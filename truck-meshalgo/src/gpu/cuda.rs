//! CUDA-accelerated mesh algorithms targeting NVIDIA Ampere (SM 8.6, RTX 3080).
#![allow(unsafe_code)]

use super::{filter_and_cluster_seeds, sample_surface_grid, GpuTriangle};
use truck_base::cgmath64::*;
use truck_base::hash::{self, HashGen};
use truck_geometry::prelude::*;
use truck_polymesh::PolygonMesh;

extern "C" {
    // Default optimized SM 8.6 PTX kernel with buffer pooling
    fn cuda_compute_distance2_point_cloud(
        h_points: *const f32,
        num_points: u32,
        h_triangles: *const f32,
        num_triangles: u32,
        h_min_distances: *mut f32,
    ) -> i32;

    // Baseline unoptimized CUDA kernel with individual per-call allocations
    fn cuda_compute_distance2_point_cloud_baseline(
        h_points: *const f32,
        num_points: u32,
        h_triangles: *const f32,
        num_triangles: u32,
        h_min_distances: *mut f32,
    ) -> i32;

    // PTX module execution via bare-metal CUDA Driver API (cuLaunchKernel)
    fn cuda_compute_distance2_point_cloud_ptx(
        h_points: *const f32,
        num_points: u32,
        h_triangles: *const f32,
        num_triangles: u32,
        h_min_distances: *mut f32,
    ) -> i32;

    // Default optimized SM 8.6 ray crossing kernel with buffer pooling
    fn cuda_compute_signed_crossing_rays(
        h_points: *const f32,
        h_ray_dirs: *const f32,
        num_points: u32,
        h_triangles: *const f32,
        num_triangles: u32,
        h_crossing_counts: *mut i32,
    ) -> i32;

    // Baseline unoptimized ray crossing kernel
    fn cuda_compute_signed_crossing_rays_baseline(
        h_points: *const f32,
        h_ray_dirs: *const f32,
        num_points: u32,
        h_triangles: *const f32,
        num_triangles: u32,
        h_crossing_counts: *mut i32,
    ) -> i32;

    // PTX module ray crossing execution via bare-metal CUDA Driver API
    fn cuda_compute_signed_crossing_rays_ptx(
        h_points: *const f32,
        h_ray_dirs: *const f32,
        num_points: u32,
        h_triangles: *const f32,
        num_triangles: u32,
        h_crossing_counts: *mut i32,
    ) -> i32;

    // Candidate 1: Mesh Normal Generation & Smoothing
    fn truck_cuda_compute_face_normals(
        h_triangles: *const f32,
        num_triangles: u32,
        h_face_normals: *mut f32,
    ) -> i32;

    fn truck_cuda_compute_smooth_normals(
        h_vertices: *const f32,
        num_vertices: u32,
        h_faces: *const u32,
        num_faces: u32,
        h_vertex_normals: *mut f32,
    ) -> i32;

    // Candidate 2: 2D Trimming Loop Inclusion Testing
    #[allow(dead_code)]
    fn truck_cuda_trimming_loop_include(
        h_segments: *const f32,
        num_segments: u32,
        h_points: *const f32,
        h_ray_dirs: *const f32,
        num_points: u32,
        h_inside_flags: *mut i32,
    ) -> i32;

    // Candidate 3: Triangle-Triangle Collision & Interference
    fn truck_cuda_collide_triangles(
        h_mesh0: *const f32,
        num_tri0: u32,
        h_mesh1: *const f32,
        num_tri1: u32,
        return_early_on_first: i32,
        h_num_interferences: *mut i32,
        h_out_segments: *mut f32,
        max_segments: u32,
    ) -> i32;

    // Candidate 4: Batch B-Spline Surface Evaluation
    #[allow(dead_code)]
    fn truck_cuda_evaluate_bspline_surface(
        h_u_knots: *const f32,
        num_u_knots: u32,
        h_v_knots: *const f32,
        num_v_knots: u32,
        u_degree: u32,
        v_degree: u32,
        h_control_points: *const f32,
        cp_stride_v: u32,
        h_uv_points: *const f32,
        num_points: u32,
        h_out_points: *mut f32,
        h_out_normals: *mut f32,
    ) -> i32;

    fn truck_cuda_trimming_loop_include_f64(
        h_segments: *const f64,
        num_segments: u32,
        h_points: *const f64,
        h_ray_dirs: *const f64,
        num_points: u32,
        h_inside_flags: *mut i32,
    ) -> i32;

    fn truck_cuda_evaluate_bspline_surface_f64(
        h_u_knots: *const f64,
        num_u_knots: u32,
        h_v_knots: *const f64,
        num_v_knots: u32,
        u_degree: u32,
        v_degree: u32,
        h_control_points: *const f64,
        cp_stride_v: u32,
        h_uv_points: *const f64,
        num_points: u32,
        h_out_points: *mut f64,
        h_out_normals: *mut f64,
    ) -> i32;

    fn truck_cuda_evaluate_nurbs_surface_f64(
        h_u_knots: *const f64,
        num_u_knots: u32,
        h_v_knots: *const f64,
        num_v_knots: u32,
        u_degree: u32,
        v_degree: u32,
        h_control_points: *const f64,
        cp_stride_v: u32,
        h_uv_points: *const f64,
        num_points: u32,
        h_out_points: *mut f64,
        h_out_normals: *mut f64,
    ) -> i32;

    fn truck_cuda_is_clung_to_by_f64(
        h_points: *const f64,
        num_points: u32,
        h_triangles: *const f64,
        num_triangles: u32,
        tol: f64,
        h_result: *mut i32,
    ) -> i32;

    fn truck_cuda_neighborhood_include_f64(
        h_points: *const f64,
        num_points: u32,
        h_triangles: *const f64,
        num_triangles: u32,
        tol: f64,
        h_result: *mut i32,
    ) -> i32;

    fn truck_cuda_find_surface_intersection_seeds(
        h_s1_samples: *const std::ffi::c_void,
        n_u1: u32,
        n_v1: u32,
        h_s2_samples: *const std::ffi::c_void,
        n_u2: u32,
        n_v2: u32,
        tol: f32,
        h_out_seeds: *mut f32,
        max_seeds: u32,
        h_seed_count: *mut u32,
    ) -> i32;

    fn truck_cuda_slice_mesh_planes(
        h_triangles: *const f32,
        num_triangles: u32,
        h_planes: *const f32,
        num_planes: u32,
        max_segments_per_plane: u32,
        h_out_counts: *mut u32,
        h_out_segments: *mut f32,
    ) -> i32;

    fn truck_cuda_mesh_volume_cog(
        h_triangles: *const f32,
        num_triangles: u32,
        h_out_accum: *mut f64,
    ) -> i32;

    fn truck_cuda_extract_planes(
        h_face_positions: *const f64,
        h_face_normals: *const f64,
        h_face_offsets: *const u32,
        num_faces: u32,
        total_vertices: u32,
        tol: f64,
        h_planar_count: *mut u32,
        h_planar_indices: *mut u32,
        h_other_count: *mut u32,
        h_other_indices: *mut u32,
    ) -> i32;

    fn truck_cuda_subdivide_triangles(
        h_in_triangles: *const f32,
        num_triangles: u32,
        h_out_triangles: *mut f32,
    ) -> i32;

    fn truck_cuda_evaluate_nurbs_curve_f64(
        h_knots: *const f64,
        num_knots: u32,
        degree: u32,
        h_control_points: *const f64,
        num_control_points: u32,
        h_t_points: *const f64,
        num_points: u32,
        h_out_points: *mut f64,
        h_out_derivatives: *mut f64,
    ) -> i32;

    fn truck_cuda_raycast_mesh(
        h_ray_origins: *const f32,
        h_ray_dirs: *const f32,
        num_rays: u32,
        h_triangles: *const f32,
        num_triangles: u32,
        h_out_distances: *mut f32,
        h_out_tri_indices: *mut i32,
        h_out_hit_points: *mut f32,
        h_out_normals: *mut f32,
    ) -> i32;

    fn truck_cuda_get_device_info(
        name_buf: *mut std::os::raw::c_char,
        name_buf_len: i32,
        major: *mut i32,
        minor: *mut i32,
        multi_processor_count: *mut i32,
        total_mem: *mut usize,
    ) -> i32;
}

