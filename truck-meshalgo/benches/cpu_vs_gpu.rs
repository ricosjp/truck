use rayon::prelude::*;
use std::f64::consts::PI;
use std::time::{Duration, Instant};
use truck_base::cgmath64::*;
use truck_meshalgo::prelude::*;
use truck_platform::wgpu::*;
use truck_platform::*;

fn init_vulkan_device() -> DeviceHandler {
    pollster::block_on(async {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::VULKAN,
            flags: InstanceFlags::from_build_config(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .expect("Failed to get Vulkan adapter");
        let info = adapter.get_info();
        eprintln!("Benchmarking on GPU: {} ({:?})\n", info.name, info.backend);
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("bench_device"),
                    ..Default::default()
                },
            )
            .await
            .expect("Failed to create device");
        DeviceHandler::new(adapter, device, queue)
    })
}

fn create_sphere(radius: f64, udiv: usize, vdiv: usize) -> PolygonMesh {
    let positions = (0..udiv)
        .flat_map(move |i| {
            (0..vdiv).map(move |j| {
                let u = 2.0 * PI * i as f64 / udiv as f64;
                let v = PI * j as f64 / (vdiv - 1) as f64;
                Point3::origin() + radius * Vector3::new(u.cos() * v.sin(), u.sin() * v.sin(), v.cos())
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
    PolygonMesh::new(
        StandardAttributes {
            positions,
            ..Default::default()
        },
        faces,
    )
}

fn cpu_distance2_point_triangle(point: Point3, triangle: [Point3; 3]) -> f64 {
    let ab = triangle[1] - triangle[0];
    let ap = point - triangle[0];
    let bc = triangle[2] - triangle[1];
    let bp = point - triangle[1];
    let ca = triangle[0] - triangle[2];
    let cp = point - triangle[2];
    let nor = ab.cross(ca);

    let coef = f64::signum(ab.cross(nor).dot(ap))
        + f64::signum(bc.cross(nor).dot(bp))
        + f64::signum(ca.cross(nor).dot(cp));
    if coef < 2.0 || nor.magnitude().so_small() {
        let a = (ap - ab * f64::clamp(ab.dot(ap) / ab.dot(ab), 0.0, 1.0)).magnitude2();
        let b = (bp - bc * f64::clamp(bc.dot(bp) / bc.dot(bc), 0.0, 1.0)).magnitude2();
        let c = (cp - ca * f64::clamp(ca.dot(cp) / ca.dot(ca), 0.0, 1.0)).magnitude2();
        f64::min(f64::min(a, b), c)
    } else {
        nor.dot(ap) * nor.dot(ap) / nor.magnitude2()
    }
}

fn extract_cpu_triangles(mesh: &PolygonMesh) -> Vec<[Point3; 3]> {
    let positions = mesh.positions();
    mesh.face_iter()
        .flat_map(|face| {
            (2..face.len()).map(move |i| {
                [
                    positions[face[0].pos],
                    positions[face[i - 1].pos],
                    positions[face[i].pos],
                ]
            })
        })
        .collect()
}

fn cpu_dist_single(triangles: &[[Point3; 3]], points: &[Point3]) -> Vec<f64> {
    points
        .iter()
        .map(|&p| {
            triangles
                .iter()
                .map(|&tri| cpu_distance2_point_triangle(p, tri))
                .fold(f64::INFINITY, f64::min)
        })
        .collect()
}

fn cpu_dist_rayon(triangles: &[[Point3; 3]], points: &[Point3]) -> Vec<f64> {
    points
        .par_iter()
        .map(|&p| {
            triangles
                .iter()
                .map(|&tri| cpu_distance2_point_triangle(p, tri))
                .fold(f64::INFINITY, f64::min)
        })
        .collect()
}

fn generate_points(n: usize) -> Vec<Point3> {
    (0..n)
        .map(|i| {
            let u = (i as f64 * 0.12345).sin() * 5.0;
            let v = (i as f64 * 0.67891).cos() * 5.0;
            let w = (i as f64 * 0.24680).sin() * 5.0;
            Point3::new(u, v, w)
        })
        .collect()
}

fn bench_fn<F: FnMut()>(name: &str, mut f: F, iters: usize) -> Duration {
    // Warmup
    f();
    let start = Instant::now();
    for _ in 0..iters {
        f();
    }
    let elapsed = start.elapsed() / iters as u32;
    eprintln!("  {:30}: {:?}", name, elapsed);
    elapsed
}

fn main() {
    let handler = init_vulkan_device();
    let vulkan_analyzer = GpuMeshAnalyzer::new(&handler);

    #[cfg(feature = "cuda")]
    let cuda_analyzer = truck_meshalgo::gpu::CudaMeshAnalyzer::new();

    let sphere_small = create_sphere(2.0, 32, 32);
    let triangles_small = extract_cpu_triangles(&sphere_small);
    let n_triangles_small = triangles_small.len();

    println!("==========================================================================================");
    println!("                           TRUCK CAD GPU vs CPU BENCHMARK                                  ");
    println!("==========================================================================================");
    println!("Sphere Mesh: {} triangles", n_triangles_small);
    println!();

    let point_counts = [1_000, 10_000, 50_000];

    struct ResultRow {
        workload: String,
        points: usize,
        triangles: usize,
        cpu_single: Duration,
        cpu_rayon: Duration,
        gpu_vulkan: Duration,
        gpu_cuda_baseline: Duration,
        gpu_ptx_sm86: Duration,
    }

    let mut results = Vec::new();

    for &n_pts in &point_counts {
        println!(">>> Benchmarking Point-Mesh Distance Evaluation (N = {} points)...", n_pts);
        let points = generate_points(n_pts);

        let iters = if n_pts <= 1_000 { 5 } else if n_pts <= 10_000 { 3 } else { 2 };

        let t_cpu_single = bench_fn("CPU Single-Thread", || {
            let res = cpu_dist_single(&triangles_small, &points);
            std::hint::black_box(res);
        }, iters);

        let t_cpu_rayon = bench_fn("CPU Rayon Multi-Thread", || {
            let res = cpu_dist_rayon(&triangles_small, &points);
            std::hint::black_box(res);
        }, iters);

        let t_vulkan = bench_fn("GPU Vulkan (RTX 3080)", || {
            let res = vulkan_analyzer.distance2_point_cloud(&sphere_small, &points);
            std::hint::black_box(res);
        }, iters);

        #[cfg(feature = "cuda")]
        let t_cuda_baseline = bench_fn("GPU CUDA Baseline", || {
            let res = cuda_analyzer.distance2_point_cloud_baseline(&sphere_small, &points);
            std::hint::black_box(res);
        }, iters);
        #[cfg(not(feature = "cuda"))]
        let t_cuda_baseline = Duration::ZERO;

        #[cfg(feature = "cuda")]
        let t_ptx_sm86 = bench_fn("GPU PTX SM 8.6 (RTX 3080)", || {
            let res = cuda_analyzer.distance2_point_cloud(&sphere_small, &points);
            std::hint::black_box(res);
        }, iters);
        #[cfg(not(feature = "cuda"))]
        let t_ptx_sm86 = Duration::ZERO;

        results.push(ResultRow {
            workload: "Point-Mesh Distance".to_string(),
            points: n_pts,
            triangles: n_triangles_small,
            cpu_single: t_cpu_single,
            cpu_rayon: t_cpu_rayon,
            gpu_vulkan: t_vulkan,
            gpu_cuda_baseline: t_cuda_baseline,
            gpu_ptx_sm86: t_ptx_sm86,
        });

        println!();
    }

    for &n_pts in &[1_000, 10_000, 50_000] {
        println!(">>> Benchmarking In-Out Solid Containment (N = {} points)...", n_pts);
        let points = generate_points(n_pts);
        let iters = if n_pts <= 1_000 { 5 } else if n_pts <= 10_000 { 3 } else { 2 };

        let t_cpu_single = bench_fn("CPU Single-Thread", || {
            let res: Vec<bool> = points.iter().map(|&p| sphere_small.inside(p)).collect();
            std::hint::black_box(res);
        }, iters);

        let t_cpu_rayon = bench_fn("CPU Rayon Multi-Thread", || {
            let res: Vec<bool> = points.par_iter().map(|&p| sphere_small.inside(p)).collect();
            std::hint::black_box(res);
        }, iters);

        let t_vulkan = bench_fn("GPU Vulkan (RTX 3080)", || {
            let res = vulkan_analyzer.inside(&sphere_small, &points);
            std::hint::black_box(res);
        }, iters);

        #[cfg(feature = "cuda")]
        let t_cuda_baseline = bench_fn("GPU CUDA Baseline", || {
            let res = cuda_analyzer.inside_baseline(&sphere_small, &points);
            std::hint::black_box(res);
        }, iters);
        #[cfg(not(feature = "cuda"))]
        let t_cuda_baseline = Duration::ZERO;

        #[cfg(feature = "cuda")]
        let t_ptx_sm86 = bench_fn("GPU PTX SM 8.6 (RTX 3080)", || {
            let res = cuda_analyzer.inside(&sphere_small, &points);
            std::hint::black_box(res);
        }, iters);
        #[cfg(not(feature = "cuda"))]
        let t_ptx_sm86 = Duration::ZERO;

        results.push(ResultRow {
            workload: "In-Out Containment".to_string(),
            points: n_pts,
            triangles: n_triangles_small,
            cpu_single: t_cpu_single,
            cpu_rayon: t_cpu_rayon,
            gpu_vulkan: t_vulkan,
            gpu_cuda_baseline: t_cuda_baseline,
            gpu_ptx_sm86: t_ptx_sm86,
        });

        println!();
    }

    // ========================================================================
    // Candidate 1: Mesh Normal Generation & Smoothing
    // ========================================================================
    {
        let sphere_dense = create_sphere(3.0, 64, 64);
        let n_tris = extract_cpu_triangles(&sphere_dense).len();
        println!(">>> Benchmarking Mesh Normal Generation ({} triangles)...", n_tris);

        let t_cpu_single = bench_fn("CPU Face Normals", || {
            let res: Vec<Vector3> = sphere_dense
                .face_iter()
                .map(|face| {
                    let p0 = sphere_dense.positions()[face[0].pos];
                    let p1 = sphere_dense.positions()[face[1].pos];
                    let p2 = sphere_dense.positions()[face[2].pos];
                    (p1 - p0).cross(p2 - p0).normalize()
                })
                .collect();
            std::hint::black_box(res);
        }, 5);

        let t_cpu_rayon = bench_fn("CPU Rayon Smooth Normals", || {
            let mut clone = sphere_dense.clone();
            clone.add_smooth_normals(0.8, true);
            std::hint::black_box(clone);
        }, 3);

        let t_vulkan = bench_fn("GPU Vulkan Face Normals", || {
            let res = vulkan_analyzer.compute_face_normals(&sphere_dense);
            std::hint::black_box(res);
        }, 10);

        #[cfg(feature = "cuda")]
        let t_cuda_base = bench_fn("GPU CUDA Face Normals", || {
            let res = cuda_analyzer.compute_face_normals(&sphere_dense);
            std::hint::black_box(res);
        }, 10);
        #[cfg(not(feature = "cuda"))]
        let t_cuda_base = Duration::ZERO;

        #[cfg(feature = "cuda")]
        let t_ptx_sm86 = bench_fn("GPU CUDA Smooth Normals", || {
            let res = cuda_analyzer.compute_smooth_normals(&sphere_dense);
            std::hint::black_box(res);
        }, 10);
        #[cfg(not(feature = "cuda"))]
        let t_ptx_sm86 = Duration::ZERO;

        results.push(ResultRow {
            workload: "Mesh Normals".to_string(),
            points: sphere_dense.positions().len(),
            triangles: n_tris,
            cpu_single: t_cpu_single,
            cpu_rayon: t_cpu_rayon,
            gpu_vulkan: t_vulkan,
            gpu_cuda_baseline: t_cuda_base,
            gpu_ptx_sm86: t_ptx_sm86,
        });

        println!();
    }

    // ========================================================================
    // Candidate 2: 2D Trimming Loop Inclusion Testing
    // ========================================================================
    {
        let n_segs = 64;
        let segments: Vec<[Point2; 2]> = (0..n_segs)
            .map(|i| {
                let theta0 = 2.0 * PI * (i as f64) / (n_segs as f64);
                let theta1 = 2.0 * PI * ((i + 1) as f64) / (n_segs as f64);
                let r0 = 1.0 + 0.3 * (theta0 * 4.0).sin();
                let r1 = 1.0 + 0.3 * (theta1 * 4.0).sin();
                [
                    Point2::new(r0 * theta0.cos(), r0 * theta0.sin()),
                    Point2::new(r1 * theta1.cos(), r1 * theta1.sin()),
                ]
            })
            .collect();

        let n_points = 50_000;
        let query_points: Vec<Point2> = (0..n_points)
            .map(|i| {
                let u = (i as f64 * 0.1234).sin() * 2.0;
                let v = (i as f64 * 0.5678).cos() * 2.0;
                Point2::new(u, v)
            })
            .collect();

        println!(">>> Benchmarking 2D Trimming Loop Inclusion ({} segs, {} points)...", n_segs, n_points);

        let t_cpu_single = bench_fn("CPU Single-Thread", || {
            let res: Vec<bool> = query_points
                .iter()
                .map(|&c| {
                    let t = 2.0 * PI * hash::HashGen::hash1(c);
                    let r = Vector2::new(t.cos(), t.sin());
                    let mut counter = 0i32;
                    let mut valid = true;
                    for seg in &segments {
                        let a = seg[0] - c;
                        let b = seg[1] - c;
                        let s0 = r.x * a.y - r.y * a.x;
                        let s1 = r.x * b.y - r.y * b.x;
                        let s2 = a.x * b.y - a.y * b.x;
                        let denom = s1 - s0;
                        if denom.abs() < 1e-12 { continue; }
                        let x = s2 / denom;
                        if x.abs() < 1e-7 && s0 * s1 < 0.0 { valid = false; break; }
                        else if x > 0.0 && s0 <= 0.0 && s1 > 0.0 { counter += 1; }
                        else if x > 0.0 && s0 >= 0.0 && s1 < 0.0 { counter -= 1; }
                    }
                    valid && counter > 0
                })
                .collect();
            std::hint::black_box(res);
        }, 2);

        let t_cpu_rayon = bench_fn("CPU Rayon Multi-Thread", || {
            let res: Vec<bool> = query_points
                .par_iter()
                .map(|&c| {
                    let t = 2.0 * PI * hash::HashGen::hash1(c);
                    let r = Vector2::new(t.cos(), t.sin());
                    let mut counter = 0i32;
                    let mut valid = true;
                    for seg in &segments {
                        let a = seg[0] - c;
                        let b = seg[1] - c;
                        let s0 = r.x * a.y - r.y * a.x;
                        let s1 = r.x * b.y - r.y * b.x;
                        let s2 = a.x * b.y - a.y * b.x;
                        let denom = s1 - s0;
                        if denom.abs() < 1e-12 { continue; }
                        let x = s2 / denom;
                        if x.abs() < 1e-7 && s0 * s1 < 0.0 { valid = false; break; }
                        else if x > 0.0 && s0 <= 0.0 && s1 > 0.0 { counter += 1; }
                        else if x > 0.0 && s0 >= 0.0 && s1 < 0.0 { counter -= 1; }
                    }
                    valid && counter > 0
                })
                .collect();
            std::hint::black_box(res);
        }, 3);

        let t_vulkan = bench_fn("GPU Vulkan (RTX 3080)", || {
            let res = vulkan_analyzer.trimming_loop_include(&segments, &query_points);
            std::hint::black_box(res);
        }, 5);

        #[cfg(feature = "cuda")]
        let t_ptx_sm86 = bench_fn("GPU PTX SM 8.6 (RTX 3080)", || {
            let res = cuda_analyzer.trimming_loop_include(&segments, &query_points);
            std::hint::black_box(res);
        }, 5);
        #[cfg(not(feature = "cuda"))]
        let t_ptx_sm86 = Duration::ZERO;

        results.push(ResultRow {
            workload: "2D Trimming Loop".to_string(),
            points: n_points,
            triangles: n_segs,
            cpu_single: t_cpu_single,
            cpu_rayon: t_cpu_rayon,
            gpu_vulkan: t_vulkan,
            gpu_cuda_baseline: t_ptx_sm86,
            gpu_ptx_sm86: t_ptx_sm86,
        });

        println!();
    }

    // ========================================================================
    // Candidate 3: Triangle-Triangle Collision & Interference
    // ========================================================================
    {
        let sphere0 = create_sphere(2.0, 24, 24);
        let sphere1 = PolygonMesh::new(
            StandardAttributes {
                positions: sphere0.positions().iter().map(|&p| p + Vector3::new(1.0, 0.5, 0.5)).collect(),
                ..Default::default()
            },
            sphere0.faces().clone(),
        );
        let n_tris = extract_cpu_triangles(&sphere0).len();

        println!(">>> Benchmarking Mesh-Mesh Interference ({} x {} tris)...", n_tris, n_tris);

        let t_cpu_single = bench_fn("CPU extract_interference", || {
            let res = sphere0.extract_interference(&sphere1);
            std::hint::black_box(res);
        }, 3);

        let t_cpu_rayon = bench_fn("CPU collide_with", || {
            let res = sphere0.collide_with(&sphere1);
            std::hint::black_box(res);
        }, 5);

        let t_vulkan = bench_fn("GPU Vulkan collide_triangles", || {
            let res = vulkan_analyzer.collide_triangles(&sphere0, &sphere1);
            std::hint::black_box(res);
        }, 5);

        #[cfg(feature = "cuda")]
        let t_ptx_sm86 = bench_fn("GPU CUDA collide_triangles", || {
            let res = cuda_analyzer.collide_triangles(&sphere0, &sphere1);
            std::hint::black_box(res);
        }, 5);
        #[cfg(not(feature = "cuda"))]
        let t_ptx_sm86 = Duration::ZERO;

        results.push(ResultRow {
            workload: "Mesh Interference".to_string(),
            points: sphere0.positions().len(),
            triangles: n_tris,
            cpu_single: t_cpu_single,
            cpu_rayon: t_cpu_rayon,
            gpu_vulkan: t_vulkan,
            gpu_cuda_baseline: t_ptx_sm86,
            gpu_ptx_sm86: t_ptx_sm86,
        });

        println!();
    }

    // ========================================================================
    // Candidate 4: Batch B-Spline Surface Evaluation
    // ========================================================================
    {
        use truck_geometry::prelude::*;

        let u_knots = vec![0.0, 0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0, 1.0];
        let v_knots = vec![0.0, 0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0, 1.0];
        let u_degree = 3;
        let v_degree = 3;

        let ctrl_pts: Vec<Vec<Point3>> = (0..5)
            .map(|i| {
                (0..5)
                    .map(|j| {
                        let x = i as f64 / 4.0;
                        let y = j as f64 / 4.0;
                        let z = (x * PI).sin() * (y * PI).sin();
                        Point3::new(x, y, z)
                    })
                    .collect()
            })
            .collect();

        let bsp = BSplineSurface::new(
            (KnotVec::from(u_knots.clone()), KnotVec::from(v_knots.clone())),
            ctrl_pts.clone(),
        );

        let n_points = 50_000;
        let query_uv: Vec<Point2> = (0..n_points)
            .map(|i| {
                let u = (i as f64 * 0.12345).sin().abs() * 0.98 + 0.01;
                let v = (i as f64 * 0.67891).cos().abs() * 0.98 + 0.01;
                Point2::new(u, v)
            })
            .collect();

        println!(">>> Benchmarking Batch B-Spline Surface Evaluation (N = {} points)...", n_points);

        let t_cpu_single = bench_fn("CPU Single-Thread", || {
            let res: Vec<(Point3, Vector3)> = query_uv
                .iter()
                .map(|&p| (bsp.subs(p.x, p.y), bsp.normal(p.x, p.y)))
                .collect();
            std::hint::black_box(res);
        }, 2);

        let t_cpu_rayon = bench_fn("CPU Rayon Multi-Thread", || {
            let res: Vec<(Point3, Vector3)> = query_uv
                .par_iter()
                .map(|&p| (bsp.subs(p.x, p.y), bsp.normal(p.x, p.y)))
                .collect();
            std::hint::black_box(res);
        }, 3);

        #[cfg(feature = "cuda")]
        let t_ptx_sm86 = bench_fn("GPU CUDA (RTX 3080)", || {
            let res = cuda_analyzer.evaluate_bspline_surface(
                &u_knots,
                &v_knots,
                u_degree,
                v_degree,
                &ctrl_pts,
                &query_uv,
            );
            std::hint::black_box(res);
        }, 5);
        #[cfg(not(feature = "cuda"))]
        let t_ptx_sm86 = Duration::ZERO;

        results.push(ResultRow {
            workload: "B-Spline Evaluation".to_string(),
            points: n_points,
            triangles: 25,
            cpu_single: t_cpu_single,
            cpu_rayon: t_cpu_rayon,
            gpu_vulkan: Duration::ZERO,
            gpu_cuda_baseline: t_ptx_sm86,
            gpu_ptx_sm86: t_ptx_sm86,
        });

        println!();
    }

    println!("\n=================================================================================================================================");
    println!("                                                    BENCHMARK SUMMARY TABLE                                                      ");
    println!("=================================================================================================================================");
    println!(
        "| {:<20} | {:>7} | {:>9} | {:>10} | {:>10} | {:>10} | {:>12} | {:>13} | {:>11} | {:>10} |",
        "Workload", "Points", "Triangles", "CPU (1T)", "CPU (Rayon)", "GPU (Vulkan)", "CUDA Base", "GPU PTX SM86", "PTX vs Base", "PTX vs 1T"
    );
    println!(
        "|{:-<22}|{:-<9}|{:-<11}|{:-<12}|{:-<12}|{:-<12}|{:-<14}|{:-<15}|{:-<13}|{:-<12}|",
        "", "", "", "", "", "", "", "", "", ""
    );

    for r in results {
        let cpu_s_ms = r.cpu_single.as_secs_f64() * 1000.0;
        let cpu_r_ms = r.cpu_rayon.as_secs_f64() * 1000.0;
        let vulkan_ms = r.gpu_vulkan.as_secs_f64() * 1000.0;
        let cuda_base_ms = r.gpu_cuda_baseline.as_secs_f64() * 1000.0;
        let ptx_ms = r.gpu_ptx_sm86.as_secs_f64() * 1000.0;
        let speedup_base = cuda_base_ms / ptx_ms;
        let speedup_1t = cpu_s_ms / ptx_ms;

        println!(
            "| {:<20} | {:>7} | {:>9} | {:>9.2}ms | {:>9.2}ms | {:>9.2}ms | {:>11.2}ms | {:>12.2}ms | {:>11.1}x | {:>10.1}x |",
            r.workload,
            r.points,
            r.triangles,
            cpu_s_ms,
            cpu_r_ms,
            vulkan_ms,
            cuda_base_ms,
            ptx_ms,
            speedup_base,
            speedup_1t
        );
    }
    println!("=================================================================================================================================");
}
