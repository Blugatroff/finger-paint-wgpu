use crate::color_mesh::instance::{ColorInstanceRaw, ColorMeshInstance};
use crate::color_mesh::ColorVertex;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{Buffer, BufferDescriptor, BufferUsage, Device};

use cgmath::Vector3;

pub struct ColorMesh {
    pub color: Vector3<f32>,
    pub instances: Vec<ColorMeshInstance>,
    pub vertex_buf: wgpu::Buffer,
    pub index_buf: Option<wgpu::Buffer>,
    pub vertex_count: usize,
    pub index_count: usize,
    pub instance_buffer: Buffer,
    pub instances_in_buffer: usize,
}

impl ColorMesh {
    pub fn from_vertices_and_indices(
        device: &Device,
        vertices: Vec<ColorVertex>,
        indices: Option<Vec<u16>>,
    ) -> Self {
        let instances = vec![];
        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("model instance buffer"),
            size: 0,
            usage: BufferUsage::VERTEX,
            mapped_at_creation: false,
        });

        Self {
            color: Vector3::new(1.0, 0.0, 1.0),
            vertex_buf: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Cubes Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: BufferUsage::VERTEX,
            }),
            index_buf: indices.as_ref().map(|indices| device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Cubes Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: BufferUsage::INDEX,
            })),
            vertex_count: vertices.len(),
            index_count: if let Some(indices) = indices {
                indices.len()
            } else {
                0
            },
            instances_in_buffer: instances.len(),
            instances,
            instance_buffer,
        }
    }
    pub fn update(&mut self, device: &Device) {
        self.instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("instance vertex buffer"),
            contents: bytemuck::cast_slice(
                &self
                    .instances
                    .iter()
                    .map(|instance: &ColorMeshInstance| instance.into())
                    .collect::<Vec<ColorInstanceRaw>>(),
            ),
            usage: BufferUsage::VERTEX,
        });
        self.instances_in_buffer = self.instances.len();
    }
    pub fn is_empty(&self) -> bool {
        self.instances_in_buffer == 0
    }
}
