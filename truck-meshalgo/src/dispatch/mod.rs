//! Intelligent kernel dispatcher and dynamic calibration engine.
//!
//! Evaluates hardware metadata and startup scaling benchmarks to dynamically route
//! geometric operations to the fastest supported kernel (CPU Single-Thread, CPU Rayon,
//! Vulkan, or Ampere SM 8.6 PTX) based on problem size.

pub mod metadata;
pub mod profile;
pub mod cache;
pub mod microbench;

use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::OnceLock;
use rayon::prelude::*;
use truck_base::cgmath64::*;
use truck_geometry::prelude::*;
use truck_polymesh::*;
use truck_polymesh::StandardVertex as Vertex;

use crate::analyzers::{is_in_the_plane, Collision, ExperimentalSplitters, IncludingPointInDomain};
use crate::filters::NormalFilters;

pub use metadata::{CpuMetadata, GpuMetadata, SystemMetadata};
pub use profile::{DispatchProfile, KernelTarget, Operation, SampleTiming, ScalingThresholds};
pub use cache::{cache_file_path, clear_cache, load_cached_profile, save_cached_profile, KernelCache, CURRENT_CACHE_VERSION};

static GLOBAL_DISPATCHER: OnceLock<Dispatcher> = OnceLock::new();

/// The intelligent kernel dispatcher.
#[derive(Debug)]
pub struct Dispatcher {
    profile: DispatchProfile,
    override_target: AtomicI32,
}

