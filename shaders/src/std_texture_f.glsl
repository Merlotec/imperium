#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec2 uv;

layout(set = 0, binding = 1) uniform texture2D colormap;
layout(set = 0, binding = 2) uniform sampler colorsampler;

layout(location = 0) out vec4 target;

void main() {
    target = texture(sampler2D(colormap, colorsampler), uv);
}