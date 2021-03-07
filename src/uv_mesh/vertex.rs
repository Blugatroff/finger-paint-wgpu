use bytemuck::{Pod, Zeroable};
use cgmath::{Vector2, Vector3};
use wgpu::{VertexBufferLayout, VertexFormat};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct UvVertex {
    pos: [f32; 4],
    normal: [f32; 4],
    uv: [f32; 2],
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
            pos: [position.x, position.y, position.z, 1.0],
            normal: [normal.x, normal.y, normal.z, 0.0],
            uv: [uv.x, uv.y],
        }
    }
    pub fn set_position(&mut self, pos: Vector3<f32>) {
        self.pos = [pos.x, pos.y, pos.z, 1.0];
    }
    pub fn set_normal(&mut self, n: Vector3<f32>) {
        self.normal = [n.x, n.y, n.z, 0.0];
    }
    pub fn set_uv(&mut self, uv: Vector2<f32>) {
        self.uv = [uv.x, uv.y];
    }
    pub fn get_position(&self) -> Vector3<f32> {
        Vector3::new(self.pos[0], self.pos[1], self.pos[2])
    }
    pub fn get_normal(&self) -> Vector3<f32> {
        Vector3::new(self.normal[0], self.normal[1], self.normal[2])
    }
    pub fn get_uv(&self) -> Vector2<f32> {
        self.uv.into()
    }
}