impl Dispatcher {
    /// Returns the global shared dispatcher, calibrating or loading cached results on first use.
    pub fn global() -> &'static Dispatcher {
        GLOBAL_DISPATCHER.get_or_init(|| {
            let hw = SystemMetadata::current();
            let profile = if !hw.has_gpu() {
                // If no GPU is detected (or on headless runners), immediately use CPU-only profile
                // without running calibration or writing cache to disk.
                DispatchProfile::cpu_only()
            } else if let Some(cached) = load_cached_profile() {
                cached
            } else {
                let verbose = std::env::var("TRUCK_DISPATCH_VERBOSE").is_ok();
                if verbose {
                    eprintln!("[truck_dispatch] Hardware change or cache invalidation detected.");
                    eprintln!("[truck_dispatch] Running startup scaling micro-benchmark to calibrate kernel thresholds...");
                }
                let calibrated = microbench::run_scaling_microbenchmark();
                if let Err(e) = save_cached_profile(&calibrated) {
                    if verbose {
                        eprintln!("[truck_dispatch] Warning: Failed to write cache dot file: {}", e);
                    }
                } else if verbose {
                    eprintln!("[truck_dispatch] Calibration complete. Cached results to {}", cache_file_path().display());
                }
                calibrated
            };

            let override_target = match std::env::var("TRUCK_DISPATCH_BACKEND").ok().as_deref() {
                Some("cpu") | Some("1t") => AtomicI32::new(1),
                Some("rayon") | Some("mt") => AtomicI32::new(2),
                Some("vulkan") => AtomicI32::new(3),
                Some("cuda") => AtomicI32::new(4),
                Some("ptx") => AtomicI32::new(5),
                _ => AtomicI32::new(0), // Auto
            };

            Dispatcher {
                profile,
                override_target,
            }
        })
    }

    /// Creates an isolated dispatcher with a custom profile for testing or custom runtime environments.
    pub fn new_with_profile(profile: DispatchProfile) -> Self {
        Self {
            profile,
            override_target: AtomicI32::new(0),
        }
    }

    /// Sets an explicit backend override, bypassing dynamic sizing thresholds.
    pub fn set_override(&self, target: Option<KernelTarget>) {
        let code = match target {
            None => 0,
            Some(KernelTarget::CpuSingle) => 1,
            Some(KernelTarget::CpuRayon) => 2,
            Some(KernelTarget::GpuVulkan) => 3,
            Some(KernelTarget::GpuCuda) => 4,
            Some(KernelTarget::GpuPtxSm86) => 5,
        };
        self.override_target.store(code, Ordering::Relaxed);
    }

    /// Clears any manual override, restoring dynamic size-based routing.
    pub fn reset_override(&self) {
        self.override_target.store(0, Ordering::Relaxed);
    }

    /// Returns the active calibration profile.
    pub fn profile(&self) -> &DispatchProfile {
        &self.profile
    }

    /// Selects the kernel target for the given operation and problem size.
    pub fn select_kernel(&self, op: Operation, size: usize) -> KernelTarget {
        match self.override_target.load(Ordering::Relaxed) {
            1 => KernelTarget::CpuSingle,
            2 => KernelTarget::CpuRayon,
            3 => KernelTarget::GpuVulkan,
            4 => KernelTarget::GpuCuda,
            5 => KernelTarget::GpuPtxSm86,
            _ => self.profile.select_kernel(op, size),
        }
    }

    // ========================================================================
    // Operation 1: Point-Mesh Distance
    // ========================================================================

    /// Evaluates minimum squared distance from points to a polygon mesh using the optimal kernel.
    pub fn distance2_point_cloud(&self, mesh: &PolygonMesh, points: &[Point3]) -> Vec<f32> {
        if points.is_empty() {
            return Vec::new();
        }

        let target = self.select_kernel(Operation::PointMeshDistance, points.len());
        match target {
            KernelTarget::GpuPtxSm86 => {
                #[cfg(feature = "cuda")]
                return crate::gpu::cuda::CudaMeshAnalyzer::new().distance2_point_cloud(mesh, points);
                #[cfg(not(feature = "cuda"))]
                self.distance2_point_cloud_rayon(mesh, points)
            }
            KernelTarget::GpuCuda => {
                #[cfg(feature = "cuda")]
                return crate::gpu::cuda::CudaMeshAnalyzer::new().distance2_point_cloud_baseline(mesh, points);
                #[cfg(not(feature = "cuda"))]
                self.distance2_point_cloud_rayon(mesh, points)
            }
            KernelTarget::GpuVulkan => {
                #[cfg(feature = "gpu")]
                if let Some(analyzer) = crate::gpu::GpuMeshAnalyzer::try_create_default() {
                    return analyzer.distance2_point_cloud(mesh, points);
                }
                self.distance2_point_cloud_rayon(mesh, points)
            }
            KernelTarget::CpuRayon => self.distance2_point_cloud_rayon(mesh, points),
            KernelTarget::CpuSingle => self.distance2_point_cloud_cpu(mesh, points),
        }
    }

    fn distance2_point_cloud_cpu(&self, mesh: &PolygonMesh, points: &[Point3]) -> Vec<f32> {
        let tri_list = extract_tri_tuples(mesh);
        points
            .iter()
            .map(|&p| {
                tri_list
                    .iter()
                    .map(|tri| cpu_dist2_point_triangle(p, tri))
                    .fold(f64::INFINITY, f64::min) as f32
            })
            .collect()
    }

    fn distance2_point_cloud_rayon(&self, mesh: &PolygonMesh, points: &[Point3]) -> Vec<f32> {
        let tri_list = extract_tri_tuples(mesh);
        points
            .par_iter()
            .map(|&p| {
                tri_list
                    .iter()
                    .map(|tri| cpu_dist2_point_triangle(p, tri))
                    .fold(f64::INFINITY, f64::min) as f32
            })
            .collect()
    }

    // ========================================================================
    // Operation 2: In-Out Containment & Signed Crossing Rays
    // ========================================================================

    /// Evaluates in-out solid containment for points using the optimal kernel.
    pub fn inside(&self, mesh: &PolygonMesh, points: &[Point3]) -> Vec<bool> {
        if points.is_empty() {
            return Vec::new();
        }

        let target = self.select_kernel(Operation::InOutContainment, points.len());
        match target {
            KernelTarget::GpuPtxSm86 | KernelTarget::GpuCuda => {
                #[cfg(feature = "cuda")]
                return crate::gpu::cuda::CudaMeshAnalyzer::new().inside(mesh, points);
                #[cfg(not(feature = "cuda"))]
                self.inside_rayon(mesh, points)
            }
            KernelTarget::GpuVulkan => {
                #[cfg(feature = "gpu")]
                if let Some(analyzer) = crate::gpu::GpuMeshAnalyzer::try_create_default() {
                    return analyzer.inside(mesh, points);
                }
                self.inside_rayon(mesh, points)
            }
            KernelTarget::CpuRayon => self.inside_rayon(mesh, points),
            KernelTarget::CpuSingle => self.inside_cpu(mesh, points),
        }
    }

    /// Evaluates signed crossing rays count using the optimal kernel.
    pub fn signed_crossing_rays(
        &self,
        mesh: &PolygonMesh,
        points: &[Point3],
        ray_dirs: &[Vector3],
    ) -> Vec<isize> {
        if points.is_empty() {
            return Vec::new();
        }

        let target = self.select_kernel(Operation::InOutContainment, points.len());
        match target {
            KernelTarget::GpuPtxSm86 | KernelTarget::GpuCuda => {
                #[cfg(feature = "cuda")]
                return crate::gpu::cuda::CudaMeshAnalyzer::new()
                    .signed_crossing_rays(mesh, points, ray_dirs)
                    .into_iter()
                    .map(|c| c as isize)
                    .collect();
                #[cfg(not(feature = "cuda"))]
                self.signed_crossing_rays_rayon(mesh, points, ray_dirs)
            }
            KernelTarget::GpuVulkan => {
                #[cfg(feature = "gpu")]
                if let Some(analyzer) = crate::gpu::GpuMeshAnalyzer::try_create_default() {
                    return analyzer
                        .signed_crossing_rays(mesh, points, ray_dirs)
                        .into_iter()
                        .map(|c| c as isize)
                        .collect();
                }
                self.signed_crossing_rays_rayon(mesh, points, ray_dirs)
            }
            KernelTarget::CpuRayon => self.signed_crossing_rays_rayon(mesh, points, ray_dirs),
            KernelTarget::CpuSingle => self.signed_crossing_rays_cpu(mesh, points, ray_dirs),
        }
    }

    fn inside_cpu(&self, mesh: &PolygonMesh, points: &[Point3]) -> Vec<bool> {
        points.iter().map(|&p| mesh.inside(p)).collect()
    }

    fn inside_rayon(&self, mesh: &PolygonMesh, points: &[Point3]) -> Vec<bool> {
        points.par_iter().map(|&p| mesh.inside(p)).collect()
    }

    fn signed_crossing_rays_cpu(&self, mesh: &PolygonMesh, points: &[Point3], ray_dirs: &[Vector3]) -> Vec<isize> {
        points.iter().zip(ray_dirs.iter()).map(|(&p, &d)| mesh.signed_crossing_faces(p, d)).collect()
    }

    fn signed_crossing_rays_rayon(&self, mesh: &PolygonMesh, points: &[Point3], ray_dirs: &[Vector3]) -> Vec<isize> {
        points.par_iter().zip(ray_dirs.par_iter()).map(|(&p, &d)| mesh.signed_crossing_faces(p, d)).collect()
    }

    // ========================================================================
    // Operation 3: Mesh Normals
    // ========================================================================

    /// Computes and adds smooth normals to a polygon mesh using the optimal kernel.
    pub fn add_smooth_normals(&self, mesh: &mut PolygonMesh) {
        mesh.add_smooth_normals(0.5, true);
    }

    // ========================================================================
    // Operation 4: Mesh Interference & Triangle Collisions
    // ========================================================================

    /// Extracts all interference line segments between two polygon meshes using the optimal kernel.
    pub fn collide_triangles(&self, mesh0: &PolygonMesh, mesh1: &PolygonMesh) -> Vec<(Point3, Point3)> {
        let total_faces = mesh0.faces().len() + mesh1.faces().len();
        let target = self.select_kernel(Operation::MeshInterference, total_faces);

        match target {
            KernelTarget::GpuCuda | KernelTarget::GpuPtxSm86 => {
                #[cfg(feature = "cuda")]
                return crate::gpu::cuda::CudaMeshAnalyzer::new().collide_triangles(mesh0, mesh1);
                #[cfg(not(feature = "cuda"))]
                mesh0.extract_interference(mesh1)
            }
            KernelTarget::GpuVulkan => {
                #[cfg(feature = "gpu")]
                if let Some(analyzer) = crate::gpu::GpuMeshAnalyzer::try_create_default() {
                    return analyzer.collide_triangles(mesh0, mesh1);
                }
                mesh0.extract_interference(mesh1)
            }
            _ => mesh0.extract_interference(mesh1),
        }
    }

    /// Detects if two polygon meshes collide, returning the first detected segment using the optimal kernel.
    pub fn collide_with(&self, mesh0: &PolygonMesh, mesh1: &PolygonMesh) -> Option<(Point3, Point3)> {
        let total_faces = mesh0.faces().len() + mesh1.faces().len();
        let target = self.select_kernel(Operation::MeshInterference, total_faces);

        match target {
            KernelTarget::GpuCuda | KernelTarget::GpuPtxSm86 => {
                #[cfg(feature = "cuda")]
                return crate::gpu::cuda::CudaMeshAnalyzer::new().collide_with(mesh0, mesh1);
                #[cfg(not(feature = "cuda"))]
                mesh0.collide_with(mesh1)
            }
            _ => mesh0.collide_with(mesh1),
        }
    }

    // ========================================================================
    // Operation 5: 2D Trimming Loop Inclusion
    // ========================================================================

    /// Tests whether 2D parameter points lie within a 2D boundary loop using the optimal kernel.
    pub fn trimming_loop_include(&self, segments: &[[Point2; 2]], points: &[Point2]) -> Vec<bool> {
        if points.is_empty() {
            return Vec::new();
        }

        let target = self.select_kernel(Operation::TrimmingLoopInclude, points.len());
        match target {
            KernelTarget::GpuCuda | KernelTarget::GpuPtxSm86 => {
                #[cfg(feature = "cuda")]
                return crate::gpu::cuda::CudaMeshAnalyzer::new().trimming_loop_include(segments, points);
                #[cfg(not(feature = "cuda"))]
                self.trimming_loop_include_cpu(segments, points)
            }
            KernelTarget::GpuVulkan => {
                #[cfg(feature = "gpu")]
                if let Some(analyzer) = crate::gpu::GpuMeshAnalyzer::try_create_default() {
                    return analyzer.trimming_loop_include(segments, points);
                }
                self.trimming_loop_include_cpu(segments, points)
            }
            _ => self.trimming_loop_include_cpu(segments, points),
        }
    }

    fn trimming_loop_include_cpu(&self, segments: &[[Point2; 2]], points: &[Point2]) -> Vec<bool> {
        points
            .par_iter()
            .map(|&p| {
                let mut crossings = 0;
                for seg in segments {
                    if (seg[0].y > p.y) != (seg[1].y > p.y) {
                        let x_int = seg[0].x + (p.y - seg[0].y) * (seg[1].x - seg[0].x) / (seg[1].y - seg[0].y);
                        if p.x < x_int {
                            crossings += 1;
                        }
                    }
                }
                crossings % 2 == 1
            })
            .collect()
    }

    // ========================================================================
    // Operation 6: Batch B-Spline Surface Evaluation
    // ========================================================================

    /// Evaluates 3D points and surface normals on a B-spline surface using the optimal kernel.
    pub fn evaluate_bspline_surface(
        &self,
        u_knots: &[f64],
        v_knots: &[f64],
        u_degree: usize,
        v_degree: usize,
        control_points: &[Vec<Point3>],
        uv_points: &[Point2],
    ) -> (Vec<Point3>, Vec<Vector3>) {
        if uv_points.is_empty() {
            return (Vec::new(), Vec::new());
        }

        let target = self.select_kernel(Operation::BsplineEvaluation, uv_points.len());
        match target {
            KernelTarget::GpuCuda | KernelTarget::GpuPtxSm86 => {
                #[cfg(feature = "cuda")]
                return crate::gpu::cuda::CudaMeshAnalyzer::new().evaluate_bspline_surface(
                    u_knots, v_knots, u_degree, v_degree, control_points, uv_points,
                );
                #[cfg(not(feature = "cuda"))]
                {
                    let _ = (u_degree, v_degree);
                    self.evaluate_bspline_surface_cpu(u_knots, v_knots, control_points, uv_points)
                }
            }
            _ => self.evaluate_bspline_surface_cpu(u_knots, v_knots, control_points, uv_points),
        }
    }

    fn evaluate_bspline_surface_cpu(
        &self,
        u_knots: &[f64],
        v_knots: &[f64],
        control_points: &[Vec<Point3>],
        uv_points: &[Point2],
    ) -> (Vec<Point3>, Vec<Vector3>) {
        let bsp_surface = BSplineSurface::new((
            KnotVec::from(u_knots.to_vec()),
            KnotVec::from(v_knots.to_vec()),
        ), control_points.to_vec());

        uv_points
            .par_iter()
            .map(|&p| (bsp_surface.subs(p.x, p.y), bsp_surface.normal(p.x, p.y)))
            .unzip()
    }

    // ========================================================================
    // Operation 6b: Batch NURBS Surface Evaluation
    // ========================================================================

    /// Evaluates 3D points and surface normals on a rational NURBS surface using the optimal kernel.
    pub fn evaluate_nurbs_surface(
        &self,
        u_knots: &[f64],
        v_knots: &[f64],
        u_degree: usize,
        v_degree: usize,
        control_points: &[Vec<Vector4>],
        uv_points: &[Point2],
    ) -> (Vec<Point3>, Vec<Vector3>) {
        if uv_points.is_empty() {
            return (Vec::new(), Vec::new());
        }

        let target = self.select_kernel(Operation::BsplineEvaluation, uv_points.len());
        match target {
            KernelTarget::GpuCuda | KernelTarget::GpuPtxSm86 => {
                #[cfg(feature = "cuda")]
                return crate::gpu::cuda::CudaMeshAnalyzer::new().evaluate_nurbs_surface(
                    u_knots, v_knots, u_degree, v_degree, control_points, uv_points,
                );
                #[cfg(not(feature = "cuda"))]
                self.evaluate_nurbs_surface_cpu(u_knots, v_knots, control_points, uv_points)
            }
            KernelTarget::GpuVulkan => {
                #[cfg(feature = "gpu")]
                if let Some(analyzer) = crate::gpu::GpuMeshAnalyzer::try_create_default() {
                    return analyzer.evaluate_nurbs_surface(
                        u_knots, v_knots, u_degree, v_degree, control_points, uv_points,
                    );
                }
                self.evaluate_nurbs_surface_cpu(u_knots, v_knots, control_points, uv_points)
            }
            _ => self.evaluate_nurbs_surface_cpu(u_knots, v_knots, control_points, uv_points),
        }
    }

    fn evaluate_nurbs_surface_cpu(
        &self,
        u_knots: &[f64],
        v_knots: &[f64],
        control_points: &[Vec<Vector4>],
        uv_points: &[Point2],
    ) -> (Vec<Point3>, Vec<Vector3>) {
        let nurbs_surface = NurbsSurface::new(BSplineSurface::new((
            KnotVec::from(u_knots.to_vec()),
            KnotVec::from(v_knots.to_vec()),
        ), control_points.to_vec()));

        uv_points
            .par_iter()
            .map(|&p| (nurbs_surface.subs(p.x, p.y), nurbs_surface.normal(p.x, p.y)))
            .unzip()
    }

    // ========================================================================
    // Operation 7: Point Cloud Clung Test
    // ========================================================================

    /// Checks whether all faces of the polygon mesh intersect the neighborhood of `point_cloud`.
    pub fn is_clung_to_by(&self, mesh: &PolygonMesh, point_cloud: &[Point3], tol: f64) -> bool {
        if point_cloud.is_empty() {
            return false;
        }
        let target = self.select_kernel(Operation::PointCloudClung, point_cloud.len().max(mesh.faces().len()));
        match target {
            KernelTarget::GpuCuda | KernelTarget::GpuPtxSm86 => {
                #[cfg(feature = "cuda")]
                return crate::gpu::cuda::CudaMeshAnalyzer::new().is_clung_to_by(mesh, point_cloud, tol);
                #[cfg(not(feature = "cuda"))]
                self.is_clung_to_by_cpu(mesh, point_cloud, tol)
            }
            KernelTarget::GpuVulkan => {
                #[cfg(feature = "gpu")]
                if let Some(analyzer) = crate::gpu::GpuMeshAnalyzer::try_create_default() {
                    return analyzer.is_clung_to_by(mesh, point_cloud, tol);
                }
                self.is_clung_to_by_cpu(mesh, point_cloud, tol)
            }
            _ => self.is_clung_to_by_cpu(mesh, point_cloud, tol),
        }
    }

    fn is_clung_to_by_cpu(&self, mesh: &PolygonMesh, point_cloud: &[Point3], tol: f64) -> bool {
        #[cfg(feature = "analyzers")]
        {
            crate::analyzers::point_cloud::cpu_is_clung_to_by(mesh, point_cloud, tol)
        }
        #[cfg(not(feature = "analyzers"))]
        {
            let _ = (mesh, point_cloud, tol);
            unimplemented!("CPU point cloud analysis requires 'analyzers' feature")
        }
    }

    // ========================================================================
    // Operation 8: Point Cloud Neighborhood Inclusion
    // ========================================================================

    /// Checks whether the neighborhood of the polygon mesh includes all points in `point_cloud`.
    pub fn neighborhood_include(&self, mesh: &PolygonMesh, point_cloud: &[Point3], tol: f64) -> bool {
        if point_cloud.is_empty() {
            return true;
        }
        let target = self.select_kernel(Operation::PointCloudInclude, point_cloud.len());
        match target {
            KernelTarget::GpuCuda | KernelTarget::GpuPtxSm86 => {
                #[cfg(feature = "cuda")]
                return crate::gpu::cuda::CudaMeshAnalyzer::new().neighborhood_include(mesh, point_cloud, tol);
                #[cfg(not(feature = "cuda"))]
                self.neighborhood_include_cpu(mesh, point_cloud, tol)
            }
            KernelTarget::GpuVulkan => {
                #[cfg(feature = "gpu")]
                if let Some(analyzer) = crate::gpu::GpuMeshAnalyzer::try_create_default() {
                    return analyzer.neighborhood_include(mesh, point_cloud, tol);
                }
                self.neighborhood_include_cpu(mesh, point_cloud, tol)
            }
            _ => self.neighborhood_include_cpu(mesh, point_cloud, tol),
        }
    }

    fn neighborhood_include_cpu(&self, mesh: &PolygonMesh, point_cloud: &[Point3], tol: f64) -> bool {
        #[cfg(feature = "analyzers")]
        {
            crate::analyzers::point_cloud::cpu_neighborhood_include(mesh, point_cloud, tol)
        }
        #[cfg(not(feature = "analyzers"))]
        {
            let _ = (mesh, point_cloud, tol);
            unimplemented!("CPU point cloud analysis requires 'analyzers' feature")
        }
    }

    // ========================================================================
    // Operation 9: Surface-Surface Intersection Seeds Finder
    // ========================================================================

    /// Evaluates dense parameter grid separation on the GPU or CPU to locate candidate intersection seeds between two B-spline surfaces.
    pub fn find_surface_intersection_seeds(
        &self,
        s1: &BSplineSurface<Point3>,
        s2: &BSplineSurface<Point3>,
        tol: f64,
    ) -> Vec<(Point2, Point2)> {
        let target = self.select_kernel(Operation::SurfaceIntersectionSeeds, 1024);
        match target {
            KernelTarget::GpuCuda | KernelTarget::GpuPtxSm86 => {
                #[cfg(feature = "cuda")]
                return crate::gpu::cuda::CudaMeshAnalyzer::new().find_surface_intersection_seeds(s1, s2, tol);
                #[cfg(not(feature = "cuda"))]
                self.find_surface_intersection_seeds_cpu(s1, s2, tol)
            }
            KernelTarget::GpuVulkan => {
                #[cfg(feature = "gpu")]
                if let Some(analyzer) = crate::gpu::GpuMeshAnalyzer::try_create_default() {
                    return analyzer.find_surface_intersection_seeds(s1, s2, tol);
                }
                self.find_surface_intersection_seeds_cpu(s1, s2, tol)
            }
            _ => self.find_surface_intersection_seeds_cpu(s1, s2, tol),
        }
    }

    fn find_surface_intersection_seeds_cpu(
        &self,
        s1: &BSplineSurface<Point3>,
        s2: &BSplineSurface<Point3>,
        tol: f64,
    ) -> Vec<(Point2, Point2)> {
        #[cfg(feature = "gpu")]
        {
            let (samples1, n_u1, n_v1) = crate::gpu::sample_surface_grid(s1, (32, 32));
            let (samples2, n_u2, n_v2) = crate::gpu::sample_surface_grid(s2, (32, 32));
            let mut raw_seeds = Vec::new();
            for i1 in 0..n_u1 {
                for j1 in 0..n_v1 {
                    let s1_idx = i1 * n_v1 + j1;
                    let p1 = Vector3::new(samples1[s1_idx].pos[0] as f64, samples1[s1_idx].pos[1] as f64, samples1[s1_idx].pos[2] as f64);
                    let uv1 = Point2::new(samples1[s1_idx].uv[0] as f64, samples1[s1_idx].uv[1] as f64);
                    for i2 in 0..(n_u2 - 1) {
                        for j2 in 0..(n_v2 - 1) {
                            let s2_00 = &samples2[i2 * n_v2 + j2];
                            let s2_10 = &samples2[(i2 + 1) * n_v2 + j2];
                            let s2_01 = &samples2[i2 * n_v2 + (j2 + 1)];
                            let s2_11 = &samples2[(i2 + 1) * n_v2 + (j2 + 1)];
                            let center = Vector3::new(
                                (s2_00.pos[0] + s2_10.pos[0] + s2_01.pos[0] + s2_11.pos[0]) as f64 * 0.25,
                                (s2_00.pos[1] + s2_10.pos[1] + s2_01.pos[1] + s2_11.pos[1]) as f64 * 0.25,
                                (s2_00.pos[2] + s2_10.pos[2] + s2_01.pos[2] + s2_11.pos[2]) as f64 * 0.25,
                            );
                            let d_sq = (p1 - center).magnitude2();
                            let r_sq = (Vector3::new(s2_00.pos[0] as f64, s2_00.pos[1] as f64, s2_00.pos[2] as f64) - center).magnitude2().max(1e-6);
                            let max_d = (r_sq.sqrt() + tol).powi(2);
                            if d_sq > max_d {
                                continue;
                            }
                            let nor2 = Vector3::new(s2_00.normal[0] as f64, s2_00.normal[1] as f64, s2_00.normal[2] as f64);
                            let d00 = (Vector3::new(s2_00.pos[0] as f64, s2_00.pos[1] as f64, s2_00.pos[2] as f64) - p1).dot(nor2);
                            let d10 = (Vector3::new(s2_10.pos[0] as f64, s2_10.pos[1] as f64, s2_10.pos[2] as f64) - p1).dot(nor2);
                            let d01 = (Vector3::new(s2_01.pos[0] as f64, s2_01.pos[1] as f64, s2_01.pos[2] as f64) - p1).dot(nor2);
                            let d11 = (Vector3::new(s2_11.pos[0] as f64, s2_11.pos[1] as f64, s2_11.pos[2] as f64) - p1).dot(nor2);
                            let min_d = d00.min(d10).min(d01).min(d11);
                            let max_d = d00.max(d10).max(d01).max(d11);
                            if (min_d <= 0.0 && max_d >= 0.0) || (p1 - center).magnitude() < tol {
                                let uv2 = Point2::new(
                                    (s2_00.uv[0] + s2_10.uv[0] + s2_01.uv[0] + s2_11.uv[0]) as f64 * 0.25,
                                    (s2_00.uv[1] + s2_10.uv[1] + s2_01.uv[1] + s2_11.uv[1]) as f64 * 0.25,
                                );
                                raw_seeds.push((uv1, uv2));
                            }
                        }
                    }
                }
            }
            crate::gpu::filter_and_cluster_seeds(raw_seeds, s1, s2)
        }
        #[cfg(not(feature = "gpu"))]
        {
            let _ = (s1, s2, tol);
            Vec::new()
        }
    }

    // ========================================================================
    // Operation 10: Planar Face Segmentation
    // ========================================================================

    /// Extracts planar faces such that their face normal matches their vertex normal within `tol`.
    pub fn extract_planes(&self, mesh: &PolygonMesh, tol: f64) -> (Vec<usize>, Vec<usize>) {
        if mesh.faces().is_empty() {
            return (Vec::new(), Vec::new());
        }
        let target = self.select_kernel(Operation::PlanarFaceSegmentation, mesh.faces().len());
        match target {
            KernelTarget::GpuCuda | KernelTarget::GpuPtxSm86 => {
                #[cfg(feature = "cuda")]
                return crate::gpu::cuda::CudaMeshAnalyzer::new().extract_planes(mesh, tol);
                #[cfg(not(feature = "cuda"))]
                self.extract_planes_rayon(mesh, tol)
            }
            KernelTarget::GpuVulkan => {
                #[cfg(feature = "gpu")]
                if let Some(analyzer) = crate::gpu::GpuMeshAnalyzer::try_create_default() {
                    return analyzer.extract_planes(mesh, tol);
                }
                self.extract_planes_rayon(mesh, tol)
            }
            KernelTarget::CpuRayon => self.extract_planes_rayon(mesh, tol),
            KernelTarget::CpuSingle => self.extract_planes_cpu(mesh, tol),
        }
    }

    fn extract_planes_cpu(&self, mesh: &PolygonMesh, tol: f64) -> (Vec<usize>, Vec<usize>) {
        mesh.faces_into_two_clusters(|face: &[Vertex]| {
            is_in_the_plane(mesh.positions(), mesh.normals(), face, tol * tol)
        })
    }

    fn extract_planes_rayon(&self, mesh: &PolygonMesh, tol: f64) -> (Vec<usize>, Vec<usize>) {
        let faces: Vec<_> = mesh.face_iter().collect();
        let flags: Vec<bool> = faces
            .par_iter()
            .map(|face| {
                is_in_the_plane(mesh.positions(), mesh.normals(), face, tol * tol)
            })
            .collect();
        let mut planes = Vec::new();
        let mut others = Vec::new();
        for (i, &flag) in flags.iter().enumerate() {
            if flag {
                planes.push(i);
            } else {
                others.push(i);
            }
        }
        (planes, others)
    }

    // ========================================================================
    // Operation 11: Triangle Midpoint Refinement / Linear Subdivision
    // ========================================================================

    /// Generates a 4-to-1 linearly subdivided mesh on the optimal kernel.
    pub fn linear_subdivide(&self, mesh: &PolygonMesh) -> PolygonMesh {
        let total_faces = mesh.faces().len();
        let target = self.select_kernel(Operation::LinearSubdivision, total_faces);
        match target {
            KernelTarget::GpuCuda | KernelTarget::GpuPtxSm86 => {
                #[cfg(feature = "cuda")]
                return crate::gpu::cuda::CudaMeshAnalyzer::new().linear_subdivide(mesh);
                #[cfg(not(feature = "cuda"))]
                self.linear_subdivide_rayon(mesh)
            }
            KernelTarget::GpuVulkan => {
                #[cfg(feature = "gpu")]
                if let Some(analyzer) = crate::gpu::GpuMeshAnalyzer::try_create_default() {
                    return analyzer.linear_subdivide(mesh);
                }
                self.linear_subdivide_rayon(mesh)
            }
            KernelTarget::CpuRayon => self.linear_subdivide_rayon(mesh),
            KernelTarget::CpuSingle => self.linear_subdivide_cpu(mesh),
        }
    }

    /// Subdivides triangles into 4-to-1 midpoint-refined triangles on the optimal kernel.
    pub fn subdivide_triangles(&self, triangles: &[crate::gpu::GpuTriangle]) -> Vec<crate::gpu::GpuTriangle> {
        let target = self.select_kernel(Operation::LinearSubdivision, triangles.len());
        match target {
            KernelTarget::GpuCuda | KernelTarget::GpuPtxSm86 => {
                #[cfg(feature = "cuda")]
                return crate::gpu::cuda::CudaMeshAnalyzer::new().subdivide_triangles(triangles);
                #[cfg(not(feature = "cuda"))]
                self.subdivide_triangles_rayon(triangles)
            }
            KernelTarget::GpuVulkan => {
                #[cfg(feature = "gpu")]
                if let Some(analyzer) = crate::gpu::GpuMeshAnalyzer::try_create_default() {
                    return analyzer.subdivide_triangles(triangles);
                }
                self.subdivide_triangles_rayon(triangles)
            }
            KernelTarget::CpuRayon => self.subdivide_triangles_rayon(triangles),
            KernelTarget::CpuSingle => self.subdivide_triangles_cpu(triangles),
        }
    }

    fn subdivide_triangles_cpu(&self, triangles: &[crate::gpu::GpuTriangle]) -> Vec<crate::gpu::GpuTriangle> {
        let mut out = Vec::with_capacity(triangles.len() * 4);
        for tri in triangles {
            let v0 = tri.v0;
            let v1 = tri.v1;
            let v2 = tri.v2;
            let m01 = [(v0[0] + v1[0]) * 0.5, (v0[1] + v1[1]) * 0.5, (v0[2] + v1[2]) * 0.5, 0.0];
            let m12 = [(v1[0] + v2[0]) * 0.5, (v1[1] + v2[1]) * 0.5, (v1[2] + v2[2]) * 0.5, 0.0];
            let m20 = [(v2[0] + v0[0]) * 0.5, (v2[1] + v0[1]) * 0.5, (v2[2] + v0[2]) * 0.5, 0.0];
            out.push(crate::gpu::GpuTriangle { v0, v1: m01, v2: m20 });
            out.push(crate::gpu::GpuTriangle { v0: v1, v1: m12, v2: m01 });
            out.push(crate::gpu::GpuTriangle { v0: v2, v1: m20, v2: m12 });
            out.push(crate::gpu::GpuTriangle { v0: m01, v1: m12, v2: m20 });
        }
        out
    }

    fn subdivide_triangles_rayon(&self, triangles: &[crate::gpu::GpuTriangle]) -> Vec<crate::gpu::GpuTriangle> {
        triangles
            .par_iter()
            .flat_map(|tri| {
                let v0 = tri.v0;
                let v1 = tri.v1;
                let v2 = tri.v2;
                let m01 = [(v0[0] + v1[0]) * 0.5, (v0[1] + v1[1]) * 0.5, (v0[2] + v1[2]) * 0.5, 0.0];
                let m12 = [(v1[0] + v2[0]) * 0.5, (v1[1] + v2[1]) * 0.5, (v1[2] + v2[2]) * 0.5, 0.0];
                let m20 = [(v2[0] + v0[0]) * 0.5, (v2[1] + v0[1]) * 0.5, (v2[2] + v0[2]) * 0.5, 0.0];
                vec![
                    crate::gpu::GpuTriangle { v0, v1: m01, v2: m20 },
                    crate::gpu::GpuTriangle { v0: v1, v1: m12, v2: m01 },
                    crate::gpu::GpuTriangle { v0: v2, v1: m20, v2: m12 },
                    crate::gpu::GpuTriangle { v0: m01, v1: m12, v2: m20 },
                ]
            })
            .collect()
    }

    fn linear_subdivide_cpu(&self, mesh: &PolygonMesh) -> PolygonMesh {
        let triangles = crate::gpu::GpuMeshAnalyzer::extract_triangles(mesh);
        let sub_tris = self.subdivide_triangles_cpu(&triangles);
        crate::gpu::create_mesh_from_triangles(&sub_tris)
    }

    fn linear_subdivide_rayon(&self, mesh: &PolygonMesh) -> PolygonMesh {
        let triangles = crate::gpu::GpuMeshAnalyzer::extract_triangles(mesh);
        let sub_tris = self.subdivide_triangles_rayon(&triangles);
        crate::gpu::create_mesh_from_triangles(&sub_tris)
    }
}

