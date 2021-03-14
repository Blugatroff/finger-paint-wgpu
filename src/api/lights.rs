use crate::constants::SHADOW_FORMAT;
use crate::{Camera, WgpuRenderer};
use std::num::NonZeroU32;

pub struct RealLightPublic {
    pub camera: Camera,
    pub color: [f32; 4],
    pub default: f32,
    pub attenuation: LightAttenuation,
}

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
    pub active: bool,
    pub resolution: [u32; 2],
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
    active: u32,
    _padding: [u32; 3],
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
            active: if self.active { 1 } else { 0 },
            _padding: [42069; 3],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SimpleLight {
    pub color: Vector4<f32>,
    pub kind: SimpleLightKind,
    pub attenuation: LightAttenuation,
}

#[derive(Clone, Copy, Debug)]
pub enum SimpleLightKind {
    Directional([f32; 3]),
    Positional([f32; 3]),
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
                SimpleLightKind::Directional([x, y, z]) => [x, y, z, 0.0],
                SimpleLightKind::Positional([x, y, z]) => [x, y, z, 1.0],
            },
            constant: light.attenuation.constant,
            linear: light.attenuation.linear,
            quadratic: light.attenuation.quadratic,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LightAttenuation {
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
}

impl Default for LightAttenuation {
    fn default() -> Self {
        Self {
            constant: 1.0,
            linear: 0.05,
            quadratic: 0.01,
        }
    }
}

pub trait RealLightApi {
    fn add_real_light(&mut self, light: RealLightPublic) -> Result<usize, String>;
    fn remove_real_light(&mut self, light: usize);
    fn get_real_light(&self, light: usize) -> Option<RealLightPublic>;
    fn set_real_light(&mut self, id: usize, light: RealLightPublic);
    fn update_real_lights(&mut self);
    fn set_shadow_resolution(&mut self, res: [u32; 2]);
}

impl RealLightApi for WgpuRenderer {
    fn add_real_light(&mut self, light: RealLightPublic) -> Result<usize, String> {
        if self.real_lights.iter().fold(0, |mut i, l| {
            if l.active {
                i += 1
            }
            i
        }) >= self.max_real_lights
        {
            Err(format!(
                "maximum number of lights reached ({})",
                self.max_real_lights
            ))
        } else {
            let mut reusing = None;
            let mut overwrite = None;
            for (i, l) in self.real_lights.iter_mut().enumerate() {
                if !l.active {
                    l.active = true;
                    l.camera = light.camera;
                    l.default = light.default;
                    l.constant = light.attenuation.constant;
                    l.linear = light.attenuation.linear;
                    l.quadratic = light.attenuation.quadratic;
                    if l.resolution != self.shadow_resolution {
                        overwrite = Some(i);
                    } else {
                        reusing = Some(i);
                    }
                }
            }
            if let Some(index) = reusing {
                self.real_lights[index].resolution = self.shadow_resolution; // this does nothing since the resolution must already be the same to reach this point
                self.real_lights[index].camera = light.camera;
                self.real_lights[index].color = light.color;
                self.real_lights[index].constant = light.attenuation.constant;
                self.real_lights[index].linear = light.attenuation.linear;
                self.real_lights[index].quadratic = light.attenuation.quadratic;
                self.real_lights[index].active = true;
                Ok(index)
            } else if let Some(index) = overwrite {
                self.real_lights[index].resolution = self.shadow_resolution;
                self.real_lights[index].camera = light.camera;
                self.real_lights[index].color = light.color;
                self.real_lights[index].constant = light.attenuation.constant;
                self.real_lights[index].linear = light.attenuation.linear;
                self.real_lights[index].quadratic = light.attenuation.quadratic;
                self.real_lights[index].active = true;
                self.real_lights[index].target_view =
                    self.shadow_texture
                        .create_view(&wgpu::TextureViewDescriptor {
                            label: Some("shadow"),
                            format: None,
                            dimension: Some(wgpu::TextureViewDimension::D2),
                            aspect: wgpu::TextureAspect::All,
                            base_mip_level: 0,
                            level_count: None,
                            base_array_layer: index as u32,
                            array_layer_count: NonZeroU32::new(1),
                        });
                Ok(index)
            } else {
                let attenuation = LightAttenuation::default();
                self.real_lights.push(RealLight {
                    camera: light.camera,
                    color: light.color,
                    target_view: self
                        .shadow_texture
                        .create_view(&wgpu::TextureViewDescriptor {
                            label: Some("shadow"),
                            format: None,
                            dimension: Some(wgpu::TextureViewDimension::D2),
                            aspect: wgpu::TextureAspect::All,
                            base_mip_level: 0,
                            level_count: None,
                            base_array_layer: self.real_lights.len() as u32,
                            array_layer_count: NonZeroU32::new(1),
                        }),
                    default: light.default,
                    constant: attenuation.constant,
                    linear: attenuation.linear,
                    quadratic: attenuation.quadratic,
                    active: true,
                    resolution: self.shadow_resolution,
                });
                Ok(self.real_lights.len() - 1)
            }
        }
    }
    fn remove_real_light(&mut self, light: usize) {
        self.real_lights[light].active = false;
    }
    fn get_real_light(&self, light: usize) -> Option<RealLightPublic> {
        if light >= self.real_lights.len() {
            return None;
        }
        let light = &self.real_lights[light];
        if light.active {
            Some(RealLightPublic {
                camera: light.camera,
                color: light.color,
                default: light.default,
                attenuation: LightAttenuation {
                    constant: light.constant,
                    linear: light.linear,
                    quadratic: light.quadratic,
                },
            })
        } else {
            None
        }
    }
    fn set_real_light(&mut self, light: usize, v: RealLightPublic) {
        if light < self.real_lights.len() {
            let light = &mut self.real_lights[light];
            if light.active {
                light.default = v.default;
                light.color = v.color;
                light.camera = v.camera;
            }
        }
    }
    fn update_real_lights(&mut self) {
        self.lights_are_dirty = true;
    }
    fn set_shadow_resolution(&mut self, res: [u32; 2]) {
        let shadow_resolution = wgpu::Extent3d {
            width: res[0],
            height: res[1],
            depth: self.max_real_lights as u32,
        };
        self.shadow_resolution = res;
        self.shadow_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: shadow_resolution,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: SHADOW_FORMAT,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
            label: None,
        });
        self.shadow_view = self
            .shadow_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        for i in 0..self.real_lights.len() {
            if let Some(light) = self.get_real_light(i) {
                self.remove_real_light(i);
                self.add_real_light(light).unwrap();
            }
        }
    }
}
