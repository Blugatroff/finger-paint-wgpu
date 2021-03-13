[[location(0)]] var<in> in_position: vec4<f32>;
[[location(1)]] var<in> in_color: vec4<f32>;

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
    out_position = u_globals.view_proj * in_position;
}

[[location(0)]]
var<out> out_color: vec4<f32>;

[[stage(vertex)]]
fn vs_main() {
    out_position = u_globals.view_proj * in_position;
    out_color = in_color;
}

[[location(0)]]
var<in> v_color: vec4<f32>;

[[location(0)]]
var<out> out_color_fs: vec4<f32>;

[[builtin(frag_coord)]]
var<in> frag_coord: vec4<f32>;

[[stage(fragment)]]
fn fs_main() {
    out_color_fs = v_color;
}