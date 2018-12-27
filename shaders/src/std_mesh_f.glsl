#version 450
#extension GL_ARB_separate_shader_objects : enable

const int MAX_LIGHTS = 20;

struct Ambient {
    float intensity;
    vec4 color;
};

struct Diffuse {
    float intensity;
    vec4 color;
    vec3 direction;
};

struct LightData {
    Ambient ambient;
    Diffuse diffuse;
};

struct LightList {
    int count;
    LightData data[MAX_LIGHTS];
};

layout(location = 0) in vec2 uv;
layout(location = 1) in vec3 norm;

layout(set = 0, binding = 1) uniform u_LightList {
    LightList lights;
};

layout(set = 1, binding = 0) uniform texture2D colormap;
layout(set = 1, binding = 1) uniform sampler colorsampler;

layout(location = 0) out vec4 target;

void main() {

    vec4 TotalAmbient = vec4(0, 0, 0, 1.0);

    vec4 TotalDiffuse = vec4(0, 0, 0, 1.0);

    for (int i = 0; i < lights.count; i++) {

        vec4 AmbientColor = lights.data[i].ambient.color * lights.data[i].ambient.intensity;

        float DiffuseFactor = dot(normalize(norm), -lights.data[i].diffuse.direction);

        vec4 DiffuseColor;

        if (DiffuseFactor > 0) {
            DiffuseColor = lights.data[i].diffuse.color * lights.data[i].diffuse.intensity * DiffuseFactor;
        }
        else {
            DiffuseColor = vec4(0, 0, 0, 0);
        }

        TotalAmbient += AmbientColor;
        TotalDiffuse += DiffuseColor;

    }

    //target = vec4(0.0, 0.0, 1.0, 1.0);
    target = texture(sampler2D(colormap, colorsampler), uv) * vec4(TotalDiffuse.xyz + TotalAmbient.xyz, 1.0);
    //target = texture(sampler2D(colormap, colorsampler), uv);
}