#include <cuda_runtime.h>
#include <float.h>
#include <math.h>

struct GpuTriangle {
    float4 v0;
    float4 v1;
    float4 v2;
};

__device__ __forceinline__ float3 cross_product(float3 a, float3 b) {
    return make_float3(
        a.y * b.z - a.z * b.y,
        a.z * b.x - a.x * b.z,
        a.x * b.y - a.y * b.x
    );
}

__device__ __forceinline__ float dot_product(float3 a, float3 b) {
    return a.x * b.x + a.y * b.y + a.z * b.z;
}

__device__ __forceinline__ float3 sub(float3 a, float3 b) {
    return make_float3(a.x - b.x, a.y - b.y, a.z - b.z);
}

__device__ __forceinline__ float3 mul(float3 a, float s) {
    return make_float3(a.x * s, a.y * s, a.z * s);
}

__device__ float distance2_point_triangle_dev(float3 point, float3 v0, float3 v1, float3 v2) {
    float3 ab = sub(v1, v0);
    float3 ap = sub(point, v0);
    float3 bc = sub(v2, v1);
    float3 bp = sub(point, v1);
    float3 ca = sub(v0, v2);
    float3 cp = sub(point, v2);
    float3 nor = cross_product(ab, ca);

    float s0 = copysignf(1.0f, dot_product(cross_product(ab, nor), ap));
    float s1 = copysignf(1.0f, dot_product(cross_product(bc, nor), bp));
    float s2 = copysignf(1.0f, dot_product(cross_product(ca, nor), cp));
    float coef = s0 + s1 + s2;
    float nor_len2 = dot_product(nor, nor);

    if (coef < 2.0f || nor_len2 < 1.0e-12f) {
        float dot_ab = dot_product(ab, ab);
        float t_a = fminf(fmaxf(dot_product(ap, ab) / fmaxf(dot_ab, 1.0e-12f), 0.0f), 1.0f);
        float3 da = sub(ap, mul(ab, t_a));
        float d2_a = dot_product(da, da);

        float dot_bc = dot_product(bc, bc);
        float t_b = fminf(fmaxf(dot_product(bp, bc) / fmaxf(dot_bc, 1.0e-12f), 0.0f), 1.0f);
        float3 db = sub(bp, mul(bc, t_b));
        float d2_b = dot_product(db, db);

        float dot_ca = dot_product(ca, ca);
        float t_c = fminf(fmaxf(dot_product(cp, ca) / fmaxf(dot_ca, 1.0e-12f), 0.0f), 1.0f);
        float3 dc = sub(cp, mul(ca, t_c));
        float d2_c = dot_product(dc, dc);

        return fminf(fminf(d2_a, d2_b), d2_c);
    } else {
        float d = dot_product(nor, ap);
        return (d * d) / nor_len2;
    }
}

__device__ int check_crossing_dev(float3 origin, float3 dir, float3 v0, float3 v1, float3 v2) {
    float3 a = sub(v0, v1);
    float3 b = sub(v0, v2);
    float3 b_cross_dir = cross_product(b, dir);
    float det = dot_product(a, b_cross_dir);
    if (fabsf(det) < 1.0e-7f) {
        return 0;
    }

    float inv_det = 1.0f / det;
    float3 rhs = sub(v0, origin);
    float u = dot_product(rhs, b_cross_dir) * inv_det;
    if (u <= 0.0f || u >= 1.0f) {
        return 0;
    }

    float v = dot_product(a, cross_product(rhs, dir)) * inv_det;
    if (v <= 0.0f || (u + v) >= 1.0f) {
        return 0;
    }

    float t = dot_product(a, cross_product(b, rhs)) * inv_det;
    if (t <= 0.0f) {
        return 0;
    }

    float3 nor = cross_product(sub(v1, v0), sub(v2, v0));
    return (dot_product(nor, dir) > 0.0f) ? 1 : -1;
}

__global__ void distance2_point_cloud_kernel(
    const float4* __restrict__ points,
    unsigned int num_points,
    const GpuTriangle* __restrict__ triangles,
    unsigned int num_triangles,
    float* __restrict__ min_distances
) {
    unsigned int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= num_points) return;

    float4 pt4 = points[idx];
    float3 pt = make_float3(pt4.x, pt4.y, pt4.z);

    float min_d2 = 1.0e30f;
    for (unsigned int i = 0; i < num_triangles; ++i) {
        GpuTriangle tri = triangles[i];
        float3 v0 = make_float3(tri.v0.x, tri.v0.y, tri.v0.z);
        float3 v1 = make_float3(tri.v1.x, tri.v1.y, tri.v1.z);
        float3 v2 = make_float3(tri.v2.x, tri.v2.y, tri.v2.z);
        float d2 = distance2_point_triangle_dev(pt, v0, v1, v2);
        min_d2 = fminf(min_d2, d2);
    }
    min_distances[idx] = min_d2;
}

