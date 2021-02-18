use cgmath::{Deg, InnerSpace, Matrix3, Point3, Rad, SquareMatrix, Vector2, Vector3};
use finger_paint_wgpu::{Camera, ColorMeshInstance, Lighting, Paragraph, RealLightApi, RealLightPublic, Render, Resize, TextSection, Transform, ColorVertex, ViewMatrixMode, WgpuRenderer, UvVertex, SimpleLight, SimpleLightApi, SimpleLightKind};
use simple_winit::input::{Input, VirtualKeyCode};
use std::f32::consts::PI;
use std::time::Duration;
use wgpu_glyph::{HorizontalAlign, VerticalAlign};

pub struct State {
    renderer: WgpuRenderer,
    time: f32,
    cube_model: usize,
    cube: usize,
    cube_2: usize,
    player_cube: usize,
    camera_controller: CameraController,
    average_frame_time: f32,
}

impl State {
    pub fn new(window: &simple_winit::winit::window::Window) -> Self {
        Self {
            renderer: WgpuRenderer::new(window),
            time: 0.0,
            cube_model: 0,
            cube: 0,
            cube_2: 0,
            player_cube: 0,
            camera_controller: CameraController {
                speed: 5.0,
                mouse_sens: 0.005,
                turn_speed: 0.5,
                heading: 0.0,
                pitch: 0.0,
            },
            average_frame_time: 1.0,
        }
    }
}

impl simple_winit::WindowLoop for State {
    fn init(&mut self) {
        let (cube_vertex_data, cube_index_data) = create_cube();
        self.cube_model = self.renderer.add_color_mesh(cube_vertex_data, cube_index_data);
        let (plane_vertex_data, plane_index_data) = create_plane(20.0);
        let plane_model = self.renderer.add_color_mesh(plane_vertex_data, plane_index_data);

        build_box(self, plane_model);

        self.cube_2 = self.renderer.add_color_mesh_instance(
            self.cube_model,
            ColorMeshInstance {
                transform: Transform {
                    position: Point3::new(0.9, 0.5, 2.0),
                    rotation: Matrix3::from_axis_angle(Vector3::unit_x(), Deg(0.01)),
                    scale: Vector3::new(1.0, 0.5, 1.0),
                },
                lighting: Lighting {
                    specular_strength: 1.0,
                    specular_spread: 32.0,
                    diffuse_strength: 1.0,
                },
            },
        );

        self.cube = self.renderer.add_color_mesh_instance(
            self.cube_model,
            ColorMeshInstance {
                transform: Transform {
                    position: Point3::new(0.0, 0.0, 0.0),
                    rotation: Matrix3::identity(),
                    scale: Vector3::new(1.0, 1.0, 1.0),
                },
                lighting: Lighting {
                    specular_strength: 2.0,
                    specular_spread: 64.0,
                    diffuse_strength: 1.0,
                },
            },
        );

        self.player_cube = self.renderer.add_color_mesh_instance(
            self.cube_model,
            ColorMeshInstance {
                transform: Transform {
                    position: Point3::new(0.0, 0.0, 0.0),
                    rotation: Matrix3::identity(),
                    scale: Vector3::new(0.1, 0.1, 0.1),
                },
                lighting: Lighting {
                    specular_strength: 2.0,
                    specular_spread: 64.0,
                    diffuse_strength: 1.0,
                },
            },
        );

        self.renderer.update_color_mesh(plane_model);
        self.renderer.update_color_mesh(self.cube_model);

        self.renderer.add_color_mesh_instance(
            self.cube_model,
            ColorMeshInstance {
                transform: Transform {
                    position: Point3::new(0.0, 20.0, 0.0),
                    rotation: Matrix3::identity(),
                    scale: Vector3::new(0.1, 0.1, 0.1),
                },
                lighting: Lighting {
                    specular_strength: 2.0,
                    specular_spread: 64.0,
                    diffuse_strength: 1.0,
                },
            },
        );
        let uv_mesh = self.renderer.add_uv_mesh(vec![
            UvVertex::new(
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
                Vector2::new(0.0, 0.0)
            ),
            UvVertex::new(
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
                Vector2::new(0.0, 1.0)
            ),
            UvVertex::new(
                Vector3::new(0.0, 0.0, 1.0),
                Vector3::new(0.0, 1.0, 0.0),
                Vector2::new(1.0, 0.0)
            ),
            UvVertex::new(
                Vector3::new(1.0, 0.0, 1.0),
                Vector3::new(0.0, 1.0, 0.0),
                Vector2::new(1.0, 1.0)
            )
        ], vec![2, 1, 0, 1, 2, 3]);
        self.renderer.add_uv_mesh_instance(uv_mesh, Transform {
            position: Point3::new(0.0, 2.0, 2.0),
            rotation: Matrix3::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0) * 2.0
        });
        self.renderer.update_uv_mesh(uv_mesh);
        self.renderer.add_simple_light(SimpleLight {
            color: [1.0, 0.75, 0.0, 1.0],
            kind: SimpleLightKind::Directional([0.0, -1.0, 1.0]),
            constant: 1.00,
            linear: 0.01,
            quadratic: 0.03,
        });
        add_point_light(self, Point3::new(00.0, 20.0, 0.0));

