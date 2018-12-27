#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec2 position;
layout(location = 1) in vec4 inColor;

layout(binding = 0) uniform Transform {
    mat4 model;
    mat4 view;
    mat4 projection;
};

flat layout(location = 0) out vec4 color;

void main() {
    color = inColor;
    gl_Position = projection * view * model * vec4(position, 0.0, 1.0);
}