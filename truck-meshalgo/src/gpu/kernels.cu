#include <cuda_runtime.h>
#include <cuda.h>
#include <float.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "kernels_sm86_ptx.h"

struct GpuTriangle {
    float4 v0;
    float4 v1;
    float4 v2;
};

// ============================================================================
// Math Utilities
// ============================================================================

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

// Ampere SM 8.6 hardware async copy instructions
__device__ __forceinline__ void cp_async_16(void* smem_ptr, const void* gmem_ptr) {
    unsigned int smem_addr = __cvta_generic_to_shared(smem_ptr);
    asm volatile("cp.async.ca.shared.global [%0], [%1], 16;\n" : : "r"(smem_addr), "l"(gmem_ptr));
}
__device__ __forceinline__ void cp_async_commit() {
    asm volatile("cp.async.commit_group;\n");
}
__device__ __forceinline__ void cp_async_wait_all() {
    asm volatile("cp.async.wait_all;\n");
}

// ============================================================================
// Baseline Kernels
// ============================================================================

__device__ float distance2_point_triangle_baseline(float3 point, float3 v0, float3 v1, float3 v2) {
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

__device__ int check_crossing_baseline(float3 origin, float3 dir, float3 v0, float3 v1, float3 v2) {
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

__global__ void distance2_point_cloud_kernel_baseline(
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
        float d2 = distance2_point_triangle_baseline(pt, v0, v1, v2);
        min_d2 = fminf(min_d2, d2);
    }
    min_distances[idx] = min_d2;
}

__global__ void signed_crossing_rays_kernel_baseline(
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
        count += check_crossing_baseline(origin, dir, v0, v1, v2);
    }
    crossing_counts[idx] = count;
}

// ============================================================================
// Iteration 3: Ampere SM 8.6 Optimized Handcrafted Kernels
// ============================================================================

struct PrecomputedTriangleDist {
    float3 v0;
    float3 v1;
    float3 v2;
    float3 ab;
    float3 bc;
    float3 ca;
    float3 nor;
    float3 e0_nor;
    float3 e1_nor;
    float3 e2_nor;
    float nor_len2;
    float inv_nor_len2;
    float inv_dot_ab;
    float inv_dot_bc;
    float inv_dot_ca;
};

#define SM86_TILE_SIZE 64

extern "C" __global__ void distance2_point_cloud_sm86(
    const float4* __restrict__ points,
    unsigned int num_points,
    const GpuTriangle* __restrict__ triangles,
    unsigned int num_triangles,
    float* __restrict__ min_distances
) {
    __shared__ float4 s_raw[SM86_TILE_SIZE * 3];
    __shared__ PrecomputedTriangleDist s_precomp[SM86_TILE_SIZE];

    unsigned int idx = blockIdx.x * blockDim.x + threadIdx.x;
    float3 pt = make_float3(0.0f, 0.0f, 0.0f);
    if (idx < num_points) {
        float4 p4 = points[idx];
        pt = make_float3(p4.x, p4.y, p4.z);
    }
    float min_d2 = 1.0e30f;

    const float4* g_tri_f4 = reinterpret_cast<const float4*>(triangles);

    for (unsigned int t = 0; t < num_triangles; t += SM86_TILE_SIZE) {
        unsigned int tile_tri_count = min((unsigned int)SM86_TILE_SIZE, num_triangles - t);
        unsigned int tile_f4_count = tile_tri_count * 3;

        // Cooperative asynchronous copy directly into shared memory via Ampere cp.async
        for (unsigned int l = threadIdx.x; l < tile_f4_count; l += blockDim.x) {
            cp_async_16(&s_raw[l], &g_tri_f4[t * 3 + l]);
        }
        cp_async_commit();
        cp_async_wait_all();
        __syncthreads();

        // Cooperative expansion of triangle invariants in shared memory
        if (threadIdx.x < tile_tri_count) {
            float4 v0_4 = s_raw[threadIdx.x * 3 + 0];
            float4 v1_4 = s_raw[threadIdx.x * 3 + 1];
            float4 v2_4 = s_raw[threadIdx.x * 3 + 2];
            float3 v0 = make_float3(v0_4.x, v0_4.y, v0_4.z);
            float3 v1 = make_float3(v1_4.x, v1_4.y, v1_4.z);
            float3 v2 = make_float3(v2_4.x, v2_4.y, v2_4.z);

            float3 ab = sub(v1, v0);
            float3 bc = sub(v2, v1);
            float3 ca = sub(v0, v2);
            float3 nor = cross_product(ab, ca);

            s_precomp[threadIdx.x].v0 = v0;
            s_precomp[threadIdx.x].v1 = v1;
            s_precomp[threadIdx.x].v2 = v2;
            s_precomp[threadIdx.x].ab = ab;
            s_precomp[threadIdx.x].bc = bc;
            s_precomp[threadIdx.x].ca = ca;
            s_precomp[threadIdx.x].nor = nor;
            s_precomp[threadIdx.x].e0_nor = cross_product(ab, nor);
            s_precomp[threadIdx.x].e1_nor = cross_product(bc, nor);
            s_precomp[threadIdx.x].e2_nor = cross_product(ca, nor);

            float nor_len2 = dot_product(nor, nor);
            s_precomp[threadIdx.x].nor_len2 = nor_len2;
            s_precomp[threadIdx.x].inv_nor_len2 = (nor_len2 > 1.0e-12f) ? (1.0f / nor_len2) : 0.0f;

            float dot_ab = dot_product(ab, ab);
            float dot_bc = dot_product(bc, bc);
            float dot_ca = dot_product(ca, ca);
            s_precomp[threadIdx.x].inv_dot_ab = 1.0f / fmaxf(dot_ab, 1.0e-12f);
            s_precomp[threadIdx.x].inv_dot_bc = 1.0f / fmaxf(dot_bc, 1.0e-12f);
            s_precomp[threadIdx.x].inv_dot_ca = 1.0f / fmaxf(dot_ca, 1.0e-12f);
        }
        __syncthreads();

        if (idx < num_points) {
            #pragma unroll 4
            for (unsigned int i = 0; i < tile_tri_count; ++i) {
                const PrecomputedTriangleDist& tri = s_precomp[i];
                float3 ap = sub(pt, tri.v0);
                float3 bp = sub(pt, tri.v1);
                float3 cp = sub(pt, tri.v2);

                float s0 = copysignf(1.0f, dot_product(tri.e0_nor, ap));
                float s1 = copysignf(1.0f, dot_product(tri.e1_nor, bp));
                float s2 = copysignf(1.0f, dot_product(tri.e2_nor, cp));
                float coef = s0 + s1 + s2;

                float d2;
                if (coef < 2.0f || tri.nor_len2 < 1.0e-12f) {
                    float t_a = fminf(fmaxf(dot_product(ap, tri.ab) * tri.inv_dot_ab, 0.0f), 1.0f);
                    float3 da = sub(ap, mul(tri.ab, t_a));
                    float d2_a = dot_product(da, da);

                    float t_b = fminf(fmaxf(dot_product(bp, tri.bc) * tri.inv_dot_bc, 0.0f), 1.0f);
                    float3 db = sub(bp, mul(tri.bc, t_b));
                    float d2_b = dot_product(db, db);

                    float t_c = fminf(fmaxf(dot_product(cp, tri.ca) * tri.inv_dot_ca, 0.0f), 1.0f);
                    float3 dc = sub(cp, mul(tri.ca, t_c));
                    float d2_c = dot_product(dc, dc);

                    d2 = fminf(fminf(d2_a, d2_b), d2_c);
                } else {
                    float d = dot_product(tri.nor, ap);
                    d2 = (d * d) * tri.inv_nor_len2;
                }
                min_d2 = fminf(min_d2, d2);
            }
        }
        __syncthreads();
    }

    if (idx < num_points) {
        min_distances[idx] = min_d2;
    }
}

struct RayTriPrecomp {
    float3 v0;
    float3 a;
    float3 b;
    float3 nor;
};

