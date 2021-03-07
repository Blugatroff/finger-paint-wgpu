use cgmath::{Point3, Deg};
use cgmath::Vector3;
use cgmath::Matrix4;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub const MAX_LIGHTS: usize = 10;
pub const SHADOW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub const SHADOW_SIZE: wgpu::Extent3d = wgpu::Extent3d {
    width: 256 * 8,
    height: 256 * 8,
    depth: MAX_LIGHTS as u32,
};
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub fn generate_matrix(aspect_ratio: f32) -> Matrix4<f32> {
    let mx_projection = cgmath::perspective(Deg(45f32), aspect_ratio, 1.0, 20.0);
    let mx_view = Matrix4::look_at_rh(
        Point3::new(5.0, 5.0, 5.0),
        Point3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    );
    OPENGL_TO_WGPU_MATRIX * mx_projection * mx_view
}
