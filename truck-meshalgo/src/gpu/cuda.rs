//! CUDA-accelerated mesh algorithms targeting NVIDIA Ampere (SM 8.6, RTX 3080).
#![allow(unsafe_code)]

use super::GpuTriangle;
use truck_base::cgmath64::*;
use truck_base::hash;
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
}
