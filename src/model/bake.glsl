#version 450

layout(location=0) in vec3 in_position;
layout(location=1) in vec3 in_normal;
layout(location=2) in vec2 in_tex_coords;

layout(location=0) out vec3 out_position;

layout(set=0, binding=0)
uniform Uniforms {
    mat4 view_proj;
    vec4 camera_pos;
    ivec4 num_lights;
};

layout(location=5) in vec4 model_matrix_0;
layout(location=6) in vec4 model_matrix_1;
layout(location=7) in vec4 model_matrix_2;
layout(location=8) in vec4 model_matrix_3;


void main() {
    mat4 model_matrix = mat4(model_matrix_0, model_matrix_1, model_matrix_2, model_matrix_3);
    gl_Position = view_proj * model_matrix * vec4(in_position, 1.0);
}