extern "C" __global__ void signed_crossing_rays_sm86(
    const float4* __restrict__ points,
    const float4* __restrict__ ray_dirs,
    unsigned int num_points,
    const GpuTriangle* __restrict__ triangles,
    unsigned int num_triangles,
    int* __restrict__ crossing_counts
) {
    __shared__ float4 s_raw[SM86_TILE_SIZE * 3];
    __shared__ RayTriPrecomp s_precomp[SM86_TILE_SIZE];

    unsigned int idx = blockIdx.x * blockDim.x + threadIdx.x;
    float3 origin = make_float3(0.0f, 0.0f, 0.0f);
    float3 dir = make_float3(0.0f, 0.0f, 0.0f);
    if (idx < num_points) {
        float4 p4 = points[idx];
        float4 d4 = ray_dirs[idx];
        origin = make_float3(p4.x, p4.y, p4.z);
        dir = make_float3(d4.x, d4.y, d4.z);
    }
    int count = 0;

    const float4* g_tri_f4 = reinterpret_cast<const float4*>(triangles);

    for (unsigned int t = 0; t < num_triangles; t += SM86_TILE_SIZE) {
        unsigned int tile_tri_count = min((unsigned int)SM86_TILE_SIZE, num_triangles - t);
        unsigned int tile_f4_count = tile_tri_count * 3;

        for (unsigned int l = threadIdx.x; l < tile_f4_count; l += blockDim.x) {
            cp_async_16(&s_raw[l], &g_tri_f4[t * 3 + l]);
        }
        cp_async_commit();
        cp_async_wait_all();
        __syncthreads();

        if (threadIdx.x < tile_tri_count) {
            float4 v0_4 = s_raw[threadIdx.x * 3 + 0];
            float4 v1_4 = s_raw[threadIdx.x * 3 + 1];
            float4 v2_4 = s_raw[threadIdx.x * 3 + 2];
            float3 v0 = make_float3(v0_4.x, v0_4.y, v0_4.z);
            float3 v1 = make_float3(v1_4.x, v1_4.y, v1_4.z);
            float3 v2 = make_float3(v2_4.x, v2_4.y, v2_4.z);
            float3 a = sub(v0, v1);
            float3 b = sub(v0, v2);
            s_precomp[threadIdx.x].v0 = v0;
            s_precomp[threadIdx.x].a = a;
            s_precomp[threadIdx.x].b = b;
            s_precomp[threadIdx.x].nor = cross_product(a, b);
        }
        __syncthreads();

        if (idx < num_points) {
            #pragma unroll 4
            for (unsigned int i = 0; i < tile_tri_count; ++i) {
                const RayTriPrecomp& tri = s_precomp[i];
                float3 b_cross_dir = cross_product(tri.b, dir);
                float det = dot_product(tri.a, b_cross_dir);
                if (fabsf(det) >= 1.0e-7f) {
                    float inv_det = 1.0f / det;
                    float3 rhs = sub(tri.v0, origin);
                    float u = dot_product(rhs, b_cross_dir) * inv_det;
                    if (u > 0.0f && u < 1.0f) {
                        float3 rhs_cross_dir = cross_product(rhs, dir);
                        float v = dot_product(tri.a, rhs_cross_dir) * inv_det;
                        if (v > 0.0f && (u + v) < 1.0f) {
                            float t_val = dot_product(tri.nor, rhs) * inv_det;
                            if (t_val > 0.0f) {
                                count += (det > 0.0f) ? 1 : -1;
                            }
                        }
                    }
                }
            }
        }
        __syncthreads();
    }

    if (idx < num_points) {
        crossing_counts[idx] = count;
    }
}

// ============================================================================
// Driver API Dynamic PTX Module Loading
// ============================================================================

static CUmodule g_driver_ptx_module = nullptr;
static CUfunction g_driver_ptx_dist_fn = nullptr;
static CUfunction g_driver_ptx_cross_fn = nullptr;
static bool g_driver_initialized = false;

static void ensure_driver_ptx_loaded() {
    if (g_driver_initialized) return;

    cuInit(0);
    // Retain primary context for current CUDA device
    int current_dev = 0;
    cudaGetDevice(&current_dev);
    CUcontext ctx;
    cuDevicePrimaryCtxRetain(&ctx, current_dev);
    cuCtxSetCurrent(ctx);

    char* ptx_null_term = (char*)malloc(truck_meshalgo_src_gpu_kernels_sm86_ptx_len + 1);
    if (!ptx_null_term) return;
    memcpy(ptx_null_term, truck_meshalgo_src_gpu_kernels_sm86_ptx, truck_meshalgo_src_gpu_kernels_sm86_ptx_len);
    ptx_null_term[truck_meshalgo_src_gpu_kernels_sm86_ptx_len] = '\0';

    CUresult res = cuModuleLoadData(&g_driver_ptx_module, ptx_null_term);
    free(ptx_null_term);
    if (res == CUDA_SUCCESS) {
        cuModuleGetFunction(&g_driver_ptx_dist_fn, g_driver_ptx_module, "distance2_point_cloud_sm86");
        cuModuleGetFunction(&g_driver_ptx_cross_fn, g_driver_ptx_module, "signed_crossing_rays_sm86");
        g_driver_initialized = true;
    } else {
        fprintf(stderr, "[truck_cuda] Warning: cuModuleLoadData failed with error %d\n", res);
    }
}

// ============================================================================
// Device Initialization & Reusable Device Scratchpads
// ============================================================================

static bool g_device_configured = false;

static void ensure_best_device() {
    if (g_device_configured) return;

    int current_dev = 0;
    if (cudaGetDevice(&current_dev) != cudaSuccess) {
        int device_count = 0;
        if (cudaGetDeviceCount(&device_count) == cudaSuccess && device_count > 0) {
            int best_dev = 0;
            int max_sms = 0;
            for (int i = 0; i < device_count; ++i) {
                int sms = 0;
                if (cudaDeviceGetAttribute(&sms, cudaDevAttrMultiProcessorCount, i) == cudaSuccess) {
                    if (sms > max_sms) {
                        max_sms = sms;
                        best_dev = i;
                    }
                }
            }
            cudaSetDevice(best_dev);
        }
    }
    g_device_configured = true;
}

// Reusable scratchpad for point-mesh distance queries
static void* g_dist_scratch = nullptr;
static size_t g_dist_scratch_cap = 0;

static void* get_dist_scratch(size_t bytes) {
    if (bytes > g_dist_scratch_cap) {
        if (g_dist_scratch) cudaFree(g_dist_scratch);
        if (cudaMalloc(&g_dist_scratch, bytes) != cudaSuccess) {
            g_dist_scratch = nullptr;
            g_dist_scratch_cap = 0;
            return nullptr;
        }
        g_dist_scratch_cap = bytes;
    }
    return g_dist_scratch;
}

// Reusable scratchpad for ray crossing queries
static void* g_cross_scratch = nullptr;
static size_t g_cross_scratch_cap = 0;

static void* get_cross_scratch(size_t bytes) {
    if (bytes > g_cross_scratch_cap) {
        if (g_cross_scratch) cudaFree(g_cross_scratch);
        if (cudaMalloc(&g_cross_scratch, bytes) != cudaSuccess) {
            g_cross_scratch = nullptr;
            g_cross_scratch_cap = 0;
            return nullptr;
        }
        g_cross_scratch_cap = bytes;
    }
    return g_cross_scratch;
}

// ============================================================================
// Exported C API Functions
// ============================================================================

extern "C" {

// Optimized SM 8.6 kernel execution with buffer pooling (Default)
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
    ensure_best_device();

    size_t points_size = num_points * sizeof(float4);
    size_t triangles_size = num_triangles * sizeof(GpuTriangle);
    size_t dist_size = num_points * sizeof(float);
    size_t total_size = points_size + triangles_size + dist_size;

    void* scratch = get_dist_scratch(total_size);
    if (!scratch) return -1;

    float4* d_points = (float4*)scratch;
    GpuTriangle* d_triangles = (GpuTriangle*)((char*)d_points + points_size);
    float* d_min_distances = (float*)((char*)d_triangles + triangles_size);

    cudaMemcpy(d_points, h_points, points_size, cudaMemcpyHostToDevice);
    cudaMemcpy(d_triangles, h_triangles, triangles_size, cudaMemcpyHostToDevice);

    unsigned int block_size = 256;
    unsigned int grid_size = (num_points + block_size - 1) / block_size;

    distance2_point_cloud_sm86<<<grid_size, block_size>>>(
        d_points, num_points, d_triangles, num_triangles, d_min_distances
    );
    cudaDeviceSynchronize();

    cudaMemcpy(h_min_distances, d_min_distances, dist_size, cudaMemcpyDeviceToHost);
    return 0;
}

