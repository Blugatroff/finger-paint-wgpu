use crate::light::{RealLightRaw, SimpleLightRaw};
#[cfg(feature = "hot_reload_shader")]
use crate::render_passes::shader_reload::ShaderHotReload;
use crate::WgpuRenderer;
use wgpu::util::DeviceExt;

pub trait Update {
    fn update(&mut self);
}

impl Update for WgpuRenderer {
    fn update(&mut self) {
        #[cfg(feature = "hot_reload_shader")]
        self.update_pipelines();

        self.global_uniforms.proj = self.camera.build_view_projection_matrix().into();
        self.global_uniforms.camera_pos = self.camera.get_position().into();
        self.global_uniforms.num_lights = [
            self.real_lights.len() as u32,
            self.simple_lights.len() as u32,
            0,
            0,
        ];
        self.simple_lights_storage_buffer =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("simple lights storage buffer"),
                    contents: bytemuck::cast_slice(
                        &self
                            .simple_lights
                            .iter()
                            .map(|light| light.into())
                            .collect::<Vec<SimpleLightRaw>>(),
                    ),
                    usage: wgpu::BufferUsage::STORAGE,
                });

        self.real_lights_storage_buffer =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("simple lights storage buffer"),
                    contents: bytemuck::cast_slice(
                        &self
                            .real_lights
                            .iter()
                            .map(|light| light.to_raw())
                            .collect::<Vec<RealLightRaw>>(),
                    ),
                    usage: wgpu::BufferUsage::STORAGE
                        | wgpu::BufferUsage::COPY_DST
                        | wgpu::BufferUsage::COPY_SRC,
                });

        if !self.color_meshes.is_empty() {
            self.queue.write_buffer(
                &self.passes.color_forward_pass.uniform_buf,
                0,
                bytemuck::cast_slice(&[self.global_uniforms]),
            );

            self.passes.color_forward_pass.bind_group =
                self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.passes.color_forward_pass.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer {
                                buffer: &self.passes.color_forward_pass.uniform_buf,
                                offset: 0,
                                size: None,
                            },
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Buffer {
                                buffer: &self.real_lights_storage_buffer,
                                offset: 0,
                                size: None,
                            },
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Buffer {
                                buffer: &self.simple_lights_storage_buffer,
                                offset: 0,
                                size: None,
                            },
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::TextureView(&self.shadow_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: wgpu::BindingResource::Sampler(&self.shadow_sampler),
                        },
                    ],
                    label: None,
                });
        }

        if !self.uv_meshes.is_empty() {
            self.queue.write_buffer(
                &self.passes.uv_forward_pass.uniform_buf,
                0,
                bytemuck::cast_slice(&[self.global_uniforms]),
            );
            self.passes.uv_forward_pass.bind_group =
                self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.passes.uv_forward_pass.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer {
                                buffer: &self.passes.uv_forward_pass.uniform_buf,
                                offset: 0,
                                size: None,
                            },
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Buffer {
                                buffer: &self.real_lights_storage_buffer,
                                offset: 0,
                                size: None,
                            },
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Buffer {
                                buffer: &self.simple_lights_storage_buffer,
                                offset: 0,
                                size: None,
                            },
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::TextureView(&self.shadow_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: wgpu::BindingResource::Sampler(&self.shadow_sampler),
                        },
                    ],
                    label: None,
                });
        }
        if !self.models.is_empty() {
            self.queue.write_buffer(
                &self.passes.model_forward_pass.uniform_buf,
                0,
                bytemuck::cast_slice(&[self.global_uniforms]),
            );
            self.passes.model_forward_pass.bind_group =
                self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.passes.model_forward_pass.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer {
                                buffer: &self.passes.model_forward_pass.uniform_buf,
                                offset: 0,
                                size: None,
                            },
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Buffer {
                                buffer: &self.real_lights_storage_buffer,
                                offset: 0,
                                size: None,
                            },
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Buffer {
                                buffer: &self.simple_lights_storage_buffer,
                                offset: 0,
                                size: None,
                            },
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::TextureView(&self.shadow_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: wgpu::BindingResource::Sampler(&self.shadow_sampler),
                        },
                    ],
                    label: None,
                });
        }
        if !self.lines.is_empty() {
            self.queue.write_buffer(
                &self.passes.line_forward_pass.uniform_buf,
                0,
                bytemuck::cast_slice(&[self.global_uniforms]),
            );
            self.passes.line_forward_pass.bind_group =
                self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.passes.line_forward_pass.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer {
                                buffer: &self.passes.line_forward_pass.uniform_buf,
                                offset: 0,
                                size: None,
                            },
                        },
                    ],
                    label: None,
                });
            self.lines.update(&self.device);
        }
    }
}
