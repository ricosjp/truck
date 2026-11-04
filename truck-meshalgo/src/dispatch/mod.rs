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
use crate::filters::{NormalFilters, StructuringFilter};

pub use metadata::{CpuMetadata, GpuMetadata, SystemMetadata};
pub use profile::{DispatchProfile, KernelTarget, Operation, SampleTiming, ScalingThresholds};
pub use cache::{cache_file_path, clear_cache, load_cached_profile, save_cached_profile, KernelCache, CURRENT_CACHE_VERSION};

#[cfg(feature = "gpu")]
pub use crate::gpu::bvh::{build_bvh, build_mesh_bvh, GpuBvhNode};
#[cfg(feature = "cuda")]
pub use crate::gpu::bvh::CudaMeshBvh;

/// Eagerly warms up the dispatcher and preloads GPU execution environments (CUDA contexts, PTX modules, scratchpads, and Vulkan compute pipelines).
///
/// Calling this during application startup or splash screens (optionally on a background thread)
/// eliminates the one-time 50ms–250ms cold-start JIT and allocation latency on the first dispatched call.
pub fn warmup() {
    Dispatcher::warmup();
}

static GLOBAL_DISPATCHER: OnceLock<Dispatcher> = OnceLock::new();

/// The intelligent kernel dispatcher.
#[derive(Debug)]
pub struct Dispatcher {
    profile: DispatchProfile,
    override_target: AtomicI32,
}

impl Dispatcher {
    /// Eagerly warms up the dispatcher and preloads GPU execution environments (CUDA contexts, PTX modules, scratchpads, and Vulkan compute pipelines).
    pub fn warmup() {
        let _ = Dispatcher::global();

        #[cfg(feature = "cuda")]
        {
            crate::gpu::cuda::CudaMeshAnalyzer::preload_all();
        }

        #[cfg(feature = "gpu")]
        {
            let _ = get_vulkan_analyzers();
        }
    }

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
                Some("bvh") => AtomicI32::new(6),
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
            Some(KernelTarget::GpuBvh) => 6,
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
            6 => KernelTarget::GpuBvh,
            _ => self.profile.select_kernel(op, size),
        }
    }
}

// ============================================================================
// Multi-GPU Batch Execution Helpers
// ============================================================================

#[cfg(feature = "cuda")]
fn multi_gpu_cuda_split<T, R, F>(items: &[T], f: F) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&crate::gpu::cuda::CudaMeshAnalyzer, &[T]) -> Vec<R> + Sync,
{
    let hw = SystemMetadata::current();
    let devices = &hw.gpu.cuda_devices;
    if devices.len() <= 1 || items.len() < 1024 {
        return f(&crate::gpu::cuda::CudaMeshAnalyzer::new(), items);
    }

    if devices.len() == 2 {
        let sm0 = devices[0].multi_processor_count.max(1) as usize;
        let sm1 = devices[1].multi_processor_count.max(1) as usize;
        let total_sms = sm0 + sm1;
        let split_idx = ((items.len() * sm0) / total_sms).clamp(1, items.len() - 1);

        let dev0 = devices[0].device_id;
        let dev1 = devices[1].device_id;

        let (mut res0, res1) = rayon::join(
            || f(&crate::gpu::cuda::CudaMeshAnalyzer::with_device(dev0), &items[..split_idx]),
            || f(&crate::gpu::cuda::CudaMeshAnalyzer::with_device(dev1), &items[split_idx..]),
        );
        res0.extend(res1);
        res0
    } else {
        let total_sms: usize = devices.iter().map(|d| d.multi_processor_count.max(1) as usize).sum();
        let mut starts_ends = Vec::with_capacity(devices.len());
        let mut start = 0;
        for (i, d) in devices.iter().enumerate() {
            if i == devices.len() - 1 {
                starts_ends.push((d.device_id, start, items.len()));
            } else {
                let cnt = (items.len() * (d.multi_processor_count.max(1) as usize)) / total_sms;
                let end = (start + cnt).min(items.len());
                starts_ends.push((d.device_id, start, end));
                start = end;
            }
        }
        starts_ends
            .into_par_iter()
            .flat_map(|(dev_id, s, e)| {
                if s < e {
                    f(&crate::gpu::cuda::CudaMeshAnalyzer::with_device(dev_id), &items[s..e])
                } else {
                    Vec::new()
                }
            })
            .collect()
    }
}

#[cfg(feature = "cuda")]
fn multi_gpu_cuda_split_pair<T, U, R, F>(items1: &[T], items2: &[U], f: F) -> Vec<R>
where
    T: Sync,
    U: Sync,
    R: Send,
    F: Fn(&crate::gpu::cuda::CudaMeshAnalyzer, &[T], &[U]) -> Vec<R> + Sync,
{
    assert_eq!(items1.len(), items2.len());
    let hw = SystemMetadata::current();
    let devices = &hw.gpu.cuda_devices;
    if devices.len() <= 1 || items1.len() < 1024 {
        return f(&crate::gpu::cuda::CudaMeshAnalyzer::new(), items1, items2);
    }

    if devices.len() == 2 {
        let sm0 = devices[0].multi_processor_count.max(1) as usize;
        let sm1 = devices[1].multi_processor_count.max(1) as usize;
        let total_sms = sm0 + sm1;
        let split_idx = ((items1.len() * sm0) / total_sms).clamp(1, items1.len() - 1);

        let dev0 = devices[0].device_id;
        let dev1 = devices[1].device_id;

        let (mut res0, res1) = rayon::join(
            || f(
                &crate::gpu::cuda::CudaMeshAnalyzer::with_device(dev0),
                &items1[..split_idx],
                &items2[..split_idx],
            ),
            || f(
                &crate::gpu::cuda::CudaMeshAnalyzer::with_device(dev1),
                &items1[split_idx..],
                &items2[split_idx..],
            ),
        );
        res0.extend(res1);
        res0
    } else {
        let total_sms: usize = devices.iter().map(|d| d.multi_processor_count.max(1) as usize).sum();
        let mut starts_ends = Vec::with_capacity(devices.len());
        let mut start = 0;
        for (i, d) in devices.iter().enumerate() {
            if i == devices.len() - 1 {
                starts_ends.push((d.device_id, start, items1.len()));
            } else {
                let cnt = (items1.len() * (d.multi_processor_count.max(1) as usize)) / total_sms;
                let end = (start + cnt).min(items1.len());
                starts_ends.push((d.device_id, start, end));
                start = end;
            }
        }
        starts_ends
            .into_par_iter()
            .flat_map(|(dev_id, s, e)| {
                if s < e {
                    f(&crate::gpu::cuda::CudaMeshAnalyzer::with_device(dev_id), &items1[s..e], &items2[s..e])
                } else {
                    Vec::new()
                }
            })
            .collect()
    }
}

