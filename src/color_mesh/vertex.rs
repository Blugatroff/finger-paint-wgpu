use bytemuck::{Pod, Zeroable};
use cgmath::{Vector3, Vector4};
use wgpu::{VertexBufferLayout, VertexFormat};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ColorVertex {
    pos: [f32; 4],
    normal: [f32; 4],
    color: [f32; 4],
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
    pub fn new(position: Vector3<f32>, normal: Vector3<f32>, color: Vector4<f32>) -> Self {
        ColorVertex {
            pos: [position.x, position.y, position.z, 1.0],
            normal: [normal.x, normal.y, normal.z, 0.0],
            color: color.into(),
        }
    }
    pub fn get_position(&self) -> Vector3<f32> {
        Vector3::new(self.pos[0], self.pos[1], self.pos[2])
    }
    pub fn get_normal(&self) -> Vector3<f32> {
        Vector3::new(self.normal[0], self.normal[1], self.normal[2])
    }
    pub fn get_color(&self) -> Vector4<f32> {
        Vector4::new(self.color[0], self.color[1], self.color[2], self.color[3])
    }
    pub fn set_position(&mut self, p: Vector3<f32>) {
        self.pos[0] = p.x;
        self.pos[1] = p.y;
        self.pos[2] = p.z;
    }
    pub fn set_normal(&mut self, n: Vector3<f32>) {
        self.normal[0] = n.x;
        self.normal[1] = n.y;
        self.normal[2] = n.z;
    }
    pub fn set_color(&mut self, c: Vector4<f32>) {
        self.color[0] = c.x;
        self.color[1] = c.y;
        self.color[2] = c.z;
        self.color[3] = c.w;
    }
}
