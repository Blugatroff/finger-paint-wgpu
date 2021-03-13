#![warn(rust_2018_idioms)]
//! This crate allows for simple rendering using WGPU.
//! You can load and draw .obj Models and Meshes, and add lights to the world

use std::path::{Path, PathBuf};

pub use cgmath;
use wgpu::{Buffer, SwapChain, SwapChainDescriptor, Texture};
use wgpu_glyph::ab_glyph::InvalidFont;
use wgpu_glyph::{ab_glyph, FontId};
pub use wgpu_glyph::{HorizontalAlign, VerticalAlign};

pub use api::lights::RealLightApi;
pub use api::lights::RealLightPublic;
pub use camera::Camera;
pub use camera::ViewMatrixMode;
use color_mesh::ColorMesh;
pub use color_mesh::ColorMeshInstance;
pub use color_mesh::ColorVertex;
pub use color_mesh::Lighting;
use constants::*;
pub use light::SimpleLight;
pub use light::SimpleLightKind;
use model::Model;
pub use model::ModelVertex;
use new::New;
pub use render::Render;
use render_passes::shader_compiler::ShaderCompiler;
pub use resize::Resize;
pub use text::Paragraph;
pub use text::TextSection;
pub use transform::Transform;
use uniforms::GlobalUniforms;
use update::Update;
use uv_mesh::UvModel;
pub use uv_mesh::UvVertex;
pub use lines::LineVertex;

pub use lines::Line;
use lines::Lines;
use render_passes::Passes;

mod api;
mod camera;
mod color_mesh;
mod constants;
mod instance;
mod light;
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
    real_lights: Vec<light::RealLight>,
    lights_are_dirty: bool,

    color_meshes: Vec<Option<ColorMesh>>,
    uv_meshes: Vec<Option<UvModel>>,
    models: Vec<Model>,
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
    /// load a mesh with colored vertices
    pub fn load_color_mesh(
        &mut self,
        vertices: Vec<ColorVertex>,
        indices: Option<Vec<u16>>,
    ) -> usize {
        self.color_meshes
            .push(Some(ColorMesh::from_vertices_and_indices(
                &self.device,
                vertices,
                indices,
            )));
        self.color_meshes.len() - 1
    }
    pub fn remove_color_mesh(&mut self, mesh: usize) {
        self.color_meshes[mesh] = None;
    }
    pub fn remove_uv_mesh(&mut self, mesh: usize) {
        self.uv_meshes[mesh] = None;
    }
    /// get all the instance of a ColorMesh
    pub fn color_mesh_instances(&mut self, mesh: usize) -> &mut Vec<ColorMeshInstance> {
        if let Some(mesh) = &mut self.color_meshes[mesh] {
            &mut mesh.instances
        } else {
            panic!("ColorMesh does not exist")
        }
    }
    /// load a model from a obj
    /// this is not working well, only simple models work properly
    pub fn load_model<P: AsRef<Path>>(&mut self, path: P) -> usize {
        self.models
            .push(Model::load(&self.device, &self.queue, path));
        self.models.len() - 1
    }
    pub fn model_instances(&mut self, model: usize) -> &mut Vec<Transform> {
        &mut self.models[model].instances
    }
    /// update the instances of a Model
    /// this has to be called in order for any changes to take effect
    pub fn update_model(&mut self, model: usize) {
        self.models[model].update(&self.device);
    }
    /// update the instances of a ColorMesh
    /// this has to be called in order for any changes to take effect
    pub fn update_color_mesh(&mut self, mesh: usize) {
        if let Some(mesh) = &mut self.color_meshes[mesh] {
            mesh.update(&self.device)
        }
    }
    /// load a UvMesh
    /// UvMesh like ColorMesh but using a texture and uv coordinates instead of colors in the vertices
    pub fn load_uv_mesh<P: AsRef<Path>>(
        &mut self,
        vertices: Vec<UvVertex>,
        indices: Option<Vec<u16>>,
        texture: P,
    ) -> usize {
        self.uv_meshes.push(Some(UvModel::new(
            vertices,
            indices,
            &self.device,
            &self.queue,
            texture,
        )));
        self.uv_meshes.len() - 1
    }
    /// get all the instance of a UvMesh
    pub fn uv_mesh_instances(&mut self, mesh: usize) -> &mut Vec<Transform> {
        if let Some(mesh) = &mut self.uv_meshes[mesh] {
            &mut mesh.instances
        } else {
            panic!("UvMesh does not exist")
        }
    }
    pub fn simple_lights(&mut self) -> &mut Vec<SimpleLight> {
        &mut self.simple_lights
    }
    /// update the instances of a UVMesh
    /// this has to be called in order for any changes to take effect
    pub fn update_uv_mesh(&mut self, mesh: usize) {
        if let Some(uv_mesh) = &mut self.uv_meshes[mesh] {
            uv_mesh.update(&self.device);
        }
    }
    /// Write a slice of bytes to the texture of a uv_mesh.
    /// When the size of the new texture is greater than the old one a new texture will have to be created. This is a bit slower.
    pub fn write_raw_texture_to_uv_mesh(&mut self, mesh: usize, size: (u32, u32), data: &[u8]) {
        if let Some(mesh) = self.uv_meshes[mesh].as_mut() {
            if mesh
                .diffuse_texture
                .write_raw(&self.device, &self.queue, size, data)
            {
                mesh.update_texture(&self.device);
            }
        }
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
    /// get access to all lines
    pub fn lines(&mut self) -> &mut Vec<Line> {
        self.lines.lines()
    }
}
