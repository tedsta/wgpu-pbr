#version 450

layout(location = 0) in vec2 v_TexCoord;
layout(location = 0) out vec4 o_Target;
layout(location = 1) out vec4 bright_color;

layout(set = 2, binding = 0) uniform MeshPart {
    layout(offset = 0) vec4 in_diffuse;
    layout(offset = 16) float metal_factor;
    layout(offset = 32) float rough_factor;
    layout(offset = 48) vec3 emissive_factor;
    layout(offset = 64) vec3 extra_emissive;
};
layout(set = 2, binding = 1) uniform sampler s_Color;
layout(set = 2, binding = 2) uniform texture2D t_Color;

void main() {
    o_Target = texture(sampler2D(t_Color, s_Color), v_TexCoord) * in_diffuse;
}