/// Metadata describing a CUDA device.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CudaDeviceInfo {
    /// Device model name (e.g. "NVIDIA GeForce RTX 3080").
    pub name: String,
    /// Compute capability string (e.g. "8.6").
    pub compute_capability: String,
    /// Number of Streaming Multiprocessors (SMs).
    pub multi_processor_count: i32,
    /// Total global device memory in bytes.
    pub total_memory_bytes: usize,
}

/// CUDA-accelerated mesh analyzer for point-cloud distance queries and in-out containment.
#[derive(Debug, Clone, Copy, Default)]
pub struct CudaMeshAnalyzer;

impl CudaMeshAnalyzer {
    /// Creates a new CUDA mesh analyzer.
    #[inline(always)]
    pub fn new() -> Self {
        Self
    }

    /// Queries the primary CUDA device information.
    pub fn get_device_info() -> Option<CudaDeviceInfo> {
        let mut name_buf = [0u8; 256];
        let mut major = 0i32;
        let mut minor = 0i32;
        let mut sms = 0i32;
        let mut total_mem = 0usize;
        let status = unsafe {
            truck_cuda_get_device_info(
                name_buf.as_mut_ptr() as *mut std::os::raw::c_char,
                name_buf.len() as i32,
                &mut major,
                &mut minor,
                &mut sms,
                &mut total_mem,
            )
        };
        if status == 0 {
            let nul_pos = name_buf.iter().position(|&b| b == 0).unwrap_or(name_buf.len());
            let name = String::from_utf8_lossy(&name_buf[..nul_pos]).trim().to_string();
            Some(CudaDeviceInfo {
                name,
                compute_capability: format!("{}.{}", major, minor),
                multi_processor_count: sms,
                total_memory_bytes: total_mem,
            })
        } else {
            None
        }
    }

    /// Extracts triangles from a polygon mesh into GPU triangle format.
    pub fn extract_triangles(mesh: &PolygonMesh) -> Vec<GpuTriangle> {
        let positions = mesh.positions();
        mesh.face_iter()
            .flat_map(|face| {
                (2..face.len()).map(move |i| {
                    let p0 = positions[face[0].pos];
                    let p1 = positions[face[i - 1].pos];
                    let p2 = positions[face[i].pos];
                    GpuTriangle::new(p0, p1, p2)
                })
            })
            .collect()
    }

    /// Computes the minimum squared distance from each query point to the polygon mesh via optimized SM 8.6 PTX.
    pub fn distance2_point_cloud(&self, mesh: &PolygonMesh, points: &[Point3]) -> Vec<f32> {
        if points.is_empty() {
            return Vec::new();
        }
        let triangles = Self::extract_triangles(mesh);
        if triangles.is_empty() {
            return vec![f32::INFINITY; points.len()];
        }

        let num_points = points.len() as u32;
        let num_triangles = triangles.len() as u32;

        let pt_vec: Vec<[f32; 4]> = points
            .iter()
            .map(|p| [p.x as f32, p.y as f32, p.z as f32, 0.0])
            .collect();

        let mut out_distances = vec![0.0f32; points.len()];

        let status = unsafe {
            cuda_compute_distance2_point_cloud(
                pt_vec.as_ptr() as *const f32,
                num_points,
                triangles.as_ptr() as *const f32,
                num_triangles,
                out_distances.as_mut_ptr(),
            )
        };
        assert_eq!(status, 0, "CUDA distance2_point_cloud kernel failed with error code: {}", status);

        out_distances
    }