        self.renderer.paragraphs().push(Paragraph {
            vertical_alignment: VerticalAlign::Top,
            horizontal_alignment: HorizontalAlign::Left,
            position: Vector2::new(0.0, 0.0),
            sections: vec![TextSection {
                text: format!("frametime: {}ms, {}fps", 0.0, 0.0),
                color: [0.0, 0.0, 0.0, 1.0],
                scale: 30.0,
                font: Default::default(),
            }],
        });
    }
    fn update(&mut self, input: &mut Input, dt: Duration) {
        let dt = dt.as_secs_f32();
        let weight = 100;
        self.average_frame_time =
            (self.average_frame_time * (weight - 1) as f32 + dt) / weight as f32;
        self.time += dt;
        if let Some(size) = input.resized() {
            self.renderer.resize(size);
        }
        self.renderer.paragraphs()[0].sections[0].text = format!(
            "frametime: {}ms\n fps: {}",
            self.average_frame_time * 1000.0,
            1.0 / self.average_frame_time
        );
        self.renderer
            .color_mesh_instance(self.cube_model, self.cube)
            .transform
            .position[1] = self.time.cos() * 2.0;

        let rotation: Matrix3<f32> = self
            .renderer
            .color_mesh_instance(self.cube_model, self.cube_2)
            .transform
            .rotation;
        self.renderer
            .color_mesh_instance(self.cube_model, self.cube_2)
            .transform
            .rotation = Matrix3::from_angle_x(Rad(dt)) * rotation;

        self.renderer
            .color_mesh_instance(self.cube_model, self.cube_2)
            .transform
            .position = Point3::new(
            self.time.cos() * 5.0,
            (self.time + self.time.sin()).cos() * 5.0 + 20.0,
            self.time.sin() * 5.0,
        );

        self.camera_controller
            .update(dt, self.renderer.camera(), input);

        self.renderer
            .color_mesh_instance(self.cube_model, self.player_cube)
            .transform
            .position = self.renderer.camera().get_position();

        self.renderer.update_color_mesh(self.cube_model);

        self.renderer.update();
    }

    fn render(&mut self) {
        self.renderer.render();
    }
    fn on_close(&mut self) {}
}

fn main() {
    let (window, event_loop) = simple_winit::create("bla2");

    let state = State::new(&window);

    simple_winit::start(state, (window, event_loop));
}

