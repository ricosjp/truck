#version 450

layout(set = 0, binding = 0) uniform Camera {
    mat4 camera_matrix;
    mat4 camera_projection;
};

struct Light {
    vec4 position;
    vec4 color;
    uvec4 light_type;
};

layout(set = 0, binding = 1) buffer Lights {
    Light lights[];
};

layout(set = 0, binding = 2) uniform Scene {
    float time;
    uint nlights;
};

layout(location = 0) in uint idx;
layout(location = 0) out mat4 cm;
layout(location = 4) out mat4 cp;
layout(location = 8) out vec4 lp0;
layout(location = 9) out vec4 lc0;
layout(location = 10) out uvec4 lt0;
layout(location = 11) out vec4 lp1;
layout(location = 12) out vec4 lc1;
layout(location = 13) out uvec4 lt1;
layout(location = 14) out float st;
layout(location = 15) out uint snl;

const vec2 vertex[4] = vec2[](
    vec2(-1.0, -1.0),
    vec2(1.0, -1.0),
    vec2(-1.0, 1.0),
    vec2(1.0, 1.0)
);

void main() {
    cm = camera_matrix;
    cp = camera_projection;
    lp0 = lights[0].position;
    lc0 = lights[0].color;
    lt0 = lights[0].light_type;
    lp1 = lights[1].position;
    lc1 = lights[1].color;
    lt1 = lights[1].light_type;
    st = time;
    snl = nlights;
    vec2 uv = vertex[idx];
    gl_Position = vec4(uv, 0.0, 1.0);
}

