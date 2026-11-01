//! Execution targets, operations, and scaling thresholds.

use serde::{Deserialize, Serialize};

/// Target backend/implementation for executing an operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum KernelTarget {
    /// Single-threaded CPU implementation.
    CpuSingle,
    /// Multi-threaded CPU implementation using Rayon.
    CpuRayon,
    /// GPU implementation using Vulkan / wgpu compute shaders.
    GpuVulkan,
    /// GPU implementation using CUDA baseline kernels.
    GpuCuda,
    /// GPU implementation using handcrafted Ampere SM 8.6 PTX kernels.
    GpuPtxSm86,
}

impl std::fmt::Display for KernelTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CpuSingle => write!(f, "CPU (1T)"),
            Self::CpuRayon => write!(f, "CPU (Rayon)"),
            Self::GpuVulkan => write!(f, "GPU (Vulkan)"),
            Self::GpuCuda => write!(f, "GPU (CUDA Baseline)"),
            Self::GpuPtxSm86 => write!(f, "GPU (Ampere PTX SM 8.6)"),
        }
    }
}

/// The algorithmic operation being dispatched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Operation {
    /// Point-to-mesh minimum squared distance queries.
    PointMeshDistance,
    /// Solid containment / in-out point classification.
    InOutContainment,
    /// Surface normal calculation and smoothing.
    MeshNormals,
    /// Triangle-triangle mesh collision and interference detection.
    MeshInterference,
    /// 2D composite polygon trimming loop winding/inclusion tests.
    TrimmingLoopInclude,
    /// Batch B-spline surface point and normal evaluation.
    BsplineEvaluation,
    /// Point cloud clinging test: all faces intersect neighborhood of point cloud.
    PointCloudClung,
    /// Point cloud neighborhood inclusion test: neighborhood includes all points.
    PointCloudInclude,
    /// Surface-surface intersection candidate grid seed finder.
    SurfaceIntersectionSeeds,
    /// Planar face extraction / segmentation.
    PlanarFaceSegmentation,
    /// Triangle linear subdivision / midpoint refinement.
    LinearSubdivision,
}

/// Recorded microbenchmark measurement sample.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SampleTiming {
    /// Problem size tested (e.g. number of query points, faces, or segments).
    pub size: usize,
    /// Kernel target measured.
    pub kernel: KernelTarget,
    /// Duration of the execution in nanoseconds.
    pub duration_nanos: u64,
}

/// Dynamic problem-size threshold profile for an operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScalingThresholds {
    /// Problem size below which single-threaded CPU is preferred.
    pub cpu_single_max: usize,
    /// Problem size below which multi-threaded Rayon CPU is preferred.
    pub cpu_rayon_max: usize,
    /// The fastest GPU kernel on this hardware when problem size > cpu_rayon_max.
    pub preferred_gpu: KernelTarget,
    /// Recorded benchmark timings for transparency and diagnostics.
    pub sample_timings: Vec<SampleTiming>,
}

impl ScalingThresholds {
    /// Creates default fallback thresholds when benchmarks have not been run.
    pub fn default_fallback(op: Operation) -> Self {
        match op {
            Operation::PointMeshDistance => Self {
                cpu_single_max: 64,
                cpu_rayon_max: 256,
                preferred_gpu: KernelTarget::GpuPtxSm86,
                sample_timings: Vec::new(),
            },
            Operation::InOutContainment => Self {
                cpu_single_max: 64,
                cpu_rayon_max: 256,
                preferred_gpu: KernelTarget::GpuPtxSm86,
                sample_timings: Vec::new(),
            },
            Operation::MeshNormals => Self {
                cpu_single_max: 512,
                cpu_rayon_max: 2048,
                preferred_gpu: KernelTarget::GpuCuda,
                sample_timings: Vec::new(),
            },
            Operation::MeshInterference => Self {
                cpu_single_max: 128,
                cpu_rayon_max: 512,
                preferred_gpu: KernelTarget::GpuCuda,
                sample_timings: Vec::new(),
            },
            Operation::TrimmingLoopInclude => Self {
                cpu_single_max: 256,
                cpu_rayon_max: 1024,
                preferred_gpu: KernelTarget::GpuCuda,
                sample_timings: Vec::new(),
            },
            Operation::BsplineEvaluation => Self {
                cpu_single_max: 128,
                cpu_rayon_max: 512,
                preferred_gpu: KernelTarget::GpuCuda,
                sample_timings: Vec::new(),
            },
            Operation::PointCloudClung => Self {
                cpu_single_max: 64,
                cpu_rayon_max: 256,
                preferred_gpu: KernelTarget::GpuCuda,
                sample_timings: Vec::new(),
            },
            Operation::PointCloudInclude => Self {
                cpu_single_max: 64,
                cpu_rayon_max: 256,
                preferred_gpu: KernelTarget::GpuCuda,
                sample_timings: Vec::new(),
            },
            Operation::SurfaceIntersectionSeeds => Self {
                cpu_single_max: 64,
                cpu_rayon_max: 256,
                preferred_gpu: KernelTarget::GpuCuda,
                sample_timings: Vec::new(),
            },
            Operation::PlanarFaceSegmentation => Self {
                cpu_single_max: 32,
                cpu_rayon_max: 128,
                preferred_gpu: KernelTarget::GpuCuda,
                sample_timings: Vec::new(),
            },
            Operation::LinearSubdivision => Self {
                cpu_single_max: 64,
                cpu_rayon_max: 256,
                preferred_gpu: KernelTarget::GpuCuda,
                sample_timings: Vec::new(),
            },
        }
    }