    /// Computes the minimum squared distance using the baseline unoptimized CUDA kernel.
    pub fn distance2_point_cloud_baseline(&self, mesh: &PolygonMesh, points: &[Point3]) -> Vec<f32> {
        if points.is_empty() {
            return Vec::new();
        }
        let triangles = Self::extract_triangles(mesh);
        if triangles.is_empty() {
            return vec![f32::INFINITY; points.len()];
        }

        let num_points = points.len() as u32;
        let num_triangles = triangles.len() as u32;

        let pt_vec: Vec<[f32; 4]> = points
            .iter()
            .map(|p| [p.x as f32, p.y as f32, p.z as f32, 0.0])
            .collect();

        let mut out_distances = vec![0.0f32; points.len()];

        let status = unsafe {
            cuda_compute_distance2_point_cloud_baseline(
                pt_vec.as_ptr() as *const f32,
                num_points,
                triangles.as_ptr() as *const f32,
                num_triangles,
                out_distances.as_mut_ptr(),
            )
        };
        assert_eq!(status, 0, "CUDA distance2_point_cloud_baseline failed with error code: {}", status);

        out_distances
    }

    /// Computes the minimum squared distance using the standalone SM 8.6 PTX module via CUDA Driver API.
    pub fn distance2_point_cloud_ptx(&self, mesh: &PolygonMesh, points: &[Point3]) -> Vec<f32> {
        if points.is_empty() {
            return Vec::new();
        }
        let triangles = Self::extract_triangles(mesh);
        if triangles.is_empty() {
            return vec![f32::INFINITY; points.len()];
        }

        let num_points = points.len() as u32;
        let num_triangles = triangles.len() as u32;

        let pt_vec: Vec<[f32; 4]> = points
            .iter()
            .map(|p| [p.x as f32, p.y as f32, p.z as f32, 0.0])
            .collect();

        let mut out_distances = vec![0.0f32; points.len()];

        let status = unsafe {
            cuda_compute_distance2_point_cloud_ptx(
                pt_vec.as_ptr() as *const f32,
                num_points,
                triangles.as_ptr() as *const f32,
                num_triangles,
                out_distances.as_mut_ptr(),
            )
        };
        assert_eq!(status, 0, "CUDA distance2_point_cloud_ptx failed with error code: {}", status);

        out_distances
    }

    /// Computes signed crossing face counts for specified rays on the GPU via optimized SM 8.6 PTX.
    pub fn signed_crossing_rays(
        &self,
        mesh: &PolygonMesh,
        points: &[Point3],
        ray_directions: &[Vector3],
    ) -> Vec<i32> {
        assert_eq!(points.len(), ray_directions.len());
        if points.is_empty() {
            return Vec::new();
        }
        let triangles = Self::extract_triangles(mesh);
        if triangles.is_empty() {
            return vec![0; points.len()];
        }

        let num_points = points.len() as u32;
        let num_triangles = triangles.len() as u32;

        let pt_vec: Vec<[f32; 4]> = points
            .iter()
            .map(|p| [p.x as f32, p.y as f32, p.z as f32, 0.0])
            .collect();
        let dir_vec: Vec<[f32; 4]> = ray_directions
            .iter()
            .map(|d| [d.x as f32, d.y as f32, d.z as f32, 0.0])
            .collect();

        let mut out_counts = vec![0i32; points.len()];

        let status = unsafe {
            cuda_compute_signed_crossing_rays(
                pt_vec.as_ptr() as *const f32,
                dir_vec.as_ptr() as *const f32,
                num_points,
                triangles.as_ptr() as *const f32,
                num_triangles,
                out_counts.as_mut_ptr(),
            )
        };
        assert_eq!(status, 0, "CUDA signed_crossing_rays failed with error code: {}", status);

        out_counts
    }

    /// Computes signed crossing face counts using baseline CUDA kernel.
    pub fn signed_crossing_rays_baseline(
        &self,
        mesh: &PolygonMesh,
        points: &[Point3],
        ray_directions: &[Vector3],
    ) -> Vec<i32> {
        assert_eq!(points.len(), ray_directions.len());
        if points.is_empty() {
            return Vec::new();
        }
        let triangles = Self::extract_triangles(mesh);
        if triangles.is_empty() {
            return vec![0; points.len()];
        }

        let num_points = points.len() as u32;
        let num_triangles = triangles.len() as u32;

        let pt_vec: Vec<[f32; 4]> = points
            .iter()
            .map(|p| [p.x as f32, p.y as f32, p.z as f32, 0.0])
            .collect();
        let dir_vec: Vec<[f32; 4]> = ray_directions
            .iter()
            .map(|d| [d.x as f32, d.y as f32, d.z as f32, 0.0])
            .collect();

        let mut out_counts = vec![0i32; points.len()];

        let status = unsafe {
            cuda_compute_signed_crossing_rays_baseline(
                pt_vec.as_ptr() as *const f32,
                dir_vec.as_ptr() as *const f32,
                num_points,
                triangles.as_ptr() as *const f32,
                num_triangles,
                out_counts.as_mut_ptr(),
            )
        };
        assert_eq!(status, 0, "CUDA signed_crossing_rays_baseline failed with error code: {}", status);

        out_counts
    }

    /// Computes signed crossing face counts using standalone SM 8.6 PTX module via CUDA Driver API.
    pub fn signed_crossing_rays_ptx(
        &self,
        mesh: &PolygonMesh,
        points: &[Point3],
        ray_directions: &[Vector3],
    ) -> Vec<i32> {
        assert_eq!(points.len(), ray_directions.len());
        if points.is_empty() {
            return Vec::new();
        }
        let triangles = Self::extract_triangles(mesh);
        if triangles.is_empty() {
            return vec![0; points.len()];
        }

        let num_points = points.len() as u32;
        let num_triangles = triangles.len() as u32;

        let pt_vec: Vec<[f32; 4]> = points
            .iter()
            .map(|p| [p.x as f32, p.y as f32, p.z as f32, 0.0])
            .collect();
        let dir_vec: Vec<[f32; 4]> = ray_directions
            .iter()
            .map(|d| [d.x as f32, d.y as f32, d.z as f32, 0.0])
            .collect();

        let mut out_counts = vec![0i32; points.len()];

        let status = unsafe {
            cuda_compute_signed_crossing_rays_ptx(
                pt_vec.as_ptr() as *const f32,
                dir_vec.as_ptr() as *const f32,
                num_points,
                triangles.as_ptr() as *const f32,
                num_triangles,
                out_counts.as_mut_ptr(),
            )
        };
        assert_eq!(status, 0, "CUDA signed_crossing_rays_ptx failed with error code: {}", status);

        out_counts
    }

