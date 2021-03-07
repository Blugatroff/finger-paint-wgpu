use super::*;

#[allow(clippy::too_many_arguments)]
pub fn create_model_render_passes(
    device: &Device,
    global_uniforms: &GlobalUniforms,
    real_lights_storage_buffer: &Buffer,
    simple_lights_storage_buffer: &Buffer,
    shadow_view: &TextureView,
    shadow_sampler: &Sampler,
    sc_desc: &SwapChainDescriptor,
    shaders: &ShaderCompiler,
) -> (Pass, Pass) {
    let bake_shader = shaders.get_shader("model_bake");
    let vs_shader = shaders.get_shader("model_vs");
    let fs_shader = shaders.get_shader("model_fs");
    //let diffuse_texture_bind_group_layout = texture::;
    let uniform_size = std::mem::size_of::<GlobalUniforms>() as wgpu::BufferAddress;
    // Create pipeline layout
    let diffuse_texture_bind_group_layout = Material::layout(device);
    let shadow_bind_group_layout =
        create_shadow_bind_group_layout(device);
    let shadow_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("shadow"),
        bind_group_layouts: &[&shadow_bind_group_layout],
        push_constant_ranges: &[],
    });
    // this buffer is not yet initialized
    let shadow_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: uniform_size,
        usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        mapped_at_creation: false,
    });

    // create bind group
    // this has the global uniforms and the uniforms of each entity
    let shadow_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &shadow_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: shadow_uniform_buffer.as_entire_binding(),
        }],
        label: None,
    });
    // Create the render pipeline for the shadow passes
    let shadow_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("shadow pass pipeline"),
        layout: Some(&shadow_pipeline_layout),
        vertex: wgpu::VertexState {
            module: bake_shader,
            entry_point: "main",
            buffers: &[ModelVertex::desc(), InstanceRaw::desc()],
        },
        fragment: None,
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::Back,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: SHADOW_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState {
                constant: 2, // corresponds to bilinear filtering
                slope_scale: 2.0,
                clamp: 0.0,
            },
            clamp_depth: device.features().contains(wgpu::Features::DEPTH_CLAMPING),
        }),
        multisample: wgpu::MultisampleState::default(),
    });
    // Create pipeline layout
    let forward_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0, // global
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<GlobalUniforms>() as u64,
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1, // real_lights
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None, //wgpu::BufferSize::new(real_light_uniform_size),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2, // simple_lights
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3, // shadow_texture
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4, // shadow sampler
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: true,
                        filtering: false,
                    },
                    count: None,
                },
            ],
            label: None,
        });
    let forward_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(&format!("forward pipeline layout: {}", "model_pipeline")),
        bind_group_layouts: &[
            &forward_bind_group_layout,
            &diffuse_texture_bind_group_layout,
        ],
        push_constant_ranges: &[],
    });

    let forward_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: bytemuck::bytes_of(global_uniforms),
        usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
    });

    // Create bind group
    let forward_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &forward_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: forward_uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: real_lights_storage_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &simple_lights_storage_buffer,
                    offset: 0,
                    size: None,
                },
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(shadow_view),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Sampler(shadow_sampler),
            },
        ],
        label: None,
    });
    // Create the render pipeline
    let forward_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&format!("forward pipeline: {}", "model_mesh_pipeline")),
        layout: Some(&forward_pipeline_layout),
        vertex: wgpu::VertexState {
            module: vs_shader,
            entry_point: "main",
            buffers: &[ModelVertex::desc(), InstanceRaw::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: fs_shader,
            entry_point: "main",
            targets: &[sc_desc.format.into()],
        }),
        primitive: wgpu::PrimitiveState {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::Back,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
            clamp_depth: false,
        }),
        multisample: wgpu::MultisampleState::default(),
    });
    (
        Pass {
            pipeline: shadow_pipeline,
            bind_group_layout: shadow_bind_group_layout,
            bind_group: shadow_bind_group,
            uniform_buf: shadow_uniform_buffer,
        },
        Pass {
            pipeline: forward_pipeline,
            bind_group_layout: forward_bind_group_layout,
            bind_group: forward_bind_group,
            uniform_buf: forward_uniform_buffer,
        },
    )
}
