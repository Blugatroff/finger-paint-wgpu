use bytemuck::Pod;
use bytemuck::Zeroable;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GlobalUniforms {
    pub proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 3],
    pub _padding_1: u32,
    pub num_lights: [u32; 4],
    pub ambient_light: [f32; 4],
    pub lighting_enabled: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct EntityUniforms {
    pub model: [[f32; 4]; 4],
    pub color: [f32; 4],
}