    /// Computes signed crossing face counts for rays starting at each query point along a common direction.
    pub fn signed_crossing_faces(
        &self,
        mesh: &PolygonMesh,
        points: &[Point3],
        ray_direction: Vector3,
    ) -> Vec<i32> {
        let dirs = vec![ray_direction; points.len()];
        self.signed_crossing_rays(mesh, points, &dirs)
    }

    /// Determines whether each point in `points` is inside the domain bounded by the closed mesh via CUDA.
    pub fn inside(&self, mesh: &PolygonMesh, points: &[Point3]) -> Vec<bool> {
        let dirs: Vec<Vector3> = points.iter().map(|&p| hash::take_one_unit(p)).collect();
        let counts = self.signed_crossing_rays(mesh, points, &dirs);
        counts.into_iter().map(|c| c >= 1).collect()
    }

    /// Determines whether each point in `points` is inside via baseline CUDA kernel.
    pub fn inside_baseline(&self, mesh: &PolygonMesh, points: &[Point3]) -> Vec<bool> {
        let dirs: Vec<Vector3> = points.iter().map(|&p| hash::take_one_unit(p)).collect();
        let counts = self.signed_crossing_rays_baseline(mesh, points, &dirs);
        counts.into_iter().map(|c| c >= 1).collect()
    }

    /// Determines whether each point in `points` is inside via Driver PTX kernel.
    pub fn inside_ptx(&self, mesh: &PolygonMesh, points: &[Point3]) -> Vec<bool> {
        let dirs: Vec<Vector3> = points.iter().map(|&p| hash::take_one_unit(p)).collect();
        let counts = self.signed_crossing_rays_ptx(mesh, points, &dirs);
        counts.into_iter().map(|c| c >= 1).collect()
    }

    // ========================================================================
    // Candidate 1: Mesh Normal Generation & Smoothing
    // ========================================================================

    /// Computes normalized face normals for all triangular faces in the mesh via CUDA.
    pub fn compute_face_normals(&self, mesh: &PolygonMesh) -> Vec<Vector3> {
        let triangles = Self::extract_triangles(mesh);
        if triangles.is_empty() {
            return Vec::new();
        }

        let num_triangles = triangles.len() as u32;
        let mut out_normals = vec![[0.0f32; 4]; triangles.len()];

        let status = unsafe {
            truck_cuda_compute_face_normals(
                triangles.as_ptr() as *const f32,
                num_triangles,
                out_normals.as_mut_ptr() as *mut f32,
            )
        };
        assert_eq!(status, 0, "truck_cuda_compute_face_normals failed with code: {}", status);

        out_normals
            .into_iter()
            .map(|n| Vector3::new(n[0] as f64, n[1] as f64, n[2] as f64))
            .collect()
    }

    /// Computes smooth area-weighted and normalized per-vertex normals via CUDA.
    pub fn compute_smooth_normals(&self, mesh: &PolygonMesh) -> Vec<Vector3> {
        let positions = mesh.positions();
        if positions.is_empty() {
            return Vec::new();
        }

        let vertices: Vec<[f32; 4]> = positions
            .iter()
            .map(|p| [p.x as f32, p.y as f32, p.z as f32, 0.0])
            .collect();

        let faces: Vec<[u32; 3]> = mesh
            .face_iter()
            .flat_map(|face| {
                (2..face.len()).map(move |i| {
                    [face[0].pos as u32, face[i - 1].pos as u32, face[i].pos as u32]
                })
            })
            .collect();

        if faces.is_empty() {
            return vec![Vector3::zero(); positions.len()];
        }

        let mut out_normals = vec![[0.0f32; 4]; positions.len()];

        let status = unsafe {
            truck_cuda_compute_smooth_normals(
                vertices.as_ptr() as *const f32,
                vertices.len() as u32,
                faces.as_ptr() as *const u32,
                faces.len() as u32,
                out_normals.as_mut_ptr() as *mut f32,
            )
        };
        assert_eq!(status, 0, "truck_cuda_compute_smooth_normals failed with code: {}", status);

        out_normals
            .into_iter()
            .map(|n| Vector3::new(n[0] as f64, n[1] as f64, n[2] as f64))
            .collect()
    }

    // ========================================================================
    // Candidate 2: 2D Trimming Loop Inclusion Testing (PolyBoundary::include)
    // ========================================================================

    /// Tests whether 2D query points are included within a 2D boundary represented by segments via CUDA.
    pub fn trimming_loop_include(
        &self,
        segments: &[[Point2; 2]],
        points: &[Point2],
    ) -> Vec<bool> {
        if points.is_empty() {
            return Vec::new();
        }
        if segments.is_empty() {
            return vec![false; points.len()];
        }

        let seg_vec: Vec<[f64; 4]> = segments
            .iter()
            .map(|seg| [seg[0].x, seg[0].y, seg[1].x, seg[1].y])
            .collect();

        let pt_vec: Vec<[f64; 2]> = points
            .iter()
            .map(|p| [p.x, p.y])
            .collect();

        let ray_vec: Vec<[f64; 2]> = points
            .iter()
            .map(|&c| {
                let t = 2.0 * std::f64::consts::PI * HashGen::hash1(c);
                [t.cos(), t.sin()]
            })
            .collect();

        let mut out_flags = vec![0i32; points.len()];

        let status = unsafe {
            truck_cuda_trimming_loop_include_f64(
                seg_vec.as_ptr() as *const f64,
                seg_vec.len() as u32,
                pt_vec.as_ptr() as *const f64,
                ray_vec.as_ptr() as *const f64,
                points.len() as u32,
                out_flags.as_mut_ptr(),
            )
        };
        assert_eq!(status, 0, "truck_cuda_trimming_loop_include_f64 failed with code: {}", status);

        out_flags.into_iter().map(|f| f > 0).collect()
    }

    // ========================================================================
    // Candidate 3: Triangle-Triangle Collision & Interference
    // ========================================================================

