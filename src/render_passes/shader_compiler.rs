#[cfg(feature = "hot_reload_shader")]
use notify::DebouncedEvent;
#[cfg(feature = "hot_reload_shader")]
use shaderc::ShaderKind;
use std::collections::HashMap;
use std::path::PathBuf;
#[cfg(feature = "hot_reload_shader")]
use std::sync::mpsc::Receiver;
use wgpu::{Device, ShaderFlags, ShaderModule, ShaderSource};

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum ShaderType {
    Glsl,
    Wgsl,
    Spirv,
}

pub struct ShaderPackage {
    pub module: ShaderModule,
    #[cfg(feature = "hot_reload_shader")]
    pub shader_type: Option<ShaderType>,
    #[cfg(feature = "hot_reload_shader")]
    pub shader_kind: Option<ShaderKind>,
    pub path: Option<PathBuf>,
}


#[cfg(feature = "hot_reload_shader")]
pub struct ShaderCompiler {
    glsl_compiler: shaderc::Compiler,
    shaders: HashMap<String, ShaderPackage>,
    shader_watchers: HashMap<String, Receiver<notify::DebouncedEvent>>,
    flags: ShaderFlags,
}

#[cfg(not(feature = "hot_reload_shader"))]
pub struct ShaderCompiler {
    shaders: HashMap<String, ShaderPackage>,
    flags: ShaderFlags,
}

impl ShaderCompiler {
    pub fn new(flags: ShaderFlags) -> Self {
        #[cfg(feature = "hot_reload_shader")]
        return Self {
            glsl_compiler: shaderc::Compiler::new().unwrap(),
            shaders: HashMap::new(),
            shader_watchers: HashMap::new(),
            flags,
        };
        #[cfg(not(feature = "hot_reload_shader"))]
        return Self {
            shaders: HashMap::new(),
            flags,
        };
    }
    pub fn get_shader(&self, name: &str) -> &ShaderModule {
        &self.shaders.get(name).unwrap().module
    }
    #[cfg(feature = "hot_reload_shader")]
    pub fn get_package(&mut self, name: &str) -> &ShaderPackage {
        self.shaders.get(name).unwrap()
    }
    #[cfg(feature = "hot_reload_shader")]
    pub fn insert_watcher(&mut self, name: &str, watcher: Receiver<DebouncedEvent>) {
        self.shader_watchers.insert(name.to_owned(), watcher);
    }
    #[cfg(feature = "hot_reload_shader")]
    pub fn get_packages_mut(&mut self) -> &mut HashMap<String, ShaderPackage> {
        &mut self.shaders
    }
    #[cfg(feature = "hot_reload_shader")]
    pub fn get_watchers(&mut self) -> &mut HashMap<String, Receiver<DebouncedEvent>> {
        &mut self.shader_watchers
    }
    pub fn load_spirv(&mut self, device: &Device, src: &[u8], name: &str) {
        self.shaders.insert(
            name.to_string(),
            ShaderPackage {
                module: device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                    label: Some("vertex_shader"),
                    source: wgpu::util::make_spirv(src),
                    flags: self.flags,
                }),
                #[cfg(feature = "hot_reload_shader")]
                shader_type: None,
                #[cfg(feature = "hot_reload_shader")]
                shader_kind: None,
                path: None,
            },
        );
    }
    #[cfg(feature = "hot_reload_shader")]
    pub fn load_glsl(
        &mut self,
        device: &Device,
        src: &str,
        name: &str,
        file_name: &str,
        shader_kind: shaderc::ShaderKind,
    ) {
        let spirv = self
            .glsl_compiler
            .compile_into_spirv(&src, shader_kind, file_name, "main", None)
            .unwrap()
            .as_binary_u8()
            .to_vec();
        self.load_spirv(device, &spirv, name);
    }
    pub fn load_wgsl(&mut self, device: &Device, src: &str, name: &str) {
        self.shaders.insert(
            name.to_string(),
            ShaderPackage {
                module: device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                    label: Some(name),
                    source: ShaderSource::Wgsl(std::borrow::Cow::Borrowed(src)),
                    flags: self.flags,
                }),
                #[cfg(feature = "hot_reload_shader")]
                shader_type: None,
                #[cfg(feature = "hot_reload_shader")]
                shader_kind: None,
                path: None,
            },
        );
    }
    #[cfg(feature = "hot_reload_shader")]
    pub fn read_from_file<P>(
        &mut self,
        device: &Device,
        path: P,
        shader_type: ShaderType,
        shader_kind: shaderc::ShaderKind,
        name: &str,
    ) where
        P: Into<PathBuf> + Clone,
    {
        {
            let path: PathBuf = path.clone().into();
            match shader_type {
                ShaderType::Glsl => {
                    self.load_glsl(
                        device,
                        &std::fs::read_to_string(&path).unwrap(),
                        name,
                        path.file_name().unwrap().to_str().unwrap(),
                        shader_kind,
                    );
                }
                ShaderType::Wgsl => {
                    self.load_wgsl(device, &std::fs::read_to_string(path).unwrap(), name);
                }
                ShaderType::Spirv => {
                    self.load_spirv(device, &std::fs::read(path).unwrap(), name);
                }
            }
        }
        let this_shader = self.shaders.get_mut(name).unwrap();
        this_shader.shader_type = Some(shader_type);
        this_shader.shader_kind = Some(shader_kind);
        this_shader.path = Some(path.into());
    }
}