// Baseline unoptimized kernel with individual per-call allocations
int cuda_compute_distance2_point_cloud_baseline(
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
    ensure_best_device();

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

    distance2_point_cloud_kernel_baseline<<<grid_size, block_size>>>(
        d_points, num_points, d_triangles, num_triangles, d_min_distances
    );
    cudaDeviceSynchronize();

    cudaMemcpy(h_min_distances, d_min_distances, dist_size, cudaMemcpyDeviceToHost);

    cudaFree(d_points);
    cudaFree(d_triangles);
    cudaFree(d_min_distances);
    return 0;
}

// PTX module execution via bare-metal CUDA Driver API
int cuda_compute_distance2_point_cloud_ptx(
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
    ensure_best_device();
    ensure_driver_ptx_loaded();
    if (!g_driver_ptx_dist_fn) return -10;

    size_t points_size = num_points * sizeof(float4);
    size_t triangles_size = num_triangles * sizeof(GpuTriangle);
    size_t dist_size = num_points * sizeof(float);
    size_t total_size = points_size + triangles_size + dist_size;

    void* scratch = get_dist_scratch(total_size);
    if (!scratch) return -1;

    float4* d_points = (float4*)scratch;
    GpuTriangle* d_triangles = (GpuTriangle*)((char*)d_points + points_size);
    float* d_min_distances = (float*)((char*)d_triangles + triangles_size);

    cudaMemcpy(d_points, h_points, points_size, cudaMemcpyHostToDevice);
    cudaMemcpy(d_triangles, h_triangles, triangles_size, cudaMemcpyHostToDevice);

    unsigned int block_size = 256;
    unsigned int grid_size = (num_points + block_size - 1) / block_size;

    void* kernel_params[] = {
        &d_points,
        (void*)&num_points,
        &d_triangles,
        (void*)&num_triangles,
        &d_min_distances
    };

    CUresult res = cuLaunchKernel(
        g_driver_ptx_dist_fn,
        grid_size, 1, 1,
        block_size, 1, 1,
        0, 0,
        kernel_params,
        nullptr
    );
    if (res != CUDA_SUCCESS) return (int)res;
    cudaDeviceSynchronize();

    cudaMemcpy(h_min_distances, d_min_distances, dist_size, cudaMemcpyDeviceToHost);
    return 0;
}

// Optimized SM 8.6 ray crossing with buffer pooling (Default)
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
    ensure_best_device();

    size_t points_size = num_points * sizeof(float4);
    size_t dirs_size = num_points * sizeof(float4);
    size_t triangles_size = num_triangles * sizeof(GpuTriangle);
    size_t count_size = num_points * sizeof(int);
    size_t total_size = points_size + dirs_size + triangles_size + count_size;

    void* scratch = get_cross_scratch(total_size);
    if (!scratch) return -1;

    float4* d_points = (float4*)scratch;
    float4* d_ray_dirs = (float4*)((char*)d_points + points_size);
    GpuTriangle* d_triangles = (GpuTriangle*)((char*)d_ray_dirs + dirs_size);
    int* d_crossing_counts = (int*)((char*)d_triangles + triangles_size);

    cudaMemcpy(d_points, h_points, points_size, cudaMemcpyHostToDevice);
    cudaMemcpy(d_ray_dirs, h_ray_dirs, dirs_size, cudaMemcpyHostToDevice);
    cudaMemcpy(d_triangles, h_triangles, triangles_size, cudaMemcpyHostToDevice);

    unsigned int block_size = 256;
    unsigned int grid_size = (num_points + block_size - 1) / block_size;

    signed_crossing_rays_sm86<<<grid_size, block_size>>>(
        d_points, d_ray_dirs, num_points, d_triangles, num_triangles, d_crossing_counts
    );
    cudaDeviceSynchronize();

    cudaMemcpy(h_crossing_counts, d_crossing_counts, count_size, cudaMemcpyDeviceToHost);
    return 0;
}

// Baseline ray crossing with per-call alloc/free
int cuda_compute_signed_crossing_rays_baseline(
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
    ensure_best_device();

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

    signed_crossing_rays_kernel_baseline<<<grid_size, block_size>>>(
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

// PTX ray crossing module execution via Driver API
int cuda_compute_signed_crossing_rays_ptx(
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
    ensure_best_device();
    ensure_driver_ptx_loaded();
    if (!g_driver_ptx_cross_fn) return -10;

    size_t points_size = num_points * sizeof(float4);
    size_t dirs_size = num_points * sizeof(float4);
    size_t triangles_size = num_triangles * sizeof(GpuTriangle);
    size_t count_size = num_points * sizeof(int);
    size_t total_size = points_size + dirs_size + triangles_size + count_size;

    void* scratch = get_cross_scratch(total_size);
    if (!scratch) return -1;

    float4* d_points = (float4*)scratch;
    float4* d_ray_dirs = (float4*)((char*)d_points + points_size);
    GpuTriangle* d_triangles = (GpuTriangle*)((char*)d_ray_dirs + dirs_size);
    int* d_crossing_counts = (int*)((char*)d_triangles + triangles_size);

    cudaMemcpy(d_points, h_points, points_size, cudaMemcpyHostToDevice);
    cudaMemcpy(d_ray_dirs, h_ray_dirs, dirs_size, cudaMemcpyHostToDevice);
    cudaMemcpy(d_triangles, h_triangles, triangles_size, cudaMemcpyHostToDevice);

    unsigned int block_size = 256;
    unsigned int grid_size = (num_points + block_size - 1) / block_size;

    void* kernel_params[] = {
        &d_points,
        &d_ray_dirs,
        (void*)&num_points,
        &d_triangles,
        (void*)&num_triangles,
        &d_crossing_counts
    };

    CUresult res = cuLaunchKernel(
        g_driver_ptx_cross_fn,
        grid_size, 1, 1,
        block_size, 1, 1,
        0, 0,
        kernel_params,
        nullptr
    );
    if (res != CUDA_SUCCESS) return (int)res;
    cudaDeviceSynchronize();

    cudaMemcpy(h_crossing_counts, d_crossing_counts, count_size, cudaMemcpyDeviceToHost);
    return 0;
}

// ============================================================================
// Candidate 1: Mesh Normal Generation & Smoothing
// ============================================================================

__device__ __forceinline__ float3 normalize_safe(float3 v) {
    float len2 = dot_product(v, v);
    if (len2 > 1.0e-12f) {
        float inv = rsqrtf(len2);
        return make_float3(v.x * inv, v.y * inv, v.z * inv);
    }
    return make_float3(0.0f, 0.0f, 0.0f);
}

extern "C" __global__ void truck_cuda_compute_face_normals_kernel(
    const GpuTriangle* __restrict__ triangles,
    unsigned int num_triangles,
    float4* __restrict__ face_normals
) {
    unsigned int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= num_triangles) return;

    GpuTriangle tri = triangles[idx];
    float3 v0 = make_float3(tri.v0.x, tri.v0.y, tri.v0.z);
    float3 v1 = make_float3(tri.v1.x, tri.v1.y, tri.v1.z);
    float3 v2 = make_float3(tri.v2.x, tri.v2.y, tri.v2.z);

    float3 nor = cross_product(sub(v1, v0), sub(v2, v0));
    float3 n = normalize_safe(nor);
    face_normals[idx] = make_float4(n.x, n.y, n.z, 0.0f);
}

extern "C" __global__ void truck_cuda_accumulate_smooth_normals_kernel(
    const float4* __restrict__ vertices,
    const uint3* __restrict__ faces,
    unsigned int num_faces,
    float* __restrict__ normal_accum_xyz
) {
    unsigned int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= num_faces) return;

    uint3 f = faces[idx];
    float4 v0_4 = vertices[f.x];
    float4 v1_4 = vertices[f.y];
    float4 v2_4 = vertices[f.z];
    float3 v0 = make_float3(v0_4.x, v0_4.y, v0_4.z);
    float3 v1 = make_float3(v1_4.x, v1_4.y, v1_4.z);
    float3 v2 = make_float3(v2_4.x, v2_4.y, v2_4.z);

    // Area-weighted unnormalized face normal
    float3 nor = cross_product(sub(v1, v0), sub(v2, v0));

    atomicAdd(&normal_accum_xyz[f.x * 3 + 0], nor.x);
    atomicAdd(&normal_accum_xyz[f.x * 3 + 1], nor.y);
    atomicAdd(&normal_accum_xyz[f.x * 3 + 2], nor.z);

    atomicAdd(&normal_accum_xyz[f.y * 3 + 0], nor.x);
    atomicAdd(&normal_accum_xyz[f.y * 3 + 1], nor.y);
    atomicAdd(&normal_accum_xyz[f.y * 3 + 2], nor.z);

    atomicAdd(&normal_accum_xyz[f.z * 3 + 0], nor.x);
    atomicAdd(&normal_accum_xyz[f.z * 3 + 1], nor.y);
    atomicAdd(&normal_accum_xyz[f.z * 3 + 2], nor.z);
}

