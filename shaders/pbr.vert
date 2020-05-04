#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 a_pos;
layout(location = 1) in vec3 a_norm;
layout(location = 2) in vec4 a_tang;
layout(location = 3) in vec2 a_uv;
// vec4[4] is used instead of mat4 due to spirv-cross bug for dx12 backend
//layout(location = 4) in vec4 model[4]; // per-instance.

layout(set = 0, binding = 0) uniform Globals {
    layout(offset = 0) mat4 view_proj;
    layout(offset = 64) vec3 camera_pos;
};
layout(set = 1, binding = 0) uniform Mesh {
    mat4 transform;
};

layout(location = 0) out vec4 frag_world_pos;
layout(location = 1) out vec3 frag_norm;
layout(location = 2) out vec3 frag_tang;
layout(location = 3) flat out float frag_tbn_handedness;
layout(location = 4) out vec2 frag_uv;
layout(location = 5) out mat3 tbn;

void main() {
    frag_uv = a_uv;
    frag_norm = normalize((transform * vec4(a_norm, 0.0)).xyz);
    frag_tang = normalize((transform * vec4(a_tang.xyz, 0.0)).xyz);
    frag_tbn_handedness = a_tang.w;
    frag_world_pos = transform * vec4(a_pos, 1.0);
    vec3 vertex_bitangent = normalize(cross(frag_norm, frag_tang)) * a_tang.w;
    tbn = mat3(frag_tang, vertex_bitangent, frag_norm);
    gl_Position = view_proj * frag_world_pos;
}