    /// Extracts interference line segments between two polygon meshes on the GPU via CUDA.
    pub fn collide_triangles(
        &self,
        mesh0: &PolygonMesh,
        mesh1: &PolygonMesh,
    ) -> Vec<(Point3, Point3)> {
        let tri0 = Self::extract_triangles(mesh0);
        let tri1 = Self::extract_triangles(mesh1);
        if tri0.is_empty() || tri1.is_empty() {
            return Vec::new();
        }

        let max_segments = 65536u32;
        let mut out_segments = vec![[0.0f32; 4]; (max_segments * 2) as usize];
        let mut num_interferences = 0i32;

        let status = unsafe {
            truck_cuda_collide_triangles(
                tri0.as_ptr() as *const f32,
                tri0.len() as u32,
                tri1.as_ptr() as *const f32,
                tri1.len() as u32,
                0,
                &mut num_interferences,
                out_segments.as_mut_ptr() as *mut f32,
                max_segments,
            )
        };
        assert_eq!(status, 0, "truck_cuda_collide_triangles failed with code: {}", status);

        let count = num_interferences.clamp(0, max_segments as i32) as usize;
        (0..count)
            .map(|i| {
                let p = Point3::new(
                    out_segments[i * 2][0] as f64,
                    out_segments[i * 2][1] as f64,
                    out_segments[i * 2][2] as f64,
                );
                let q = Point3::new(
                    out_segments[i * 2 + 1][0] as f64,
                    out_segments[i * 2 + 1][1] as f64,
                    out_segments[i * 2 + 1][2] as f64,
                );
                (p, q)
            })
            .collect()
    }

    /// Tests if two polygon meshes collide, returning the first detected interference line segment.
    pub fn collide_with(
        &self,
        mesh0: &PolygonMesh,
        mesh1: &PolygonMesh,
    ) -> Option<(Point3, Point3)> {
        let tri0 = Self::extract_triangles(mesh0);
        let tri1 = Self::extract_triangles(mesh1);
        if tri0.is_empty() || tri1.is_empty() {
            return None;
        }

        let mut out_segments = vec![[0.0f32; 4]; 2];
        let mut num_interferences = 0i32;

        let status = unsafe {
            truck_cuda_collide_triangles(
                tri0.as_ptr() as *const f32,
                tri0.len() as u32,
                tri1.as_ptr() as *const f32,
                tri1.len() as u32,
                1,
                &mut num_interferences,
                out_segments.as_mut_ptr() as *mut f32,
                1,
            )
        };
        assert_eq!(status, 0, "truck_cuda_collide_triangles failed with code: {}", status);

        if num_interferences > 0 {
            let p = Point3::new(
                out_segments[0][0] as f64,
                out_segments[0][1] as f64,
                out_segments[0][2] as f64,
            );
            let q = Point3::new(
                out_segments[1][0] as f64,
                out_segments[1][1] as f64,
                out_segments[1][2] as f64,
            );
            Some((p, q))
        } else {
            None
        }
    }

    // ========================================================================
    // Candidate 4: Batch B-Spline Surface Evaluation
    // ========================================================================

    /// Evaluates a B-spline surface on a batch of `(u, v)` coordinates on the GPU via CUDA.
    ///
    /// Returns a tuple of `(evaluated_3d_points, evaluated_surface_normals)`.
    pub fn evaluate_bspline_surface(
        &self,
        u_knots: &[f64],
        v_knots: &[f64],
        u_degree: usize,
        v_degree: usize,
        control_points: &[Vec<Point3>],
        uv_points: &[Point2],
    ) -> (Vec<Point3>, Vec<Vector3>) {
        if uv_points.is_empty() || control_points.is_empty() || control_points[0].is_empty() {
            return (Vec::new(), Vec::new());
        }

        let num_u_knots = u_knots.len() as u32;
        let num_v_knots = v_knots.len() as u32;
        let num_points = uv_points.len() as u32;

        let cp_stride_v = control_points[0].len() as u32;
        let mut flat_cps: Vec<[f64; 4]> = Vec::with_capacity(control_points.len() * control_points[0].len());
        for row in control_points {
            for pt in row {
                flat_cps.push([pt.x, pt.y, pt.z, 1.0]);
            }
        }

        let uv_f64: Vec<[f64; 2]> = uv_points
            .iter()
            .map(|p| [p.x, p.y])
            .collect();

        let mut out_points = vec![[0.0f64; 4]; uv_points.len()];
        let mut out_normals = vec![[0.0f64; 4]; uv_points.len()];

        let status = unsafe {
            truck_cuda_evaluate_bspline_surface_f64(
                u_knots.as_ptr(),
                num_u_knots,
                v_knots.as_ptr(),
                num_v_knots,
                u_degree as u32,
                v_degree as u32,
                flat_cps.as_ptr() as *const f64,
                cp_stride_v,
                uv_f64.as_ptr() as *const f64,
                num_points,
                out_points.as_mut_ptr() as *mut f64,
                out_normals.as_mut_ptr() as *mut f64,
            )
        };
        assert_eq!(status, 0, "truck_cuda_evaluate_bspline_surface_f64 failed with code: {}", status);

        let pts = out_points
            .into_iter()
            .map(|p| Point3::new(p[0], p[1], p[2]))
            .collect();
        let nors = out_normals
            .into_iter()
            .map(|n| Vector3::new(n[0], n[1], n[2]))
            .collect();

        (pts, nors)
    }

