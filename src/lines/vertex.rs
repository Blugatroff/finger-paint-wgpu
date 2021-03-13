use cgmath::{Vector3, Vector4};
use wgpu::{VertexBufferLayout, VertexFormat};

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct LineVertex {
    pos: [f32; 4],
    color: [f32; 4],
}

impl LineVertex {
    pub fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    shader_location: 0,
                    offset: 0,
                    format: VertexFormat::Float4,
                },
                wgpu::VertexAttribute {
                    shader_location: 1,
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    format: VertexFormat::Float4,
                },
            ],
        }
    }
    pub fn new(position: Vector3<f32>, color: Vector4<f32>) -> Self {
        LineVertex {
            pos: [position.x, position.y, position.z, 1.0],
            color: color.into(),
        }
    }
}
