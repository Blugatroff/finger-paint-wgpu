use bytemuck::{Pod, Zeroable};
use wgpu::{VertexBufferLayout, VertexFormat};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ColorVertex {
    _pos: [f32; 4],
    _normal: [f32; 4],
    _color: [f32; 4],
}

impl ColorVertex {
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
                    format: VertexFormat::Float4,
                },
            ],
        }
    }
    pub fn new(position: [f32; 3], normal: [f32; 3], color: [f32; 4]) -> Self {
        ColorVertex {
            _pos: [position[0], position[1], position[2], 1.0],
            _normal: [normal[0], normal[1], normal[2], 0.0],
            _color: color
        }
    }
}
