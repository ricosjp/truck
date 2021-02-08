#version 450

const float EPS = 1.0e-5;

const mat4 acm = mat4(
    1.0, 2.1, 3.2, 4.3, 5.4, 6.5, 7.6, 8.7, 9.8, 10.9, 11.0, 12.0, 13.0, 14.0, 15.0, 16.23
);

const mat4 acp = mat4(
    11.714964805291158, -20.083602331793195, 0.2296881862103637, 1.0000000000018192,
    -7.919279773964541, 12.919469128453738, -1.3377250768579256, -2.0000000000036002,
    -23.645933626763625, 36.55672789512988, 1.9863855950847729, 1.0000000000017617,
    19.301563694896643, -28.84390979125007, -0.8783487044372066, 0.000000000000025531539193725142
);

struct Light {
    vec4 position;
    vec4 color;
    uvec4 light_type;
};

layout(set = 0, binding = 0) uniform Camera {
    mat4 camera_matrix;
    mat4 camera_projection;
};

layout(set = 0, binding = 1) buffer Lights {
    Light lights[];
};

vec4 alp0 = vec4(0.1, 0.2, 0.3, 1.0);
vec4 alc0 = vec4(0.4, 0.5, 0.6, 1.0);
uvec4 alt0 = uvec4(0, 0, 0, 0);
vec4 alp1 = vec4(1.1, 1.2, 1.3, 1.0);
vec4 alc1 = vec4(1.4, 1.5, 1.6, 1.0);
uvec4 alt1 = uvec4(1, 0, 0, 0);

layout(set = 0, binding = 2) uniform Scene {
    float time;
    uint nlights;
};

uint asnl = 2;

layout(location = 0) in mat4 cm;
layout(location = 4) in mat4 cp;
layout(location = 8) in vec4 lp0;
layout(location = 9) in vec4 lc0;
layout(location = 10) in flat uvec4 lt0;
layout(location = 11) in vec4 lp1;
layout(location = 12) in vec4 lc1;
layout(location = 13) in flat uvec4 lt1;
layout(location = 14) in float st;
layout(location = 15) in flat uint snl;
layout(location = 0) out vec4 color;

void main() {
    if (cm != camera_matrix) {
        color = vec4(0.0, 0.0, 0.0, 1.0);
    } else if (distance(cm[0], acm[0]) > EPS) {
        color = vec4(0.0, 0.0, 0.0, 1.0);
    } else if (distance(cm[1], acm[1]) > EPS) {
        color = vec4(0.0, 0.0, 0.0, 1.0);
    } else if (distance(cm[2], acm[2]) > EPS) {
        color = vec4(0.0, 0.0, 0.0, 1.0);
    } else if (distance(cm[3], acm[3]) > EPS) {
        color = vec4(0.0, 0.0, 0.0, 1.0);
    } else if (cp != camera_projection) {
        color = vec4(0.1, 0.1, 0.1, 1.0);
    } else if (distance(cp[0], acp[0]) > EPS) {
        color = vec4(0.1, 0.1, 0.1, 1.0);
    } else if (distance(cp[1], acp[1]) > EPS) {
        color = vec4(0.1, 0.1, 0.1, 1.0);
    } else if (distance(cp[2], acp[2]) > EPS) {
        color = vec4(0.1, 0.1, 0.1, 1.0);
    } else if (distance(cp[3], acp[3]) > EPS) {
        color = vec4(0.1, 0.1, 0.1, 1.0);
    } else if (lp0 != lights[0].position) {
        color = vec4(0.2, 0.2, 0.2, 1.0);
    } else if (distance(lp0, alp0) > EPS) {
        color = vec4(0.2, 0.2, 0.2, 1.0);
    } else if (lc0 != lights[0].color) {
        color = vec4(0.3, 0.3, 0.3, 1.0);
    } else if (distance(lc0, alc0) > EPS) {
        color = vec4(0.3, 0.3, 0.3, 1.0);
    } else if (lt0 != lights[0].light_type) {
        color = vec4(0.4, 0.4, 0.4, 1.0);
    } else if (lt0 != alt0) {
        color = vec4(0.4, 0.4, 0.4, 1.0);
    } else if (lp1 != lights[1].position) {
        color = vec4(0.5, 0.5, 0.5, 1.0);
    } else if (distance(lp1, alp1) > EPS) {
        color = vec4(0.5, 0.5, 0.5, 1.0);
    } else if (lc1 != lights[1].color) {
        color = vec4(0.6, 0.6, 0.6, 1.0);
    } else if (distance(lc1, alc1) > EPS) {
        color = vec4(0.6, 0.6, 0.6, 1.0);
    } else if (lt1 != lights[1].light_type) {
        color = vec4(0.7, 0.7, 0.7, 1.0);
    } else if (lt1 != alt1) {
        color = vec4(0.7, 0.7, 0.7, 1.0);
    } else if (st != time) {
        color = vec4(0.8, 0.8, 0.8, 1.0);
    } else if (snl != nlights) {
        color = vec4(0.9, 0.9, 0.9, 1.0);
    } else if (snl != asnl) {
        color = vec4(0.9, 0.9, 0.9, 1.0);
    } else {
        color = vec4(0.2, 0.4, 0.6, 0.8);
    }
}
