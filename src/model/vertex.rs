use bytemuck::{Pod, Zeroable};
use cgmath::{Vector2, Vector3};
use wgpu::{VertexBufferLayout, VertexFormat};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ModelVertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
}

impl ModelVertex {
    pub fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    shader_location: 0,
                    offset: 0,
                    format: VertexFormat::Float3,
                },
                wgpu::VertexAttribute {
                    shader_location: 1,
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    format: VertexFormat::Float3,
                },
                wgpu::VertexAttribute {
                    shader_location: 2,
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    format: VertexFormat::Float2,
                },
                wgpu::VertexAttribute {
                    shader_location: 3,
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    format: VertexFormat::Float3,
                },
                wgpu::VertexAttribute {
                    shader_location: 4,
                    offset: std::mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                    format: VertexFormat::Float3,
                },
            ],
        }
    }
    pub fn new(
        position: Vector3<f32>,
        normal: Vector3<f32>,
        uv: Vector2<f32>,
        tangent: Vector3<f32>,
        bitangent: Vector3<f32>,
    ) -> Self {
        ModelVertex {
            pos: position.into(),
            normal: normal.into(),
            uv: [uv.x, uv.y],
            tangent: tangent.into(),
            bitangent: bitangent.into(),
        }
    }
}
