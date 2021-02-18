use crate::texture;
use crate::texture::Texture;
use std::path::Path;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BindGroup, BindGroupLayout, Buffer, BufferDescriptor, BufferUsage, Device, Queue};
use crate::model_mesh::ModelVertex;

pub struct ModelMesh {
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u16>,
    pub material: usize,

    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

impl ModelMesh {
    pub fn new<P: AsRef<Path>>(
        vertices: Vec<ModelVertex>,
        indices: Vec<u16>,
        device: &Device,
        material: usize
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cubes Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsage::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cubes Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsage::INDEX,
        });

        Self {
            vertices,
            indices,
            material,
            vertex_buffer,
            index_buffer,
        }
    }
}
