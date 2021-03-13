[[location(0)]] var<in> in_position: vec4<f32>;
[[location(1)]] var<in> in_normal: vec4<f32>;
[[location(2)]] var<in> in_uv: vec2<f32>;

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
    out_position = u_globals.view_proj * model_matrix * vec4<f32>(in_position);
}


[[location(0)]]
var<out> out_normal_vs: vec3<f32>;
[[location(1)]]
var<out> out_position_vs: vec4<f32>;
[[location(2)]]
var<out> out_uv_vs: vec2<f32>;

[[stage(vertex)]]
fn vs_main() {
    const model_matrix: mat4x4<f32> = mat4x4<f32>(model_matrix_0, model_matrix_1, model_matrix_2, model_matrix_3);

    out_normal_vs = mat3x3<f32>(model_matrix.x.xyz, model_matrix.y.xyz, model_matrix.z.xyz) * vec3<f32>(in_normal.xyz);
    out_position_vs = model_matrix * in_position;
    out_position = u_globals.view_proj * out_position_vs;
    out_uv_vs = in_uv;
}

// fragment shader

[[block]]
struct SimpleLight {
    color: vec4<f32>;
    pos: vec4<f32>;
    constant: f32;
    linear: f32;
    quadratic: f32;
};

[[block]]
struct SimpleLights {
    data: [[stride(44)]] array<SimpleLight>;
};

[[block]]
struct RealLight {
    proj: mat4x4<f32>;
    pos: vec4<f32>;
    color: vec4<f32>;
    default: f32;
    constant: f32;
    linear: f32;
    quadratic: f32;
    active: u32;
};

[[block]]
struct Lights {
    data: [[stride(128)]] array<RealLight>;
};

[[group(0), binding(1)]]
var<storage> real_lights: [[access(read)]] Lights;
[[group(0), binding(2)]]
var<storage> simple_lights: [[access(read)]] SimpleLights;
[[group(0), binding(3)]]
var t_shadow: texture_depth_2d_array;
[[group(0), binding(4)]]
var sampler_shadow: sampler;

fn fetch_shadow(light_id: u32, homogeneous_coords: vec4<f32>) -> f32 {
    if (homogeneous_coords.w <= 0.0) {
        return 1.0;
    }
    // compensate for the Y-flip difference between the NDC and texture coordinates
    const flip_correction: vec2<f32> = vec2<f32>(0.5, -0.5);
    // compute texture coordinates for shadow lookup
    const light_local: vec2<f32> = homogeneous_coords.xy * flip_correction + vec2<f32>(0.5, 0.5);
    // do the lookup, using HW PCF and comparison
    return textureSampleCompare(t_shadow, sampler_shadow, light_local, i32(light_id), homogeneous_coords.z);
}

// reflect function
// I dont know why i cant call it reflect but hey
fn rf(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return a - (2.0 * dot(a, b) * b);
}

fn ln(v: vec3<f32>) -> f32 {
    return (v.x * v.x + v.y * v.y + v.z * v.z);
}

[[location(0)]]
var<in> in_normal_fs: vec3<f32>;
[[location(1)]]
var<in> in_position_fs: vec4<f32>;
[[location(2)]]
var<in> uv_in_fs: vec2<f32>;

[[location(0)]]
var<out> out_color_fs: vec4<f32>;

const c_max_lights: u32 = 10u;

[[group(1), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(1), binding(1)]]
var sampler_diffuse: sampler;

[[stage(fragment)]]
fn fs_main() {
    const object_color: vec4<f32> = textureSample(t_diffuse, sampler_diffuse, uv_in_fs);

    if (u_globals.enable_lighting != 0) {
        const view_dir: vec3<f32> = normalize(u_globals.camera_pos.xyz - in_position_fs.xyz);
        const normal: vec3<f32> = normalize(in_normal_fs);

        // accumulate color
        var color: vec4<f32> = u_globals.ambient_color;
        var i: u32 = 0u;

        loop {
            if (i >= min(u_globals.num_lights.x, c_max_lights)) {
                break;
            }

            const light: RealLight = real_lights.data[i];
            if (light.active == 0) {
                continue;
            }

            // project into the light space
            var light_view_space_pos: vec4<f32> = light.proj * in_position_fs;
            const correction: f32 = 1.0 / light_view_space_pos.w;
            light_view_space_pos = light_view_space_pos * correction;
            var shadow: f32 = light.default;

            if (
                light_view_space_pos.x < 1.0 &&
                light_view_space_pos.y < 1.0 &&
                light_view_space_pos.x > -1.0 &&
                light_view_space_pos.y > -1.0
            ) {
                shadow = fetch_shadow(i, light_view_space_pos);
            }

            // attenuation
            const d: f32 = distance(light.pos.xyz, in_position_fs.xyz);
            const attenuation: f32 = 1.0 / (light.constant + light.linear * d + light.quadratic * (d * d));

            // diffuse lighting
            const light_dir: vec3<f32> = normalize(light.pos.xyz - in_position_fs.xyz);

            const diffuse: f32 = max(0.0, dot(normal, light_dir));

            // specular lighting
            const reflect_dir: vec3<f32> = rf(-light_dir, normal);

            const spec: f32 = pow(max(dot(view_dir, reflect_dir), 0.0), 32);

            color = color + (diffuse + spec) * shadow * light.color * attenuation * light.color.w;

            continuing {
                i = i + 1u;
            }
        }

        i = 0;

        loop {
            if (i >= u_globals.num_lights.y) {
                break;
            }
            const light: SimpleLight = simple_lights.data[i];

            var attenuation: f32;

            var light_dir: vec3<f32>;
            if (light.pos.w == 0.0) {
                light_dir = normalize(-light.pos.xyz);
                attenuation = 1.0;
            } else {
                const d: f32 = distance(light.pos.xyz, in_position_fs.xyz);
                attenuation = 1.0 / (light.constant + light.linear * d + light.quadratic * (d * d));
                light_dir = normalize(light.pos.xyz - in_position_fs.xyz);
            }

            // diffuse lighting
            const diffuse: f32 = max(0.0, dot(normal, light_dir));
            //const diffuse: f32 = 0.0;

            // specular lighting
            const reflect_dir: vec3<f32> = rf(-light_dir, normal);
            const spec: f32 = pow(max(dot(view_dir, reflect_dir), 0.0), 32);

            color = color + (diffuse + spec) * light.color * attenuation * light.color.w;

            continuing {
                i = i + 1u;
            }
        }

        // multiply the light by material color
        out_color_fs = color * object_color;
    } else {
        out_color_fs = object_color;
    }
}