pub fn create_cube() -> (Vec<ColorVertex>, Vec<u16>) {
    let vertex_data = [
        // top (0, 0, 1)
        ColorVertex::new([-1.0, -1.0, 1.0], [0.0, 0.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
        ColorVertex::new([1.0, -1.0, 1.0], [0.0, 0.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
        ColorVertex::new([1.0, 1.0, 1.0], [0.0, 0.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
        ColorVertex::new([-1.0, 1.0, 1.0], [0.0, 0.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
        // bottom (0, 0, -1)
        ColorVertex::new([-1.0, 1.0, -1.0], [0.0, 0.0, -1.0], [1.0, 1.0, 1.0, 1.0]),
        ColorVertex::new([1.0, 1.0, -1.0], [0.0, 0.0, -1.0], [1.0, 1.0, 1.0, 1.0]),
        ColorVertex::new([1.0, -1.0, -1.0], [0.0, 0.0, -1.0], [1.0, 1.0, 1.0, 1.0]),
        ColorVertex::new([-1.0, -1.0, -1.0], [0.0, 0.0, -1.0], [1.0, 1.0, 1.0, 1.0]),
        // right (1, 0, 0)
        ColorVertex::new([1.0, -1.0, -1.0], [1.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
        ColorVertex::new([1.0, 1.0, -1.0], [1.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
        ColorVertex::new([1.0, 1.0, 1.0], [1.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
        ColorVertex::new([1.0, -1.0, 1.0], [1.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
        // left (-1, 0, 0)
        ColorVertex::new([-1.0, -1.0, 1.0], [-1.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
        ColorVertex::new([-1.0, 1.0, 1.0], [-1.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
        ColorVertex::new([-1.0, 1.0, -1.0], [-1.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
        ColorVertex::new([-1.0, -1.0, -1.0], [-1.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
        // front (0, 1, 0)
        ColorVertex::new([1.0, 1.0, -1.0], [0.0, 1.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
        ColorVertex::new([-1.0, 1.0, -1.0], [0.0, 1.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
        ColorVertex::new([-1.0, 1.0, 1.0], [0.0, 1.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
        ColorVertex::new([1.0, 1.0, 1.0], [0.0, 1.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
        // back (0, -1, 0)
        ColorVertex::new([1.0, -1.0, 1.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
        ColorVertex::new([-1.0, -1.0, 1.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
        ColorVertex::new([-1.0, -1.0, -1.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
        ColorVertex::new([1.0, -1.0, -1.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
    ];

    let index_data: &[u16] = &[
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];

    (vertex_data.to_vec(), index_data.to_vec())
}

pub fn create_plane(size: f32) -> (Vec<ColorVertex>, Vec<u16>) {
    let vertex_data = [
        ColorVertex::new([size, -size, 0.0], [0.0, 0.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
        ColorVertex::new([size, size, 0.0], [0.0, 0.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
        ColorVertex::new([-size, -size, 0.0], [0.0, 0.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
        ColorVertex::new([-size, size, 0.0], [0.0, 0.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
    ];

    let index_data: &[u16] = &[0, 1, 2, 2, 1, 3];

    (vertex_data.to_vec(), index_data.to_vec())
}

pub struct CameraController {
    speed: f32,
    mouse_sens: f32,
    turn_speed: f32,
    heading: f32,
    pitch: f32,
}
impl CameraController {
    pub fn update(&mut self, dt: f32, camera: &mut Camera, input: &mut Input) {
        camera.set_far(1000.0);
        camera.set_near(0.05);
        let mouse_diff = input.mouse_diff();
        if mouse_diff != (0.0, 0.0) {
            self.heading += mouse_diff.0 * -self.mouse_sens;
            self.pitch += mouse_diff.1 * -self.mouse_sens;
        }
        if input.key_held(VirtualKeyCode::Left) || input.key_held(VirtualKeyCode::J) {
            self.heading += dt * self.turn_speed;
        }
        if input.key_held(VirtualKeyCode::Right) || input.key_held(VirtualKeyCode::L) {
            self.heading -= dt * self.turn_speed;
        }
        if input.key_held(VirtualKeyCode::Down) || input.key_held(VirtualKeyCode::K) {
            self.pitch -= dt * self.turn_speed;
        }
        if input.key_held(VirtualKeyCode::Up) || input.key_held(VirtualKeyCode::I) {
            self.pitch += dt * self.turn_speed;
        }
        self.pitch = if self.pitch < -PI / 2.0 + 0.005 {
            -PI / 2.0 + 0.005
        } else if self.pitch > PI / 2.0 - 0.005 {
            PI / 2.0 - 0.005
        } else {
            self.pitch
        };
        camera.set_direction(Vector3::new(
            self.pitch.cos() * self.heading.sin(),
            self.pitch.sin(),
            self.pitch.cos() * self.heading.cos(),
        ));
        let direction: Vector3<f32> = camera.get_direction().into();
        let plane_direction = Vector3::new(direction.x, 0.0, direction.z).normalize();
        let right = Vector3::new(
            (self.heading - PI / 2.0).sin(),
            0.0,
            (self.heading - PI / 2.0).cos(),
        )
        .normalize();

        let speed = if input.key_held(VirtualKeyCode::LShift) {
            self.speed * 5.0
        } else {
            self.speed
        };
        if input.key_held(VirtualKeyCode::W) {
            camera.set_position(camera.get_position() + plane_direction * dt * speed);
        }
        if input.key_held(VirtualKeyCode::S) {
            camera.set_position(camera.get_position() - plane_direction * dt * speed);
        }
        if input.key_held(VirtualKeyCode::D) {
            camera.set_position(camera.get_position() + right * dt * speed);
        }
        if input.key_held(VirtualKeyCode::A) {
            camera.set_position(camera.get_position() - right * dt * speed);
        }
        if input.key_held(VirtualKeyCode::Space) {
            camera.set_position(camera.get_position() + camera.get_up() * dt * speed);
        }
        if input.key_held(VirtualKeyCode::LControl) {
            camera.set_position(camera.get_position() - camera.get_up() * dt * speed);
        }
    }
}

fn add_point_light(state: &mut State, pos: Point3<f32>) {
    let fov: f32 = PI / 2.0;
    state
        .renderer
        .add_real_light(RealLightPublic {
            camera: Camera::new(
                pos,
                pos + Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
                1.0,
                ViewMatrixMode::Perspective {
                    near: 0.1,
                    far: 100.0,
                    fov,
                },
            ),
            color: [1.0, 0.0, 0.0, 1.0],
            default: 0.0,
        })
        .unwrap();
    state
        .renderer
        .add_real_light(RealLightPublic {
            camera: Camera::new(
                pos,
                pos + Vector3::new(-1.0, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
                1.0,
                ViewMatrixMode::Perspective {
                    near: 0.1,
                    far: 100.0,
                    fov,
                },
            ),
            color: [0.0, 1.0, 1.0, 1.0],
            default: 0.0,
        })
        .unwrap();
    state
        .renderer
        .add_real_light(RealLightPublic {
            camera: Camera::new(
                pos,
                pos + Vector3::new(0.0, 1.0, 0.0),
                Vector3::new(0.0, 0.0, 1.0),
                1.0,
                ViewMatrixMode::Perspective {
                    near: 0.1,
                    far: 100.0,
                    fov,
                },
            ),
            color: [0.0, 1.0, 0.0, 1.0],
            default: 0.0,
        })
        .unwrap();
    state
        .renderer
        .add_real_light(RealLightPublic {
            camera: Camera::new(
                pos,
                pos + Vector3::new(0.0, -1.0, 0.0),
                Vector3::new(0.0, 0.0, 1.0),
                1.0,
                ViewMatrixMode::Perspective {
                    near: 0.1,
                    far: 100.0,
                    fov,
                },
            ),
            color: [1.0, 0.0, 1.0, 1.0],
            default: 0.0,
        })
        .unwrap();
    state
        .renderer
        .add_real_light(RealLightPublic {
            camera: Camera::new(
                pos,
                pos + Vector3::new(0.0, 0.0, -1.0),
                Vector3::new(0.0, 1.0, 0.0),
                1.0,
                ViewMatrixMode::Perspective {
                    near: 0.1,
                    far: 100.0,
                    fov,
                },
            ),
            color: [1.0, 1.0, 0.0, 1.0],
            default: 0.0,
        })
        .unwrap();
    state
        .renderer
        .add_real_light(RealLightPublic {
            camera: Camera::new(
                pos,
                pos + Vector3::new(0.0, 0.0, 1.0),
                Vector3::new(0.0, 1.0, 0.0),
                1.0,
                ViewMatrixMode::Perspective {
                    near: 0.1,
                    far: 100.0,
                    fov,
                },
            ),
            color: [0.0, 0.0, 1.0, 1.0],
            default: 0.0,
        })
        .unwrap();
}

fn build_box(state: &mut State, plane_model: usize) {
    let plane_scale: Vector3<f32> = Vector3::new(1.0, 1.0, 1.0);
    state.renderer.add_color_mesh_instance(
        plane_model,
        ColorMeshInstance {
            transform: Transform {
                position: Point3::new(-20.0, 20.0, 0.0),
                rotation: Matrix3::from_angle_y(Rad(PI / 2.0)),
                scale: plane_scale,
            },
            lighting: Lighting {
                specular_strength: 1.0,
                specular_spread: 32.0,
                diffuse_strength: 1.0,
            },
        },
    );
    state.renderer.add_color_mesh_instance(
        plane_model,
        ColorMeshInstance {
            transform: Transform {
                position: Point3::new(20.0, 20.0, 0.0),
                rotation: Matrix3::from_angle_y(Rad(-PI / 2.0)),
                scale: plane_scale,
            },
            lighting: Lighting {
                specular_strength: 1.0,
                specular_spread: 32.0,
                diffuse_strength: 1.0,
            },
        },
    );
    state.renderer.add_color_mesh_instance(
        plane_model,
        ColorMeshInstance {
            transform: Transform {
                position: Point3::new(0.0, 20.0, -20.0),
                rotation: Matrix3::identity(),
                scale: plane_scale,
            },
            lighting: Lighting {
                specular_strength: 1.0,
                specular_spread: 32.0,
                diffuse_strength: 1.0,
            },
        },
    );
    state.renderer.add_color_mesh_instance(
        plane_model,
        ColorMeshInstance {
            transform: Transform {
                position: Point3::new(0.0, 20.0, 20.0),
                rotation: Matrix3::from_angle_x(Rad(PI)),
                scale: plane_scale,
            },
            lighting: Lighting {
                specular_strength: 1.0,
                specular_spread: 32.0,
                diffuse_strength: 1.0,
            },
        },
    );
    state.renderer.add_color_mesh_instance(
        plane_model,
        ColorMeshInstance {
            transform: Transform {
                position: Point3::new(0.0, 0.0, 0.0),
                rotation: Matrix3::from_angle_x(Rad(-PI / 2.0)),
                scale: plane_scale,
            },
            lighting: Lighting {
                specular_strength: 1.0,
                specular_spread: 32.0,
                diffuse_strength: 1.0,
            },
        },
    );
    state.renderer.add_color_mesh_instance(
        plane_model,
        ColorMeshInstance {
            transform: Transform {
                position: Point3::new(0.0, 40.0, 0.0),
                rotation: Matrix3::from_angle_x(Rad(PI / 2.0)),
                scale: plane_scale,
            },
            lighting: Lighting {
                specular_strength: 1.0,
                specular_spread: 32.0,
                diffuse_strength: 1.0,
            },
        },
    );
}