__global__ void signed_crossing_rays_kernel(
    const float4* __restrict__ points,
    const float4* __restrict__ ray_dirs,
    unsigned int num_points,
    const GpuTriangle* __restrict__ triangles,
    unsigned int num_triangles,
    int* __restrict__ crossing_counts
) {
    unsigned int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= num_points) return;

    float4 p4 = points[idx];
    float4 d4 = ray_dirs[idx];
    float3 origin = make_float3(p4.x, p4.y, p4.z);
    float3 dir = make_float3(d4.x, d4.y, d4.z);

    int count = 0;
    for (unsigned int i = 0; i < num_triangles; ++i) {
        GpuTriangle tri = triangles[i];
        float3 v0 = make_float3(tri.v0.x, tri.v0.y, tri.v0.z);
        float3 v1 = make_float3(tri.v1.x, tri.v1.y, tri.v1.z);
        float3 v2 = make_float3(tri.v2.x, tri.v2.y, tri.v2.z);
        count += check_crossing_dev(origin, dir, v0, v1, v2);
    }
    crossing_counts[idx] = count;
}

extern "C" {

int cuda_compute_distance2_point_cloud(
    const float* h_points,
    unsigned int num_points,
    const float* h_triangles,
    unsigned int num_triangles,
    float* h_min_distances
) {
    if (num_points == 0) return 0;
    if (num_triangles == 0) {
        for (unsigned int i = 0; i < num_points; ++i) {
            h_min_distances[i] = 1.0e30f;
        }
        return 0;
    }

    size_t points_size = num_points * sizeof(float4);
    size_t triangles_size = num_triangles * sizeof(GpuTriangle);
    size_t dist_size = num_points * sizeof(float);

    float4* d_points = NULL;
    GpuTriangle* d_triangles = NULL;
    float* d_min_distances = NULL;

    if (cudaMalloc(&d_points, points_size) != cudaSuccess) return -1;
    if (cudaMalloc(&d_triangles, triangles_size) != cudaSuccess) {
        cudaFree(d_points);
        return -2;
    }
    if (cudaMalloc(&d_min_distances, dist_size) != cudaSuccess) {
        cudaFree(d_points);
        cudaFree(d_triangles);
        return -3;
    }

    cudaMemcpy(d_points, h_points, points_size, cudaMemcpyHostToDevice);
    cudaMemcpy(d_triangles, h_triangles, triangles_size, cudaMemcpyHostToDevice);

    unsigned int block_size = 256;
    unsigned int grid_size = (num_points + block_size - 1) / block_size;

    distance2_point_cloud_kernel<<<grid_size, block_size>>>(
        d_points, num_points, d_triangles, num_triangles, d_min_distances
    );
    cudaDeviceSynchronize();

    cudaMemcpy(h_min_distances, d_min_distances, dist_size, cudaMemcpyDeviceToHost);

    cudaFree(d_points);
    cudaFree(d_triangles);
    cudaFree(d_min_distances);

    return 0;
}

int cuda_compute_signed_crossing_rays(
    const float* h_points,
    const float* h_ray_dirs,
    unsigned int num_points,
    const float* h_triangles,
    unsigned int num_triangles,
    int* h_crossing_counts
) {
    if (num_points == 0) return 0;
    if (num_triangles == 0) {
        for (unsigned int i = 0; i < num_points; ++i) {
            h_crossing_counts[i] = 0;
        }
        return 0;
    }

    size_t points_size = num_points * sizeof(float4);
    size_t dirs_size = num_points * sizeof(float4);
    size_t triangles_size = num_triangles * sizeof(GpuTriangle);
    size_t count_size = num_points * sizeof(int);

    float4* d_points = NULL;
    float4* d_ray_dirs = NULL;
    GpuTriangle* d_triangles = NULL;
    int* d_crossing_counts = NULL;

    if (cudaMalloc(&d_points, points_size) != cudaSuccess) return -1;
    if (cudaMalloc(&d_ray_dirs, dirs_size) != cudaSuccess) {
        cudaFree(d_points);
        return -2;
    }
    if (cudaMalloc(&d_triangles, triangles_size) != cudaSuccess) {
        cudaFree(d_points);
        cudaFree(d_ray_dirs);
        return -3;
    }
    if (cudaMalloc(&d_crossing_counts, count_size) != cudaSuccess) {
        cudaFree(d_points);
        cudaFree(d_ray_dirs);
        cudaFree(d_triangles);
        return -4;
    }

    cudaMemcpy(d_points, h_points, points_size, cudaMemcpyHostToDevice);
    cudaMemcpy(d_ray_dirs, h_ray_dirs, dirs_size, cudaMemcpyHostToDevice);
    cudaMemcpy(d_triangles, h_triangles, triangles_size, cudaMemcpyHostToDevice);

    unsigned int block_size = 256;
    unsigned int grid_size = (num_points + block_size - 1) / block_size;

    signed_crossing_rays_kernel<<<grid_size, block_size>>>(
        d_points, d_ray_dirs, num_points, d_triangles, num_triangles, d_crossing_counts
    );
    cudaDeviceSynchronize();

    cudaMemcpy(h_crossing_counts, d_crossing_counts, count_size, cudaMemcpyDeviceToHost);

    cudaFree(d_points);
    cudaFree(d_ray_dirs);
    cudaFree(d_triangles);
    cudaFree(d_crossing_counts);

    return 0;
}

}