extern "C" __global__ void truck_cuda_normalize_vertex_normals_kernel(
    const float* __restrict__ normal_accum_xyz,
    unsigned int num_vertices,
    float4* __restrict__ out_normals
) {
    unsigned int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= num_vertices) return;

    float3 n = make_float3(
        normal_accum_xyz[idx * 3 + 0],
        normal_accum_xyz[idx * 3 + 1],
        normal_accum_xyz[idx * 3 + 2]
    );
    float3 norm = normalize_safe(n);
    out_normals[idx] = make_float4(norm.x, norm.y, norm.z, 0.0f);
}

int truck_cuda_compute_face_normals(
    const float* h_triangles,
    unsigned int num_triangles,
    float* h_face_normals
) {
    if (num_triangles == 0) return 0;
    ensure_best_device();

    size_t tri_bytes = num_triangles * sizeof(GpuTriangle);
    size_t nor_bytes = num_triangles * sizeof(float4);

    GpuTriangle* d_triangles = NULL;
    float4* d_face_normals = NULL;

    if (cudaMalloc(&d_triangles, tri_bytes) != cudaSuccess) return -1;
    if (cudaMalloc(&d_face_normals, nor_bytes) != cudaSuccess) {
        cudaFree(d_triangles);
        return -2;
    }

    cudaMemcpy(d_triangles, h_triangles, tri_bytes, cudaMemcpyHostToDevice);

    unsigned int block_size = 256;
    unsigned int grid_size = (num_triangles + block_size - 1) / block_size;
    truck_cuda_compute_face_normals_kernel<<<grid_size, block_size>>>(
        d_triangles, num_triangles, d_face_normals
    );
    cudaDeviceSynchronize();

    cudaMemcpy(h_face_normals, d_face_normals, nor_bytes, cudaMemcpyDeviceToHost);

    cudaFree(d_triangles);
    cudaFree(d_face_normals);
    return 0;
}

int truck_cuda_compute_smooth_normals(
    const float* h_vertices,
    unsigned int num_vertices,
    const unsigned int* h_faces,
    unsigned int num_faces,
    float* h_vertex_normals
) {
    if (num_vertices == 0 || num_faces == 0) return 0;
    ensure_best_device();

    size_t vert_bytes = num_vertices * sizeof(float4);
    size_t face_bytes = num_faces * sizeof(uint3);
    size_t accum_bytes = num_vertices * 3 * sizeof(float);
    size_t out_bytes = num_vertices * sizeof(float4);

    float4* d_vertices = NULL;
    uint3* d_faces = NULL;
    float* d_accum = NULL;
    float4* d_out_normals = NULL;

    if (cudaMalloc(&d_vertices, vert_bytes) != cudaSuccess) return -1;
    if (cudaMalloc(&d_faces, face_bytes) != cudaSuccess) {
        cudaFree(d_vertices);
        return -2;
    }
    if (cudaMalloc(&d_accum, accum_bytes) != cudaSuccess) {
        cudaFree(d_vertices);
        cudaFree(d_faces);
        return -3;
    }
    if (cudaMalloc(&d_out_normals, out_bytes) != cudaSuccess) {
        cudaFree(d_vertices);
        cudaFree(d_faces);
        cudaFree(d_accum);
        return -4;
    }

    cudaMemcpy(d_vertices, h_vertices, vert_bytes, cudaMemcpyHostToDevice);
    cudaMemcpy(d_faces, h_faces, face_bytes, cudaMemcpyHostToDevice);
    cudaMemset(d_accum, 0, accum_bytes);

    unsigned int block_size = 256;
    unsigned int grid_faces = (num_faces + block_size - 1) / block_size;
    truck_cuda_accumulate_smooth_normals_kernel<<<grid_faces, block_size>>>(
        d_vertices, d_faces, num_faces, d_accum
    );

    unsigned int grid_verts = (num_vertices + block_size - 1) / block_size;
    truck_cuda_normalize_vertex_normals_kernel<<<grid_verts, block_size>>>(
        d_accum, num_vertices, d_out_normals
    );
    cudaDeviceSynchronize();

    cudaMemcpy(h_vertex_normals, d_out_normals, out_bytes, cudaMemcpyDeviceToHost);

    cudaFree(d_vertices);
    cudaFree(d_faces);
    cudaFree(d_accum);
    cudaFree(d_out_normals);
    return 0;
}

// ============================================================================
// Candidate 2: 2D Trimming Loop Inclusion Testing (PolyBoundary::include)
// ============================================================================

extern "C" __global__ void truck_cuda_trimming_loop_include_kernel(
    const float4* __restrict__ segments,
    unsigned int num_segments,
    const float2* __restrict__ points,
    const float2* __restrict__ ray_dirs,
    unsigned int num_points,
    int* __restrict__ inside_flags
) {
    __shared__ float4 s_segments[SM86_TILE_SIZE];

    unsigned int idx = blockIdx.x * blockDim.x + threadIdx.x;
    float2 c = make_float2(0.0f, 0.0f);
    float2 r = make_float2(0.0f, 0.0f);
    if (idx < num_points) {
        c = points[idx];
        r = ray_dirs[idx];
    }
    int counter = 0;

    for (unsigned int t = 0; t < num_segments; t += SM86_TILE_SIZE) {
        unsigned int tile_seg_count = min((unsigned int)SM86_TILE_SIZE, num_segments - t);

        for (unsigned int l = threadIdx.x; l < tile_seg_count; l += blockDim.x) {
            cp_async_16(&s_segments[l], &segments[t + l]);
        }
        cp_async_commit();
        cp_async_wait_all();
        __syncthreads();

        if (idx < num_points) {
            #pragma unroll 4
            for (unsigned int s = 0; s < tile_seg_count; ++s) {
                float4 seg = s_segments[s];
                float2 a = make_float2(seg.x - c.x, seg.y - c.y);
                float2 b = make_float2(seg.z - c.x, seg.w - c.y);

                float s0 = r.x * a.y - r.y * a.x;
                float s1 = r.x * b.y - r.y * b.x;
                float s2 = a.x * b.y - a.y * b.x;

                float denom = s1 - s0;
                if (fabsf(denom) > 1.0e-12f) {
                    float x = s2 / denom;
                    if (fabsf(x) < 1.0e-6f && s0 * s1 < 0.0f) {
                        // Point on segment boundary
                    } else if (x > 0.0f && s0 <= 0.0f && s1 > 0.0f) {
                        counter += 1;
                    } else if (x > 0.0f && s0 >= 0.0f && s1 < 0.0f) {
                        counter -= 1;
                    }
                }
            }
        }
        __syncthreads();
    }

    if (idx < num_points) {
        inside_flags[idx] = (counter > 0) ? 1 : 0;
    }
}

