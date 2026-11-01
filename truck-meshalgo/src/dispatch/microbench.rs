//! Startup scaling micro-benchmark that calibrates kernel thresholds for current CPU & GPU.

use std::f64::consts::PI;
use std::time::{Duration, Instant};
use rayon::prelude::*;
use truck_base::cgmath64::*;
use truck_geometry::prelude::*;
use truck_polymesh::*;

use crate::analyzers::{Collision, IncludingPointInDomain};
use crate::filters::{NormalFilters, StructuringFilter};

use super::profile::{DispatchProfile, KernelTarget, Operation, SampleTiming, ScalingThresholds};

/// Creates a small synthetic test mesh for microbenchmarks.
fn create_test_mesh(udiv: usize, vdiv: usize) -> PolygonMesh {
    let positions = (0..udiv)
        .flat_map(move |i| {
            (0..vdiv).map(move |j| {
                let u = 2.0 * PI * i as f64 / udiv as f64;
                let v = PI * j as f64 / (vdiv - 1) as f64;
                Point3::origin() + Vector3::new(u.cos() * v.sin(), u.sin() * v.sin(), v.cos())
            })
        })
        .collect::<Vec<_>>();
    let faces = Faces::from_iter((0..udiv).flat_map(move |i| {
        (0..vdiv - 1).map(move |j| {
            [
                i * vdiv + j,
                i * vdiv + (j + 1) % vdiv,
                (i + 1) % udiv * vdiv + (j + 1) % vdiv,
                (i + 1) % udiv * vdiv + j,
            ]
        })
    }));
    let mut mesh = PolygonMesh::new(StandardAttributes { positions, ..Default::default() }, faces);
    mesh.triangulate();
    mesh
}

/// Generates deterministic query points.
fn generate_test_points(n: usize) -> Vec<Point3> {
    (0..n)
        .map(|i| {
            let t = i as f64 * 0.1234567;
            Point3::new(t.sin() * 2.0, t.cos() * 2.0, (t * 0.5).sin() * 2.0)
        })
        .collect()
}

fn time_run<F: FnMut()>(mut f: F) -> Duration {
    // Warmup
    f();
    // 2 timed runs
    let t0 = Instant::now();
    f();
    f();
    t0.elapsed() / 2
}

