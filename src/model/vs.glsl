#version 450

layout(location=0) in vec3 in_position;
layout(location=1) in vec3 in_normal;
layout(location=2) in vec2 in_tex_coords;
layout(location=3) in vec3 in_tangent;
layout(location=4) in vec3 in_bitangent;

layout(location=0) out vec2 out_tex_coords;
layout(location=1) out vec3 out_position;
layout(location=2) out vec3 out_normal;
layout(location=3) out mat3 out_tangent_matrix;

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

    mat3 normal_matrix = mat3(transpose(inverse(model_matrix)));

    vec3 normal = normalize(normal_matrix * in_normal);
    vec3 tangent = normalize(normal_matrix * in_tangent);
    vec3 bitangent = normalize(normal_matrix * in_bitangent);

    out_tangent_matrix = transpose(mat3(
        tangent,
        bitangent,
        normal
    ));

    out_tex_coords = in_tex_coords;
    vec4 model_space = model_matrix * vec4(in_position, 1.0);
    out_position = model_space.xyz;
    gl_Position = view_proj * model_space;
    out_normal = mat3(model_matrix_0.xyz, model_matrix_1.xyz, model_matrix_2.xyz) * in_normal;
}
