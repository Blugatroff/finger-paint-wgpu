use super::*;
use crate::lines::LineVertex;

#[allow(clippy::too_many_arguments)]
pub fn create_line_pipelines(
    device: &Device,
    global_uniforms: &GlobalUniforms,
    sc_desc: &SwapChainDescriptor,
    shaders: &ShaderCompiler,
) -> (Pass, Pass) {
    let uniform_size = std::mem::size_of::<GlobalUniforms>() as wgpu::BufferAddress;
    // Create pipeline layout
    let shadow_bind_group_layout = create_shadow_bind_group_layout(device);
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

    let shader = shaders.get_shader("line_shader");
    // Create the render pipeline for the shadow passes
    let shadow_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("shadow pass pipeline"),
        layout: Some(&shadow_pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: "vs_bake",
            buffers: &[LineVertex::desc()],
        },
        fragment: None,
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::LineList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::None,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: SHADOW_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState {
                constant: 2,
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
                            std::mem::size_of::<GlobalUniforms>() as _,
                        ),
                    },
                    count: None,
                },
            ],
            label: None,
        });
    let forward_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(&format!("forward pipeline layout: {}", "color mesh")),
        bind_group_layouts: &[&forward_bind_group_layout],
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
        ],
        label: None,
    });

    // Create the render pipeline
    let forward_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&format!("forward pipeline: {}", "color mesh")),
        layout: Some(&forward_pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: "vs_main",
            buffers: &[LineVertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: "fs_main",
            targets: &[sc_desc.format.into()],
        }),
        primitive: wgpu::PrimitiveState {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::None,
            polygon_mode: wgpu::PolygonMode::Fill,
            topology: wgpu::PrimitiveTopology::LineList,
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