/// Runs the scaling micro-benchmark and produces a calibrated DispatchProfile.
pub fn run_scaling_microbenchmark() -> DispatchProfile {
    let mesh = create_test_mesh(12, 12);
    let sizes = [32, 256, 2048];

    // Check GPU availabilities
    #[cfg(feature = "cuda")]
    let cuda_analyzer = crate::gpu::cuda::CudaMeshAnalyzer::new();
    #[cfg(feature = "cuda")]
    let cuda_available = crate::gpu::cuda::CudaMeshAnalyzer::get_device_info().is_some();
    #[cfg(not(feature = "cuda"))]
    let _cuda_available = false;

    #[cfg(feature = "gpu")]
    let vulkan_analyzer = crate::gpu::GpuMeshAnalyzer::try_create_default();
    #[cfg(not(feature = "gpu"))]
    let vulkan_analyzer: Option<()> = None;

    // ------------------------------------------------------------------------
    // 1. PointMeshDistance
    // ------------------------------------------------------------------------
    let mut dist_samples = Vec::new();
    let tri_list: Vec<[Point3; 3]> = {
        let pos = mesh.positions();
        mesh.face_iter()
            .map(|f| [pos[f[0].pos], pos[f[1].pos], pos[f[2].pos]])
            .collect()
    };

    let cpu_dist_pt_tri = |p: Point3, tri: &[Point3; 3]| -> f64 {
        let ab = tri[1] - tri[0];
        let ap = p - tri[0];
        let bc = tri[2] - tri[1];
        let bp = p - tri[1];
        let ca = tri[0] - tri[2];
        let cp = p - tri[2];
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
    };

    for &size in &sizes {
        let pts = generate_test_points(size);

        // CPU Single
        let d_cpu1 = time_run(|| {
            let res: Vec<f64> = pts.iter().map(|&p| {
                tri_list.iter().map(|tri| cpu_dist_pt_tri(p, tri)).fold(f64::INFINITY, f64::min)
            }).collect();
            std::hint::black_box(res);
        });
        dist_samples.push(SampleTiming {
            size,
            kernel: KernelTarget::CpuSingle,
            duration_nanos: d_cpu1.as_nanos() as u64,
        });

        // CPU Rayon
        let d_rayon = time_run(|| {
            let res: Vec<f64> = pts.par_iter().map(|&p| {
                tri_list.iter().map(|tri| cpu_dist_pt_tri(p, tri)).fold(f64::INFINITY, f64::min)
            }).collect();
            std::hint::black_box(res);
        });
        dist_samples.push(SampleTiming {
            size,
            kernel: KernelTarget::CpuRayon,
            duration_nanos: d_rayon.as_nanos() as u64,
        });

        // Vulkan
        #[cfg(feature = "gpu")]
        if let Some(ref v_analyzer) = vulkan_analyzer {
            let d_vulkan = time_run(|| {
                let res = v_analyzer.distance2_point_cloud(&mesh, &pts);
                std::hint::black_box(res);
            });
            dist_samples.push(SampleTiming {
                size,
                kernel: KernelTarget::GpuVulkan,
                duration_nanos: d_vulkan.as_nanos() as u64,
            });
        }

        // CUDA & PTX
        #[cfg(feature = "cuda")]
        if cuda_available {
            let d_cuda = time_run(|| {
                let res = cuda_analyzer.distance2_point_cloud_baseline(&mesh, &pts);
                std::hint::black_box(res);
            });
            dist_samples.push(SampleTiming {
                size,
                kernel: KernelTarget::GpuCuda,
                duration_nanos: d_cuda.as_nanos() as u64,
            });

            let d_ptx = time_run(|| {
                let res = cuda_analyzer.distance2_point_cloud(&mesh, &pts);
                std::hint::black_box(res);
            });
            dist_samples.push(SampleTiming {
                size,
                kernel: KernelTarget::GpuPtxSm86,
                duration_nanos: d_ptx.as_nanos() as u64,
            });
        }
    }

    let point_mesh_distance = calibrate_thresholds(Operation::PointMeshDistance, &dist_samples);

    // ------------------------------------------------------------------------
    // 2. InOutContainment
    // ------------------------------------------------------------------------
    let mut in_out_samples = Vec::new();
    let ray_dirs: Vec<Vector3> = (0..sizes[sizes.len() - 1])
        .map(|i| {
            let t = i as f64 * 0.456789;
            Vector3::new(t.cos(), t.sin(), (t * 0.3).cos()).normalize()
        })
        .collect();

    for &size in &sizes {
        let pts = generate_test_points(size);
        let dirs = &ray_dirs[..size];

        // CPU Single
        let d_cpu1 = time_run(|| {
            let res: Vec<isize> = pts.iter().zip(dirs.iter()).map(|(&p, &d)| {
                mesh.signed_crossing_faces(p, d)
            }).collect();
            std::hint::black_box(res);
        });
        in_out_samples.push(SampleTiming {
            size,
            kernel: KernelTarget::CpuSingle,
            duration_nanos: d_cpu1.as_nanos() as u64,
        });

        // CPU Rayon
        let d_rayon = time_run(|| {
            let res: Vec<isize> = pts.par_iter().zip(dirs.par_iter()).map(|(&p, &d)| {
                mesh.signed_crossing_faces(p, d)
            }).collect();
            std::hint::black_box(res);
        });
        in_out_samples.push(SampleTiming {
            size,
            kernel: KernelTarget::CpuRayon,
            duration_nanos: d_rayon.as_nanos() as u64,
        });

        // Vulkan
        #[cfg(feature = "gpu")]
        if let Some(ref v_analyzer) = vulkan_analyzer {
            let d_vulkan = time_run(|| {
                let res = v_analyzer.signed_crossing_rays(&mesh, &pts, dirs);
                std::hint::black_box(res);
            });
            in_out_samples.push(SampleTiming {
                size,
                kernel: KernelTarget::GpuVulkan,
                duration_nanos: d_vulkan.as_nanos() as u64,
            });
        }

        // CUDA & PTX
        #[cfg(feature = "cuda")]
        if cuda_available {
            let d_ptx = time_run(|| {
                let res = cuda_analyzer.signed_crossing_rays(&mesh, &pts, dirs);
                std::hint::black_box(res);
            });
            in_out_samples.push(SampleTiming {
                size,
                kernel: KernelTarget::GpuPtxSm86,
                duration_nanos: d_ptx.as_nanos() as u64,
            });
        }
    }

    let in_out_containment = calibrate_thresholds(Operation::InOutContainment, &in_out_samples);

    // ------------------------------------------------------------------------
    // 3. MeshNormals
    // ------------------------------------------------------------------------
    let mut normal_samples = Vec::new();
    let norm_sizes = [64, 512, 4096];
    for &num_faces in &norm_sizes {
        let udiv = (num_faces as f64).sqrt().ceil() as usize;
        let n_mesh = create_test_mesh(udiv.max(6), udiv.max(6));
        let actual_faces = n_mesh.faces().len();

        let d_cpu1 = time_run(|| {
            let mut m = n_mesh.clone();
            m.add_smooth_normals(0.5, true);
            std::hint::black_box(m);
        });
        normal_samples.push(SampleTiming {
            size: actual_faces,
            kernel: KernelTarget::CpuSingle,
            duration_nanos: d_cpu1.as_nanos() as u64,
        });

        #[cfg(feature = "cuda")]
        if cuda_available {
            let d_cuda = time_run(|| {
                let res = cuda_analyzer.compute_smooth_normals(&n_mesh);
                std::hint::black_box(res);
            });
            normal_samples.push(SampleTiming {
                size: actual_faces,
                kernel: KernelTarget::GpuCuda,
                duration_nanos: d_cuda.as_nanos() as u64,
            });
        }
    }

    let mesh_normals = calibrate_thresholds(Operation::MeshNormals, &normal_samples);

    // ------------------------------------------------------------------------
    // 4. MeshInterference
    // ------------------------------------------------------------------------
    let mut interference_samples = Vec::new();
    let mesh2 = create_test_mesh(10, 10);
    let d_cpu_col = time_run(|| {
        let res = mesh.extract_interference(&mesh2);
        std::hint::black_box(res);
    });
    interference_samples.push(SampleTiming {
        size: mesh.faces().len() + mesh2.faces().len(),
        kernel: KernelTarget::CpuSingle,
        duration_nanos: d_cpu_col.as_nanos() as u64,
    });

    #[cfg(feature = "cuda")]
    if cuda_available {
        let d_cuda_col = time_run(|| {
            let res = cuda_analyzer.collide_triangles(&mesh, &mesh2);
            std::hint::black_box(res);
        });
        interference_samples.push(SampleTiming {
            size: mesh.faces().len() + mesh2.faces().len(),
            kernel: KernelTarget::GpuCuda,
            duration_nanos: d_cuda_col.as_nanos() as u64,
        });
    }

    let mesh_interference = calibrate_thresholds(Operation::MeshInterference, &interference_samples);

    // ------------------------------------------------------------------------
    // 5. TrimmingLoopInclude
    // ------------------------------------------------------------------------
    let mut trim_samples = Vec::new();
    let poly_segs: Vec<[Point2; 2]> = (0..16)
        .map(|i| {
            let a0 = 2.0 * PI * i as f64 / 16.0;
            let a1 = 2.0 * PI * (i + 1) as f64 / 16.0;
            [Point2::new(a0.cos(), a0.sin()), Point2::new(a1.cos(), a1.sin())]
        })
        .collect();

    for &size in &[64, 512, 4096] {
        let pts_2d: Vec<Point2> = (0..size)
            .map(|i| {
                let t = i as f64 * 0.1;
                Point2::new(t.sin() * 0.8, t.cos() * 0.8)
            })
            .collect();

        let d_cpu1 = time_run(|| {
            let res: Vec<bool> = pts_2d.iter().map(|&p| {
                // Ray crossing in 2D
                let mut crossings = 0;
                for seg in &poly_segs {
                    if (seg[0].y > p.y) != (seg[1].y > p.y) {
                        let x_int = seg[0].x + (p.y - seg[0].y) * (seg[1].x - seg[0].x) / (seg[1].y - seg[0].y);
                        if p.x < x_int {
                            crossings += 1;
                        }
                    }
                }
                crossings % 2 == 1
            }).collect();
            std::hint::black_box(res);
        });
        trim_samples.push(SampleTiming {
            size,
            kernel: KernelTarget::CpuSingle,
            duration_nanos: d_cpu1.as_nanos() as u64,
        });

        #[cfg(feature = "cuda")]
        if cuda_available {
            let d_cuda = time_run(|| {
                let res = cuda_analyzer.trimming_loop_include(&poly_segs, &pts_2d);
                std::hint::black_box(res);
            });
            trim_samples.push(SampleTiming {
                size,
                kernel: KernelTarget::GpuCuda,
                duration_nanos: d_cuda.as_nanos() as u64,
            });
        }
    }

    let trimming_loop_include = calibrate_thresholds(Operation::TrimmingLoopInclude, &trim_samples);

    // ------------------------------------------------------------------------
    // 6. BsplineEvaluation
    // ------------------------------------------------------------------------
    let mut bsp_samples = Vec::new();
    let u_knots = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
    let v_knots = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
    let cps = vec![
        vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.2), Point3::new(0.0, 2.0, 0.0), Point3::new(0.0, 3.0, 0.0)],
        vec![Point3::new(1.0, 0.0, 0.2), Point3::new(1.0, 1.0, 1.0), Point3::new(1.0, 2.0, 0.8), Point3::new(1.0, 3.0, 0.2)],
        vec![Point3::new(2.0, 0.0, 0.0), Point3::new(2.0, 1.0, 0.8), Point3::new(2.0, 2.0, 1.0), Point3::new(2.0, 3.0, 0.0)],
        vec![Point3::new(3.0, 0.0, 0.0), Point3::new(3.0, 1.0, 0.2), Point3::new(3.0, 2.0, 0.0), Point3::new(3.0, 3.0, 0.0)],
    ];
    let bsp_surface = BSplineSurface::new((
        KnotVec::from(u_knots.clone()),
        KnotVec::from(v_knots.clone()),
    ), cps.clone());

    for &size in &[64, 512, 4096] {
        let pts_2d: Vec<Point2> = (0..size)
            .map(|i| {
                let t = i as f64 / size as f64;
                Point2::new(t, t)
            })
            .collect();

        let d_cpu1 = time_run(|| {
            let res: Vec<(Point3, Vector3)> = pts_2d.iter().map(|&p| {
                (bsp_surface.subs(p.x, p.y), bsp_surface.normal(p.x, p.y))
            }).collect();
            std::hint::black_box(res);
        });
        bsp_samples.push(SampleTiming {
            size,
            kernel: KernelTarget::CpuSingle,
            duration_nanos: d_cpu1.as_nanos() as u64,
        });

        #[cfg(feature = "cuda")]
        if cuda_available {
            let d_cuda = time_run(|| {
                let res = cuda_analyzer.evaluate_bspline_surface(&u_knots, &v_knots, 3, 3, &cps, &pts_2d);
                std::hint::black_box(res);
            });
            bsp_samples.push(SampleTiming {
                size,
                kernel: KernelTarget::GpuCuda,
                duration_nanos: d_cuda.as_nanos() as u64,
            });
        }
    }

    let bspline_evaluation = calibrate_thresholds(Operation::BsplineEvaluation, &bsp_samples);

    DispatchProfile {
        point_mesh_distance,
        in_out_containment,
        mesh_normals,
        mesh_interference,
        trimming_loop_include,
        bspline_evaluation,
        point_cloud_clung: ScalingThresholds::default_fallback(Operation::PointCloudClung),
        point_cloud_include: ScalingThresholds::default_fallback(Operation::PointCloudInclude),
        surface_intersection_seeds: ScalingThresholds::default_fallback(Operation::SurfaceIntersectionSeeds),
        planar_face_segmentation: ScalingThresholds::default_fallback(Operation::PlanarFaceSegmentation),
        linear_subdivision: ScalingThresholds::default_fallback(Operation::LinearSubdivision),
        nurbs_curve_evaluation: ScalingThresholds::default_fallback(Operation::NurbsCurveEvaluation),
        raycast_mesh: ScalingThresholds::default_fallback(Operation::RaycastMesh),
    }
}

