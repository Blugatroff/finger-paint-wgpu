use bytemuck::{Pod, Zeroable};
use wgpu::{VertexBufferLayout, VertexFormat};
use cgmath::{Vector2, Vector3};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct UvVertex {
    _pos: [f32; 4],
    _normal: [f32; 4],
    _uv: [f32; 2]
}

impl UvVertex {
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
                wgpu::VertexAttribute {
                    shader_location: 2,
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    format: VertexFormat::Float2,
                },
            ],
        }
    }
    pub fn new(position: Vector3<f32>, normal: Vector3<f32>, uv: Vector2<f32>) -> Self {
        UvVertex {
            _pos: [position.x, position.y, position.z, 1.0],
            _normal: [normal.x, normal.y, normal.z, 0.0],
            _uv: [uv.x, uv.y]
        }
    }
}
