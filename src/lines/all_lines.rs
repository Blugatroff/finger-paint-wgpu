use crate::lines::Line;
use wgpu::{Buffer, BufferUsage, Device};
use wgpu::util::DeviceExt;

pub struct Lines {
    lines: Vec<Line>,
    changed: bool,
    pub vertex_buffer: Buffer,
    vertices_in_buffer: usize,
}

impl Lines {
    pub fn new(device: &Device) -> Self {
        let lines = vec![];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&lines),
            usage: BufferUsage::VERTEX,
        });
        Self {
            lines,
            changed: true,
            vertex_buffer,
            vertices_in_buffer: 0
        }
    }
    pub fn lines(&mut self) -> &mut Vec<Line> {
        self.changed = true;
        &mut self.lines
    }
    pub fn update(&mut self, device: &Device) {
        if self.changed {
            self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.lines),
                usage: BufferUsage::VERTEX,
            });
            self.vertices_in_buffer = self.lines.len() * 2;
        }
    }
    pub fn number_of_vertices(&self) -> usize {
        self.vertices_in_buffer
    }
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}