/// Computes the optimal threshold crossover boundaries from recorded timings.
fn calibrate_thresholds(op: Operation, samples: &[SampleTiming]) -> ScalingThresholds {
    if samples.is_empty() {
        return ScalingThresholds::default_fallback(op);
    }

    // Group by size
    let mut sizes: Vec<usize> = samples.iter().map(|s| s.size).collect();
    sizes.sort_unstable();
    sizes.dedup();

    let mut fastest_by_size = Vec::new();
    for &sz in &sizes {
        let matching: Vec<&SampleTiming> = samples.iter().filter(|s| s.size == sz).collect();
        if let Some(min_sample) = matching.iter().min_by_key(|s| s.duration_nanos) {
            fastest_by_size.push((sz, min_sample.kernel));
        }
    }

    let is_gpu = |k: KernelTarget| matches!(
        k,
        KernelTarget::GpuVulkan | KernelTarget::GpuCuda | KernelTarget::GpuPtxSm86
    );

    // Determine preferred GPU by comparing GPU kernels at the largest tested size
    let max_size = sizes.last().copied().unwrap_or(0);
    let max_gpu_samples: Vec<&SampleTiming> = samples
        .iter()
        .filter(|s| s.size == max_size && is_gpu(s.kernel))
        .collect();

    let preferred_gpu = if let Some(best_gpu) = max_gpu_samples.iter().min_by_key(|s| s.duration_nanos) {
        best_gpu.kernel
    } else {
        let all_gpu: Vec<&SampleTiming> = samples.iter().filter(|s| is_gpu(s.kernel)).collect();
        if let Some(best) = all_gpu.iter().min_by_key(|s| s.duration_nanos) {
            best.kernel
        } else {
            KernelTarget::CpuRayon
        }
    };

    // Determine crossover limits dynamically based on local measurements:
    // Start at 0 so if GPU is faster even for the smallest tested size, GPU is used immediately.
    let mut cpu_single_max = 0;
    let mut cpu_rayon_max = 0;

    for (sz, kernel) in fastest_by_size {
        match kernel {
            KernelTarget::CpuSingle => {
                cpu_single_max = cpu_single_max.max(sz);
                cpu_rayon_max = cpu_rayon_max.max(sz);
            }
            KernelTarget::CpuRayon => {
                cpu_rayon_max = cpu_rayon_max.max(sz);
            }
            _ => {
                // GPU won at this size
            }
        }
    }

    let any_gpu_available = samples.iter().any(|s| is_gpu(s.kernel));
    if !any_gpu_available {
        cpu_rayon_max = usize::MAX;
    }

    ScalingThresholds {
        cpu_single_max,
        cpu_rayon_max,
        preferred_gpu,
        sample_timings: samples.to_vec(),
    }
}
