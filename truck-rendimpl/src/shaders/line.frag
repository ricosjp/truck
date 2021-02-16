#version 450

layout(set = 1, binding = 1) uniform Color {
    vec4 color;
};

layout(location = 0) out vec4 outColor;

void main() {
    outColor = color;
}
