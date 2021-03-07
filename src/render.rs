use crate::light::RealLightRaw;
use crate::WgpuRenderer;
use std::mem;
use wgpu_glyph::{BuiltInLineBreaker, Layout, Section, Text};

pub trait Render {
    fn render(&mut self);
}

impl Render for WgpuRenderer {
    fn render(&mut self) {
        let frame = self.swap_chain.get_current_frame().unwrap();

        if self.lights_are_dirty {
            self.lights_are_dirty = false;
            for (i, light) in self.real_lights.iter().enumerate() {
                self.queue.write_buffer(
                    &self.real_lights_storage_buffer,
                    (i * mem::size_of::<RealLightRaw>()) as wgpu::BufferAddress,
                    bytemuck::bytes_of(&light.to_raw()),
                );
            }
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        encoder.push_debug_group("shadow passes for color_mesh");
        for (i, light) in self.real_lights.iter().enumerate() {
            encoder.push_debug_group(&format!(
                "shadow pass color_mesh {} (light at position {:?})",
                i,
                light.camera.get_position()
            ));

            if !self.color_meshes.is_empty() {
                // copy the view_proj_matrix from the current light into the global_uniforms of the shadow pass
                encoder.copy_buffer_to_buffer(
                    &self.real_lights_storage_buffer, // the buffer all the lights are stored in
                    (i * mem::size_of::<RealLightRaw>()) as wgpu::BufferAddress,
                    &self.passes.color_shadow_pass.uniform_buf, // the globals of the shadow pass
                    0, // the destination is always at the start of the global_uniform_buffer because it gets overwritten everytime
                    64, // mat4x4 = 4float * 4float = 16 float = 64 bytes
                );
            }
            if !self.uv_meshes.is_empty() {
                encoder.copy_buffer_to_buffer(
                    &self.real_lights_storage_buffer,
                    (i * mem::size_of::<RealLightRaw>()) as wgpu::BufferAddress,
                    &self.passes.uv_shadow_pass.uniform_buf,
                    0,
                    64,
                );
            }
            if !self.models.is_empty() {
                encoder.copy_buffer_to_buffer(
                    &self.real_lights_storage_buffer,
                    (i * mem::size_of::<RealLightRaw>()) as wgpu::BufferAddress,
                    &self.passes.model_shadow_pass.uniform_buf,
                    0,
                    64,
                );
            }
            encoder.insert_debug_marker("render color meshes");
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[], // no color attachments needed because we only care about the depth
                    depth_stencil_attachment: Some(
                        wgpu::RenderPassDepthStencilAttachmentDescriptor {
                            attachment: &light.target_view, // depth is written into the texture of the light
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: true,
                            }),
                            stencil_ops: None,
                        },
                    ),
                });
                if !self.models.is_empty() {
                    pass.set_pipeline(&self.passes.model_shadow_pass.pipeline);
                    pass.set_bind_group(0, &self.passes.model_shadow_pass.bind_group, &[]); // the globals

                    for model in &self.models {
                        for mesh in &model.meshes {
                            pass.set_bind_group(1, &model.materials[mesh.material].bind_group, &[]);
                            pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                            pass.set_vertex_buffer(1, model.instance_buffer.slice(..));
                            pass.set_index_buffer(
                                mesh.index_buffer.slice(..),
                                wgpu::IndexFormat::Uint32,
                            );
                            pass.draw_indexed(
                                0..mesh.indices.len() as u32,
                                0,
                                0..model.instances.len() as u32,
                            );
                        }
                    }
                }
                if !self.color_meshes.is_empty() {
                    pass.set_pipeline(&self.passes.color_shadow_pass.pipeline);
                    pass.set_bind_group(0, &self.passes.color_shadow_pass.bind_group, &[]); // the globals
                    for model in self.color_meshes.iter().flatten() {
                        if !model.is_empty() {
                            pass.set_vertex_buffer(0, model.vertex_buf.slice(..));
                            pass.set_vertex_buffer(1, model.instance_buffer.slice(..));
                            if let Some(index_buf) = &model.index_buf {
                                pass.set_index_buffer(
                                    index_buf.slice(..),
                                    wgpu::IndexFormat::Uint16,
                                );
                                pass.draw_indexed(
                                    0..model.index_count as u32,
                                    0,
                                    0..model.instances.len() as u32,
                                );
                            } else {
                                pass.draw(
                                    0..model.index_count as u32,
                                    0..model.instances.len() as u32,
                                );
                            }
                        }
                    }
                }
                if !self.uv_meshes.is_empty() {
                    pass.set_pipeline(&self.passes.uv_shadow_pass.pipeline);
                    pass.set_bind_group(0, &self.passes.uv_shadow_pass.bind_group, &[]); // the globals

                    for uv_mesh in self.uv_meshes.iter().flatten() {
                        if !uv_mesh.is_empty() {
                            pass.set_bind_group(1, &uv_mesh.diffuse_bind_group, &[]);
                            pass.set_vertex_buffer(0, uv_mesh.vertex_buffer.slice(..));
                            pass.set_vertex_buffer(1, uv_mesh.instance_buffer.slice(..));
                            if let Some(index_buffer) = &uv_mesh.index_buffer {
                                pass.set_index_buffer(
                                    index_buffer.slice(..),
                                    wgpu::IndexFormat::Uint16,
                                );
                                pass.draw_indexed(
                                    0..uv_mesh.index_count as u32,
                                    0,
                                    0..uv_mesh.instances.len() as u32,
                                );
                            } else {
                                pass.draw(
                                    0..uv_mesh.index_count as u32,
                                    0..uv_mesh.index_count as u32,
                                );
                            }
                        }
                    }
                }
            }
            encoder.pop_debug_group();
        }
        encoder.pop_debug_group();

        // forward pass
        encoder.push_debug_group("forward rendering pass");
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.output.view, // no we write to the screen
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.forward_depth, // just a normal depth buffer
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: false,
                    }),
                    stencil_ops: None,
                }),
            });
            if !self.color_meshes.is_empty() {
                pass.set_pipeline(&self.passes.color_forward_pass.pipeline);
                pass.set_bind_group(0, &self.passes.color_forward_pass.bind_group, &[]); // the globals and simple lights

                for color_mesh in self.color_meshes.iter().flatten() {
                    if !color_mesh.is_empty() {
                        if let Some(index_buf) = &color_mesh.index_buf {
                            pass.set_index_buffer(index_buf.slice(..), wgpu::IndexFormat::Uint16);
                        }
                        pass.set_vertex_buffer(0, color_mesh.vertex_buf.slice(..));
                        pass.set_vertex_buffer(1, color_mesh.instance_buffer.slice(..));

                        pass.draw(
                            0..color_mesh.vertex_count as u32,
                            0..color_mesh.instances.len() as u32,
                        );
                        pass.draw_indexed(
                            0..color_mesh.index_count as u32,
                            0,
                            0..color_mesh.instances.len() as u32,
                        );
                    }
                }
            }
            if !self.uv_meshes.is_empty() {
                pass.set_pipeline(&self.passes.uv_forward_pass.pipeline);
                pass.set_bind_group(0, &self.passes.uv_forward_pass.bind_group, &[]); // the globals and simple lights
                for uv_mesh in self.uv_meshes.iter().flatten() {
                    if !uv_mesh.is_empty() {
                        pass.set_bind_group(1, &uv_mesh.diffuse_bind_group, &[]);
                        pass.set_vertex_buffer(0, uv_mesh.vertex_buffer.slice(..));
                        pass.set_vertex_buffer(1, uv_mesh.instance_buffer.slice(..));
                        if let Some(index_buffer) = &uv_mesh.index_buffer {
                            pass.set_index_buffer(
                                index_buffer.slice(..),
                                wgpu::IndexFormat::Uint16,
                            );
                            pass.draw_indexed(
                                0..uv_mesh.index_count as u32,
                                0,
                                0..uv_mesh.instances.len() as u32,
                            );
                        } else {
                            pass.draw(0..uv_mesh.index_count as u32, 0..uv_mesh.index_count as u32);
                        }
                    }
                }
            }
            if !self.models.is_empty() {
                pass.set_pipeline(&self.passes.model_forward_pass.pipeline);
                pass.set_bind_group(0, &self.passes.model_forward_pass.bind_group, &[]); // the globals
                for model in &self.models {
                    if !model.is_empty() {
                        for mesh in &model.meshes {
                            pass.set_bind_group(1, &model.materials[mesh.material].bind_group, &[]);
                            pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                            pass.set_vertex_buffer(1, model.instance_buffer.slice(..));
                            pass.set_index_buffer(
                                mesh.index_buffer.slice(..),
                                wgpu::IndexFormat::Uint32,
                            );
                            pass.draw_indexed(
                                0..mesh.indices.len() as u32,
                                0,
                                0..model.instances.len() as u32,
                            );
                        }
                    }
                }
            }
        }
        encoder.pop_debug_group();

        encoder.push_debug_group("text rendering");
        {
            for paragraph in &self.paragraphs {
                self.glyph_brush.queue(Section {
                    screen_position: (paragraph.position.x, paragraph.position.y),
                    bounds: (self.sc_desc.width as f32, self.sc_desc.height as f32),
                    layout: Layout::Wrap {
                        line_breaker: BuiltInLineBreaker::AnyCharLineBreaker,
                        h_align: paragraph.horizontal_alignment,
                        v_align: paragraph.vertical_alignment,
                    },
                    text: paragraph
                        .sections
                        .iter()
                        .map(|section| {
                            Text::new(&section.text)
                                .with_color(section.color)
                                .with_scale(section.scale)
                                .with_font_id(section.font)
                        })
                        .collect(),
                });
            }

            // Draw the text!
            self.glyph_brush
                .draw_queued(
                    &self.device,
                    &mut self.staging_belt,
                    &mut encoder,
                    &frame.output.view,
                    self.sc_desc.width,
                    self.sc_desc.height,
                )
                .expect("Draw queued");

            // Submit the work!
            self.staging_belt.finish();
        }
        encoder.pop_debug_group();
        self.queue.submit(std::iter::once(encoder.finish()));
        // Recall unused staging buffers
        use futures::task::SpawnExt;

        self.local_spawner
            .spawn(self.staging_belt.recall())
            .expect("Recall staging belt");

        self.local_pool.run_until_stalled();
    }
}
