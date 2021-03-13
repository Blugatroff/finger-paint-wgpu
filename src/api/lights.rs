use crate::constants::MAX_LIGHTS;
use crate::light::{RealLight, LightAttenuation};
use crate::{Camera, WgpuRenderer};
use std::num::NonZeroU32;

pub trait RealLightApi {
    fn add_real_light(&mut self, light: RealLightPublic) -> Result<usize, String>;
    fn remove_real_light(&mut self, light: usize);
    fn get_real_light(&self, light: usize) -> Option<RealLightPublic>;
    fn set_real_light(&mut self, id: usize, light: RealLightPublic);
    fn update_real_lights(&mut self);
}

impl RealLightApi for WgpuRenderer {
    fn add_real_light(&mut self, light: RealLightPublic) -> Result<usize, String> {
        if self.real_lights.iter().fold(0, |mut i, l| {
            if l.active {
                i += 1
            }
            i
        }) >= MAX_LIGHTS
        {
            Err(format!("maximum number of lights reached ({})", MAX_LIGHTS))
        } else {
            for l in &mut self.real_lights {
                if !l.active {
                    l.active = true;
                    l.camera = light.camera;
                    l.default = light.default;
                    l.constant = light.attenuation.constant;
                    l.linear = light.attenuation.linear;
                    l.quadratic = light.attenuation.quadratic;
                }
            }
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
                constant: 1.0,
                linear: 0.01,
                quadratic: 0.03,
                active: true,
            });
            Ok(self.real_lights.len() - 1)
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
}

pub struct RealLightPublic {
    pub camera: Camera,
    pub color: [f32; 4],
    pub default: f32,
    pub attenuation: LightAttenuation,
}