    /// Evaluates a rational NURBS surface on a batch of `(u, v)` coordinates on the GPU via CUDA.
    ///
    /// Returns a tuple of `(evaluated_3d_points, evaluated_surface_normals)`.
    pub fn evaluate_nurbs_surface(
        &self,
        u_knots: &[f64],
        v_knots: &[f64],
        u_degree: usize,
        v_degree: usize,
        control_points: &[Vec<Vector4>],
        uv_points: &[Point2],
    ) -> (Vec<Point3>, Vec<Vector3>) {
        if uv_points.is_empty() || control_points.is_empty() || control_points[0].is_empty() {
            return (Vec::new(), Vec::new());
        }

        let num_u_knots = u_knots.len() as u32;
        let num_v_knots = v_knots.len() as u32;
        let num_points = uv_points.len() as u32;

        let cp_stride_v = control_points[0].len() as u32;
        let mut flat_cps: Vec<[f64; 4]> = Vec::with_capacity(control_points.len() * control_points[0].len());
        for row in control_points {
            for pt in row {
                flat_cps.push([pt.x, pt.y, pt.z, pt.w]);
            }
        }

        let uv_f64: Vec<[f64; 2]> = uv_points
            .iter()
            .map(|p| [p.x, p.y])
            .collect();

        let mut out_points = vec![[0.0f64; 4]; uv_points.len()];
        let mut out_normals = vec![[0.0f64; 4]; uv_points.len()];

        let status = unsafe {
            truck_cuda_evaluate_nurbs_surface_f64(
                u_knots.as_ptr(),
                num_u_knots,
                v_knots.as_ptr(),
                num_v_knots,
                u_degree as u32,
                v_degree as u32,
                flat_cps.as_ptr() as *const f64,
                cp_stride_v,
                uv_f64.as_ptr() as *const f64,
                num_points,
                out_points.as_mut_ptr() as *mut f64,
                out_normals.as_mut_ptr() as *mut f64,
            )
        };
        assert_eq!(status, 0, "truck_cuda_evaluate_nurbs_surface_f64 failed with code: {}", status);

        let pts = out_points
            .into_iter()
            .map(|p| Point3::new(p[0], p[1], p[2]))
            .collect();
        let nors = out_normals
            .into_iter()
            .map(|n| Vector3::new(n[0], n[1], n[2]))
            .collect();

        (pts, nors)
    }

    /// Extracts triangles from a polygon mesh as flat double-precision vertex coordinates `[f64; 4]`.
    pub fn extract_triangles_f64(mesh: &PolygonMesh) -> Vec<[f64; 4]> {
        let pos = mesh.positions();
        mesh.face_iter()
            .flat_map(|face| {
                (2..face.len()).flat_map(move |i| {
                    let p0 = pos[face[0].pos];
                    let p1 = pos[face[i - 1].pos];
                    let p2 = pos[face[i].pos];
                    vec![
                        [p0.x, p0.y, p0.z, 0.0],
                        [p1.x, p1.y, p1.z, 0.0],
                        [p2.x, p2.y, p2.z, 0.0],
                    ]
                })
            })
            .collect()
    }

    /// Checks whether all faces of the mesh intersect the neighborhood of `point_cloud` on the GPU.
    pub fn is_clung_to_by(&self, mesh: &PolygonMesh, point_cloud: &[Point3], tol: f64) -> bool {
        if point_cloud.is_empty() {
            return false;
        }
        let tri_coords = Self::extract_triangles_f64(mesh);
        if tri_coords.is_empty() {
            return false;
        }
        let pt_coords: Vec<[f64; 4]> = point_cloud
            .iter()
            .map(|p| [p.x, p.y, p.z, 0.0])
            .collect();

        let mut result = 1i32;
        let status = unsafe {
            truck_cuda_is_clung_to_by_f64(
                pt_coords.as_ptr() as *const f64,
                point_cloud.len() as u32,
                tri_coords.as_ptr() as *const f64,
                (tri_coords.len() / 3) as u32,
                tol,
                &mut result,
            )
        };
        assert_eq!(status, 0, "truck_cuda_is_clung_to_by_f64 failed with code: {}", status);
        result != 0
    }

    /// Checks whether the neighborhood of the mesh includes all points in `point_cloud` on the GPU.
    pub fn neighborhood_include(&self, mesh: &PolygonMesh, point_cloud: &[Point3], tol: f64) -> bool {
        if point_cloud.is_empty() {
            return true;
        }
        let tri_coords = Self::extract_triangles_f64(mesh);
        if tri_coords.is_empty() {
            return false;
        }
        let pt_coords: Vec<[f64; 4]> = point_cloud
            .iter()
            .map(|p| [p.x, p.y, p.z, 0.0])
            .collect();

        let mut result = 1i32;
        let status = unsafe {
            truck_cuda_neighborhood_include_f64(
                pt_coords.as_ptr() as *const f64,
                point_cloud.len() as u32,
                tri_coords.as_ptr() as *const f64,
                (tri_coords.len() / 3) as u32,
                tol,
                &mut result,
            )
        };
        assert_eq!(status, 0, "truck_cuda_neighborhood_include_f64 failed with code: {}", status);
        result != 0
    }

    /// Evaluates dense parameter grid separation on the GPU to locate candidate intersection seeds between two B-spline surfaces.
    pub fn find_surface_intersection_seeds(
        &self,
        s1: &BSplineSurface<Point3>,
        s2: &BSplineSurface<Point3>,
        tol: f64,
    ) -> Vec<(Point2, Point2)> {
        self.find_surface_intersection_seeds_with_res(s1, s2, (32, 32), (32, 32), tol)
    }

    /// Evaluates dense parameter grid separation on the GPU with custom grid resolutions.
    pub fn find_surface_intersection_seeds_with_res(
        &self,
        s1: &BSplineSurface<Point3>,
        s2: &BSplineSurface<Point3>,
        res1: (usize, usize),
        res2: (usize, usize),
        tol: f64,
    ) -> Vec<(Point2, Point2)> {
        let (samples1, n_u1, n_v1) = sample_surface_grid(s1, res1);
        let (samples2, n_u2, n_v2) = sample_surface_grid(s2, res2);

        const MAX_SEEDS: usize = 8192;
        let mut out_seeds = vec![[0.0f32; 4]; MAX_SEEDS];
        let mut seed_count = 0u32;

        let status = unsafe {
            truck_cuda_find_surface_intersection_seeds(
                samples1.as_ptr() as *const std::ffi::c_void,
                n_u1 as u32,
                n_v1 as u32,
                samples2.as_ptr() as *const std::ffi::c_void,
                n_u2 as u32,
                n_v2 as u32,
                tol as f32,
                out_seeds.as_mut_ptr() as *mut f32,
                MAX_SEEDS as u32,
                &mut seed_count,
            )
        };
        assert_eq!(status, 0, "truck_cuda_find_surface_intersection_seeds failed with code: {}", status);

        let valid_count = (seed_count as usize).min(MAX_SEEDS);
        let raw_seeds: Vec<(Point2, Point2)> = out_seeds[..valid_count]
            .iter()
            .map(|s| (Point2::new(s[0] as f64, s[1] as f64), Point2::new(s[2] as f64, s[3] as f64)))
            .collect();

        filter_and_cluster_seeds(raw_seeds, s1, s2)
    }

