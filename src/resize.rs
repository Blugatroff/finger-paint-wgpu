use crate::WgpuRenderer;
use crate::constants::{generate_matrix, DEPTH_FORMAT};

pub trait Resize {
    fn resize(&mut self, size: (i32, i32));
}

impl Resize for WgpuRenderer {
    fn resize(&mut self, size: (i32, i32)) {
        self.sc_desc.width = size.0.max(1) as u32;
        self.sc_desc.height = size.1.max(1) as u32;

        self.camera.set_aspect_ratio(self.aspect());

        // update view-projection matrix
        let mx_total: [[f32; 4]; 4] = generate_matrix(self.sc_desc.width as f32 / self.sc_desc.height as f32).into();
        self.queue.write_buffer(
            &self.passes.color_forward_pass.uniform_buf,
            0,
            bytemuck::cast_slice(&mx_total),
        );

        let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.sc_desc.width,
                height: self.sc_desc.height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            label: None,
        });
        self.forward_depth = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }
}