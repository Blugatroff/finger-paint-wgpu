#![warn(rust_2018_idioms)]
//! This crate allows for simple rendering using WGPU.
//! You can load and draw .obj Models and Meshes, and add lights to the world

use crate::api::lights::{RealLight, SimpleLight};
use color_mesh::ColorMesh;
use constants::*;
use lines::Lines;
use model::Model;
use new::New;
use render_passes::shader_compiler::ShaderCompiler;
use render_passes::Passes;
use std::path::PathBuf;
use uniforms::GlobalUniforms;
use update::Update;
use uv_mesh::UvModel;
use wgpu::{Buffer, SwapChain, SwapChainDescriptor, Texture};
use wgpu_glyph::ab_glyph::InvalidFont;
use wgpu_glyph::{ab_glyph, FontId};

pub use api::lights::LightAttenuation;
pub use api::lights::RealLightApi;
pub use api::lights::RealLightPublic;
pub use api::meshes::MeshApi;
pub use camera::Camera;
pub use camera::ViewMatrixMode;
pub use cgmath;
pub use color_mesh::ColorMeshInstance;
pub use color_mesh::ColorVertex;
pub use color_mesh::Lighting;
pub use lines::Line;
pub use lines::LineVertex;
pub use model::ModelVertex;
pub use render::Render;
pub use resize::Resize;
pub use text::Paragraph;
pub use text::TextSection;
pub use transform::Transform;
pub use uv_mesh::UvVertex;
pub use wgpu_glyph::{HorizontalAlign, VerticalAlign};
pub use api::meshes::ColorMeshHandle;
pub use api::meshes::UvMeshHandle;
pub use api::meshes::ModelHandle;

mod api;
mod camera;
mod color_mesh;
mod constants;
mod instance;
mod lines;
mod model;
mod new;
mod render;
mod render_passes;
mod resize;
mod text;
mod texture;
mod transform;
mod uniforms;
mod update;
mod uv_mesh;

/// This contains the State of the Renderer
pub struct WgpuRenderer {
    _instance: wgpu::Instance,
    _adapter: wgpu::Adapter,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: SwapChainDescriptor,
    swap_chain: SwapChain,
    global_uniforms: GlobalUniforms,
    real_lights: Vec<RealLight>,
    lights_are_dirty: bool,

    color_meshes: Vec<Option<ColorMesh>>,
    uv_meshes: Vec<Option<UvModel>>,
    models: Vec<Option<Model>>,
    lines: Lines,

    passes: Passes,

    shadow_texture: Texture,
    forward_depth: wgpu::TextureView,
    camera: Camera,
    glyph_brush: wgpu_glyph::GlyphBrush<()>,
    staging_belt: wgpu::util::StagingBelt,
    local_pool: futures::executor::LocalPool,
    local_spawner: futures::executor::LocalSpawner,
    paragraphs: Vec<Paragraph>,
    simple_lights: Vec<SimpleLight>,
    simple_lights_storage_buffer: Buffer,
    real_lights_storage_buffer: Buffer,
    shadow_view: wgpu::TextureView,
    shadow_sampler: wgpu::Sampler,
    max_real_lights: u32,
    shadow_resolution: [u32; 2],
    #[allow(dead_code)]
    shaders: ShaderCompiler,
}

impl WgpuRenderer {
    /// create the renderer by supplying a Window
    pub fn new(window: &winit::window::Window, hot_reload_shader: Option<PathBuf>) -> Self {
        New::new(window, hot_reload_shader)
    }
    pub fn render(&mut self) {
        Render::render(self);
    }
    pub fn simple_lights(&mut self) -> &mut Vec<SimpleLight> {
        &mut self.simple_lights
    }
    /// get access to the camera
    pub fn camera(&mut self) -> &mut Camera {
        &mut self.camera
    }
    /// update the renderer, this should be called once every frame
    pub fn update(&mut self) {
        Update::update(self);
    }
    /// get the aspect ratio of the window
    pub fn aspect(&self) -> f32 {
        self.sc_desc.width as f32 / self.sc_desc.height as f32
    }
    /// get mutable access to all the paragraphs displayed on screen
    pub fn paragraphs(&mut self) -> &mut Vec<Paragraph> {
        &mut self.paragraphs
    }
    /// load a font from a static slice of bytes
    /// will fail if the bytes are an invalid font
    pub fn add_font(&mut self, data: &'static [u8]) -> Result<FontId, InvalidFont> {
        let font = ab_glyph::FontArc::try_from_slice(data)?;
        Ok(self.glyph_brush.add_font(font))
    }
    /// This allows turning on/off all lighting calculations in the shaders.
    /// The default is on
    pub fn enable_lighting(&mut self, enabled: bool) {
        self.global_uniforms.lighting_enabled = if enabled { 1 } else { 0 };
    }
}