int truck_cuda_trimming_loop_include(
    const float* h_segments,
    unsigned int num_segments,
    const float* h_points,
    const float* h_ray_dirs,
    unsigned int num_points,
    int* h_inside_flags
) {
    if (num_points == 0) return 0;
    if (num_segments == 0) {
        for (unsigned int i = 0; i < num_points; ++i) h_inside_flags[i] = 0;
        return 0;
    }
    ensure_best_device();

    size_t seg_bytes = num_segments * sizeof(float4);
    size_t pt_bytes = num_points * sizeof(float2);
    size_t out_bytes = num_points * sizeof(int);

    float4* d_segments = NULL;
    float2* d_points = NULL;
    float2* d_ray_dirs = NULL;
    int* d_inside_flags = NULL;

    if (cudaMalloc(&d_segments, seg_bytes) != cudaSuccess) return -1;
    if (cudaMalloc(&d_points, pt_bytes) != cudaSuccess) {
        cudaFree(d_segments);
        return -2;
    }
    if (cudaMalloc(&d_ray_dirs, pt_bytes) != cudaSuccess) {
        cudaFree(d_segments);
        cudaFree(d_points);
        return -3;
    }
    if (cudaMalloc(&d_inside_flags, out_bytes) != cudaSuccess) {
        cudaFree(d_segments);
        cudaFree(d_points);
        cudaFree(d_ray_dirs);
        return -4;
    }

    cudaMemcpy(d_segments, h_segments, seg_bytes, cudaMemcpyHostToDevice);
    cudaMemcpy(d_points, h_points, pt_bytes, cudaMemcpyHostToDevice);
    cudaMemcpy(d_ray_dirs, h_ray_dirs, pt_bytes, cudaMemcpyHostToDevice);

    unsigned int block_size = 256;
    unsigned int grid_size = (num_points + block_size - 1) / block_size;
    truck_cuda_trimming_loop_include_kernel<<<grid_size, block_size>>>(
        d_segments, num_segments, d_points, d_ray_dirs, num_points, d_inside_flags
    );
    cudaDeviceSynchronize();

    cudaMemcpy(h_inside_flags, d_inside_flags, out_bytes, cudaMemcpyDeviceToHost);

    cudaFree(d_segments);
    cudaFree(d_points);
    cudaFree(d_ray_dirs);
    cudaFree(d_inside_flags);
    return 0;
}

// ============================================================================
// Candidate 3: Triangle-Triangle Collision & Interference
// ============================================================================

__device__ __forceinline__ float dist2_3d(float3 a, float3 b) {
    float3 d = sub(a, b);
    return dot_product(d, d);
}

__device__ bool collide_seg_triangle_dev(
    float3 p, float3 q,
    float3 v0, float3 v1, float3 v2,
    float3* out_h
) {
    float3 ab = sub(v1, v0);
    float3 bc = sub(v2, v1);
    float3 ca = sub(v0, v2);
    float3 nor = cross_product(ab, ca);
    float nor_len2 = dot_product(nor, nor);
    if (nor_len2 < 1.0e-12f) return false;

    float3 ap = sub(p, v0);
    float3 aq = sub(q, v0);
    float dotapnor = dot_product(ap, nor);
    float dotaqnor = dot_product(aq, nor);
    if (dotapnor * dotaqnor > 0.0f) return false;

    float denom = dotapnor - dotaqnor;
    if (fabsf(denom) < 1.0e-12f) return false;

    float t = dotapnor / denom;
    float3 h = make_float3(
        p.x + t * (q.x - p.x),
        p.y + t * (q.y - p.y),
        p.z + t * (q.z - p.z)
    );

    float s0 = copysignf(1.0f, dot_product(cross_product(ab, nor), sub(h, v0)) + 1.0e-6f);
    float s1 = copysignf(1.0f, dot_product(cross_product(bc, nor), sub(h, v1)) + 1.0e-6f);
    float s2 = copysignf(1.0f, dot_product(cross_product(ca, nor), sub(h, v2)) + 1.0e-6f);

    if (s0 + s1 + s2 >= 2.0f) {
        *out_h = h;
        return true;
    }
    return false;
}

__device__ bool collide_triangles_dev(
    float3 a0, float3 a1, float3 a2,
    float3 b0, float3 b1, float3 b2,
    float3* out_p, float3* out_q
) {
    float3 p, q;
    bool has_p = false;
    bool has_q = false;

    float3 seg_pairs[6][2] = {
        {a0, a1}, {a1, a2}, {a2, a0},
        {b0, b1}, {b1, b2}, {b2, b0}
    };

    for (int e = 0; e < 6; ++e) {
        float3 pt;
        bool hit;
        if (e < 3) {
            hit = collide_seg_triangle_dev(seg_pairs[e][0], seg_pairs[e][1], b0, b1, b2, &pt);
        } else {
            hit = collide_seg_triangle_dev(seg_pairs[e][0], seg_pairs[e][1], a0, a1, a2, &pt);
        }

        if (hit) {
            if (!has_p) {
                p = pt;
                has_p = true;
            } else if (!has_q) {
                q = pt;
                has_q = true;
            } else {
                float d0 = dist2_3d(pt, p);
                float d1 = dist2_3d(pt, q);
                float d2 = dist2_3d(p, q);
                if (d2 < d0) {
                    q = pt;
                } else if (d2 < d1) {
                    p = pt;
                }
            }
        }
    }

    if (has_p && has_q) {
        *out_p = p;
        *out_q = q;
        return true;
    }
    return false;
}

extern "C" __global__ void truck_cuda_collide_triangles_kernel(
    const GpuTriangle* __restrict__ mesh0,
    unsigned int num_tri0,
    const GpuTriangle* __restrict__ mesh1,
    unsigned int num_tri1,
    int return_early_on_first,
    int* __restrict__ num_interferences,
    float4* __restrict__ out_segments,
    unsigned int max_segments
) {
    unsigned int idx0 = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx0 >= num_tri0) return;

    if (return_early_on_first && *num_interferences > 0) return;

    GpuTriangle t0 = mesh0[idx0];
    float3 a0 = make_float3(t0.v0.x, t0.v0.y, t0.v0.z);
    float3 a1 = make_float3(t0.v1.x, t0.v1.y, t0.v1.z);
    float3 a2 = make_float3(t0.v2.x, t0.v2.y, t0.v2.z);

    float a_min_x = fminf(fminf(a0.x, a1.x), a2.x);
    float a_max_x = fmaxf(fmaxf(a0.x, a1.x), a2.x);
    float a_min_y = fminf(fminf(a0.y, a1.y), a2.y);
    float a_max_y = fmaxf(fmaxf(a0.y, a1.y), a2.y);
    float a_min_z = fminf(fminf(a0.z, a1.z), a2.z);
    float a_max_z = fmaxf(fmaxf(a0.z, a1.z), a2.z);

    for (unsigned int idx1 = 0; idx1 < num_tri1; ++idx1) {
        if (return_early_on_first && *num_interferences > 0) return;

        GpuTriangle t1 = mesh1[idx1];
        float3 b0 = make_float3(t1.v0.x, t1.v0.y, t1.v0.z);
        float3 b1 = make_float3(t1.v1.x, t1.v1.y, t1.v1.z);
        float3 b2 = make_float3(t1.v2.x, t1.v2.y, t1.v2.z);

        float b_min_x = fminf(fminf(b0.x, b1.x), b2.x);
        float b_max_x = fmaxf(fmaxf(b0.x, b1.x), b2.x);
        float b_min_y = fminf(fminf(b0.y, b1.y), b2.y);
        float b_max_y = fmaxf(fmaxf(b0.y, b1.y), b2.y);
        float b_min_z = fminf(fminf(b0.z, b1.z), b2.z);
        float b_max_z = fmaxf(fmaxf(b0.z, b1.z), b2.z);

        // Bounding box disjoint check
        if (a_max_x < b_min_x || b_max_x < a_min_x ||
            a_max_y < b_min_y || b_max_y < a_min_y ||
            a_max_z < b_min_z || b_max_z < a_min_z) {
            continue;
        }

        float3 p, q;
        if (collide_triangles_dev(a0, a1, a2, b0, b1, b2, &p, &q)) {
            int slot = atomicAdd(num_interferences, 1);
            if (out_segments && slot < (int)max_segments) {
                out_segments[slot * 2 + 0] = make_float4(p.x, p.y, p.z, 0.0f);
                out_segments[slot * 2 + 1] = make_float4(q.x, q.y, q.z, 0.0f);
            }
            if (return_early_on_first) return;
        }
    }
}

