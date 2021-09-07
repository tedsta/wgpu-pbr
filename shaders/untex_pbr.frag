#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec4 f_world_pos;
layout(location = 1) in vec2 f_uv;
// XXX mat3 isn't interpolated so we pass in rows individually
layout(location = 2) in vec3 f_tbn_t;
layout(location = 3) in vec3 f_tbn_b;
layout(location = 4) in vec3 f_tbn_n;

struct Light {
    vec3 position;
    float intensity;
    vec3 color;
};

struct SpotLight {
    vec3 position;
    float angle;
    vec3 color;
    float range;
    vec3 direction;
    float smoothness;
    float intensity;
};

/*layout(set = 0, binding = 1) uniform textureCube spec_cube_map;
layout(set = 0, binding = 2) uniform textureCube irradiance_cube_map;
layout(set = 0, binding = 3) uniform texture2D spec_brdf_map;*/

layout(std140, set = 0, binding = 0) uniform Args {
    layout(offset = 0) mat4 proj_view;
    layout(offset = 64) vec3 camera_pos;
    layout(offset = 76) int point_light_count;
    layout(offset = 80) Light point_lights[32];
    layout(offset = 1104) int spot_light_count;
    layout(offset = 1120) SpotLight spot_lights[32];
};

layout(set = 2, binding = 0) uniform MeshPart {
    layout(offset = 0) vec4 in_diffuse;
    layout(offset = 16) float metal_factor;
    layout(offset = 32) float rough_factor;
    layout(offset = 48) vec3 emissive_factor;
    layout(offset = 64) vec3 extra_emissive;
};

layout(location = 0) out vec4 color;
layout(location = 1) out vec4 bright_color;

const float PI = 3.14159265359;

float tex_coord(float coord, vec2 offset) {
    return offset.x + coord * (offset.y - offset.x);
}

vec2 tex_coords(vec2 coord, vec2 u, vec2 v) {
    return vec2(tex_coord(coord.x, u), tex_coord(coord.y, v));
}

float normal_distribution(vec3 N, vec3 H, float a) {
    float a2 = a * a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH*NdotH;

    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return (a2 + 0.0000001) / denom;
}

float geometry(float NdotV, float NdotL, float r2) {
    float a1 = r2 + 1.0;
    float k = a1 * a1 / 8.0;
    float denom = NdotV * (1.0 - k) + k;
    float ggx1 = NdotV / denom;
    denom = NdotL * (1.0 - k) + k;
    float ggx2 = NdotL / denom;
    return ggx1 * ggx2;
}

vec3 fresnel(float HdotV, vec3 fresnel_base) {
    return fresnel_base + (1.0 - fresnel_base) * pow(1.0 - HdotV, 5.0);
}

vec3 compute_light(vec3 attenuation,
                   vec3 light_color,
                   vec3 view_direction,
                   vec3 light_direction,
                   vec3 albedo,
                   vec3 normal,
                   float roughness2,
                   float metallic,
                   vec3 fresnel_base) {

    vec3 halfway = normalize(view_direction + light_direction);
    float normal_distribution = normal_distribution(normal, halfway, roughness2);

    float NdotV = max(dot(normal, view_direction), 0.0);
    float NdotL = max(dot(normal, light_direction), 0.0);
    float HdotV = max(dot(halfway, view_direction), 0.0);
    float geometry = geometry(NdotV, NdotL, roughness2);


    vec3 fresnel = fresnel(HdotV, fresnel_base);
    vec3 diffuse = vec3(1.0) - fresnel;
    diffuse *= 1.0 - metallic;

    vec3 nominator = normal_distribution * geometry * fresnel;
    float denominator = 4 * NdotV * NdotL + 0.0001;
    vec3 specular = nominator / denominator;

    vec3 resulting_light = (diffuse * albedo / PI + specular) * light_color * attenuation * NdotL;
    return resulting_light;
}