    /// Slices a polygon mesh against multiple planes on the GPU using native CUDA with SM 8.6 warp-aggregated atomics.
    pub fn slice_mesh_planes(&self, mesh: &PolygonMesh, planes: &[[f64; 4]]) -> Vec<Vec<(Point3, Point3)>> {
        if planes.is_empty() {
            return Vec::new();
        }
        let triangles = Self::extract_triangles(mesh);
        if triangles.is_empty() {
            return vec![Vec::new(); planes.len()];
        }

        let num_triangles = triangles.len() as u32;
        let num_planes = planes.len() as u32;
        let max_segments_per_plane = num_triangles.min(262_144);

        let planes_f32: Vec<[f32; 4]> = planes
            .iter()
            .map(|p| [p[0] as f32, p[1] as f32, p[2] as f32, p[3] as f32])
            .collect();

        let mut out_counts = vec![0u32; planes.len()];
        let total_segments = (num_planes as usize) * (max_segments_per_plane as usize);
        let mut out_segments = vec![0.0f32; total_segments * 8];

        let status = unsafe {
            truck_cuda_slice_mesh_planes(
                triangles.as_ptr() as *const f32,
                num_triangles,
                planes_f32.as_ptr() as *const f32,
                num_planes,
                max_segments_per_plane,
                out_counts.as_mut_ptr(),
                out_segments.as_mut_ptr(),
            )
        };
        assert_eq!(status, 0, "truck_cuda_slice_mesh_planes failed with code: {}", status);

        (0..planes.len())
            .map(|k| {
                let count = (out_counts[k] as usize).min(max_segments_per_plane as usize);
                let base = k * (max_segments_per_plane as usize) * 8;
                (0..count)
                    .map(|i| {
                        let seg_base = base + i * 8;
                        let p1 = Point3::new(
                            out_segments[seg_base] as f64,
                            out_segments[seg_base + 1] as f64,
                            out_segments[seg_base + 2] as f64,
                        );
                        let p2 = Point3::new(
                            out_segments[seg_base + 4] as f64,
                            out_segments[seg_base + 5] as f64,
                            out_segments[seg_base + 6] as f64,
                        );
                        (p1, p2)
                    })
                    .collect()
            })
            .collect()
    }

    /// Evaluates divergence-theorem prism volume and center of gravity of a mesh on the GPU using parallel warp-shuffle and block reductions.
    pub fn mass_properties(&self, mesh: &PolygonMesh) -> (f64, Point3) {
        let triangles = Self::extract_triangles(mesh);
        if triangles.is_empty() {
            return (0.0, Point3::origin());
        }

        let num_triangles = triangles.len() as u32;
        let mut accum = [0.0f64; 4];

        let status = unsafe {
            truck_cuda_mesh_volume_cog(
                triangles.as_ptr() as *const f32,
                num_triangles,
                accum.as_mut_ptr(),
            )
        };
        assert_eq!(status, 0, "truck_cuda_mesh_volume_cog failed with code: {}", status);

        let vol = accum[3];
        let cog = if vol.abs() > 1e-12 {
            Point3::new(accum[0] / vol, accum[1] / vol, accum[2] / vol)
        } else {
            Point3::origin()
        };

        (vol, cog)
    }

    /// Evaluates the divergence-theorem prism volume of a mesh on the GPU via CUDA.
    #[inline(always)]
    pub fn volume(&self, mesh: &PolygonMesh) -> f64 {
        self.mass_properties(mesh).0
    }

    /// Evaluates the center of gravity of a mesh on the GPU via CUDA.
    #[inline(always)]
    pub fn center_of_gravity(&self, mesh: &PolygonMesh) -> Point3 {
        self.mass_properties(mesh).1
    }

    /// Extracts planar faces such that their face normal matches their vertex normal within `tol` via CUDA.
    pub fn extract_planes(&self, mesh: &PolygonMesh, tol: f64) -> (Vec<usize>, Vec<usize>) {
        if mesh.faces().is_empty() {
            return (Vec::new(), Vec::new());
        }
        let (offsets, face_pos, face_nor) = super::prepare_extract_planes_buffers_f64(mesh);
        let num_faces = mesh.faces().len() as u32;
        let total_vertices = face_pos.len() as u32;

        let mut planar_count: u32 = 0;
        let mut other_count: u32 = 0;
        let mut planar_indices = vec![0u32; num_faces as usize];
        let mut other_indices = vec![0u32; num_faces as usize];

        let status = unsafe {
            truck_cuda_extract_planes(
                face_pos.as_ptr() as *const f64,
                face_nor.as_ptr() as *const f64,
                offsets.as_ptr(),
                num_faces,
                total_vertices,
                tol,
                &mut planar_count,
                planar_indices.as_mut_ptr(),
                &mut other_count,
                other_indices.as_mut_ptr(),
            )
        };
        assert_eq!(status, 0, "truck_cuda_extract_planes failed with code: {}", status);

        planar_indices.truncate(planar_count as usize);
        other_indices.truncate(other_count as usize);

        let mut planes: Vec<usize> = planar_indices.into_iter().map(|i| i as usize).collect();
        let mut others: Vec<usize> = other_indices.into_iter().map(|i| i as usize).collect();

        planes.sort_unstable();
        others.sort_unstable();

        (planes, others)
    }

