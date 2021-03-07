use crate::Camera;
use bytemuck::Pod;
use bytemuck::Zeroable;
use cgmath::Vector4;

pub struct RealLight {
    pub camera: Camera,
    pub color: [f32; 4],
    pub target_view: wgpu::TextureView,
    pub default: f32,
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct RealLightRaw {
    proj: [[f32; 4]; 4],
    pos: [f32; 4],
    color: [f32; 4],
    default: f32,
    constant: f32,
    linear: f32,
    quadratic: f32,
}

impl RealLight {
    pub fn to_raw(&self) -> RealLightRaw {
        let pos = self.camera.get_position();
        RealLightRaw {
            proj: self.camera.build_view_projection_matrix().into(),
            pos: [pos.x, pos.y, pos.z, 1.0],
            color: [self.color[0], self.color[1], self.color[2], self.color[3]],
            default: self.default,
            constant: self.constant,
            linear: self.linear,
            quadratic: self.quadratic,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SimpleLight {
    pub color: Vector4<f32>,
    pub kind: SimpleLightKind,
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum SimpleLightKind {
    Directional([f32; 3]),
    Positional([f32; 3])
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct SimpleLightRaw {
    pub color: [f32; 4],
    pub position_or_direction: [f32; 4],
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
}

impl From<&SimpleLight> for SimpleLightRaw {
    fn from(light: &SimpleLight) -> Self {
        Self {
            color: light.color.into(),
            position_or_direction: match light.kind {
                SimpleLightKind::Directional([x, y, z]) => {
                    [x, y, z, 0.0]
                }
                SimpleLightKind::Positional([x, y, z]) => {
                    [x, y, z, 1.0]
                }
            },
            constant: light.constant,
            linear: light.linear,
            quadratic: light.quadratic,
        }
    }
}

