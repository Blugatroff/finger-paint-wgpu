use crate::instance::InstanceRaw;
use crate::texture::Texture;
use crate::{texture, Transform};
use std::path::Path;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BindGroup, BindGroupLayout, Buffer, BufferDescriptor, BufferUsage, Device, Queue};

mod material;
mod mesh;
mod vertex;

pub use material::*;
pub use mesh::*;
pub use vertex::ModelVertex;

pub struct Model {
    pub meshes: Vec<ModelMesh>,
    pub materials: Vec<Material>,
    pub instance_buffer: Buffer,
    pub instances: Vec<Transform>,
    instances_in_buffer: usize,
}

impl Model {
    pub fn from_mesh_and_materials(
        device: &Device,
        meshes: Vec<ModelMesh>,
        materials: Vec<Material>,
    ) -> Self {
        let instances = vec![];
        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("model instance buffer"),
            size: 0,
            usage: BufferUsage::VERTEX,
            mapped_at_creation: false,
        });

        Self {
            meshes,
            materials,
            instance_buffer,
            instances_in_buffer: instances.len(),
            instances,
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
    pub fn is_empty(&self) -> bool {
        self.instances_in_buffer == 0
    }
    pub fn instances_in_buffer(&self) -> usize {
        self.instances_in_buffer
    }
    pub fn load<P: AsRef<Path>>(device: &wgpu::Device, queue: &wgpu::Queue, path: P) -> Self {
        let (obj_models, obj_materials) = tobj::load_obj(path.as_ref(), true).unwrap();

        // We're assuming that the texture files are stored with the obj file
        let containing_folder = path.as_ref().parent().unwrap();

        let mut materials = Vec::new();
        for mat in obj_materials {
            let diffuse_path = mat.diffuse_texture;

            let normal_path = mat.normal_texture;

            materials.push(Material::new(
                device,
                queue,
                containing_folder.join(diffuse_path),
                containing_folder.join(normal_path),
            ));
        }
        if materials.is_empty() {
            materials.push(Material::from_textures(
                device,
                texture::Texture::from_image(
                    device,
                    queue,
                    &texture::create_colored([255, 0, 255, 255]),
                    None,
                    wgpu::FilterMode::Nearest,
                    wgpu::FilterMode::Nearest,
                ),
                texture::Texture::from_image(
                    device,
                    queue,
                    &texture::create_colored([0, 0, 255, 255]),
                    None,
                    wgpu::FilterMode::Nearest,
                    wgpu::FilterMode::Nearest,
                ),
            ));
        }

        let mut meshes = Vec::new();
        for m in obj_models {
            let mut vertices = Vec::new();
            for i in 0..m.mesh.positions.len() / 3 {
                vertices.push(ModelVertex {
                    pos: [
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    ],
                    uv: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
                    normal: [
                        m.mesh.normals[i * 3],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    ],
                    // We'll calculate these later
                    tangent: [0.0; 3],
                    bitangent: [0.0; 3],
                });
            }

            let indices = m.mesh.indices;

            // Calculate tangents and bitangets. We're going to
            // use the triangles, so we need to loop through the
            // indices in chunks of 3
            for c in indices.chunks(3) {
                let v0 = vertices[c[0] as usize];
                let v1 = vertices[c[1] as usize];
                let v2 = vertices[c[2] as usize];

                let pos0: cgmath::Vector3<f32> = v0.pos.into();
                let pos1: cgmath::Vector3<f32> = v1.pos.into();
                let pos2: cgmath::Vector3<f32> = v2.pos.into();

                let uv0: cgmath::Vector2<f32> = v0.uv.into();
                let uv1: cgmath::Vector2<f32> = v1.uv.into();
                let uv2: cgmath::Vector2<f32> = v2.uv.into();

                // Calculate the edges of the triangle
                let delta_pos1 = pos1 - pos0;
                let delta_pos2 = pos2 - pos0;

                // This will give us a direction to calculate the
                // tangent and bitangent
                let delta_uv1 = uv1 - uv0;
                let delta_uv2 = uv2 - uv0;

                // Solving the following system of equations will
                // give us the tangent and bitangent.
                //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
                //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
                // Luckily, the place I found this equation provided
                // the solution!
                let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
                let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
                let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * r;

                // We'll use the same tangent/bitangent for each vertex in the triangle
                vertices[c[0] as usize].tangent = tangent.into();
                vertices[c[1] as usize].tangent = tangent.into();
                vertices[c[2] as usize].tangent = tangent.into();

                vertices[c[0] as usize].bitangent = bitangent.into();
                vertices[c[1] as usize].bitangent = bitangent.into();
                vertices[c[2] as usize].bitangent = bitangent.into();
            }

            meshes.push(ModelMesh::new(
                device,
                vertices,
                indices,
                m.mesh.material_id.unwrap_or(0),
            ));
        }

        Self::from_mesh_and_materials(device, meshes, materials)
    }
}
