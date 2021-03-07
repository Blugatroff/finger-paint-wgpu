use super::*;

pub struct ModelMesh {
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,

    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: usize,

    pub material: usize,
}

impl ModelMesh {
    pub fn new(
        device: &Device,
        vertices: Vec<ModelVertex>,
        indices: Vec<u32>,
        material: usize,
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
            vertex_buffer,
            index_buffer,
            index_count: 0,
            material,
        }
    }
}
