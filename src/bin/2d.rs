use cgmath::{Matrix3, Point3, Vector3};
use finger_paint_wgpu::cgmath::{SquareMatrix, Vector2};
use finger_paint_wgpu::{
    Camera, HorizontalAlign, Paragraph, Resize, TextSection, Transform, UvVertex, VerticalAlign,
    ViewMatrixMode, WgpuRenderer,
};
use simple_winit::input::Input;
use simple_winit::InputEvent;
use std::time::Duration;
use winit::event::VirtualKeyCode;

#[derive(Clone)]
pub struct Canvas {
    width: u32,
    height: u32,
    data: Vec<u8>,
}
impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        let mut data = Vec::new();
        data.resize((width * height * 4) as usize, 0);
        Self {
            width,
            height,
            data,
        }
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.data.resize((self.width * self.height * 4) as usize, 0);
    }
    pub fn write_pixel(&mut self, x: u32, y: u32, c: [u8; 4]) {
        if self.width <= x {
            dbg!(x);
            dbg!(self.width);
            panic!("pixel out of bounds of canvas");
        }
        if self.height <= y {
            dbg!(y);
            dbg!(self.height);
            panic!("pixel out of bounds of canvas");
        }
        let i = ((y * self.width + x) * 4) as usize;
        self.data[i] = c[0];
        self.data[i + 1] = c[1];
        self.data[i + 2] = c[2];
        self.data[i + 3] = c[3];
    }
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    pub fn raw_data(&self) -> &[u8] {
        &self.data
    }
    pub fn clear(&mut self, c: [u8; 4]) {
        for p in 0..self.width * self.height {
            let i = (p * 4) as usize;
            self.data[i] = c[0];
            self.data[i + 1] = c[1];
            self.data[i + 2] = c[2];
            self.data[i + 3] = c[3];
        }
    }
}

pub struct State {
    renderer: WgpuRenderer,
    time: f32,
    average_frame_time: f32,
    canvas: Canvas,
    grass_block: usize,
}

impl State {
    pub fn new(window: &simple_winit::winit::window::Window) -> Self {
        Self {
            renderer: WgpuRenderer::new(window, Some(std::path::PathBuf::from("./"))),
            time: 0.0,
            average_frame_time: 1.0,
            grass_block: 0,
            canvas: Canvas::new(20, 20),
        }
    }
}

impl simple_winit::WindowLoop for State {
    fn init(&mut self) {
        let plane = self.renderer.load_uv_mesh(
            vec![
                UvVertex::new(
                    Vector3::new(0.0, 0.0, 0.0),
                    Vector3::new(0.0, 1.0, 0.0),
                    Vector2::new(0.0, 0.0),
                ),
                UvVertex::new(
                    Vector3::new(1.0, 0.0, 0.0),
                    Vector3::new(0.0, 1.0, 0.0),
                    Vector2::new(0.0, 1.0),
                ),
                UvVertex::new(
                    Vector3::new(0.0, 0.0, 1.0),
                    Vector3::new(0.0, 1.0, 0.0),
                    Vector2::new(1.0, 0.0),
                ),
                UvVertex::new(
                    Vector3::new(1.0, 0.0, 1.0),
                    Vector3::new(0.0, 1.0, 0.0),
                    Vector2::new(1.0, 1.0),
                ),
            ],
            Some(vec![2, 1, 0, 1, 2, 3]),
            "",
        );
        self.renderer.uv_mesh_instances(plane).push(Transform {
            position: Vector3::new(0.0, 1.0, 2.0),
            rotation: Matrix3::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        });
        self.renderer.update_uv_mesh(plane);

        self.renderer.paragraphs().push(Paragraph {
            vertical_alignment: VerticalAlign::Top,
            horizontal_alignment: HorizontalAlign::Left,
            position: Vector2::new(0.0, 0.0),
            sections: vec![TextSection {
                text: "".into(),
                color: [1.0, 1.0, 1.0, 1.0],
                scale: 25.0,
                font: Default::default(),
            }],
        });
        self.renderer.enable_lighting(false);
        *self.renderer.camera() = Camera::new(
            Point3::new(0.0, 5.0, 0.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            self.renderer.aspect(),
            ViewMatrixMode::Orthographic {
                left: 2.0,
                right: 3.0,
                bottom: 0.0,
                top: 1.0,
                near: 0.1,
                far: 50.0,
            },
        );
    }
    fn update(&mut self, input: &mut Input, dt: Duration) {
        let dt = dt.as_secs_f32();
        let weight = 100;
        self.average_frame_time =
            (self.average_frame_time * (weight - 1) as f32 + dt) / weight as f32;
        self.time += dt;
        if let Some(size) = input.resized() {
            self.renderer.resize(size);
            //self.canvas.resize(size.0 as u32, size.1 as u32);
        }
        self.renderer.paragraphs()[0].sections[0].text = format!(
            "frametime: {}ms\nfps: {}",
            self.average_frame_time * 1000.0,
            1.0 / self.average_frame_time
        );
        if input.key_held(VirtualKeyCode::H) {
            self.canvas.clear([0, 0, 0, 0]);
        }

        let t = (self.time * 10.0) as u32;
        let x = t % self.canvas.size().0;
        let y = (t / self.canvas.size().0) % self.canvas.size().1;
        self.canvas.write_pixel(x, y, [255, 0, 0, 255]);

        self.renderer.write_raw_texture_to_uv_mesh(
            self.grass_block,
            self.canvas.size(),
            self.canvas.raw_data(),
        );

        self.renderer.update();
    }

    fn render(&mut self) {
        self.renderer.render();
    }
    fn on_close(&mut self) {}

    fn input_event(&mut self, _event: InputEvent) {}
}

fn main() {
    let (window, event_loop) = simple_winit::create("finger_paint_wgpu 2d");

    let state = State::new(&window);

    simple_winit::start(state, (window, event_loop));
}
