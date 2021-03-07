use wgpu::util::DeviceExt;
use wgpu::{BindGroupLayout, Buffer, Device, Sampler, SwapChainDescriptor, TextureView};

pub use color::*;
pub use model::*;
use shader_compiler::ShaderCompiler;
pub use uv::*;

use crate::color_mesh::ColorInstanceRaw;
use crate::constants::{DEPTH_FORMAT, SHADOW_FORMAT};
use crate::instance::InstanceRaw;
use crate::model::Material;
use crate::uniforms::GlobalUniforms;
use crate::ColorVertex;
use crate::ModelVertex;
use crate::{texture, UvVertex};

mod color;
mod model;
pub mod shader_compiler;
#[cfg(feature = "hot_reload_shader")]
pub mod shader_reload;
mod uv;

pub struct Passes {
    pub color_shadow_pass: Pass,
    pub color_forward_pass: Pass,
    pub uv_shadow_pass: Pass,
    pub uv_forward_pass: Pass,
    pub model_shadow_pass: Pass,
    pub model_forward_pass: Pass,
}

pub struct Pass {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub uniform_buf: wgpu::Buffer,
}

pub fn create_shadow_bind_group_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("global uniform layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0, // global
            visibility: wgpu::ShaderStage::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(
                    std::mem::size_of::<GlobalUniforms>() as wgpu::BufferAddress
                ),
            },
            count: None,
        }],
    })
}

impl Passes {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device: &Device,
        global_uniforms: &GlobalUniforms,
        real_lights_storage_buffer: &Buffer,
        simple_lights_storage_buffer: &Buffer,
        shadow_view: &TextureView,
        shadow_sampler: &Sampler,
        sc_desc: &SwapChainDescriptor,
        shaders: &ShaderCompiler,
    ) -> Self {
        let color_mesh_pipelines = create_color_mesh_pipelines(
            device,
            global_uniforms,
            real_lights_storage_buffer,
            simple_lights_storage_buffer,
            shadow_view,
            shadow_sampler,
            sc_desc,
            shaders,
        );
        let uv_mesh_pipelines = create_uv_mesh_pipelines(
            device,
            global_uniforms,
            real_lights_storage_buffer,
            simple_lights_storage_buffer,
            shadow_view,
            shadow_sampler,
            sc_desc,
            shaders,
        );
        let (model_shadow_pass, model_forward_pass) = create_model_render_passes(
            device,
            global_uniforms,
            real_lights_storage_buffer,
            simple_lights_storage_buffer,
            shadow_view,
            shadow_sampler,
            sc_desc,
            shaders,
        );
        Self {
            color_shadow_pass: color_mesh_pipelines.0,
            color_forward_pass: color_mesh_pipelines.1,
            uv_shadow_pass: uv_mesh_pipelines.0,
            uv_forward_pass: uv_mesh_pipelines.1,
            model_shadow_pass,
            model_forward_pass,
        }
    }
}
