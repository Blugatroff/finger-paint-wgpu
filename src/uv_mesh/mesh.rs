use crate::instance::InstanceRaw;
use crate::texture::Texture;
use crate::uv_mesh::vertex::UvVertex;
use crate::{texture, Transform};
use std::path::Path;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BindGroup, BindGroupLayout, Buffer, BufferDescriptor, BufferUsage, Device, Queue};

pub struct UvModel {
    pub vertices: Vec<UvVertex>,
    pub indices: Option<Vec<u16>>,
    pub diffuse_texture: Texture,

    pub instances: Vec<Transform>,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: Option<wgpu::Buffer>,
    pub index_count: usize,
    pub instance_buffer: Buffer,
    pub instances_in_buffer: usize,
    pub diffuse_bind_group: BindGroup,
    pub diffuse_bind_group_layout: BindGroupLayout,
}

impl UvModel {
    pub fn new<P: AsRef<Path>>(
        vertices: Vec<UvVertex>,
        indices: Option<Vec<u16>>,
        device: &Device,
        queue: &Queue,
        path: P,
    ) -> Self {
        let diffuse_texture = Texture::load(
            device,
            queue,
            path,
            wgpu::FilterMode::Nearest,
            wgpu::FilterMode::Nearest,
        )
        .unwrap();

        let instances = vec![];
        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("model instance buffer"),
            size: 0,
            usage: BufferUsage::VERTEX,
            mapped_at_creation: false,
        });
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cubes Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsage::VERTEX,
        });
        let index_buffer = indices.as_ref().map(|indices| device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cubes Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: BufferUsage::INDEX,
        }));

        let diffuse_bind_group_layout = texture::create_default_bind_group_layout(&device);
        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("diffuse bind group"),
            layout: &diffuse_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
        });

        Self {
            vertices,
            index_count: if let Some(indices) = &indices { indices.len() } else { 0 },
            indices,
            diffuse_texture,
            instances,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            diffuse_bind_group,
            diffuse_bind_group_layout,
            instances_in_buffer: 0,
        }
    }
    pub fn update(&mut self, device: &Device) {
        self.instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("instance vertex buffer"),
            contents: bytemuck::cast_slice(
                &self
                    .instances
                    .iter()
                    .map(|transform: &Transform| transform.into())
                    .collect::<Vec<InstanceRaw>>(),
            ),
            usage: BufferUsage::VERTEX,
        });
        self.instances_in_buffer = self.instances.len();
    }
    pub fn update_texture(&mut self, device: &Device) {
        self.diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("diffuse bind group"),
            layout: &self.diffuse_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.diffuse_texture.sampler),
                },
            ],
        });
    }
    pub fn is_empty(&self) -> bool {
        self.instances_in_buffer == 0
    }
}
