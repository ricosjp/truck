#version 450
#define BUF_SIZE 32
#define EPSILON 1.0e-7

layout(location = 0) in uint idx;

layout(set = 0, binding = 0) uniform Camera {
    mat4 _camera_matrix;
    mat4 camera_projection;
};

layout(set = 1, binding = 0) buffer ControlPoints {
    vec4 control_points[];
};

layout(set = 1, binding = 1) buffer UKnotVec {
    float uknot_vec[];
};

layout(set = 1, binding = 2) buffer VKnotVec {
    float vknot_vec[];
};

layout(set = 1, binding = 3) buffer UDivision {
    float udiv[];
};

layout(set = 1, binding = 4) buffer VDivision {
    float vdiv[];
};

layout(set = 1, binding = 6) uniform SurfaceInfo {
    uint ctrl_row_length;
    uint ctrl_column_length;
    uint uknots_length;
    uint vknots_length;
    uint param_row_length;
    uint param_column_length;
    uint _boundary_length;
};

layout(location = 0) out vec3 position;
layout(location = 1) out vec2 uv;
layout(location = 2) out vec3 normal;

bool near(float a, float b) { return abs(a - b) < EPSILON; }

float inv_or_zero(float x) {
    if (abs(x) < EPSILON) return 0.0;
    else return 1.0 / x;
}

uint udegree() { return uknots_length - ctrl_row_length - 1; }
uint vdegree() { return vknots_length - ctrl_column_length - 1; }
uint umultiplicity(uint idx) {
    uint count = 0;
    for (int i = 0; i < uknots_length; i++) {
        if (near(uknot_vec[i], uknot_vec[idx])) count++;
    }
    return count;
}
uint vmultiplicity(uint idx) {
    uint count = 0;
    for (int i = 0; i < vknots_length; i++) {
        if (near(vknot_vec[i], vknot_vec[idx])) count++;
    }
    return count;
}

uint ufloor(float t) {
    for (uint i = 1; i <= uknots_length; i++) {
        uint j = uknots_length - i;
        if (uknot_vec[j] < t) return j;
    }
    return umultiplicity(0) - 1;
}

uint vfloor(float t) {
    for (uint i = 1; i <= vknots_length; i++) {
        uint j = vknots_length - i;
        if (vknot_vec[j] < t) return j;
    }
    return vmultiplicity(0) - 1;
}

float[BUF_SIZE] ubbf(float t, uint degree) {
    uint n = uknots_length - 1;

    uint idx = ufloor(t);
    if (idx == n) idx = n - umultiplicity(n);

    float[BUF_SIZE] res;
    for (uint i = 0; i < uknots_length; i++) {
        res[i] = 0.0;
    }
    res[idx] = 1.0;
    
    for (uint k = 1; k <= degree; k++) {
        uint base = 0;
        if (idx >= k) base = idx - k;
        float delta = uknot_vec[base + k] - uknot_vec[base];
        uint maximum = idx;
        if (idx + k + 1 >= uknots_length)  maximum = uknots_length - k - 2;
        float a = inv_or_zero(delta) * (t - uknot_vec[base]);
        for (uint i = base; i <= maximum; i++) {
            float delta = uknot_vec[i + k + 1] - uknot_vec[i + 1];
            float b = inv_or_zero(delta) * (uknot_vec[i + k + 1] - t);
            res[i] = a * res[i] + b * res[i + 1];
            a = 1.0 - b;
        }
    }

    return res;
}

float[BUF_SIZE] vbbf(float t, uint degree) {
    uint n = vknots_length - 1;

    uint idx = vfloor(t);
    if (idx == n) idx = n - vmultiplicity(n);

    float[BUF_SIZE] res;
    for (uint i = 0; i < vknots_length; i++) {
        res[i] = 0.0;
    }
    res[idx] = 1.0;
    
    for (uint k = 1; k <= degree; k++) {
        uint base = 0;
        if (idx >= k) base = idx - k;
        float delta = vknot_vec[base + k] - vknot_vec[base];
        uint maximum = idx;
        if (idx + k + 1 >= vknots_length)  maximum = vknots_length - k - 2;
        float a = inv_or_zero(delta) * (t - vknot_vec[base]);
        for (uint i = base; i <= maximum; i++) {
            float delta = vknot_vec[i + k + 1] - vknot_vec[i + 1];
            float b = inv_or_zero(delta) * (vknot_vec[i + k + 1] - t);
            res[i] = a * res[i] + b * res[i + 1];
            a = 1.0 - b;
        }
    }

    return res;
}

vec4 subs(float u, float v) {
    float[BUF_SIZE] ubbf = ubbf(u, udegree());
    float[BUF_SIZE] vbbf = vbbf(v, vdegree());
    vec4 res = vec4(0.0, 0.0, 0.0, 0.0);
    for (uint i = 0; i < ctrl_row_length; i++) {
        for (uint j = 0; j < ctrl_column_length; j++) {
            uint idx = i * ctrl_column_length + j;
            res += control_points[idx] * ubbf[i] * vbbf[j];
        }
    }
    return res;
}

vec4 uder(float u, float v) {
    uint k = udegree();
    float[BUF_SIZE] ubbf = ubbf(u, k - 1);
    float[BUF_SIZE] vbbf = vbbf(v, vdegree());
    vec4 res = vec4(0.0, 0.0, 0.0, 0.0);
    for (uint i = 0; i < ctrl_row_length - 1; i++) {
        for (uint j = 0; j < ctrl_column_length; j++) {
            uint idx = i * ctrl_column_length + j;
            float denom0 = uknot_vec[i + k] - uknot_vec[i];
            float denom1 = uknot_vec[i + k + 1] - uknot_vec[i + 1];
            float dubbf = k * (ubbf[i] / denom0 - ubbf[i + 1] / denom1);
            res += control_points[idx] * dubbf * vbbf[j];
        }
    }
    return res;
}

vec4 vder(float u, float v) {
    uint k = vdegree();
    float[BUF_SIZE] ubbf = ubbf(u, udegree());
    float[BUF_SIZE] vbbf = vbbf(v, k - 1);
    vec4 res = vec4(0.0, 0.0, 0.0, 0.0);
    for (uint i = 0; i < ctrl_row_length - 1; i++) {
        for (uint j = 0; j < ctrl_column_length; j++) {
            uint idx = i * ctrl_column_length + j;
            float denom0 = vknot_vec[j + k] - vknot_vec[j];
            float denom1 = vknot_vec[j + k + 1] - vknot_vec[j + 1];
            float dvbbf = k * (vbbf[j] / denom0 - vbbf[j + 1] / denom1);
            res += control_points[idx] * ubbf[i] * dvbbf;
        }
    }
    return res;
}

vec3 rat_proj(vec4 vec) { return vec.xyz / vec.w; }
vec3 der_proj(vec4 vec, vec4 der) {
    return (vec.w * der.xyz - der.w * vec.xyz) / (der.w * der.w);
}

void main() {
    float u = udiv[idx / param_column_length];
    float v = vdiv[idx % param_column_length];
    vec4 val = subs(u, v);
    vec3 ud = der_proj(val, uder(u, v));
    vec3 vd = der_proj(val, vder(u, v));

    position = rat_proj(val);
    uv = vec2(u, v);
    normal = cross(ud, vd);
    gl_Position = camera_projection * vec4(position, 1.0);
}
