#version 450
#extension GL_ARB_separate_shader_objects : enable

const int MAX_LIGHTS = 20;
// The specular exponent that will exist if the metallic value is exactly 0.
const float MAX_SPECULAR = 128;

const float PI = 3.14159265359;

struct LightData {
    vec3 pos;
    vec3 color;
};

struct LightsList {
    int count;
    LightData data[MAX_LIGHTS];
};

const int MAX_MATERIALS = 20;
const float DEFAULT_MATERIAL_BRIGHTNESS = 1.0;
const float DEFAULT_MATERIAL_ROUGHNESS = 0.6;

const int USE_ALBEDO_BIT = 0x01;
const int USE_NORMAL_BIT = 0x02;
const int USE_METALLIC_BIT = 0x04;
const int USE_ROUGHNESS_BIT = 0x10;

struct Material {
    vec3 albedo_global;
    int options;
    float metallic_global;
    float roughness_global;
};

struct MaterialsList {
    int count;
    Material data[MAX_MATERIALS];
};

layout(location = 0) in vec2 uv;
layout(location = 1) in vec3 norm;
layout(location = 2) in vec3 frag_pos;
layout(location = 3) in vec3 view_pos;

layout(set = 0, binding = 1) uniform u_LightList {
    LightsList lights;
};

layout(set = 1, binding = 0) uniform u_Material {
    Material material;
};

layout(set = 1, binding = 1) uniform sampler samp;
layout(set = 1, binding = 2) uniform texture2D albedo;
layout(set = 1, binding = 3) uniform texture2D normal;
layout(set = 1, binding = 4) uniform texture2D metallic;
layout(set = 1, binding = 5) uniform texture2D roughness;

layout(location = 0) out vec4 target;

struct Frag {
    vec3 albedo;
    vec3 normal;
    float metallic;
    float roughness;
};

vec3 fresnelSchlick(float cosTheta, vec3 F0)
{
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

float DistributionGGX(vec3 N, vec3 H, float roughness)
{
    float a      = roughness*roughness;
    float a2     = a*a;
    float NdotH  = max(dot(N, H), 0.0);
    float NdotH2 = NdotH*NdotH;

    float num   = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return num / denom;
}

float GeometrySchlickGGX(float NdotV, float roughness)
{
    float r = (roughness + 1.0);
    float k = (r*r) / 8.0;

    float num   = NdotV;
    float denom = NdotV * (1.0 - k) + k;

    return num / denom;
}
float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness)
{
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2  = GeometrySchlickGGX(NdotV, roughness);
    float ggx1  = GeometrySchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}

vec4 fwd_render_frag(Frag frag) {

    vec3 N = normalize(norm);
    vec3 V = normalize(view_pos - frag_pos);

    vec3 F0 = vec3(0.04);
    F0 = mix(F0, frag.albedo, frag.metallic);

    // reflectance equation
    vec3 Lo = vec3(0.0);
    for(int i = 0; i < lights.count; ++i)
    {
        // calculate per-light radiance
        vec3 L = normalize(lights.data[i].pos - frag_pos);
        vec3 H = normalize(V + L);
        float distance    = length(lights.data[i].pos  - frag_pos);
        float attenuation = 1.0 / (distance * distance);
        vec3 radiance     = lights.data[i].color * attenuation;

        // cook-torrance brdf
        float NDF = DistributionGGX(N, H, frag.roughness);
        float G   = GeometrySmith(N, V, L, frag.roughness);
        vec3 F    = fresnelSchlick(max(dot(H, V), 0.0), F0);

        vec3 kS = F;
        vec3 kD = vec3(1.0) - kS;
        kD *= 1.0 - frag.metallic;

        vec3 numerator    = NDF * G * F;
        float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0);
        vec3 specular     = numerator / max(denominator, 0.001);

        // add to outgoing radiance Lo
        float NdotL = max(dot(N, L), 0.0);
        Lo += (kD * frag.albedo / PI + specular) * radiance * NdotL;
    }

    vec3 ambient = vec3(0.03) * frag.albedo;
    vec3 color = ambient + Lo;

    color = color / (color + vec3(1.0));
    color = pow(color, vec3(1.0/2.2));

    return vec4(color, 1.0);
}

void main() {

    // Adjust texture coordinates.
   // vec2 tex_coords = vec2(uv.x, 1.0 - uv.y);
    vec2 tex_coords = uv;

    Frag frag;

    if ((material.options & USE_ALBEDO_BIT) != 0) {
        frag.albedo = texture(sampler2D(albedo, samp), tex_coords).xyz;
    } else {
        frag.albedo = material.albedo_global;
    }

    if ((material.options & USE_NORMAL_BIT) != 0) {
        frag.normal = texture(sampler2D(normal, samp), tex_coords).xyz;
    } else {
        frag.normal = norm;
    }

    if ((material.options & USE_METALLIC_BIT) != 0) {
        frag.metallic = texture(sampler2D(metallic, samp), tex_coords).x;
    } else {
        frag.metallic = material.metallic_global;
    }

    if ((material.options & USE_ROUGHNESS_BIT) != 0) {
        frag.roughness = texture(sampler2D(roughness, samp), tex_coords).x;
    } else {
        frag.roughness = material.roughness_global;
    }

    target = fwd_render_frag(frag);

   // target = texture(sampler2D(albedo, samp), uv);
}