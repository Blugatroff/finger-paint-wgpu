use crate::instance::InstanceRaw;
use cgmath::SquareMatrix;
use cgmath::{Matrix3, Matrix4, Vector3};

#[derive(Copy, Clone, Debug)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Matrix3<f32>,
    pub scale: Vector3<f32>,
}

impl std::default::Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Matrix3::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

impl std::ops::Mul for Transform {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        Self {
            position: self.position + other.position,
            rotation: self.rotation * other.rotation,
            scale: Vector3::new(
                self.scale.x * other.scale.x,
                self.scale.y * other.scale.y,
                self.scale.z * other.scale.z,
            ),
        }
    }
}
impl From<&Transform> for Matrix4<f32> {
    fn from(instance: &Transform) -> Self {
        let rotation_matrix: Matrix4<f32> = Matrix4::from(instance.rotation);

        Matrix4::from_translation(instance.position)
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

impl From<&Transform> for InstanceRaw {
    fn from(transform: &Transform) -> Self {
        InstanceRaw {
            mat: transform.into(),
        }
    }
}
