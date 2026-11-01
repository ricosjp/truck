//! CUDA-accelerated mesh algorithms targeting NVIDIA Ampere (SM 8.6, RTX 3080).
#![allow(unsafe_code)]

use super::GpuTriangle;
use truck_base::cgmath64::*;
use truck_base::hash::{self, HashGen};
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
}
