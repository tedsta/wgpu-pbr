#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 a_pos;
layout(location = 1) in vec3 a_norm;
layout(location = 2) in vec4 a_tang;
layout(location = 3) in vec2 a_uv;

layout(set = 0, binding = 0) uniform Globals {
    layout(offset = 0) mat4 view_proj;
    layout(offset = 64) vec3 camera_pos;
};
layout(set = 1, binding = 0) uniform Mesh {
    mat4 transform;
};

layout(location = 0) out vec4 frag_world_pos;
layout(location = 1) out vec2 frag_uv;
// XXX mat3 isn't interpolated so we pass in rows individually
layout(location = 2) out vec3 tbn_t;
layout(location = 3) out vec3 tbn_b;
layout(location = 4) out vec3 tbn_n;

void main() {
    frag_uv = a_uv;
    frag_world_pos = transform * vec4(a_pos, 1.0);

    vec3 frag_norm = normalize(mat3(transform) * a_norm);
    vec3 frag_tang = normalize(mat3(transform) * a_tang.xyz);
    vec3 vertex_bitangent = cross(frag_norm, frag_tang) * a_tang.w;
    tbn_t = frag_tang;
    tbn_b = vertex_bitangent;
    tbn_n = frag_norm;

    gl_Position = view_proj * frag_world_pos;
}
