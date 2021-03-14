use crate::color_mesh::ColorMesh;
use crate::model::Model;
use crate::uv_mesh::UvModel;
use crate::{ColorMeshInstance, ColorVertex, Line, Transform, UvVertex, WgpuRenderer};
use std::path::{Path, PathBuf};

pub struct ColorMeshHandle {
    index: usize,
}
impl ColorMeshHandle {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}
pub struct UvMeshHandle {
    index: usize,
}
impl UvMeshHandle {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}
pub struct ModelHandle {
    index: usize,
}
impl ModelHandle {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}

#[rustfmt::skip]
pub trait MeshApi {
    fn load_color_mesh(&mut self, vertices: Vec<ColorVertex>, indices: Option<Vec<u16>>) -> ColorMeshHandle;
    fn remove_color_mesh(&mut self, mesh: &ColorMeshHandle);
    fn color_mesh_instances(&mut self, mesh: &ColorMeshHandle) -> &mut Vec<ColorMeshInstance>;
    fn update_color_mesh(&mut self, mesh: &ColorMeshHandle);

    fn load_uv_mesh<P: AsRef<Path>>(&mut self, vertices: Vec<UvVertex>, indices: Option<Vec<u16>>, texture: P) -> UvMeshHandle;
    fn remove_uv_mesh(&mut self, mesh: &UvMeshHandle);
    fn uv_mesh_instances(&mut self, mesh: &UvMeshHandle) -> &mut Vec<Transform>;
    fn update_uv_mesh(&mut self, mesh: &UvMeshHandle);
    fn write_raw_texture_to_uv_mesh(&mut self, mesh: &UvMeshHandle, size: (u32, u32), data: &[u8]);

    fn load_model<P: AsRef<Path>>(&mut self, path: P) -> ModelHandle where PathBuf: std::convert::From<P>;
    fn remove_model(&mut self, model: ModelHandle);
    fn model_instances(&mut self, model: &ModelHandle) -> Option<&mut Vec<Transform>>;
    fn update_model(&mut self, model: &ModelHandle);

    fn lines(&mut self) -> &mut Vec<Line>;
}
impl MeshApi for WgpuRenderer {
    /// load a mesh with colored vertices
    fn load_color_mesh(
        &mut self,
        vertices: Vec<ColorVertex>,
        indices: Option<Vec<u16>>,
    ) -> ColorMeshHandle {
        ColorMeshHandle::new(put_in_first_slot(
            &mut self.color_meshes,
            ColorMesh::from_vertices_and_indices(&self.device, vertices, indices),
        ))
    }
    fn remove_color_mesh(&mut self, mesh: &ColorMeshHandle) {
        self.color_meshes[mesh.index] = None;
    }
    fn remove_uv_mesh(&mut self, mesh: &UvMeshHandle) {
        self.uv_meshes[mesh.index] = None;
    }
    /// get all the instance of a ColorMesh
    fn color_mesh_instances(&mut self, mesh: &ColorMeshHandle) -> &mut Vec<ColorMeshInstance> {
        if let Some(mesh) = &mut self.color_meshes[mesh.index] {
            &mut mesh.instances
        } else {
            panic!("ColorMesh does not exist")
        }
    }
    /// load a model from a obj
    /// this is not working well, only simple models work properly
    fn load_model<P: AsRef<Path>>(&mut self, path: P) -> ModelHandle
    where
        PathBuf: std::convert::From<P>,
    {
        let path: PathBuf = path.into();
        let ext = dbg!(path.extension().unwrap());

        if ext == "glb" {
            ModelHandle::new(put_in_first_slot(
                &mut self.models,
                Model::load_gltf(&self.device, &self.queue, path),
            ))
        } else if ext == "obj" {
            ModelHandle::new(put_in_first_slot(
                &mut self.models,
                Model::load(&self.device, &self.queue, path),
            ))
        } else {
            panic!("format not supported");
        }
    }
    fn model_instances(&mut self, model: &ModelHandle) -> Option<&mut Vec<Transform>> {
        self.models[model.index]
            .as_mut()
            .map(|model| &mut model.instances)
    }
    /// update the instances of a Model
    /// this has to be called in order for any changes to take effect
    fn update_model(&mut self, model: &ModelHandle) {
        if let Some(model) = &mut self.models[model.index] {
            model.update(&self.device);
        }
    }
    /// update the instances of a ColorMesh
    /// this has to be called in order for any changes to take effect
    fn update_color_mesh(&mut self, mesh: &ColorMeshHandle) {
        if let Some(mesh) = &mut self.color_meshes[mesh.index] {
            mesh.update(&self.device)
        }
    }
    /// load a UvMesh
    /// UvMesh like ColorMesh but using a texture and uv coordinates instead of colors in the vertices
    fn load_uv_mesh<P: AsRef<Path>>(
        &mut self,
        vertices: Vec<UvVertex>,
        indices: Option<Vec<u16>>,
        texture: P,
    ) -> UvMeshHandle {
        UvMeshHandle::new(put_in_first_slot(
            &mut self.uv_meshes,
            UvModel::new(vertices, indices, &self.device, &self.queue, texture),
        ))
    }
    /// get all the instance of a UvMesh
    fn uv_mesh_instances(&mut self, mesh: &UvMeshHandle) -> &mut Vec<Transform> {
        if let Some(mesh) = &mut self.uv_meshes[mesh.index] {
            &mut mesh.instances
        } else {
            panic!("UvMesh does not exist")
        }
    }
    /// update the instances of a UVMesh
    /// this has to be called in order for any changes to take effect
    fn update_uv_mesh(&mut self, mesh: &UvMeshHandle) {
        if let Some(uv_mesh) = &mut self.uv_meshes[mesh.index] {
            uv_mesh.update(&self.device);
        }
    }
    /// Write a slice of bytes to the texture of a uv_mesh.
    /// When the size of the new texture is greater than the old one a new texture will have to be created. This is a bit slower.
    fn write_raw_texture_to_uv_mesh(&mut self, mesh: &UvMeshHandle, size: (u32, u32), data: &[u8]) {
        if let Some(mesh) = self.uv_meshes[mesh.index].as_mut() {
            if mesh
                .diffuse_texture
                .write_raw(&self.device, &self.queue, size, data)
            {
                mesh.update_texture(&self.device);
            }
        }
    }
    fn remove_model(&mut self, model: ModelHandle) {
        self.models[model.index] = None;
    }
    /// get access to all lines
    fn lines(&mut self) -> &mut Vec<Line> {
        self.lines.lines()
    }
}

fn put_in_first_slot<T>(vec: &mut Vec<Option<T>>, object: T) -> usize {
    for (i, o) in vec.iter_mut().enumerate() {
        if o.is_none() {
            *o = Some(object);
            return i;
        }
    }
    vec.push(Some(object));
    vec.len() - 1
}
