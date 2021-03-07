use crate::constants::OPENGL_TO_WGPU_MATRIX;
use cgmath::{Matrix4, Point3, Vector3, Rad};

#[derive(Copy, Clone)]
pub struct Camera {
    eye: Point3<f32>,
    target: Point3<f32>,
    up: Vector3<f32>,
    aspect: f32,
    mode: ViewMatrixMode,
}
#[derive(Copy, Clone)]
pub enum ViewMatrixMode {
    Perspective {
        near: f32,
        far: f32,
        fov: f32,
    },
    Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    },
}
impl Camera {
    pub fn new(
        eye: Point3<f32>,
        target: Point3<f32>,
        up: Vector3<f32>,
        aspect: f32,
        mode: ViewMatrixMode,
    ) -> Self {
        Self {
            eye,
            target,
            up,
            aspect,
            mode,
        }
    }
    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);

        let proj = match self.mode {
            ViewMatrixMode::Perspective { near, far, fov } => {
                cgmath::perspective(Rad(fov), self.aspect, near, far)
            }
            ViewMatrixMode::Orthographic {
                left,
                right,
                bottom,
                top,
                near,
                far,
            } => cgmath::ortho(left, right, bottom, top, near, far),
        };
        OPENGL_TO_WGPU_MATRIX * proj * view
    }
    pub fn set_far(&mut self, f: f32) {
        match &mut self.mode {
            ViewMatrixMode::Perspective { far, .. } => *far = f,
            ViewMatrixMode::Orthographic { far, .. } => *far = f,
        }
    }
    pub fn set_near(&mut self, n: f32) {
        match &mut self.mode {
            ViewMatrixMode::Perspective { near, .. } => *near = n,
            ViewMatrixMode::Orthographic { near, .. } => *near = n,
        }
    }
    pub fn set_fov(&mut self, f: f32) {
        if let ViewMatrixMode::Perspective { fov, .. } = &mut self.mode {
            *fov = f
        }
    }
    pub fn get_direction(&self) -> Vector3<f32> {
        self.target - self.eye
    }
    pub fn set_direction(&mut self, direction: Vector3<f32>) {
        self.target = self.eye + direction;
    }
    pub fn set_aspect_ratio(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
    pub fn set_up(&mut self, up: Vector3<f32>) {
        self.up = up;
    }
    pub fn set_matrix_mode(&mut self, mode: ViewMatrixMode) {
        self.mode = mode;
    }
    pub fn get_position(&self) -> Point3<f32> {
        self.eye
    }
    pub fn set_position(&mut self, position: Point3<f32>) {
        let d = self.target - self.eye;
        self.eye = position;
        self.target = self.eye + d;
    }
    pub fn get_mode(&self) -> &ViewMatrixMode {
        &self.mode
    }
    pub fn get_up(&self) -> Vector3<f32> {
        self.up
    }
}