int truck_cuda_collide_triangles(
    const float* h_mesh0,
    unsigned int num_tri0,
    const float* h_mesh1,
    unsigned int num_tri1,
    int return_early_on_first,
    int* h_num_interferences,
    float* h_out_segments,
    unsigned int max_segments
) {
    if (num_tri0 == 0 || num_tri1 == 0) {
        *h_num_interferences = 0;
        return 0;
    }
    ensure_best_device();

    size_t mesh0_bytes = num_tri0 * sizeof(GpuTriangle);
    size_t mesh1_bytes = num_tri1 * sizeof(GpuTriangle);
    size_t segs_bytes = max_segments * 2 * sizeof(float4);

    GpuTriangle* d_mesh0 = NULL;
    GpuTriangle* d_mesh1 = NULL;
    int* d_num_interferences = NULL;
    float4* d_out_segments = NULL;

    if (cudaMalloc(&d_mesh0, mesh0_bytes) != cudaSuccess) return -1;
    if (cudaMalloc(&d_mesh1, mesh1_bytes) != cudaSuccess) {
        cudaFree(d_mesh0);
        return -2;
    }
    if (cudaMalloc(&d_num_interferences, sizeof(int)) != cudaSuccess) {
        cudaFree(d_mesh0);
        cudaFree(d_mesh1);
        return -3;
    }
    if (max_segments > 0) {
        if (cudaMalloc(&d_out_segments, segs_bytes) != cudaSuccess) {
            cudaFree(d_mesh0);
            cudaFree(d_mesh1);
            cudaFree(d_num_interferences);
            return -4;
        }
    }

    cudaMemcpy(d_mesh0, h_mesh0, mesh0_bytes, cudaMemcpyHostToDevice);
    cudaMemcpy(d_mesh1, h_mesh1, mesh1_bytes, cudaMemcpyHostToDevice);
    cudaMemset(d_num_interferences, 0, sizeof(int));

    unsigned int block_size = 256;
    unsigned int grid_size = (num_tri0 + block_size - 1) / block_size;
    truck_cuda_collide_triangles_kernel<<<grid_size, block_size>>>(
        d_mesh0, num_tri0, d_mesh1, num_tri1,
        return_early_on_first,
        d_num_interferences,
        d_out_segments, max_segments
    );
    cudaDeviceSynchronize();

    cudaMemcpy(h_num_interferences, d_num_interferences, sizeof(int), cudaMemcpyDeviceToHost);
    int count = *h_num_interferences;
    if (count > (int)max_segments) count = (int)max_segments;
    if (count > 0 && h_out_segments && d_out_segments) {
        cudaMemcpy(h_out_segments, d_out_segments, count * 2 * sizeof(float4), cudaMemcpyDeviceToHost);
    }

    cudaFree(d_mesh0);
    cudaFree(d_mesh1);
    cudaFree(d_num_interferences);
    if (d_out_segments) cudaFree(d_out_segments);
    return 0;
}

// ============================================================================
// Candidate 4: Batch B-Spline Surface Evaluation
// ============================================================================

__device__ float inv_or_zero_f(float delta) {
    return (fabsf(delta) > 1.0e-12f) ? (1.0f / delta) : 0.0f;
}

__device__ int truck_cuda_bspline_basis_functions(
    float t,
    int degree,
    int der_rank,
    const float* __restrict__ knots,
    int num_knots,
    float* __restrict__ eval_out,
    int* __restrict__ out_len
) {
    int n = num_knots - 1;

    // Floor search
    int idx = 0;
    while (idx < n && knots[idx + 1] <= t) {
        idx++;
    }
    if (idx == n) {
        int mult = 0;
        for (int k = n; k >= 0; --k) {
            if (fabsf(knots[k] - knots[n]) < 1.0e-6f) mult++;
            else break;
        }
        idx = n - mult;
    }

    int global_base = (idx >= degree) ? (idx - degree) : 0;
    float eval[16] = {0.0f};
    eval[idx - global_base] = 1.0f;

    for (int k = 1; k <= (degree - der_rank); ++k) {
        int base = (idx >= k) ? (idx - k) : 0;
        float delta = knots[base + k] - knots[base];
        float a = inv_or_zero_f(delta) * (t - knots[base]);
        int max_i = (idx < (n - k - 1)) ? idx : (n - k - 1);
        for (int i = base; i <= max_i; ++i) {
            float delta2 = knots[i + k + 1] - knots[i + 1];
            float b = inv_or_zero_f(delta2) * (knots[i + k + 1] - t);
            int j = i - global_base;
            eval[j] = a * eval[j] + b * eval[j + 1];
            a = 1.0f - b;
        }
    }

    for (int k = (degree - der_rank + 1); k <= degree; ++k) {
        int base = (idx >= k) ? (idx - k) : 0;
        float delta = knots[base + k] - knots[base];
        float a = inv_or_zero_f(delta);
        int max_i = (idx < (n - k - 1)) ? idx : (n - k - 1);
        for (int i = base; i <= max_i; ++i) {
            float delta2 = knots[i + k + 1] - knots[i + 1];
            float b = inv_or_zero_f(delta2);
            int j = i - global_base;
            eval[j] = (a * eval[j] - b * eval[j + 1]) * (float)k;
            a = b;
        }
    }

    int len = degree + 1;
    if (n - degree - global_base < len) len = n - degree - global_base;
    *out_len = len;

    for (int i = 0; i < len; ++i) {
        eval_out[i] = eval[i];
    }
    return global_base;
}

extern "C" __global__ void truck_cuda_evaluate_bspline_surface_kernel(
    const float* __restrict__ u_knots,
    unsigned int num_u_knots,
    const float* __restrict__ v_knots,
    unsigned int num_v_knots,
    unsigned int u_degree,
    unsigned int v_degree,
    const float4* __restrict__ control_points,
    unsigned int cp_stride_v,
    const float2* __restrict__ uv_points,
    unsigned int num_points,
    float4* __restrict__ out_points,
    float4* __restrict__ out_normals
) {
    unsigned int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= num_points) return;

    float2 uv = uv_points[idx];

    float Nu[16], dNu[16];
    float Nv[16], dNv[16];
    int u_len = 0, u_der_len = 0;
    int v_len = 0, v_der_len = 0;

    int u_base = truck_cuda_bspline_basis_functions(uv.x, (int)u_degree, 0, u_knots, (int)num_u_knots, Nu, &u_len);
    truck_cuda_bspline_basis_functions(uv.x, (int)u_degree, 1, u_knots, (int)num_u_knots, dNu, &u_der_len);

    int v_base = truck_cuda_bspline_basis_functions(uv.y, (int)v_degree, 0, v_knots, (int)num_v_knots, Nv, &v_len);
    truck_cuda_bspline_basis_functions(uv.y, (int)v_degree, 1, v_knots, (int)num_v_knots, dNv, &v_der_len);

    float3 pt = make_float3(0.0f, 0.0f, 0.0f);
    float3 Su = make_float3(0.0f, 0.0f, 0.0f);
    float3 Sv = make_float3(0.0f, 0.0f, 0.0f);

    for (int i = 0; i < u_len; ++i) {
        float nu = Nu[i];
        float dnu = dNu[i];
        int ui = u_base + i;

        for (int j = 0; j < v_len; ++j) {
            float nv = Nv[j];
            float dnv = dNv[j];
            int vj = v_base + j;

            float4 cp = control_points[ui * cp_stride_v + vj];
            float3 P = make_float3(cp.x, cp.y, cp.z);

            float w = nu * nv;
            pt.x += P.x * w;
            pt.y += P.y * w;
            pt.z += P.z * w;

            float wu = dnu * nv;
            Su.x += P.x * wu;
            Su.y += P.y * wu;
            Su.z += P.z * wu;

            float wv = nu * dnv;
            Sv.x += P.x * wv;
            Sv.y += P.y * wv;
            Sv.z += P.z * wv;
        }
    }

    float3 nor = normalize_safe(cross_product(Su, Sv));

    out_points[idx] = make_float4(pt.x, pt.y, pt.z, 1.0f);
    out_normals[idx] = make_float4(nor.x, nor.y, nor.z, 0.0f);
}

