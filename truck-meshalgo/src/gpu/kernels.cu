#include <cuda_runtime.h>
#include <cuda.h>
#include <float.h>
#include <math.h>
#include <stdio.h>
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

    CUresult res = cuModuleLoadData(&g_driver_ptx_module, truck_meshalgo_src_gpu_kernels_sm86_ptx);
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

} // extern "C"
