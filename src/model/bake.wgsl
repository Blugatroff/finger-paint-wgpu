[[location(0)]] var<in> in_position: vec3<f32>;
[[location(1)]] var<in> in_normal: vec3<f32>;
[[location(2)]] var<in> in_color: vec4<f32>;

[[location(5)]]
var<in> model_matrix_0: vec4<f32>;
[[location(6)]]
var<in> model_matrix_1: vec4<f32>;
[[location(7)]]
var<in> model_matrix_2: vec4<f32>;
[[location(8)]]
var<in> model_matrix_3: vec4<f32>;

[[builtin(position)]]
var<out> out_position: vec4<f32>;

[[block]]
struct Globals {
    view_proj: mat4x4<f32>;
    camera_pos: vec4<f32>;
    num_lights: vec4<u32>;
    ambient_color: vec4<f32>;
    enable_lighting: u32;
};

[[group(0), binding(0)]]
var<uniform> u_globals: Globals;

[[stage(vertex)]]
fn vs_bake() {
    const model_matrix: mat4x4<f32> = mat4x4<f32>(model_matrix_0, model_matrix_1, model_matrix_2, model_matrix_3);
    out_position = u_globals.view_proj * model_matrix * vec4<f32>(in_position, 1.0);
}