use crate::render_passes::shader_compiler::{ShaderPackage, ShaderType};
use crate::render_passes::{
    create_color_mesh_pipelines, create_model_render_passes, create_uv_mesh_pipelines,
};
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::time::Duration;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use crate::WgpuRenderer;
use crate::render_passes::line::create_line_pipelines;

pub trait ShaderHotReload {
    fn init_shader_watch(&mut self);
    fn update_pipelines(&mut self);
}

fn check_for_writes(rx: &mut Receiver<DebouncedEvent>) -> bool {
    let mut update = false;

    for event in rx.try_iter() {
        dbg!(&event);
        if let DebouncedEvent::NoticeWrite(_) = event {
            update = true;
            break;
        }
    }
    update
}
fn watch_file(path: &Path) -> std::sync::mpsc::Receiver<notify::DebouncedEvent> {
    let (tx, rx) = channel();
    let path = path.to_owned();
    std::thread::spawn(move || {
        let (inner_tx, inner_rx) = channel();
        let mut watcher: RecommendedWatcher =
            Watcher::new(inner_tx, Duration::from_secs(2)).unwrap();

        watcher.watch(path, RecursiveMode::Recursive).unwrap();

        for event in inner_rx.iter() {
            tx.send(event).unwrap();
        }
    });
    rx
}
impl ShaderHotReload for WgpuRenderer {
    fn init_shader_watch(&mut self) {
        let mut watchers: Vec<(String, PathBuf)> = Vec::new();
        for (name, ShaderPackage { path, .. }) in self.shaders.get_packages_mut().iter_mut() {
            if let Some(path) = path {
                watchers.push((name.clone(), path.clone()));
            }
        }
        for (name, path) in watchers {
            let rx = watch_file(&*path);
            self.shaders.insert_watcher(&name, rx);
        }
    }
    fn update_pipelines(&mut self) {
        let mut to_update = Vec::new();
        for (name, watcher) in self.shaders.get_watchers().iter_mut() {
            if check_for_writes(watcher) {
                to_update.push(name.clone());
            }
        }
        for name in to_update {
            dbg!(&name);
            let package = self.shaders.get_package(&name);
            let shader_kind = package.shader_kind;
            let shader_type = package.shader_type;
            let path = package.path.clone();

            if let (Some(shader_kind), Some(shader_type), Some(path)) =
                dbg!((shader_kind, shader_type, path))
            {
                println!("reloading shader: {}", &name);
                match shader_type {
                    ShaderType::Glsl => {
                        self.shaders.read_from_file(
                            &self.device,
                            path,
                            shader_type,
                            shader_kind,
                            &name,
                        );
                    }
                    ShaderType::Wgsl => {
                        self.shaders.load_wgsl(
                            &self.device,
                            &std::fs::read_to_string(&path).unwrap(),
                            &name,
                        );
                        let s = self.shaders.get_packages_mut().get_mut(&name).unwrap();

                        s.shader_kind = Some(shader_kind);
                        s.shader_type = Some(shader_type);
                        s.path = Some(path);
                    }
                    ShaderType::Spirv => {
                        self.shaders
                            .load_spirv(&self.device, &std::fs::read(path).unwrap(), &name)
                    }
                }

                match name.as_str() {
                    "color_mesh" => {
                        let (shadow_pass, forward_pass) = create_color_mesh_pipelines(
                            &self.device,
                            &self.global_uniforms,
                            &self.real_lights_storage_buffer,
                            &self.simple_lights_storage_buffer,
                            &self.shadow_view,
                            &self.shadow_sampler,
                            &self.sc_desc,
                            &self.shaders,
                        );
                        self.passes.color_shadow_pass = shadow_pass;
                        self.passes.color_forward_pass = forward_pass;
                    }
                    "uv_mesh" => {
                        let (shadow_pass, forward_pass) = create_uv_mesh_pipelines(
                            &self.device,
                            &self.global_uniforms,
                            &self.real_lights_storage_buffer,
                            &self.simple_lights_storage_buffer,
                            &self.shadow_view,
                            &self.shadow_sampler,
                            &self.sc_desc,
                            &self.shaders,
                        );
                        self.passes.uv_shadow_pass = shadow_pass;
                        self.passes.uv_forward_pass = forward_pass;
                    }
                    "model_bake" | "model_vs" | "model_fs" => {
                        let (shadow_pass, forward_pass) = create_model_render_passes(
                            &self.device,
                            &self.global_uniforms,
                            &self.real_lights_storage_buffer,
                            &self.simple_lights_storage_buffer,
                            &self.shadow_view,
                            &self.shadow_sampler,
                            &self.sc_desc,
                            &self.shaders,
                        );
                        self.passes.model_shadow_pass = shadow_pass;
                        self.passes.model_forward_pass = forward_pass;
                    }
                    "line_shader" => {
                        let (line_shadow_pass, line_forward_pass) = create_line_pipelines(&self.device, &self.global_uniforms, &self.sc_desc, &self.shaders);
                        self.passes.line_shadow_pass = line_shadow_pass;
                        self.passes.line_forward_pass = line_forward_pass;
                    }
                    _ => {}
                }
            }
        }
    }
}
