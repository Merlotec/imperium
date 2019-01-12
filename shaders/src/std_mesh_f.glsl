#version 450
#extension GL_ARB_separate_shader_objects : enable

const int MAX_LIGHTS = 20;
// The specular exponent that will exist if the metallic value is exactly 0.
const float MAX_SPECULAR = 128;

struct LightData {
    vec3 pos;
    vec3 color;
    float ambient;
    float diffuse;
    float specular;
};

struct LightsList {
    int count;
    LightData data[MAX_LIGHTS];
};

const int MAX_MATERIALS = 20;
const float DEFAULT_MATERIAL_BRIGHTNESS = 1.0;
const float DEFAULT_MATERIAL_ROUGHNESS = 0.6;

struct Material {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float roughness;
};

struct MaterialsList {
    int count;
    Material data[MAX_MATERIALS];
};

layout(location = 0) in vec2 uv;
layout(location = 1) in vec3 norm;
layout(location = 2) in vec3 frag_pos;
layout(location = 3) in vec3 view_pos;
layout(location = 4) in flat int material_index;

layout(location = 5) in flat mat4 view_matrix;

layout(set = 0, binding = 1) uniform u_LightList {
    LightsList lights;
};

layout(set = 1, binding = 0) uniform texture2D colormap;
layout(set = 1, binding = 1) uniform sampler colorsampler;

layout(set = 1, binding = 2) uniform u_MaterialList {
    MaterialsList materials;
};

layout(location = 0) out vec4 target;

Material get_material(int index) {
    if (index < materials.count && index >= 0) {
        return materials.data[index];
    }
    return Material(vec3(DEFAULT_MATERIAL_BRIGHTNESS), vec3(DEFAULT_MATERIAL_BRIGHTNESS), vec3(DEFAULT_MATERIAL_BRIGHTNESS), DEFAULT_MATERIAL_ROUGHNESS);
}

vec4 calculate_lighting(vec3 view_dir, Material material) {

    vec3 total_ambient = vec3(0, 0, 0);
    vec3 total_diffuse = vec3(0, 0, 0);
    vec3 total_specular = vec3(0, 0, 0);

    for (int i = 0; i < lights.count; i++) {

        //vec3 light_pos = vec3(view_matrix * vec4(lights.data[i].pos, 1.0));

        vec3 light_dir = normalize(lights.data[i].pos - frag_pos);

        // Ambient
        vec3 ambient_color = material.ambient * lights.data[i].color * lights.data[i].ambient;

        // Diffuse
        float diffuse_factor = max(dot(normalize(norm), light_dir), 0.0);
        vec3 diffuse_color;
        diffuse_color = lights.data[i].color * lights.data[i].diffuse * material.diffuse * diffuse_factor;

        // Specular
        vec3 reflect_dir = normalize(reflect(-light_dir, norm));
        float specular_exponent = MAX_SPECULAR - (MAX_SPECULAR * min(material.roughness, 1.0));
        float spec_factor = pow(max(dot(view_dir, reflect_dir), 0.0), 32);
        vec3 specular_color = material.specular * spec_factor * lights.data[i].specular;

        total_ambient += ambient_color;
        total_diffuse += diffuse_color;
        total_specular += specular_color;
    }
    return vec4(total_ambient + total_diffuse + total_specular, 1.0);
}

void main() {

    // Adjust texture coordinates.
    vec2 tex_coords = vec2(uv.x, 1.0 - uv.y);
    vec3 view_dir = normalize(view_pos - frag_pos);
    Material material = get_material(material_index);
    target = texture(sampler2D(colormap, colorsampler), tex_coords) * calculate_lighting(view_dir, material);

    //target = texture(sampler2D(colormap, colorsampler), uv);


}