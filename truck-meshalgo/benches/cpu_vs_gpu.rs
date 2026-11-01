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