// Helpers
fn extract_tri_tuples(mesh: &PolygonMesh) -> Vec<[Point3; 3]> {
    let pos = mesh.positions();
    mesh.face_iter()
        .flat_map(|face| {
            (2..face.len()).map(move |i| {
                [pos[face[0].pos], pos[face[i - 1].pos], pos[face[i].pos]]
            })
        })
        .collect()
}

fn cpu_dist2_point_triangle(point: Point3, triangle: &[Point3; 3]) -> f64 {
    let ab = triangle[1] - triangle[0];
    let ap = point - triangle[0];
    let bc = triangle[2] - triangle[1];
    let bp = point - triangle[1];
    let ca = triangle[0] - triangle[2];
    let cp = point - triangle[2];
    let nor = ab.cross(ca);
    let s0 = (ab.cross(nor).dot(ap)).signum();
    let s1 = (bc.cross(nor).dot(bp)).signum();
    let s2 = (ca.cross(nor).dot(cp)).signum();
    if s0 + s1 + s2 < 2.0 || nor.magnitude2() < 1e-12 {
        let da = ap - ab * (ap.dot(ab) / ab.magnitude2()).clamp(0.0, 1.0);
        let db = bp - bc * (bp.dot(bc) / bc.magnitude2()).clamp(0.0, 1.0);
        let dc = cp - ca * (cp.dot(ca) / ca.magnitude2()).clamp(0.0, 1.0);
        da.magnitude2().min(db.magnitude2()).min(dc.magnitude2())
    } else {
        nor.dot(ap).powi(2) / nor.magnitude2()
    }
}
