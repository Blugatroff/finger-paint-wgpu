use crate::constants::MAX_LIGHTS;
use crate::light::RealLight;
use crate::{Camera, WgpuRenderer};
use std::num::NonZeroU32;

pub trait RealLightApi {
    fn add_real_light(&mut self, light: RealLightPublic) -> Result<usize, AddLightError>;
    fn remove_real_light(&mut self, light: usize);
    fn get_real_light(&mut self, light: usize) -> RealLightPublic;
    fn set_real_light(&mut self, id: usize, light: RealLightPublic);
}

impl RealLightApi for WgpuRenderer {
    fn add_real_light(&mut self, light: RealLightPublic) -> Result<usize, AddLightError> {
        if self.real_lights.len() >= MAX_LIGHTS {
            Err(AddLightError::MaximumNumberOfLightReached)
        } else {
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
            });
            Ok(self.real_lights.len() - 1)
        }
    }
    fn remove_real_light(&mut self, light: usize) {
        self.real_lights.remove(light);
    }
    fn get_real_light(&mut self, light: usize) -> RealLightPublic {
        let l = &self.real_lights[light];
        RealLightPublic {
            camera: l.camera,
            color: l.color,
            default: l.default,
        }
    }
    fn set_real_light(&mut self, light: usize, v: RealLightPublic) {
        let l = &mut self.real_lights[light];

        l.default = v.default;
        l.color = v.color;
        l.camera = v.camera;
    }
}

pub struct RealLightPublic {
    pub camera: Camera,
    pub color: [f32; 4],
    pub default: f32,
}

#[derive(Copy, Clone, Debug)]
pub enum AddLightError {
    MaximumNumberOfLightReached,
}