    /// Creates a CPU-only scaling profile where problem sizes up to `single_max` use single-threaded CPU
    /// and any size above uses multi-threaded Rayon CPU (never offloading to GPU).
    pub fn cpu_only(single_max: usize) -> Self {
        Self {
            cpu_single_max: single_max,
            cpu_rayon_max: usize::MAX,
            preferred_gpu: KernelTarget::CpuRayon,
            sample_timings: Vec::new(),
        }
    }

    /// Selects the optimal kernel target based on problem size.
    pub fn select_kernel(&self, size: usize) -> KernelTarget {
        if !self.sample_timings.is_empty() {
            let matching: Vec<&SampleTiming> = self.sample_timings.iter().filter(|s| s.size == size).collect();
            if let Some(min_sample) = matching.iter().min_by_key(|s| s.duration_nanos) {
                return min_sample.kernel;
            }
        }
        if size <= self.cpu_single_max {
            KernelTarget::CpuSingle
        } else if size <= self.cpu_rayon_max {
            KernelTarget::CpuRayon
        } else {
            self.preferred_gpu
        }
    }
}

/// Full collection of calibrated scaling profiles across all operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DispatchProfile {
    /// Thresholds for point-mesh distance.
    pub point_mesh_distance: ScalingThresholds,
    /// Thresholds for in-out containment.
    pub in_out_containment: ScalingThresholds,
    /// Thresholds for mesh normals.
    pub mesh_normals: ScalingThresholds,
    /// Thresholds for mesh interference.
    pub mesh_interference: ScalingThresholds,
    /// Thresholds for trimming loop inclusion.
    pub trimming_loop_include: ScalingThresholds,
    /// Thresholds for B-spline surface evaluation.
    pub bspline_evaluation: ScalingThresholds,
    /// Thresholds for point cloud clung test.
    pub point_cloud_clung: ScalingThresholds,
    /// Thresholds for point cloud neighborhood inclusion test.
    pub point_cloud_include: ScalingThresholds,
    /// Thresholds for surface-surface intersection seed finder.
    pub surface_intersection_seeds: ScalingThresholds,
    /// Thresholds for planar face extraction / segmentation.
    pub planar_face_segmentation: ScalingThresholds,
    /// Thresholds for triangle linear subdivision.
    pub linear_subdivision: ScalingThresholds,
}

impl Default for DispatchProfile {
    fn default() -> Self {
        Self {
            point_mesh_distance: ScalingThresholds::default_fallback(Operation::PointMeshDistance),
            in_out_containment: ScalingThresholds::default_fallback(Operation::InOutContainment),
            mesh_normals: ScalingThresholds::default_fallback(Operation::MeshNormals),
            mesh_interference: ScalingThresholds::default_fallback(Operation::MeshInterference),
            trimming_loop_include: ScalingThresholds::default_fallback(Operation::TrimmingLoopInclude),
            bspline_evaluation: ScalingThresholds::default_fallback(Operation::BsplineEvaluation),
            point_cloud_clung: ScalingThresholds::default_fallback(Operation::PointCloudClung),
            point_cloud_include: ScalingThresholds::default_fallback(Operation::PointCloudInclude),
            surface_intersection_seeds: ScalingThresholds::default_fallback(Operation::SurfaceIntersectionSeeds),
            planar_face_segmentation: ScalingThresholds::default_fallback(Operation::PlanarFaceSegmentation),
            linear_subdivision: ScalingThresholds::default_fallback(Operation::LinearSubdivision),
        }
    }
}

impl DispatchProfile {
    /// Creates a CPU-only profile across all operations for headless systems or systems without GPU.
    pub fn cpu_only() -> Self {
        Self {
            point_mesh_distance: ScalingThresholds::cpu_only(64),
            in_out_containment: ScalingThresholds::cpu_only(64),
            mesh_normals: ScalingThresholds::cpu_only(512),
            mesh_interference: ScalingThresholds::cpu_only(128),
            trimming_loop_include: ScalingThresholds::cpu_only(256),
            bspline_evaluation: ScalingThresholds::cpu_only(128),
            point_cloud_clung: ScalingThresholds::cpu_only(64),
            point_cloud_include: ScalingThresholds::cpu_only(64),
            surface_intersection_seeds: ScalingThresholds::cpu_only(64),
            planar_face_segmentation: ScalingThresholds::cpu_only(32),
            linear_subdivision: ScalingThresholds::cpu_only(64),
        }
    }

    /// Selects the optimal kernel target for a given operation and problem size.
    pub fn select_kernel(&self, op: Operation, size: usize) -> KernelTarget {
        match op {
            Operation::PointMeshDistance => self.point_mesh_distance.select_kernel(size),
            Operation::InOutContainment => self.in_out_containment.select_kernel(size),
            Operation::MeshNormals => self.mesh_normals.select_kernel(size),
            Operation::MeshInterference => self.mesh_interference.select_kernel(size),
            Operation::TrimmingLoopInclude => self.trimming_loop_include.select_kernel(size),
            Operation::BsplineEvaluation => self.bspline_evaluation.select_kernel(size),
            Operation::PointCloudClung => self.point_cloud_clung.select_kernel(size),
            Operation::PointCloudInclude => self.point_cloud_include.select_kernel(size),
            Operation::SurfaceIntersectionSeeds => self.surface_intersection_seeds.select_kernel(size),
            Operation::PlanarFaceSegmentation => self.planar_face_segmentation.select_kernel(size),
            Operation::LinearSubdivision => self.linear_subdivision.select_kernel(size),
        }
    }
}