int truck_cuda_evaluate_bspline_surface(
    const float* h_u_knots,
    unsigned int num_u_knots,
    const float* h_v_knots,
    unsigned int num_v_knots,
    unsigned int u_degree,
    unsigned int v_degree,
    const float* h_control_points,
    unsigned int cp_stride_v,
    const float* h_uv_points,
    unsigned int num_points,
    float* h_out_points,
    float* h_out_normals
) {
    if (num_points == 0) return 0;
    ensure_best_device();

    unsigned int u_ctrl = num_u_knots - 1 - u_degree;
    size_t u_knot_bytes = num_u_knots * sizeof(float);
    size_t v_knot_bytes = num_v_knots * sizeof(float);
    size_t cp_bytes = u_ctrl * cp_stride_v * sizeof(float4);
    size_t uv_bytes = num_points * sizeof(float2);
    size_t out_bytes = num_points * sizeof(float4);

    float* d_u_knots = NULL;
    float* d_v_knots = NULL;
    float4* d_control_points = NULL;
    float2* d_uv_points = NULL;
    float4* d_out_points = NULL;
    float4* d_out_normals = NULL;

    if (cudaMalloc(&d_u_knots, u_knot_bytes) != cudaSuccess) return -1;
    if (cudaMalloc(&d_v_knots, v_knot_bytes) != cudaSuccess) {
        cudaFree(d_u_knots);
        return -2;
    }
    if (cudaMalloc(&d_control_points, cp_bytes) != cudaSuccess) {
        cudaFree(d_u_knots);
        cudaFree(d_v_knots);
        return -3;
    }
    if (cudaMalloc(&d_uv_points, uv_bytes) != cudaSuccess) {
        cudaFree(d_u_knots);
        cudaFree(d_v_knots);
        cudaFree(d_control_points);
        return -4;
    }
    if (cudaMalloc(&d_out_points, out_bytes) != cudaSuccess) {
        cudaFree(d_u_knots);
        cudaFree(d_v_knots);
        cudaFree(d_control_points);
        cudaFree(d_uv_points);
        return -5;
    }
    if (cudaMalloc(&d_out_normals, out_bytes) != cudaSuccess) {
        cudaFree(d_u_knots);
        cudaFree(d_v_knots);
        cudaFree(d_control_points);
        cudaFree(d_uv_points);
        cudaFree(d_out_points);
        return -6;
    }

    cudaMemcpy(d_u_knots, h_u_knots, u_knot_bytes, cudaMemcpyHostToDevice);
    cudaMemcpy(d_v_knots, h_v_knots, v_knot_bytes, cudaMemcpyHostToDevice);
    cudaMemcpy(d_control_points, h_control_points, cp_bytes, cudaMemcpyHostToDevice);
    cudaMemcpy(d_uv_points, h_uv_points, uv_bytes, cudaMemcpyHostToDevice);

    unsigned int block_size = 256;
    unsigned int grid_size = (num_points + block_size - 1) / block_size;
    truck_cuda_evaluate_bspline_surface_kernel<<<grid_size, block_size>>>(
        d_u_knots, num_u_knots, d_v_knots, num_v_knots,
        u_degree, v_degree,
        d_control_points, cp_stride_v,
        d_uv_points, num_points,
        d_out_points, d_out_normals
    );
    cudaDeviceSynchronize();

    cudaMemcpy(h_out_points, d_out_points, out_bytes, cudaMemcpyDeviceToHost);
    cudaMemcpy(h_out_normals, d_out_normals, out_bytes, cudaMemcpyDeviceToHost);

    cudaFree(d_u_knots);
    cudaFree(d_v_knots);
    cudaFree(d_control_points);
    cudaFree(d_uv_points);
    cudaFree(d_out_points);
    cudaFree(d_out_normals);
    return 0;
}

// ============================================================================
// Double-Precision (FP64) Implementations for Exact CAD Parity
// ============================================================================

__device__ __forceinline__ double3 cross_product_d(double3 a, double3 b) {
    return make_double3(
        a.y * b.z - a.z * b.y,
        a.z * b.x - a.x * b.z,
        a.x * b.y - a.y * b.x
    );
}

__device__ __forceinline__ double3 normalize_safe_d(double3 v) {
    double l2 = v.x * v.x + v.y * v.y + v.z * v.z;
    if (l2 < 1.0e-24) {
        return make_double3(0.0, 0.0, 0.0);
    }
    double inv_l = rsqrt(l2);
    return make_double3(v.x * inv_l, v.y * inv_l, v.z * inv_l);
}

__device__ double inv_or_zero_d(double delta) {
    return (fabs(delta) > 1.0e-12) ? (1.0 / delta) : 0.0;
}

extern "C" __global__ void truck_cuda_trimming_loop_include_f64_kernel(
    const double4* __restrict__ segments,
    unsigned int num_segments,
    const double2* __restrict__ points,
    const double2* __restrict__ ray_dirs,
    unsigned int num_points,
    int* __restrict__ inside_flags
) {
    unsigned int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= num_points) return;

    double2 c = points[idx];
    double2 r = ray_dirs[idx];
    int counter = 0;
    bool indeterminate = false;

    for (unsigned int s = 0; s < num_segments; ++s) {
        double4 seg = segments[s];
        double2 a = make_double2(seg.x - c.x, seg.y - c.y);
        double2 b = make_double2(seg.z - c.x, seg.w - c.y);

        double s0 = r.x * a.y - r.y * a.x;
        double s1 = r.x * b.y - r.y * b.x;
        double s2 = a.x * b.y - a.y * b.x;

        double denom = s1 - s0;
        if (fabs(denom) > 1.0e-12) {
            double x = s2 / denom;
            if (fabs(x) < 1.0e-6 && s0 * s1 < 0.0) {
                indeterminate = true;
                break;
            } else if (x > 0.0 && s0 <= 0.0 && s1 > 0.0) {
                counter += 1;
            } else if (x > 0.0 && s0 >= 0.0 && s1 < 0.0) {
                counter -= 1;
            }
        }
    }

    inside_flags[idx] = (!indeterminate && counter > 0) ? 1 : 0;
}

int truck_cuda_trimming_loop_include_f64(
    const double* h_segments,
    unsigned int num_segments,
    const double* h_points,
    const double* h_ray_dirs,
    unsigned int num_points,
    int* h_inside_flags
) {
    if (num_points == 0) return 0;
    if (num_segments == 0) {
        for (unsigned int i = 0; i < num_points; ++i) h_inside_flags[i] = 0;
        return 0;
    }
    ensure_best_device();

    size_t seg_bytes = num_segments * sizeof(double4);
    size_t pt_bytes = num_points * sizeof(double2);
    size_t out_bytes = num_points * sizeof(int);

    double4* d_segments = NULL;
    double2* d_points = NULL;
    double2* d_ray_dirs = NULL;
    int* d_inside_flags = NULL;

    if (cudaMalloc(&d_segments, seg_bytes) != cudaSuccess) return -1;
    if (cudaMalloc(&d_points, pt_bytes) != cudaSuccess) {
        cudaFree(d_segments);
        return -2;
    }
    if (cudaMalloc(&d_ray_dirs, pt_bytes) != cudaSuccess) {
        cudaFree(d_segments);
        cudaFree(d_points);
        return -3;
    }
    if (cudaMalloc(&d_inside_flags, out_bytes) != cudaSuccess) {
        cudaFree(d_segments);
        cudaFree(d_points);
        cudaFree(d_ray_dirs);
        return -4;
    }

    cudaMemcpy(d_segments, h_segments, seg_bytes, cudaMemcpyHostToDevice);
    cudaMemcpy(d_points, h_points, pt_bytes, cudaMemcpyHostToDevice);
    cudaMemcpy(d_ray_dirs, h_ray_dirs, pt_bytes, cudaMemcpyHostToDevice);

    unsigned int block_size = 256;
    unsigned int grid_size = (num_points + block_size - 1) / block_size;
    truck_cuda_trimming_loop_include_f64_kernel<<<grid_size, block_size>>>(
        d_segments, num_segments,
        d_points, d_ray_dirs, num_points,
        d_inside_flags
    );
    cudaDeviceSynchronize();

    cudaMemcpy(h_inside_flags, d_inside_flags, out_bytes, cudaMemcpyDeviceToHost);

    cudaFree(d_segments);
    cudaFree(d_points);
    cudaFree(d_ray_dirs);
    cudaFree(d_inside_flags);
    return 0;
}