#[cfg(feature = "gpu")]
static GLOBAL_VULKAN_ANALYZERS: OnceLock<Vec<crate::gpu::GpuMeshAnalyzer>> = OnceLock::new();

#[cfg(feature = "gpu")]
fn get_vulkan_analyzers() -> &'static [crate::gpu::GpuMeshAnalyzer] {
    GLOBAL_VULKAN_ANALYZERS.get_or_init(crate::gpu::GpuMeshAnalyzer::try_create_all).as_slice()
}

#[cfg(feature = "gpu")]
fn multi_gpu_vulkan_split<T, R, F>(items: &[T], f: F) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&crate::gpu::GpuMeshAnalyzer, &[T]) -> Vec<R> + Sync,
{
    let analyzers = get_vulkan_analyzers();
    if analyzers.is_empty() {
        return Vec::new();
    }
    if analyzers.len() <= 1 || items.len() < 1024 {
        return f(&analyzers[0], items);
    }

    if analyzers.len() == 2 {
        let split_idx = (items.len() / 2).clamp(1, items.len() - 1);
        let (mut res0, res1) = rayon::join(
            || f(&analyzers[0], &items[..split_idx]),
            || f(&analyzers[1], &items[split_idx..]),
        );
        res0.extend(res1);
        res0
    } else {
        let chunk_size = (items.len() + analyzers.len() - 1) / analyzers.len();
        analyzers
            .par_iter()
            .enumerate()
            .flat_map(|(i, analyzer)| {
                let start = (i * chunk_size).min(items.len());
                let end = ((i + 1) * chunk_size).min(items.len());
                if start < end {
                    f(analyzer, &items[start..end])
                } else {
                    Vec::new()
                }
            })
            .collect()
    }
}

#[cfg(feature = "gpu")]
fn multi_gpu_vulkan_split_pair<T, U, R, F>(items1: &[T], items2: &[U], f: F) -> Vec<R>
where
    T: Sync,
    U: Sync,
    R: Send,
    F: Fn(&crate::gpu::GpuMeshAnalyzer, &[T], &[U]) -> Vec<R> + Sync,
{
    assert_eq!(items1.len(), items2.len());
    let analyzers = get_vulkan_analyzers();
    if analyzers.is_empty() {
        return Vec::new();
    }
    if analyzers.len() <= 1 || items1.len() < 1024 {
        return f(&analyzers[0], items1, items2);
    }

    if analyzers.len() == 2 {
        let split_idx = (items1.len() / 2).clamp(1, items1.len() - 1);
        let (mut res0, res1) = rayon::join(
            || f(&analyzers[0], &items1[..split_idx], &items2[..split_idx]),
            || f(&analyzers[1], &items1[split_idx..], &items2[split_idx..]),
        );
        res0.extend(res1);
        res0
    } else {
        let chunk_size = (items1.len() + analyzers.len() - 1) / analyzers.len();
        analyzers
            .par_iter()
            .enumerate()
            .flat_map(|(i, analyzer)| {
                let start = (i * chunk_size).min(items1.len());
                let end = ((i + 1) * chunk_size).min(items1.len());
                if start < end {
                    f(analyzer, &items1[start..end], &items2[start..end])
                } else {
                    Vec::new()
                }
            })
            .collect()
    }
}

