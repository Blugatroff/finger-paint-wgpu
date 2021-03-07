use crate::Transform;
use wgpu::{VertexBufferLayout, VertexFormat};
use bytemuck::{Pod, Zeroable};

#[derive(Copy, Clone, Debug)]
pub struct Instance {
    pub transform: Transform,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct InstanceRaw {
    pub mat: [[f32; 4]; 4],
}

impl From<&Instance> for InstanceRaw {
    fn from(instance: &Instance) -> Self {
        Self {
            mat: (&instance.transform).into(),
        }
    }
}

impl InstanceRaw {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    shader_location: 5,
                    offset: 0,
                    format: VertexFormat::Float4,
                },
                wgpu::VertexAttribute {
                    shader_location: 6,
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    format: VertexFormat::Float4,
                },
                wgpu::VertexAttribute {
                    shader_location: 7,
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    format: VertexFormat::Float4,
                },
                wgpu::VertexAttribute {
                    shader_location: 8,
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    format: VertexFormat::Float4,
                },
            ],
        }
    }
}