__device__ int truck_cuda_bspline_basis_functions_f64(
    double t,
    int degree,
    int der_rank,
    const double* __restrict__ knots,
    int num_knots,
    double* __restrict__ eval_out,
    int* __restrict__ out_len
) {
    int n = num_knots - 1;

    // Floor search
    int idx = 0;
    while (idx < n && knots[idx + 1] <= t) {
        idx++;
    }
    if (idx == n) {
        int mult = 0;
        for (int k = n; k >= 0; --k) {
            if (fabs(knots[k] - knots[n]) < 1.0e-7) mult++;
            else break;
        }
        idx = n - mult;
    }

    int global_base = (idx >= degree) ? (idx - degree) : 0;
    double eval[16] = {0.0};
    eval[idx - global_base] = 1.0;

    for (int k = 1; k <= (degree - der_rank); ++k) {
        int base = (idx >= k) ? (idx - k) : 0;
        double delta = knots[base + k] - knots[base];
        double a = inv_or_zero_d(delta) * (t - knots[base]);
        int max_i = (idx < (n - k - 1)) ? idx : (n - k - 1);
        for (int i = base; i <= max_i; ++i) {
            double delta2 = knots[i + k + 1] - knots[i + 1];
            double b = inv_or_zero_d(delta2) * (knots[i + k + 1] - t);
            int j = i - global_base;
            eval[j] = a * eval[j] + b * eval[j + 1];
            a = 1.0 - b;
        }
    }

    for (int k = (degree - der_rank + 1); k <= degree; ++k) {
        int base = (idx >= k) ? (idx - k) : 0;
        double delta = knots[base + k] - knots[base];
        double a = inv_or_zero_d(delta);
        int max_i = (idx < (n - k - 1)) ? idx : (n - k - 1);
        for (int i = base; i <= max_i; ++i) {
            double delta2 = knots[i + k + 1] - knots[i + 1];
            double b = inv_or_zero_d(delta2);
            int j = i - global_base;
            eval[j] = (a * eval[j] - b * eval[j + 1]) * (double)k;
            a = b;
        }
    }

    int len = degree + 1;
    if (n - degree - global_base < len) len = n - degree - global_base;
    *out_len = len;

    for (int i = 0; i < len; ++i) {
        eval_out[i] = eval[i];
    }
    return global_base;
}

extern "C" __global__ void truck_cuda_evaluate_bspline_surface_f64_kernel(
    const double* __restrict__ u_knots,
    unsigned int num_u_knots,
    const double* __restrict__ v_knots,
    unsigned int num_v_knots,
    unsigned int u_degree,
    unsigned int v_degree,
    const double4* __restrict__ control_points,
    unsigned int cp_stride_v,
    const double2* __restrict__ uv_points,
    unsigned int num_points,
    double4* __restrict__ out_points,
    double4* __restrict__ out_normals
) {
    unsigned int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= num_points) return;

    double2 uv = uv_points[idx];

    double Nu[16], dNu[16];
    double Nv[16], dNv[16];
    int u_len = 0, u_der_len = 0;
    int v_len = 0, v_der_len = 0;

    int u_base = truck_cuda_bspline_basis_functions_f64(uv.x, (int)u_degree, 0, u_knots, (int)num_u_knots, Nu, &u_len);
    truck_cuda_bspline_basis_functions_f64(uv.x, (int)u_degree, 1, u_knots, (int)num_u_knots, dNu, &u_der_len);

    int v_base = truck_cuda_bspline_basis_functions_f64(uv.y, (int)v_degree, 0, v_knots, (int)num_v_knots, Nv, &v_len);
    truck_cuda_bspline_basis_functions_f64(uv.y, (int)v_degree, 1, v_knots, (int)num_v_knots, dNv, &v_der_len);

    double3 pt = make_double3(0.0, 0.0, 0.0);
    double3 Su = make_double3(0.0, 0.0, 0.0);
    double3 Sv = make_double3(0.0, 0.0, 0.0);

    for (int i = 0; i < u_len; ++i) {
        double nu = Nu[i];
        double dnu = dNu[i];
        int ui = u_base + i;

        for (int j = 0; j < v_len; ++j) {
            double nv = Nv[j];
            double dnv = dNv[j];
            int vj = v_base + j;

            double4 cp = control_points[ui * cp_stride_v + vj];
            double3 P = make_double3(cp.x, cp.y, cp.z);

            double w = nu * nv;
            pt.x += P.x * w;
            pt.y += P.y * w;
            pt.z += P.z * w;

            double wu = dnu * nv;
            Su.x += P.x * wu;
            Su.y += P.y * wu;
            Su.z += P.z * wu;

            double wv = nu * dnv;
            Sv.x += P.x * wv;
            Sv.y += P.y * wv;
            Sv.z += P.z * wv;
        }
    }

    double3 nor = normalize_safe_d(cross_product_d(Su, Sv));

    out_points[idx] = make_double4(pt.x, pt.y, pt.z, 1.0);
    out_normals[idx] = make_double4(nor.x, nor.y, nor.z, 0.0);
}

int truck_cuda_evaluate_bspline_surface_f64(
    const double* h_u_knots,
    unsigned int num_u_knots,
    const double* h_v_knots,
    unsigned int num_v_knots,
    unsigned int u_degree,
    unsigned int v_degree,
    const double* h_control_points,
    unsigned int cp_stride_v,
    const double* h_uv_points,
    unsigned int num_points,
    double* h_out_points,
    double* h_out_normals
) {
    if (num_points == 0) return 0;
    ensure_best_device();

    unsigned int u_ctrl = num_u_knots - 1 - u_degree;
    size_t u_knot_bytes = num_u_knots * sizeof(double);
    size_t v_knot_bytes = num_v_knots * sizeof(double);
    size_t cp_bytes = u_ctrl * cp_stride_v * sizeof(double4);
    size_t uv_bytes = num_points * sizeof(double2);
    size_t out_bytes = num_points * sizeof(double4);

    double* d_u_knots = NULL;
    double* d_v_knots = NULL;
    double4* d_control_points = NULL;
    double2* d_uv_points = NULL;
    double4* d_out_points = NULL;
    double4* d_out_normals = NULL;

    if (cudaMalloc(&d_u_knots, u_knot_bytes) != cudaSuccess) return -1;
    if (cudaMalloc(&d_v_knots, v_knot_bytes) != cudaSuccess) {
        cudaFree(d_u_knots);
        return -2;
    }
    if (cudaMalloc(&d_control_points, cp_bytes) != cudaSuccess) {
        cudaFree(d_u_knots);
        cudaFree(d_v_knots);
        return -3;
    }
    if (cudaMalloc(&d_uv_points, uv_bytes) != cudaSuccess) {
        cudaFree(d_u_knots);
        cudaFree(d_v_knots);
        cudaFree(d_control_points);
        return -4;
    }
    if (cudaMalloc(&d_out_points, out_bytes) != cudaSuccess) {
        cudaFree(d_u_knots);
        cudaFree(d_v_knots);
        cudaFree(d_control_points);
        cudaFree(d_uv_points);
        return -5;
    }
    if (cudaMalloc(&d_out_normals, out_bytes) != cudaSuccess) {
        cudaFree(d_u_knots);
        cudaFree(d_v_knots);
        cudaFree(d_control_points);
        cudaFree(d_uv_points);
        cudaFree(d_out_points);
        return -6;
    }

    cudaMemcpy(d_u_knots, h_u_knots, u_knot_bytes, cudaMemcpyHostToDevice);
    cudaMemcpy(d_v_knots, h_v_knots, v_knot_bytes, cudaMemcpyHostToDevice);
    cudaMemcpy(d_control_points, h_control_points, cp_bytes, cudaMemcpyHostToDevice);
    cudaMemcpy(d_uv_points, h_uv_points, uv_bytes, cudaMemcpyHostToDevice);

    unsigned int block_size = 256;
    unsigned int grid_size = (num_points + block_size - 1) / block_size;
    truck_cuda_evaluate_bspline_surface_f64_kernel<<<grid_size, block_size>>>(
        d_u_knots, num_u_knots, d_v_knots, num_v_knots,
        u_degree, v_degree,
        d_control_points, cp_stride_v,
        d_uv_points, num_points,
        d_out_points, d_out_normals
    );
    cudaDeviceSynchronize();

    cudaMemcpy(h_out_points, d_out_points, out_bytes, cudaMemcpyDeviceToHost);
    cudaMemcpy(h_out_normals, d_out_normals, out_bytes, cudaMemcpyDeviceToHost);

    cudaFree(d_u_knots);
    cudaFree(d_v_knots);
    cudaFree(d_control_points);
    cudaFree(d_uv_points);
    cudaFree(d_out_points);
    cudaFree(d_out_normals);
    return 0;
}

} // extern "C"
