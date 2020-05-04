#version 450

layout(location = 0) in vec3 a_Pos;
layout(location = 1) in vec3 a_Norm;
layout(location = 2) in vec4 a_Tang;
layout(location = 3) in vec2 a_TexCoord;
layout(location = 0) out vec2 v_TexCoord;
layout(location = 1) out vec4 v_Position;

layout(set = 0, binding = 0) uniform Globals {
    layout(offset = 0) mat4 view_proj;
    layout(offset = 64) vec3 camera_pos;
};
layout(set = 1, binding = 0) uniform Mesh {
    mat4 transform;
};

void main() {
    v_TexCoord = a_TexCoord;
    v_Position = transform * vec4(a_Pos, 1.0);
    gl_Position = view_proj * v_Position;
}