void main() {
    /*vec3 albedo = texture(sampler2D(albedo_map, tex_sampler), f_uv).rgb;
    vec3 normal = texture(sampler2D(normal_map, tex_sampler), f_uv).rgb;
    vec2 metallic_roughness = texture(sampler2D(metallic_roughness_map, tex_sampler), f_uv).bg;
    float metallic = metallic_roughness.r;
    float roughness = metallic_roughness.g;
    float ambient_occlusion = texture(sampler2D(ao_map, tex_sampler), f_uv).r;
    vec3 emission = texture(sampler2D(emissive_map, tex_sampler), f_uv).rgb;
    vec3 emissive_factor = vec3(1.0, 1.0, 1.0);*/

    vec3 albedo = in_diffuse.xyz;
    vec3 normal = f_tbn_n;
    float metallic = metal_factor;
    float roughness = rough_factor;
    float ambient_occlusion = 0.5;
    vec3 emission = extra_emissive;

    /////////////////////////////////

    float roughness2 = roughness * roughness;
    vec3 fresnel_base = mix(vec3(0.04), albedo, metallic);

    vec3 view_direction = normalize(camera_pos - f_world_pos.xyz);
    vec3 lighted = vec3(0.0);
    for (int i = 0; i < point_light_count; i++) {
        vec3 light_direction = point_lights[i].position - f_world_pos.xyz;
        float attenuation = point_lights[i].intensity / dot(light_direction, light_direction);

        vec3 light = compute_light(vec3(attenuation),
                                   point_lights[i].color,
                                   view_direction,
                                   normalize(light_direction),
                                   albedo,
                                   normal,
                                   roughness2,
                                   metallic,
                                   fresnel_base);

        lighted += light;
    }

    /*for (int i = 0; i < directional_light_count; i++) {
        vec3 light_direction = -normalize(dlight[i].direction);
        float attenuation = dlight[i].intensity;

        vec3 light = compute_light(vec3(attenuation),
                                   dlight[i].color,
                                   view_direction,
                                   light_direction,
                                   albedo,
                                   normal,
                                   roughness2,
                                   metallic,
                                   fresnel_base);

        lighted += light;
    }*/

    for (int i = 0; i < spot_light_count; i++) {
        vec3 light_vec = spot_lights[i].position - f_world_pos.xyz;
        vec3 normalized_light_vec = normalize(light_vec);

        // The distance between the current fragment and the "core" of the light
        float light_length = length(light_vec);

        // The allowed "length", everything after this won't be lit.
        // Later on we are dividing by this range, so it can't be 0
        float range = max(spot_lights[i].range, 0.00001);

        // get normalized range, so everything 0..1 could be lit, everything else can't.
        float normalized_range = light_length / max(0.00001, range);

        // The attenuation for the "range". If we would only consider this, we'd have a
        // point light instead, so we need to also check for the spot angle and direction.
        float range_attenuation = max(0.0, 1.0 - normalized_range);

        // this is actually the cosine of the angle, so it can be compared with the
        // "dotted" frag_angle below a lot cheaper.
        float spot_angle = max(spot_lights[i].angle, 0.00001);
        vec3 spot_direction = normalize(spot_lights[i].direction);
        float smoothness = 1.0 - spot_lights[i].smoothness;

        // Here we check if the current fragment is within the "ring" of the spotlight.
        float frag_angle = dot(spot_direction, -normalized_light_vec);

        // so that the ring_attenuation won't be > 1
        frag_angle = max(frag_angle, spot_angle);

        // How much is this outside of the ring? (let's call it "rim")
        // Also smooth this out.
        float rim_attenuation = pow(max((1.0 - frag_angle) / (1.0 - spot_angle), 0.00001), smoothness);

        // How much is this inside the "ring"?
        float ring_attenuation = 1.0 - rim_attenuation;

        // combine the attenuations and intensity
        float attenuation = range_attenuation * ring_attenuation * spot_lights[i].intensity;

        vec3 light = compute_light(vec3(attenuation),
                                   spot_lights[i].color,
                                   view_direction,
                                   normalize(light_vec),
                                   albedo,
                                   normal,
                                   roughness2,
                                   metallic,
                                   fresnel_base);
        lighted += light;
    }

    vec3 ambient_color = vec3(0.01, 0.01, 0.01);
    vec3 ambient = ambient_color * albedo * ambient_occlusion;
    color = vec4(ambient + lighted + emission, 1.0);
    //vec3 color = ambient + lighted + emission;

    //out_color = vec4(color, alpha) * vertex.color;
}
