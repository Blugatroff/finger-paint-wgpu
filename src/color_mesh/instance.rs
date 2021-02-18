use bytemuck::{Pod, Zeroable};
use cgmath::{EuclideanSpace, Matrix3, Matrix4, Point3, Vector3};
use wgpu::{VertexBufferLayout, VertexFormat};

#[derive(Copy, Clone, Debug)]
pub struct ColorMeshInstance {
    pub transform: Transform,
    pub lighting: Lighting,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Lighting {
    pub specular_strength: f32,
    pub specular_spread: f32,
    pub diffuse_strength: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct InstanceRaw {
    mat: [[f32; 4]; 4],
    specular_strength: f32,
    specular_spread: f32,
    diffuse_strength: f32,
}

impl From<&ColorMeshInstance> for InstanceRaw {
    fn from(instance: &ColorMeshInstance) -> Self {
        Self {
            mat: (&instance.transform).into(),
            specular_strength: instance.lighting.specular_strength,
            specular_spread: instance.lighting.specular_spread,
            diffuse_strength: instance.lighting.diffuse_strength,
        }
    }
}

impl InstanceRaw {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    shader_location: 5,
                    offset: 0,
                    format: VertexFormat::Float4,
                },
                wgpu::VertexAttribute {
                    shader_location: 6,
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    format: VertexFormat::Float4,
                },
                wgpu::VertexAttribute {
                    shader_location: 7,
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    format: VertexFormat::Float4,
                },
                wgpu::VertexAttribute {
                    shader_location: 8,
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    format: VertexFormat::Float4,
                },
                wgpu::VertexAttribute {
                    shader_location: 9,
                    offset: std::mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    format: VertexFormat::Float,
                },
                wgpu::VertexAttribute {
                    shader_location: 10,
                    offset: std::mem::size_of::<[f32; 17]>() as wgpu::BufferAddress,
                    format: VertexFormat::Float,
                },
                wgpu::VertexAttribute {
                    shader_location: 11,
                    offset: std::mem::size_of::<[f32; 18]>() as wgpu::BufferAddress,
                    format: VertexFormat::Float,
                },
            ],
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Transform {
    pub position: Point3<f32>,
    pub rotation: Matrix3<f32>,
    pub scale: Vector3<f32>,
}
impl From<&Transform> for Matrix4<f32> {
    fn from(instance: &Transform) -> Self {
        let rotation_matrix: Matrix4<f32> = Matrix4::from(instance.rotation);

        Matrix4::from_translation(instance.position.to_vec())
            * rotation_matrix
            * Matrix4::from_nonuniform_scale(instance.scale.x, instance.scale.y, instance.scale.z)
    }
}
impl From<&Transform> for [[f32; 4]; 4] {
    fn from(instance: &Transform) -> Self {
        let mat: Matrix4<f32> = instance.into();
        mat.into()
    }
}