    /// Subdivides triangles into 4-to-1 midpoint-refined triangles on the GPU via CUDA.
    pub fn subdivide_triangles(&self, triangles: &[GpuTriangle]) -> Vec<GpuTriangle> {
        if triangles.is_empty() {
            return Vec::new();
        }
        let num_triangles = triangles.len() as u32;
        let mut out_triangles = vec![
            GpuTriangle {
                v0: [0.0; 4],
                v1: [0.0; 4],
                v2: [0.0; 4],
            };
            triangles.len() * 4
        ];

        let status = unsafe {
            truck_cuda_subdivide_triangles(
                triangles.as_ptr() as *const f32,
                num_triangles,
                out_triangles.as_mut_ptr() as *mut f32,
            )
        };
        assert_eq!(status, 0, "truck_cuda_subdivide_triangles failed with code: {}", status);

        out_triangles
    }

    /// Generates a 4-to-1 linearly subdivided mesh on the GPU via CUDA.
    pub fn linear_subdivide(&self, mesh: &PolygonMesh) -> PolygonMesh {
        let triangles = super::GpuMeshAnalyzer::extract_triangles(mesh);
        if triangles.is_empty() {
            return mesh.clone();
        }
        let sub_tris = self.subdivide_triangles(&triangles);
        super::create_mesh_from_triangles(&sub_tris)
    }

    /// Evaluates a rational NURBS curve on a batch of parameters on the GPU via CUDA.
    ///
    /// Returns a tuple of `(evaluated_3d_points, evaluated_derivatives)`.
    pub fn evaluate_nurbs_curve(
        &self,
        knots: &[f64],
        degree: usize,
        control_points: &[Vector4],
        t_points: &[f64],
    ) -> (Vec<Point3>, Vec<Vector3>) {
        if t_points.is_empty() || control_points.is_empty() {
            return (Vec::new(), Vec::new());
        }

        let num_knots = knots.len() as u32;
        let num_control_points = control_points.len() as u32;
        let num_points = t_points.len() as u32;

        let flat_cps: Vec<[f64; 4]> = control_points
            .iter()
            .map(|p| [p.x, p.y, p.z, p.w])
            .collect();

        let mut out_points = vec![[0.0f64; 4]; t_points.len()];
        let mut out_derivatives = vec![[0.0f64; 4]; t_points.len()];

        let status = unsafe {
            truck_cuda_evaluate_nurbs_curve_f64(
                knots.as_ptr(),
                num_knots,
                degree as u32,
                flat_cps.as_ptr() as *const f64,
                num_control_points,
                t_points.as_ptr(),
                num_points,
                out_points.as_mut_ptr() as *mut f64,
                out_derivatives.as_mut_ptr() as *mut f64,
            )
        };
        assert_eq!(status, 0, "truck_cuda_evaluate_nurbs_curve_f64 failed with code: {}", status);

        let pts = out_points
            .into_iter()
            .map(|p| Point3::new(p[0], p[1], p[2]))
            .collect();
        let ders = out_derivatives
            .into_iter()
            .map(|n| Vector3::new(n[0], n[1], n[2]))
            .collect();

        (pts, ders)
    }

    /// Raycasts an array of rays against a polygon mesh on the GPU via CUDA.
    ///
    /// For each ray, returns `Some((distance, hit_point, outward_normal))` if an intersection
    /// exists, or `None` if the ray misses the mesh.
    pub fn raycast_mesh(
        &self,
        mesh: &PolygonMesh,
        rays: &[(Point3, Vector3)],
    ) -> Vec<Option<(f64, Point3, Vector3)>> {
        if rays.is_empty() {
            return Vec::new();
        }

        let triangles = super::GpuMeshAnalyzer::extract_triangles(mesh);
        self.raycast_triangles(&triangles, rays)
    }

    /// Alias for [`Self::raycast_mesh`].
    #[inline(always)]
    pub fn raycast(
        &self,
        mesh: &PolygonMesh,
        rays: &[(Point3, Vector3)],
    ) -> Vec<Option<(f64, Point3, Vector3)>> {
        self.raycast_mesh(mesh, rays)
    }

    /// Raycasts an array of rays against a pre-extracted slice of GPU triangles.
    pub fn raycast_triangles(
        &self,
        triangles: &[GpuTriangle],
        rays: &[(Point3, Vector3)],
    ) -> Vec<Option<(f64, Point3, Vector3)>> {
        if rays.is_empty() {
            return Vec::new();
        }

        let num_rays = rays.len() as u32;
        let num_triangles = triangles.len() as u32;

        let ray_origins: Vec<[f32; 4]> = rays
            .iter()
            .map(|(o, _)| [o.x as f32, o.y as f32, o.z as f32, 1.0f32])
            .collect();
        let ray_dirs: Vec<[f32; 4]> = rays
            .iter()
            .map(|(_, d)| {
                let norm = d.normalize();
                [norm.x as f32, norm.y as f32, norm.z as f32, 0.0f32]
            })
            .collect();

        let mut out_distances = vec![0.0f32; rays.len()];
        let mut out_tri_indices = vec![-1i32; rays.len()];
        let mut out_hit_points = vec![[0.0f32; 4]; rays.len()];
        let mut out_normals = vec![[0.0f32; 4]; rays.len()];

        let status = unsafe {
            truck_cuda_raycast_mesh(
                ray_origins.as_ptr() as *const f32,
                ray_dirs.as_ptr() as *const f32,
                num_rays,
                triangles.as_ptr() as *const f32,
                num_triangles,
                out_distances.as_mut_ptr(),
                out_tri_indices.as_mut_ptr(),
                out_hit_points.as_mut_ptr() as *mut f32,
                out_normals.as_mut_ptr() as *mut f32,
            )
        };
        assert_eq!(status, 0, "truck_cuda_raycast_mesh failed with code: {}", status);

        out_distances
            .into_iter()
            .zip(out_hit_points)
            .zip(out_normals)
            .map(|((dist, hit_pt), norm)| {
                if dist >= 0.0 {
                    Some((
                        dist as f64,
                        Point3::new(hit_pt[0] as f64, hit_pt[1] as f64, hit_pt[2] as f64),
                        Vector3::new(norm[0] as f64, norm[1] as f64, norm[2] as f64),
                    ))
                } else {
                    None
                }
            })
            .collect()
    }
}