impl Dispatcher {
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
            KernelTarget::GpuBvh | KernelTarget::GpuPtxSm86 => {
                #[cfg(feature = "cuda")]
                return multi_gpu_cuda_split(points, |analyzer, pts| {
                    analyzer.distance2_point_cloud(mesh, pts)
                });
                #[cfg(not(feature = "cuda"))]
                self.distance2_point_cloud_rayon(mesh, points)
            }
            KernelTarget::GpuCuda => {
                #[cfg(feature = "cuda")]
                return multi_gpu_cuda_split(points, |analyzer, pts| {
                    analyzer.distance2_point_cloud_baseline(mesh, pts)
                });
                #[cfg(not(feature = "cuda"))]
                self.distance2_point_cloud_rayon(mesh, points)
            }
            KernelTarget::GpuVulkan => {
                #[cfg(feature = "gpu")]
                {
                    let res = multi_gpu_vulkan_split(points, |analyzer, pts| {
                        analyzer.distance2_point_cloud(mesh, pts)
                    });
                    if !res.is_empty() {
                        return res;
                    }
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
            KernelTarget::GpuBvh => {
                #[cfg(feature = "cuda")]
                return multi_gpu_cuda_split(points, |analyzer, pts| {
                    analyzer.inside_bvh(mesh, pts)
                });
                #[cfg(not(feature = "cuda"))]
                self.inside_rayon(mesh, points)
            }
            KernelTarget::GpuPtxSm86 | KernelTarget::GpuCuda => {
                #[cfg(feature = "cuda")]
                return multi_gpu_cuda_split(points, |analyzer, pts| {
                    analyzer.inside(mesh, pts)
                });
                #[cfg(not(feature = "cuda"))]
                self.inside_rayon(mesh, points)
            }
            KernelTarget::GpuVulkan => {
                #[cfg(feature = "gpu")]
                {
                    let res = multi_gpu_vulkan_split(points, |analyzer, pts| {
                        analyzer.inside(mesh, pts)
                    });
                    if !res.is_empty() {
                        return res;
                    }
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
            KernelTarget::GpuBvh => {
                #[cfg(feature = "cuda")]
                return multi_gpu_cuda_split_pair(points, ray_dirs, |analyzer, pts, dirs| {
                    analyzer.signed_crossing_rays_bvh(mesh, pts, dirs)
                })
                .into_iter()
                .map(|c| c as isize)
                .collect();
                #[cfg(not(feature = "cuda"))]
                self.signed_crossing_rays_rayon(mesh, points, ray_dirs)
            }
            KernelTarget::GpuPtxSm86 | KernelTarget::GpuCuda => {
                #[cfg(feature = "cuda")]
                return multi_gpu_cuda_split_pair(points, ray_dirs, |analyzer, pts, dirs| {
                    analyzer.signed_crossing_rays(mesh, pts, dirs)
                })
                .into_iter()
                .map(|c| c as isize)
                .collect();
                #[cfg(not(feature = "cuda"))]
                self.signed_crossing_rays_rayon(mesh, points, ray_dirs)
            }
            KernelTarget::GpuVulkan => {
                #[cfg(feature = "gpu")]
                {
                    let res = multi_gpu_vulkan_split_pair(points, ray_dirs, |analyzer, pts, dirs| {
                        analyzer.signed_crossing_rays(mesh, pts, dirs)
                    });
                    if !res.is_empty() {
                        return res.into_iter().map(|c| c as isize).collect();
                    }
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
    // Operation 6c: Batch Rational NURBS & B-Spline Curve Evaluation
    // ========================================================================

    /// Evaluates 3D points and derivatives on a rational NURBS curve using the optimal kernel.
    pub fn evaluate_nurbs_curve(
        &self,
        knots: &[f64],
        degree: usize,
        control_points: &[Vector4],
        t_points: &[f64],
    ) -> (Vec<Point3>, Vec<Vector3>) {
        if t_points.is_empty() {
            return (Vec::new(), Vec::new());
        }

        let target = self.select_kernel(Operation::NurbsCurveEvaluation, t_points.len());
        match target {
            KernelTarget::GpuCuda | KernelTarget::GpuPtxSm86 => {
                #[cfg(feature = "cuda")]
                return crate::gpu::cuda::CudaMeshAnalyzer::new().evaluate_nurbs_curve(
                    knots, degree, control_points, t_points,
                );
                #[cfg(not(feature = "cuda"))]
                self.evaluate_nurbs_curve_cpu(knots, control_points, t_points)
            }
            KernelTarget::GpuVulkan => {
                #[cfg(feature = "gpu")]
                if let Some(analyzer) = crate::gpu::GpuMeshAnalyzer::try_create_default() {
                    return analyzer.evaluate_nurbs_curve(
                        knots, degree, control_points, t_points,
                    );
                }
                self.evaluate_nurbs_curve_cpu(knots, control_points, t_points)
            }
            _ => self.evaluate_nurbs_curve_cpu(knots, control_points, t_points),
        }
    }

    fn evaluate_nurbs_curve_cpu(
        &self,
        knots: &[f64],
        control_points: &[Vector4],
        t_points: &[f64],
    ) -> (Vec<Point3>, Vec<Vector3>) {
        let nurbs_curve = NurbsCurve::new(BSplineCurve::new(
            KnotVec::from(knots.to_vec()),
            control_points.to_vec(),
        ));

        t_points
            .par_iter()
            .map(|&t| (nurbs_curve.subs(t), nurbs_curve.der(t)))
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
            KernelTarget::GpuCuda | KernelTarget::GpuPtxSm86 | KernelTarget::GpuBvh => {
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
            KernelTarget::GpuCuda | KernelTarget::GpuPtxSm86 | KernelTarget::GpuBvh => {
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
            KernelTarget::GpuCuda | KernelTarget::GpuPtxSm86 | KernelTarget::GpuBvh => {
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

    // ========================================================================
    // Operation 13: Batch Ray-to-Mesh Raycasting
    // ========================================================================

    /// Raycasts an array of rays against a polygon mesh using the optimal kernel.
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

        let target = self.select_kernel(Operation::RaycastMesh, rays.len());
        match target {
            KernelTarget::GpuBvh => {
                #[cfg(feature = "cuda")]
                return multi_gpu_cuda_split(rays, |analyzer, r| {
                    analyzer.raycast_mesh_bvh(mesh, r)
                });
                #[cfg(not(feature = "cuda"))]
                crate::analyzers::raycast_mesh_rayon(mesh, rays)
            }
            KernelTarget::GpuCuda | KernelTarget::GpuPtxSm86 => {
                #[cfg(feature = "cuda")]
                return multi_gpu_cuda_split(rays, |analyzer, r| {
                    analyzer.raycast_mesh(mesh, r)
                });
                #[cfg(not(feature = "cuda"))]
                crate::analyzers::raycast_mesh_rayon(mesh, rays)
            }
            KernelTarget::GpuVulkan => {
                #[cfg(feature = "gpu")]
                {
                    let res = multi_gpu_vulkan_split(rays, |analyzer, r| {
                        analyzer.raycast_mesh(mesh, r)
                    });
                    if !res.is_empty() {
                        return res;
                    }
                }
                crate::analyzers::raycast_mesh_rayon(mesh, rays)
            }
            KernelTarget::CpuRayon => crate::analyzers::raycast_mesh_rayon(mesh, rays),
            KernelTarget::CpuSingle => crate::analyzers::raycast_mesh_cpu(mesh, rays),
        }
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

    // ========================================================================
    // Operation 14: Loop Subdivision
    // ========================================================================

    /// Performs full smooth Loop subdivision refinement on the mesh.
    pub fn loop_subdivide(&self, mesh: &PolygonMesh) -> PolygonMesh {
        let total_faces = mesh.faces().len();
        let target = self.select_kernel(Operation::LoopSubdivision, total_faces);
        match target {
            KernelTarget::GpuCuda | KernelTarget::GpuPtxSm86 | KernelTarget::GpuBvh => {
                #[cfg(feature = "cuda")]
                return crate::gpu::cuda::CudaMeshAnalyzer::new().loop_subdivide(mesh);
                #[cfg(not(feature = "cuda"))]
                self.loop_subdivide_rayon(mesh)
            }
            KernelTarget::GpuVulkan => {
                #[cfg(feature = "gpu")]
                if let Some(analyzer) = crate::gpu::GpuMeshAnalyzer::try_create_default() {
                    return analyzer.loop_subdivide(mesh);
                }
                self.loop_subdivide_rayon(mesh)
            }
            KernelTarget::CpuRayon => self.loop_subdivide_rayon(mesh),
            KernelTarget::CpuSingle => self.loop_subdivide_cpu(mesh),
        }
    }

    /// In-place smooth Loop subdivision refinement on the mesh.
    pub fn loop_subdivision<'a>(&self, mesh: &'a mut PolygonMesh) -> &'a mut PolygonMesh {
        *mesh = self.loop_subdivide(mesh);
        mesh
    }

    fn loop_subdivide_cpu(&self, mesh: &PolygonMesh) -> PolygonMesh {
        let tri_faces = mesh.tri_faces();
        if tri_faces.is_empty() {
            return mesh.clone();
        }
        let topo = crate::gpu::prepare_loop_subdivision_topology(mesh);
        let num_vertices = mesh.positions().len();
        let in_positions = mesh.positions();

        let mut positions = Vec::with_capacity(num_vertices + topo.edges_f64.len());
        // Even vertices
        for (v, info) in topo.vertex_info.iter().enumerate() {
            let p = in_positions[v];
            let new_p = match info.boundary_type {
                2 => p, // Corner
                1 => p + ((in_positions[info.b0 as usize] - p) + (in_positions[info.b1 as usize] - p)) / 8.0,
                _ => {
                    let n = info.valence as f64;
                    if n > 0.0 {
                        let alpha = 3.0 / 8.0 + (2.0 * std::f64::consts::PI / n).cos() / 4.0;
                        let beta = (5.0 / 8.0 - alpha * alpha) / n;
                        let mut sum_v = Vector3::zero();
                        for j in 0..info.valence {
                            let nb = topo.vertex_neighbors[(info.offset + j) as usize] as usize;
                            sum_v += in_positions[nb] - p;
                        }
                        p + sum_v * beta
                    } else {
                        p
                    }
                }
            };
            positions.push(new_p);
        }

        // Odd vertices
        for edge in topo.edges_f64.iter() {
            let p0 = in_positions[edge.v0 as usize];
            let p1 = in_positions[edge.v1 as usize];
            let new_p = if edge.is_boundary != 0 {
                p0.midpoint(p1)
            } else {
                let pw0 = in_positions[edge.w0 as usize];
                let pw1 = in_positions[edge.w1 as usize];
                p0 + (p1 - p0) * edge.gamma + (pw0 - p0) / 8.0 + (pw1 - p1) / 8.0
            };
            positions.push(new_p);
        }

        let mut out_mesh = mesh.clone();
        {
            let editor = out_mesh.debug_editor();
            editor.attributes.positions = positions;
            *editor.faces = Faces::from_tri_and_quad_faces(topo.new_tri_faces, Vec::new());
        }
        out_mesh
    }

    fn loop_subdivide_rayon(&self, mesh: &PolygonMesh) -> PolygonMesh {
        let tri_faces = mesh.tri_faces();
        if tri_faces.is_empty() {
            return mesh.clone();
        }
        let topo = crate::gpu::prepare_loop_subdivision_topology(mesh);
        let in_positions = mesh.positions();

        // Even vertices in parallel
        let mut even_positions: Vec<Point3> = topo.vertex_info
            .par_iter()
            .enumerate()
            .map(|(v, info)| {
                let p = in_positions[v];
                match info.boundary_type {
                    2 => p,
                    1 => p + ((in_positions[info.b0 as usize] - p) + (in_positions[info.b1 as usize] - p)) / 8.0,
                    _ => {
                        let n = info.valence as f64;
                        if n > 0.0 {
                            let alpha = 3.0 / 8.0 + (2.0 * std::f64::consts::PI / n).cos() / 4.0;
                            let beta = (5.0 / 8.0 - alpha * alpha) / n;
                            let mut sum_v = Vector3::zero();
                            for j in 0..info.valence {
                                let nb = topo.vertex_neighbors[(info.offset + j) as usize] as usize;
                                sum_v += in_positions[nb] - p;
                            }
                            p + sum_v * beta
                        } else {
                            p
                        }
                    }
                }
            })
            .collect();

        // Odd vertices in parallel
        let odd_positions: Vec<Point3> = topo.edges_f64
            .par_iter()
            .map(|edge| {
                let p0 = in_positions[edge.v0 as usize];
                let p1 = in_positions[edge.v1 as usize];
                if edge.is_boundary != 0 {
                    p0.midpoint(p1)
                } else {
                    let pw0 = in_positions[edge.w0 as usize];
                    let pw1 = in_positions[edge.w1 as usize];
                    p0 + (p1 - p0) * edge.gamma + (pw0 - p0) / 8.0 + (pw1 - p1) / 8.0
                }
            })
            .collect();

        even_positions.extend(odd_positions);

        let mut out_mesh = mesh.clone();
        {
            let editor = out_mesh.debug_editor();
            editor.attributes.positions = even_positions;
            *editor.faces = Faces::from_tri_and_quad_faces(topo.new_tri_faces, Vec::new());
        }
        out_mesh
    }

    // ========================================================================
    // Operation 15: Feature Edges & Boundaries
    // ========================================================================

    /// Extracts crease edges and open boundary feature edges as 3D line segments.
    pub fn extract_feature_edges(&self, mesh: &PolygonMesh, crease_angle_rad: f64) -> Vec<(Point3, Point3)> {
        let total_faces = mesh.faces().len();
        let target = self.select_kernel(Operation::FeatureEdges, total_faces);
        match target {
            KernelTarget::GpuCuda | KernelTarget::GpuPtxSm86 | KernelTarget::GpuBvh => {
                #[cfg(feature = "cuda")]
                return crate::gpu::cuda::CudaMeshAnalyzer::new().extract_feature_edges(mesh, crease_angle_rad);
                #[cfg(not(feature = "cuda"))]
                self.extract_feature_edges_rayon(mesh, crease_angle_rad)
            }
            KernelTarget::GpuVulkan => {
                #[cfg(feature = "gpu")]
                if let Some(analyzer) = crate::gpu::GpuMeshAnalyzer::try_create_default() {
                    return analyzer.extract_feature_edges(mesh, crease_angle_rad);
                }
                self.extract_feature_edges_rayon(mesh, crease_angle_rad)
            }
            KernelTarget::CpuRayon => self.extract_feature_edges_rayon(mesh, crease_angle_rad),
            KernelTarget::CpuSingle => self.extract_feature_edges_cpu(mesh, crease_angle_rad),
        }
    }

    /// Extracts open boundary edges as 3D line segments.
    pub fn extract_boundary_edges(&self, mesh: &PolygonMesh) -> Vec<(Point3, Point3)> {
        let total_faces = mesh.faces().len();
        let target = self.select_kernel(Operation::FeatureEdges, total_faces);
        match target {
            KernelTarget::GpuCuda | KernelTarget::GpuPtxSm86 | KernelTarget::GpuBvh => {
                #[cfg(feature = "cuda")]
                return crate::gpu::cuda::CudaMeshAnalyzer::new().extract_boundary_edges(mesh);
                #[cfg(not(feature = "cuda"))]
                self.extract_boundary_edges_rayon(mesh)
            }
            KernelTarget::GpuVulkan => {
                #[cfg(feature = "gpu")]
                if let Some(analyzer) = crate::gpu::GpuMeshAnalyzer::try_create_default() {
                    return analyzer.extract_boundary_edges(mesh);
                }
                self.extract_boundary_edges_rayon(mesh)
            }
            KernelTarget::CpuRayon => self.extract_boundary_edges_rayon(mesh),
            KernelTarget::CpuSingle => self.extract_boundary_edges_cpu(mesh),
        }
    }

    fn extract_feature_edges_cpu(&self, mesh: &PolygonMesh, crease_angle_rad: f64) -> Vec<(Point3, Point3)> {
        let tri_faces = mesh.tri_faces();
        if tri_faces.is_empty() {
            return Vec::new();
        }
        let topo = crate::gpu::prepare_feature_edges_topology(mesh);
        let positions = mesh.positions();
        let cos_crease = crease_angle_rad.cos();

        topo.edges
            .into_iter()
            .filter_map(|e| {
                let is_feature = if e.f1 < 0 {
                    true
                } else {
                    let n0 = Vector3::new(
                        topo.face_normals_f64[e.f0 as usize][0],
                        topo.face_normals_f64[e.f0 as usize][1],
                        topo.face_normals_f64[e.f0 as usize][2],
                    );
                    let n1 = Vector3::new(
                        topo.face_normals_f64[e.f1 as usize][0],
                        topo.face_normals_f64[e.f1 as usize][1],
                        topo.face_normals_f64[e.f1 as usize][2],
                    );
                    n0.dot(n1) < cos_crease
                };
                if is_feature {
                    Some((positions[e.v0 as usize], positions[e.v1 as usize]))
                } else {
                    None
                }
            })
            .collect()
    }

    fn extract_feature_edges_rayon(&self, mesh: &PolygonMesh, crease_angle_rad: f64) -> Vec<(Point3, Point3)> {
        let tri_faces = mesh.tri_faces();
        if tri_faces.is_empty() {
            return Vec::new();
        }
        let topo = crate::gpu::prepare_feature_edges_topology(mesh);
        let positions = mesh.positions();
        let cos_crease = crease_angle_rad.cos();

        topo.edges
            .into_par_iter()
            .filter_map(|e| {
                let is_feature = if e.f1 < 0 {
                    true
                } else {
                    let n0 = Vector3::new(
                        topo.face_normals_f64[e.f0 as usize][0],
                        topo.face_normals_f64[e.f0 as usize][1],
                        topo.face_normals_f64[e.f0 as usize][2],
                    );
                    let n1 = Vector3::new(
                        topo.face_normals_f64[e.f1 as usize][0],
                        topo.face_normals_f64[e.f1 as usize][1],
                        topo.face_normals_f64[e.f1 as usize][2],
                    );
                    n0.dot(n1) < cos_crease
                };
                if is_feature {
                    Some((positions[e.v0 as usize], positions[e.v1 as usize]))
                } else {
                    None
                }
            })
            .collect()
    }

    fn extract_boundary_edges_cpu(&self, mesh: &PolygonMesh) -> Vec<(Point3, Point3)> {
        let tri_faces = mesh.tri_faces();
        if tri_faces.is_empty() {
            return Vec::new();
        }
        let topo = crate::gpu::prepare_feature_edges_topology(mesh);
        let positions = mesh.positions();

        topo.edges
            .into_iter()
            .filter_map(|e| {
                if e.f1 < 0 {
                    Some((positions[e.v0 as usize], positions[e.v1 as usize]))
                } else {
                    None
                }
            })
            .collect()
    }

    fn extract_boundary_edges_rayon(&self, mesh: &PolygonMesh) -> Vec<(Point3, Point3)> {
        let tri_faces = mesh.tri_faces();
        if tri_faces.is_empty() {
            return Vec::new();
        }
        let topo = crate::gpu::prepare_feature_edges_topology(mesh);
        let positions = mesh.positions();

        topo.edges
            .into_par_iter()
            .filter_map(|e| {
                if e.f1 < 0 {
                    Some((positions[e.v0 as usize], positions[e.v1 as usize]))
                } else {
                    None
                }
            })
            .collect()
    }

    // ========================================================================
    // Operation 16: Quadric Error Metric (QEM) Edge Collapse Costs
    // ========================================================================

    /// Evaluates QEM edge collapse costs and optimal target points using the optimal kernel.
    pub fn compute_edge_collapse_costs(&self, mesh: &PolygonMesh) -> Vec<([usize; 2], f64, Point3)> {
        let triangulated;
        let mesh = if mesh.tri_faces().is_empty() && !mesh.faces().is_empty() {
            let mut m = mesh.clone();
            m.triangulate();
            triangulated = m;
            &triangulated
        } else {
            mesh
        };
        let tri_faces = mesh.tri_faces();
        if tri_faces.is_empty() {
            return Vec::new();
        }
        let target = self.select_kernel(Operation::QemEdgeCollapse, tri_faces.len());
        match target {
            KernelTarget::GpuCuda | KernelTarget::GpuPtxSm86 | KernelTarget::GpuBvh => {
                #[cfg(feature = "cuda")]
                return crate::gpu::cuda::CudaMeshAnalyzer::new().compute_edge_collapse_costs(mesh);
                #[cfg(not(feature = "cuda"))]
                self.compute_edge_collapse_costs_rayon(mesh)
            }
            KernelTarget::GpuVulkan => {
                #[cfg(feature = "gpu")]
                if let Some(analyzer) = crate::gpu::GpuMeshAnalyzer::try_create_default() {
                    return analyzer.compute_edge_collapse_costs(mesh);
                }
                self.compute_edge_collapse_costs_rayon(mesh)
            }
            KernelTarget::CpuRayon => self.compute_edge_collapse_costs_rayon(mesh),
            KernelTarget::CpuSingle => self.compute_edge_collapse_costs_cpu(mesh),
        }
    }

    fn compute_edge_collapse_costs_cpu(&self, mesh: &PolygonMesh) -> Vec<([usize; 2], f64, Point3)> {
        cpu_compute_quadrics_and_edge_costs(mesh, false)
    }

    fn compute_edge_collapse_costs_rayon(&self, mesh: &PolygonMesh) -> Vec<([usize; 2], f64, Point3)> {
        cpu_compute_quadrics_and_edge_costs(mesh, true)
    }

    // ========================================================================
    // Operation 17: 2D Multi-Loop Point-in-Polygon Engine
    // ========================================================================

    /// Tests whether 2D points lie inside complex 2D planar profiles with nested loops using the optimal kernel.
    pub fn point_in_2d_loops(&self, points: &[Point2], loops: &[Vec<Point2>]) -> Vec<bool> {
        if points.is_empty() {
            return Vec::new();
        }
        let target = self.select_kernel(Operation::PointIn2dLoops, points.len());
        match target {
            KernelTarget::GpuCuda | KernelTarget::GpuPtxSm86 | KernelTarget::GpuBvh => {
                #[cfg(feature = "cuda")]
                return crate::gpu::cuda::CudaMeshAnalyzer::new().point_in_2d_loops(points, loops);
                #[cfg(not(feature = "cuda"))]
                self.point_in_2d_loops_rayon(points, loops)
            }
            KernelTarget::GpuVulkan => {
                #[cfg(feature = "gpu")]
                if let Some(analyzer) = crate::gpu::GpuMeshAnalyzer::try_create_default() {
                    return analyzer.point_in_2d_loops(points, loops);
                }
                self.point_in_2d_loops_rayon(points, loops)
            }
            KernelTarget::CpuRayon => self.point_in_2d_loops_rayon(points, loops),
            KernelTarget::CpuSingle => self.point_in_2d_loops_cpu(points, loops),
        }
    }

    fn point_in_2d_loops_cpu(&self, points: &[Point2], loops: &[Vec<Point2>]) -> Vec<bool> {
        cpu_point_in_2d_loops(points, loops, false)
    }

    fn point_in_2d_loops_rayon(&self, points: &[Point2], loops: &[Vec<Point2>]) -> Vec<bool> {
        cpu_point_in_2d_loops(points, loops, true)
    }

    // ========================================================================
    // Operation 18: 3D Segment / Toolpath Clearance & Gouge Checking
    // ========================================================================

    /// Evaluates minimum clearance distances from 3D line segments to a mesh using the optimal kernel.
    pub fn segment_mesh_distances(&self, mesh: &PolygonMesh, segments: &[(Point3, Point3)]) -> Vec<f64> {
        if segments.is_empty() {
            return Vec::new();
        }
        let target = self.select_kernel(Operation::SegmentMeshDistance, segments.len());
        match target {
            KernelTarget::GpuCuda | KernelTarget::GpuPtxSm86 | KernelTarget::GpuBvh => {
                #[cfg(feature = "cuda")]
                return multi_gpu_cuda_split(segments, |analyzer, segs| {
                    analyzer.segment_mesh_distances(mesh, segs)
                });
                #[cfg(not(feature = "cuda"))]
                self.segment_mesh_distances_rayon(mesh, segments)
            }
            KernelTarget::GpuVulkan => {
                #[cfg(feature = "gpu")]
                {
                    let res = multi_gpu_vulkan_split(segments, |analyzer, segs| {
                        analyzer.segment_mesh_distances(mesh, segs)
                    });
                    if !res.is_empty() {
                        return res;
                    }
                }
                self.segment_mesh_distances_rayon(mesh, segments)
            }
            KernelTarget::CpuRayon => self.segment_mesh_distances_rayon(mesh, segments),
            KernelTarget::CpuSingle => self.segment_mesh_distances_cpu(mesh, segments),
        }
    }

    fn segment_mesh_distances_cpu(&self, mesh: &PolygonMesh, segments: &[(Point3, Point3)]) -> Vec<f64> {
        cpu_segment_mesh_distances(mesh, segments, false)
    }

    fn segment_mesh_distances_rayon(&self, mesh: &PolygonMesh, segments: &[(Point3, Point3)]) -> Vec<f64> {
        cpu_segment_mesh_distances(mesh, segments, true)
    }

    // ========================================================================
    // Operation 19: Parallel Vertex Welding & Attribute Deduplication
    // ========================================================================

    /// Welds and deduplicates 3D attributes within tolerance `tol` using the optimal kernel.
    pub fn weld_attributes_3d(&self, attrs: &[Point3], tol: f64) -> Vec<usize> {
        if attrs.is_empty() {
            return Vec::new();
        }
        let target = self.select_kernel(Operation::WeldAttributes, attrs.len());
        match target {
            KernelTarget::GpuCuda | KernelTarget::GpuPtxSm86 | KernelTarget::GpuBvh => {
                #[cfg(feature = "cuda")]
                return crate::gpu::cuda::CudaMeshAnalyzer::new().weld_attributes_3d(attrs, tol);
                #[cfg(not(feature = "cuda"))]
                self.weld_attributes_3d_rayon(attrs, tol)
            }
            KernelTarget::GpuVulkan => {
                #[cfg(feature = "gpu")]
                if let Some(analyzer) = crate::gpu::GpuMeshAnalyzer::try_create_default() {
                    return analyzer.weld_attributes_3d(attrs, tol);
                }
                self.weld_attributes_3d_rayon(attrs, tol)
            }
            KernelTarget::CpuRayon => self.weld_attributes_3d_rayon(attrs, tol),
            KernelTarget::CpuSingle => self.weld_attributes_3d_cpu(attrs, tol),
        }
    }

    /// Welds and deduplicates 2D attributes within tolerance `tol` using the optimal kernel.
    pub fn weld_attributes_2d(&self, attrs: &[Point2], tol: f64) -> Vec<usize> {
        if attrs.is_empty() {
            return Vec::new();
        }
        let target = self.select_kernel(Operation::WeldAttributes, attrs.len());
        match target {
            KernelTarget::GpuCuda | KernelTarget::GpuPtxSm86 | KernelTarget::GpuBvh => {
                #[cfg(feature = "cuda")]
                return crate::gpu::cuda::CudaMeshAnalyzer::new().weld_attributes_2d(attrs, tol);
                #[cfg(not(feature = "cuda"))]
                self.weld_attributes_2d_rayon(attrs, tol)
            }
            KernelTarget::GpuVulkan => {
                #[cfg(feature = "gpu")]
                if let Some(analyzer) = crate::gpu::GpuMeshAnalyzer::try_create_default() {
                    return analyzer.weld_attributes_2d(attrs, tol);
                }
                self.weld_attributes_2d_rayon(attrs, tol)
            }
            KernelTarget::CpuRayon => self.weld_attributes_2d_rayon(attrs, tol),
            KernelTarget::CpuSingle => self.weld_attributes_2d_cpu(attrs, tol),
        }
    }

    fn weld_attributes_3d_cpu(&self, attrs: &[Point3], tol: f64) -> Vec<usize> {
        crate::filters::sub_put_together_same_attrs(attrs, tol)
    }

    fn weld_attributes_3d_rayon(&self, attrs: &[Point3], tol: f64) -> Vec<usize> {
        crate::filters::sub_put_together_same_attrs(attrs, tol)
    }

    fn weld_attributes_2d_cpu(&self, attrs: &[Point2], tol: f64) -> Vec<usize> {
        crate::filters::sub_put_together_same_attrs(attrs, tol)
    }

    fn weld_attributes_2d_rayon(&self, attrs: &[Point2], tol: f64) -> Vec<usize> {
        crate::filters::sub_put_together_same_attrs(attrs, tol)
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

fn cpu_compute_quadrics_and_edge_costs(
    mesh: &PolygonMesh,
    parallel: bool,
) -> Vec<([usize; 2], f64, Point3)> {
    let triangulated;
    let mesh = if mesh.tri_faces().is_empty() && !mesh.faces().is_empty() {
        let mut m = mesh.clone();
        m.triangulate();
        triangulated = m;
        &triangulated
    } else {
        mesh
    };
    let tri_faces = mesh.tri_faces();
    if tri_faces.is_empty() {
        return Vec::new();
    }
    let positions = mesh.positions();
    let num_vertices = positions.len();

    let mut edge_set = rustc_hash::FxHashSet::default();
    for tri in tri_faces {
        let v0 = tri[0].pos;
        let v1 = tri[1].pos;
        let v2 = tri[2].pos;
        edge_set.insert((v0.min(v1), v0.max(v1)));
        edge_set.insert((v1.min(v2), v1.max(v2)));
        edge_set.insert((v2.min(v0), v2.max(v0)));
    }
    let mut edges: Vec<[usize; 2]> = edge_set.into_iter().map(|(a, b)| [a, b]).collect();
    edges.sort_unstable();

    // Compute vertex quadrics
    let mut vertex_quadrics = vec![[0.0f64; 10]; num_vertices];
    for tri in tri_faces {
        let p0 = positions[tri[0].pos];
        let p1 = positions[tri[1].pos];
        let p2 = positions[tri[2].pos];
        let u = p1 - p0;
        let v = p2 - p0;
        let n = u.cross(v);
        let len = n.magnitude();
        let (a, b, c) = if len > 1e-12 {
            let inv = 1.0 / len;
            (n.x * inv, n.y * inv, n.z * inv)
        } else {
            (0.0, 0.0, 1.0)
        };
        let d = -(a * p0.x + b * p0.y + c * p0.z);
        let k = [
            a * a, a * b, a * c, a * d,
            b * b, b * c, b * d,
            c * c, c * d,
            d * d,
        ];
        for &idx in &[tri[0].pos, tri[1].pos, tri[2].pos] {
            for m in 0..10 {
                vertex_quadrics[idx][m] += k[m];
            }
        }
    }

    let eval_edge = |edge: [usize; 2]| -> ([usize; 2], f64, Point3) {
        let v0 = edge[0];
        let v1 = edge[1];
        let p0 = positions[v0];
        let p1 = positions[v1];
        let q0 = vertex_quadrics[v0];
        let q1 = vertex_quadrics[v1];

        let mut q = [0.0f64; 10];
        for m in 0..10 {
            q[m] = q0[m] + q1[m];
        }

        let eval_q = |p: Point3| -> f64 {
            let cost = q[0] * p.x * p.x + 2.0 * q[1] * p.x * p.y + 2.0 * q[2] * p.x * p.z + 2.0 * q[3] * p.x
                     + q[4] * p.y * p.y + 2.0 * q[5] * p.y * p.z + 2.0 * q[6] * p.y
                     + q[7] * p.z * p.z + 2.0 * q[8] * p.z + q[9];
            f64::max(0.0, cost)
        };

        let pm = Point3::new(0.5 * (p0.x + p1.x), 0.5 * (p0.y + p1.y), 0.5 * (p0.z + p1.z));
        let c0 = eval_q(p0);
        let c1 = eval_q(p1);
        let cm = eval_q(pm);

        let mut best_cost = cm;
        let mut best_target = pm;

        if c0 < best_cost {
            best_cost = c0;
            best_target = p0;
        }
        if c1 < best_cost {
            best_cost = c1;
            best_target = p1;
        }

        let c00 = q[4] * q[7] - q[5] * q[5];
        let c01 = q[2] * q[5] - q[1] * q[7];
        let c02 = q[1] * q[5] - q[2] * q[4];
        let c11 = q[0] * q[7] - q[2] * q[2];
        let c12 = q[1] * q[2] - q[0] * q[5];
        let c22 = q[0] * q[4] - q[1] * q[1];
        let det = q[0] * c00 + q[1] * c01 + q[2] * c02;

        if det.abs() > 1e-8 {
            let inv_det = 1.0 / det;
            let opt_x = -(c00 * q[3] + c01 * q[6] + c02 * q[8]) * inv_det;
            let opt_y = -(c01 * q[3] + c11 * q[6] + c12 * q[8]) * inv_det;
            let opt_z = -(c02 * q[3] + c12 * q[6] + c22 * q[8]) * inv_det;
            if opt_x.is_finite() && opt_y.is_finite() && opt_z.is_finite() {
                let opt_pt = Point3::new(opt_x, opt_y, opt_z);
                let c_opt = eval_q(opt_pt);
                if c_opt <= best_cost {
                    best_cost = c_opt;
                    best_target = opt_pt;
                }
            }
        }

        (edge, best_cost, best_target)
    };

    if parallel {
        edges.into_par_iter().map(eval_edge).collect()
    } else {
        edges.into_iter().map(eval_edge).collect()
    }
}

fn cpu_point_in_2d_loops(
    points: &[Point2],
    loops: &[Vec<Point2>],
    parallel: bool,
) -> Vec<bool> {
    if points.is_empty() {
        return Vec::new();
    }
    let mut segments = Vec::new();
    for lp in loops {
        let n = lp.len();
        if n < 3 {
            continue;
        }
        let is_closed = lp[0].distance2(lp[n - 1]) < 1e-14;
        let seg_count = if is_closed { n - 1 } else { n };
        for i in 0..seg_count {
            segments.push((lp[i], lp[(i + 1) % n]));
        }
    }

    let eval_point = |p: Point2| -> bool {
        let mut crossings = 0;
        for &(p0, p1) in &segments {
            if (p0.y > p.y) != (p1.y > p.y) {
                let x_int = p0.x + (p.y - p0.y) * (p1.x - p0.x) / (p1.y - p0.y);
                if p.x < x_int {
                    crossings += 1;
                }
            }
        }
        crossings % 2 == 1
    };

    if parallel {
        points.par_iter().copied().map(eval_point).collect()
    } else {
        points.iter().copied().map(eval_point).collect()
    }
}

fn cpu_dist2_segment_segment(p0: Point3, p1: Point3, q0: Point3, q1: Point3) -> f64 {
    let u = p1 - p0;
    let v = q1 - q0;
    let w0 = p0 - q0;
    let a = u.dot(u);
    let b = u.dot(v);
    let c = v.dot(v);
    let d = u.dot(w0);
    let e = v.dot(w0);
    let cap_d = a * c - b * b;
    let (mut s_n, mut s_d, mut t_n, t_d) = if cap_d < 1e-12 {
        (0.0, 1.0, e, c)
    } else {
        let mut sn = b * e - c * d;
        let mut tn = a * e - b * d;
        if sn < 0.0 {
            sn = 0.0;
            tn = e;
        } else if sn > cap_d {
            sn = cap_d;
            tn = e + b;
        }
        (sn, cap_d, tn, c)
    };

    if t_n <= 0.0 {
        t_n = 0.0;
        if -d < 0.0 {
            s_n = 0.0;
        } else if -d > a {
            s_n = s_d;
        } else {
            s_n = -d;
            s_d = a;
        }
    } else if t_n >= t_d {
        t_n = t_d;
        if (-d + b) < 0.0 {
            s_n = 0.0;
        } else if (-d + b) > a {
            s_n = s_d;
        } else {
            s_n = -d + b;
            s_d = a;
        }
    }

    let sc = if s_n.abs() < 1e-12 { 0.0 } else { s_n / s_d };
    let tc = if t_n.abs() < 1e-12 { 0.0 } else { t_n / t_d };
    let dp = (p0 + u * sc) - (q0 + v * tc);
    dp.dot(dp)
}

fn cpu_segment_intersects_triangle(p0: Point3, p1: Point3, v0: Point3, v1: Point3, v2: Point3) -> bool {
    let ab = v1 - v0;
    let ca = v0 - v2;
    let nor = ab.cross(ca);
    let nor_len2 = nor.dot(nor);
    if nor_len2 < 1e-20 {
        return false;
    }
    let d0 = nor.dot(p0 - v0);
    let d1 = nor.dot(p1 - v0);
    if (d0 > 1e-12 && d1 > 1e-12) || (d0 < -1e-12 && d1 < -1e-12) {
        return false;
    }
    let denom = d0 - d1;
    if denom.abs() < 1e-14 {
        return false;
    }
    let t = d0 / denom;
    if !(0.0..=1.0).contains(&t) {
        return false;
    }
    let pint = p0 + (p1 - p0) * t;
    let bc = v2 - v1;
    let s0 = (ab.cross(nor)).dot(pint - v0);
    let s1 = (bc.cross(nor)).dot(pint - v1);
    let s2 = (ca.cross(nor)).dot(pint - v2);
    let eps = -1e-9 * nor_len2.sqrt();
    (s0 >= eps && s1 >= eps && s2 >= eps) || (s0 <= -eps && s1 <= -eps && s2 <= -eps)
}

fn cpu_dist2_segment_triangle(p0: Point3, p1: Point3, tri: &[Point3; 3]) -> f64 {
    let (v0, v1, v2) = (tri[0], tri[1], tri[2]);
    if cpu_segment_intersects_triangle(p0, p1, v0, v1, v2) {
        return 0.0;
    }
    let mut d2 = cpu_dist2_point_triangle(p0, tri);
    d2 = d2.min(cpu_dist2_point_triangle(p1, tri));
    d2 = d2.min(cpu_dist2_segment_segment(p0, p1, v0, v1));
    d2 = d2.min(cpu_dist2_segment_segment(p0, p1, v1, v2));
    d2 = d2.min(cpu_dist2_segment_segment(p0, p1, v2, v0));
    d2
}

fn cpu_segment_mesh_distances(
    mesh: &PolygonMesh,
    segments: &[(Point3, Point3)],
    parallel: bool,
) -> Vec<f64> {
    if segments.is_empty() {
        return Vec::new();
    }
    let triangles = extract_tri_tuples(mesh);
    if triangles.is_empty() {
        return vec![f64::INFINITY; segments.len()];
    }

    let eval_seg = |&(p0, p1): &(Point3, Point3)| -> f64 {
        let mut min_d2 = f64::INFINITY;
        for tri in &triangles {
            let d2 = cpu_dist2_segment_triangle(p0, p1, tri);
            if d2 < min_d2 {
                min_d2 = d2;
                if min_d2 <= 0.0 {
                    break;
                }
            }
        }
        min_d2.sqrt()
    };

    if parallel {
        segments.par_iter().map(eval_seg).collect()
    } else {
        segments.iter().map(eval_seg).collect()
    }
}
