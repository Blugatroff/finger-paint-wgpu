use crate::camera::{Camera, ViewMatrixMode};
use crate::lines::Lines;
use crate::render_passes::shader_compiler::ShaderCompiler;
#[cfg(feature = "hot_reload_shader")]
use crate::render_passes::shader_compiler::ShaderType;
#[cfg(feature = "hot_reload_shader")]
use crate::render_passes::shader_reload::ShaderHotReload;
use crate::render_passes::Passes;
use crate::uniforms::GlobalUniforms;
use crate::{WgpuRenderer, DEPTH_FORMAT, SHADOW_FORMAT};
use cgmath::{Point3, Vector3};
use futures::executor::block_on;
use std::f32::consts::PI;
use std::mem;
use std::path::PathBuf;
use wgpu::util::DeviceExt;
use wgpu::Features;
use crate::api::lights::{RealLightRaw, SimpleLight, SimpleLightRaw};

pub trait New {
    fn new(window: &winit::window::Window, hot_reload: Option<PathBuf>) -> Self;
}

impl New for WgpuRenderer {
    #[allow(unused_variables)]
    fn new(window: &winit::window::Window, hot_reload: Option<PathBuf>) -> Self {
        let max_real_lights: u32 = 10;
        let shadow_resolution = [512, 512];
        let instance = wgpu::Instance::new(wgpu::BackendBit::VULKAN);
        let surface = unsafe { instance.create_surface(window) };
        let size = window.inner_size();
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
        }))
        .expect("No suitable GPU adapters found on the system!");

        let trace_dir = std::env::var("WGPU_TRACE");

        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: Features::default(),
                limits: wgpu::Limits::default(),
            },
            trace_dir.ok().as_ref().map(std::path::Path::new),
        ))
        .expect("Unable to find a suitable GPU adapter!");

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        // Create staging belt and a local pool
        let staging_belt = wgpu::util::StagingBelt::new(1024);
        let local_pool = futures::executor::LocalPool::new();
        let local_spawner = local_pool.spawner();

        let inconsolata = wgpu_glyph::ab_glyph::FontArc::try_from_slice(include_bytes!(
            "../res/Inconsolata-Regular.ttf"
        ))
        .unwrap();

        let glyph_brush = wgpu_glyph::GlyphBrushBuilder::using_font(inconsolata)
            .build(&device, wgpu::TextureFormat::Bgra8UnormSrgb);

        // ==================== create the shadow textures ==================== \\
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        // create a shadow texture, the shadows (distance of fragment to light source) will be put in layers of this
        let shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: shadow_resolution[0],
                height: shadow_resolution[1],
                depth: max_real_lights as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: SHADOW_FORMAT,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
            label: None,
        });

        // this will later be used to create the bind_group for the forward pass
        let shadow_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let real_lights = vec![];
        // ==================== create the storage buffer for the lights ==================== \\
        // the size of the buffer is determined by the max number of lights (10) and the size of each individual LightRaw
        let real_light_uniform_size =
            (max_real_lights as usize * mem::size_of::<RealLightRaw>()) as wgpu::BufferAddress;
        // this buffer is not initialized yet
        let real_lights_storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: real_light_uniform_size,
            usage: wgpu::BufferUsage::STORAGE
                | wgpu::BufferUsage::COPY_SRC
                | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let camera = Camera::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            1.0,
            ViewMatrixMode::Perspective {
                near: 0.1,
                far: 100.0,
                fov: PI / 2.0,
            },
        );
        let global_uniforms = GlobalUniforms {
            proj: camera.build_view_projection_matrix().into(),
            camera_pos: camera.get_position().into(),
            _padding_1: 0,
            num_lights: [real_lights.len() as u32, 0, 0, 0],
            ambient_light: [0.05, 0.05, 0.05, 1.0],
            lighting_enabled: 1,
        };

        let simple_lights: Vec<SimpleLight> = vec![];
        let simple_lights_storage_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("simple lights storage buffer"),
                contents: bytemuck::cast_slice(
                    &simple_lights
                        .iter()
                        .map(|light| light.into())
                        .collect::<Vec<SimpleLightRaw>>(),
                ),
                usage: wgpu::BufferUsage::STORAGE,
            });

        let mut shader_flags = wgpu::ShaderFlags::VALIDATION;

        // ==================== don't know what this is exactly ==================== \\
        if let wgpu::Backend::Vulkan = adapter.get_info().backend {
            shader_flags |= wgpu::ShaderFlags::EXPERIMENTAL_TRANSLATION;
        }

        let mut shaders = ShaderCompiler::new(shader_flags);

        #[cfg(feature = "hot_reload_shader")]
        {
            use shaderc::ShaderKind;
            let path = hot_reload.unwrap();
            shaders.read_from_file(&device, path.join("./src/color_mesh/shader.wgsl"), ShaderType::Wgsl, ShaderKind::Vertex, "color_mesh");
            shaders.read_from_file(&device, path.join("./src/uv_mesh/shader.wgsl"), ShaderType::Wgsl,ShaderKind::Vertex,  "uv_mesh");
            shaders.read_from_file(&device, path.join("./src/model/bake.wgsl"), ShaderType::Wgsl, ShaderKind::Vertex, "model_bake");
            shaders.read_from_file(&device, path.join("./src/model/vs.glsl"), ShaderType::Glsl, ShaderKind::Vertex, "model_vs");
            shaders.read_from_file( &device, path.join("./src/model/fs.glsl"), ShaderType::Glsl, ShaderKind::Fragment, "model_fs");
            shaders.read_from_file(&device, path.join("./src/lines/shader.wgsl"), ShaderType::Wgsl, ShaderKind::Vertex, "line_shader");
        }
        #[cfg(not(feature = "hot_reload_shader"))]
        {
            shaders.load_wgsl(
                &device,
                include_str!("color_mesh/shader.wgsl"),
                "color_mesh",
            );
            shaders.load_wgsl(&device, include_str!("uv_mesh/shader.wgsl"), "uv_mesh");
            shaders.load_wgsl(&device, include_str!("model/bake.wgsl"), "model_bake");
            shaders.load_spirv(&device, include_bytes!("model/vs.glsl.spv"), "model_vs");
            shaders.load_spirv(&device, include_bytes!("model/fs.glsl.spv"), "model_fs");
            shaders.load_wgsl(&device, include_str!("lines/shader.wgsl"), "line_shader");
        }

        let passes = Passes::new(
            &device,
            &global_uniforms,
            &real_lights_storage_buffer,
            &simple_lights_storage_buffer,
            &shadow_view,
            &shadow_sampler,
            &sc_desc,
            &shaders,
        );

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: sc_desc.width,
                height: sc_desc.height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            label: None,
        });

        let lines = Lines::new(&device);

        #[allow(clippy::let_and_return, unused_mut)]
        let mut renderer = Self {
            _instance: instance,
            _adapter: adapter,
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,

            real_lights,
            lights_are_dirty: true,

            color_meshes: vec![],
            uv_meshes: vec![],
            models: vec![],
            lines,

            passes,

            forward_depth: depth_texture.create_view(&wgpu::TextureViewDescriptor::default()),
            shadow_texture,
            global_uniforms,
            camera,
            glyph_brush,
            staging_belt,
            local_pool,
            local_spawner,
            paragraphs: vec![],
            simple_lights,
            simple_lights_storage_buffer,
            real_lights_storage_buffer,
            shadow_view,
            shadow_sampler,
            max_real_lights,
            shadow_resolution,
            shaders,
        };

        #[cfg(feature = "hot_reload_shader")]
        renderer.init_shader_watch();

        renderer
    }
}
