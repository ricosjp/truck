#version 450
#define BUF_SIZE 32

layout(location = 0) in uint idx;

layout(set = 0, binding = 0) uniform Camera {
    mat4 camera_matrix;
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

uint u_floor(float t) {
    for (uint i = 0; i < len; i++) {
        if (u < uknot_vec[i]) {
            if (i > 0) return i - 1;
            else return i;
        }
    }
    return len;
}

double[BBF_SIZE] u_bbf(double u, uint degree) {
    uint udegree = uknots_length - ctrl_row_length - 1;

    double[BBF_SIZE] res;
    for (uint i = 0; i < len; i++) {
        res[i] = 0.0;
    }
    
    
    uint idx = len;
    for (uint i = 0; i < len; i++) {
        if (u < uknot[i]) {
            idx = i - 1;
            break;
        }
    }
    if (idx == len) {
        res[len - degree - 2] = 1.0;
        return res;
    }
    res[idx] = 1.0;

    for (uint k = 1; k <= degree; k++) {
        uint base = 0;
        if (idx >= k) base = idx - k;
        double delta = uknot[base + k] - uknot[base];
        uint maximum = idx;
        if (idx + k + 1 >= len)  maximum = len - k - 2;
        double a = inv_or_zero(delta) * (u - uknot[base]);
        for (uint i = base; i <= maximum; i++) {
            double delta = uknot[i + k + 1] - uknot[i + 1];
            double b = inv_or_zero(delta) * (uknot[i + k + 1] - u);
            res[i] = a * res[i] + b * res[i + 1];
            a = 1.0 - b;
        }
    }

    return res;
}

void main() {
    uint uidx = idx / param_column_size;
    uint vidx = idx % param_column_size;

}
