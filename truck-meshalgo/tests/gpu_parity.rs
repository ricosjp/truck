#![cfg(feature = "gpu")]

use truck_base::cgmath64::*;
use truck_meshalgo::prelude::*;
use truck_platform::wgpu::*;
use truck_platform::*;

mod common;

static SHARED_DEVICE: std::sync::OnceLock<DeviceHandler> = std::sync::OnceLock::new();

fn init_vulkan_device() -> DeviceHandler {
    SHARED_DEVICE
        .get_or_init(|| {
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
                eprintln!(
                    "Running GPU Parity Test on: {:?} ({:?})",
                    adapter.get_info().name,
                    adapter.get_info().backend
                );
                let (device, queue) = adapter
                    .request_device(
                        &DeviceDescriptor {
                            label: Some("gpu_parity_device"),
                            ..Default::default()
                        },
                    )
                    .await
                    .expect("Failed to create device");
                DeviceHandler::new(adapter, device, queue)
            })
        })
        .clone()
}

// Ground-truth CPU distance function from truck-meshalgo
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

fn cpu_min_distance2(mesh: &PolygonMesh, point: Point3) -> f64 {
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
        .map(|tri| cpu_distance2_point_triangle(point, tri))
        .fold(f64::INFINITY, f64::min)
}

fn create_unit_cube() -> PolygonMesh {
    PolygonMesh::new(
        StandardAttributes {
            positions: vec![
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(1.0, 0.0, 0.0),
                Point3::new(1.0, 1.0, 0.0),
                Point3::new(0.0, 1.0, 0.0),
                Point3::new(0.0, 0.0, 1.0),
                Point3::new(1.0, 0.0, 1.0),
                Point3::new(1.0, 1.0, 1.0),
                Point3::new(0.0, 1.0, 1.0),
            ],
            ..Default::default()
        },
        Faces::from_iter(vec![
            [3, 2, 1, 0], // bottom
            [4, 5, 6, 7], // top
            [0, 1, 5, 4], // front
            [2, 3, 7, 6], // back
            [0, 4, 7, 3], // left
            [1, 2, 6, 5], // right
        ]),
    )
}

#[test]
fn test_gpu_distance_parity_cube() {
    let handler = init_vulkan_device();
    let analyzer = GpuMeshAnalyzer::new(&handler);
    let cube = create_unit_cube();

    // Generate diverse test points
    let mut test_points = vec![
        Point3::new(0.5, 0.5, 0.5),   // center
        Point3::new(0.0, 0.0, 0.0),   // vertex
        Point3::new(1.0, 1.0, 1.0),   // vertex
        Point3::new(0.5, 0.5, 1.0),   // on face
        Point3::new(0.5, 0.0, 0.0),   // on edge
        Point3::new(2.0, 2.0, 2.0),   // outside
        Point3::new(-1.0, -2.0, -3.0), // far outside
    ];

    // Grid of points across [-1, 2]^3
    for x in [-0.5, 0.25, 0.5, 0.75, 1.5] {
        for y in [-0.5, 0.25, 0.5, 0.75, 1.5] {
            for z in [-0.5, 0.25, 0.5, 0.75, 1.5] {
                test_points.push(Point3::new(x, y, z));
            }
        }
    }

    let gpu_distances = analyzer.distance2_point_cloud(&cube, &test_points);
    assert_eq!(gpu_distances.len(), test_points.len());

    for (i, &pt) in test_points.iter().enumerate() {
        let cpu_dist2 = cpu_min_distance2(&cube, pt);
        let gpu_dist2 = gpu_distances[i] as f64;
        let diff = (gpu_dist2 - cpu_dist2).abs();
        let rel_err = diff / cpu_dist2.max(1.0);

        assert!(
            rel_err < 1e-4,
            "Point #{}: {:?} - CPU dist2: {}, GPU dist2: {}, diff: {}, rel_err: {}",
            i,
            pt,
            cpu_dist2,
            gpu_dist2,
            diff,
            rel_err
        );
    }
}

#[test]
fn test_gpu_distance_parity_sphere() {
    let handler = init_vulkan_device();
    let analyzer = GpuMeshAnalyzer::new(&handler);
    let sphere_mesh = common::shapes::sphere(Point3::origin(), 1.0, 16, 16);

    let mut test_points = Vec::new();
    // Concentric shells of points
    for r in [0.0, 0.5, 0.9, 1.0, 1.1, 2.0, 5.0] {
        for theta_i in 0..8 {
            let theta = theta_i as f64 * std::f64::consts::PI / 4.0;
            test_points.push(Point3::new(r * theta.cos(), r * theta.sin(), 0.0));
            test_points.push(Point3::new(0.0, r * theta.sin(), r * theta.cos()));
        }
    }

    let gpu_distances = analyzer.distance2_point_cloud(&sphere_mesh, &test_points);
    assert_eq!(gpu_distances.len(), test_points.len());

    for (i, &pt) in test_points.iter().enumerate() {
        let cpu_dist2 = cpu_min_distance2(&sphere_mesh, pt);
        let gpu_dist2 = gpu_distances[i] as f64;
        let diff = (gpu_dist2 - cpu_dist2).abs();
        let rel_err = diff / cpu_dist2.max(1.0);

        assert!(
            rel_err < 1e-3,
            "Point #{}: {:?} - CPU dist2: {}, GPU dist2: {}, diff: {}, rel_err: {}",
            i,
            pt,
            cpu_dist2,
            gpu_dist2,
            diff,
            rel_err
        );
    }
}

#[test]
fn test_gpu_in_out_containment_parity_cube() {
    let handler = init_vulkan_device();
    let analyzer = GpuMeshAnalyzer::new(&handler);
    let cube = create_unit_cube();

    let ray_dir = Vector3::new(0.577350269, 0.577350269, 0.577350269);

    let test_points = vec![
        // Points strictly inside the cube
        Point3::new(0.5, 0.5, 0.5),
        Point3::new(0.2, 0.3, 0.4),
        Point3::new(0.8, 0.7, 0.6),
        Point3::new(0.1, 0.1, 0.1),
        // Points strictly outside the cube
        Point3::new(-0.2, 0.5, 0.5),
        Point3::new(1.2, 0.5, 0.5),
        Point3::new(0.5, -0.2, 0.5),
        Point3::new(0.5, 1.2, 0.5),
        Point3::new(0.5, 0.5, -0.2),
        Point3::new(0.5, 0.5, 1.2),
        Point3::new(3.0, 3.0, 3.0),
        Point3::new(-2.0, -1.0, 4.0),
    ];

    // Evaluate on CPU
    let cpu_inside: Vec<bool> = test_points.iter().map(|&p| cube.inside(p)).collect();
    let cpu_signed: Vec<isize> = test_points
        .iter()
        .map(|&p| cube.signed_crossing_faces(p, ray_dir))
        .collect();

    // Evaluate on GPU
    let gpu_signed = analyzer.signed_crossing_faces(&cube, &test_points, ray_dir);
    let gpu_inside = analyzer.inside(&cube, &test_points);

    for i in 0..test_points.len() {
        assert_eq!(
            gpu_signed[i] as isize, cpu_signed[i],
            "Signed crossing count mismatch at point #{}: {:?}, GPU: {}, CPU: {}",
            i, test_points[i], gpu_signed[i], cpu_signed[i]
        );
        assert_eq!(
            gpu_inside[i], cpu_inside[i],
            "Inside/outside mismatch at point #{}: {:?}, GPU: {}, CPU: {}",
            i, test_points[i], gpu_inside[i], cpu_inside[i]
        );
    }
}

#[test]
fn test_gpu_large_scale_parity_and_timing() {
    let handler = init_vulkan_device();
    let analyzer = GpuMeshAnalyzer::new(&handler);
    let sphere_mesh = common::shapes::sphere(Point3::origin(), 2.0, 32, 32);

    let n_points = 5000;
    let points: Vec<Point3> = (0..n_points)
        .map(|i| {
            let u = (i as f64 * 0.12345).sin() * 4.0;
            let v = (i as f64 * 0.67891).cos() * 4.0;
            let w = (i as f64 * 0.24680).sin() * 4.0;
            Point3::new(u, v, w)
        })
        .collect();

    // Time GPU distance calculation
    let start = std::time::Instant::now();
    let gpu_distances = analyzer.distance2_point_cloud(&sphere_mesh, &points);
    let gpu_duration = start.elapsed();
    eprintln!(
        "GPU computed {} points against {} triangles in {:?}",
        n_points,
        sphere_mesh.faces().len(),
        gpu_duration
    );

    assert_eq!(gpu_distances.len(), n_points);

    // Verify parity against CPU for a subsample of 100 points
    for i in (0..n_points).step_by(50) {
        let pt = points[i];
        let cpu_dist2 = cpu_min_distance2(&sphere_mesh, pt);
        let gpu_dist2 = gpu_distances[i] as f64;
        let diff = (gpu_dist2 - cpu_dist2).abs();
        let rel_err = diff / cpu_dist2.max(1.0);

        assert!(
            rel_err < 1e-3,
            "Mismatch at point #{}: {:?} - CPU: {}, GPU: {}, rel_err: {}",
            i, pt, cpu_dist2, gpu_dist2, rel_err
        );
    }

    // Test GPU inside/outside classification
    let inside_flags = analyzer.inside(&sphere_mesh, &points);
    assert_eq!(inside_flags.len(), n_points);
    for i in (0..n_points).step_by(50) {
        let pt = points[i];
        let cpu_in = sphere_mesh.inside(pt);
        let gpu_in = inside_flags[i];
        assert_eq!(
            gpu_in, cpu_in,
            "Inside mismatch at point #{}: {:?}, GPU: {}, CPU: {}",
            i, pt, gpu_in, cpu_in
        );
    }
}

#[cfg(feature = "cuda")]
#[test]
fn test_cuda_distance_and_containment_parity() {
    let cuda_analyzer = CudaMeshAnalyzer::new();
    let cube = create_unit_cube();

    let mut test_points = vec![
        Point3::new(0.5, 0.5, 0.5),
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(0.5, 0.5, 1.0),
        Point3::new(2.0, 2.0, 2.0),
        Point3::new(-1.0, -2.0, -3.0),
    ];

    for x in [-0.5, 0.25, 0.5, 0.75, 1.5] {
        for y in [-0.5, 0.25, 0.5, 0.75, 1.5] {
            for z in [-0.5, 0.25, 0.5, 0.75, 1.5] {
                test_points.push(Point3::new(x, y, z));
            }
        }
    }

    // Distance test
    let cuda_distances = cuda_analyzer.distance2_point_cloud(&cube, &test_points);
    assert_eq!(cuda_distances.len(), test_points.len());

    for (i, &pt) in test_points.iter().enumerate() {
        let cpu_dist2 = cpu_min_distance2(&cube, pt);
        let cuda_dist2 = cuda_distances[i] as f64;
        let diff = (cuda_dist2 - cpu_dist2).abs();
        let rel_err = diff / cpu_dist2.max(1.0);

        assert!(
            rel_err < 1e-4,
            "CUDA Distance mismatch at point #{}: {:?} - CPU: {}, CUDA: {}, diff: {}, rel_err: {}",
            i, pt, cpu_dist2, cuda_dist2, diff, rel_err
        );
    }

    // Containment test
    let cpu_inside: Vec<bool> = test_points.iter().map(|&p| cube.inside(p)).collect();
    let cuda_inside = cuda_analyzer.inside(&cube, &test_points);
    for i in 0..test_points.len() {
        assert_eq!(
            cuda_inside[i], cpu_inside[i],
            "CUDA inside mismatch at point #{}: {:?}, CUDA: {}, CPU: {}",
            i, test_points[i], cuda_inside[i], cpu_inside[i]
        );
    }
}

#[cfg(feature = "cuda")]
#[test]
fn test_vulkan_vs_cuda_exact_parity() {
    let handler = init_vulkan_device();
    let vulkan_analyzer = GpuMeshAnalyzer::new(&handler);
    let cuda_analyzer = CudaMeshAnalyzer::new();

    let sphere_mesh = common::shapes::sphere(Point3::origin(), 1.5, 24, 24);
    let n_points = 500;
    let points: Vec<Point3> = (0..n_points)
        .map(|i| {
            let u = (i as f64 * 0.314).sin() * 3.0;
            let v = (i as f64 * 0.271).cos() * 3.0;
            let w = (i as f64 * 0.141).sin() * 3.0;
            Point3::new(u, v, w)
        })
        .collect();

    let vulkan_dist = vulkan_analyzer.distance2_point_cloud(&sphere_mesh, &points);
    let cuda_dist = cuda_analyzer.distance2_point_cloud(&sphere_mesh, &points);

    assert_eq!(vulkan_dist.len(), n_points);
    assert_eq!(cuda_dist.len(), n_points);

    for i in 0..n_points {
        let diff = (vulkan_dist[i] - cuda_dist[i]).abs();
        let rel_diff = diff / vulkan_dist[i].max(1.0);
        assert!(
            rel_diff < 1e-4,
            "Vulkan vs CUDA distance difference at index {}: Vulkan={}, CUDA={}, diff={}",
            i, vulkan_dist[i], cuda_dist[i], diff
        );
    }

    let vulkan_inside = vulkan_analyzer.inside(&sphere_mesh, &points);
    let cuda_inside = cuda_analyzer.inside(&sphere_mesh, &points);
    assert_eq!(vulkan_inside, cuda_inside, "Vulkan and CUDA inside classifications differ!");
}

#[cfg(feature = "cuda")]
#[test]
fn test_cuda_baseline_vs_sm86_ptx_exact_parity() {
    let cuda = CudaMeshAnalyzer::new();
    let sphere = common::shapes::sphere(Point3::origin(), 1.2, 20, 20);

    let n_points = 1000;
    let points: Vec<Point3> = (0..n_points)
        .map(|i| {
            let u = (i as f64 * 0.421).sin() * 2.5;
            let v = (i as f64 * 0.733).cos() * 2.5;
            let w = (i as f64 * 0.189).sin() * 2.5;
            Point3::new(u, v, w)
        })
        .collect();

    // 1. Distance Parity: Baseline vs SM86 Runtime vs Driver PTX
    let d_opt = cuda.distance2_point_cloud(&sphere, &points);
    let d_base = cuda.distance2_point_cloud_baseline(&sphere, &points);
    let d_ptx = cuda.distance2_point_cloud_ptx(&sphere, &points);

    assert_eq!(d_opt.len(), n_points);
    assert_eq!(d_base.len(), n_points);
    assert_eq!(d_ptx.len(), n_points);

    for i in 0..n_points {
        let diff_opt_base = (d_opt[i] - d_base[i]).abs();
        let diff_ptx_base = (d_ptx[i] - d_base[i]).abs();
        assert!(
            diff_opt_base < 1e-4,
            "Distance mismatch at #{}: Opt={}, Base={}, diff={}",
            i, d_opt[i], d_base[i], diff_opt_base
        );
        assert!(
            diff_ptx_base < 1e-4,
            "Distance mismatch at #{}: PTX={}, Base={}, diff={}",
            i, d_ptx[i], d_base[i], diff_ptx_base
        );
    }

    // 2. Ray Crossing Parity: Baseline vs SM86 Runtime vs Driver PTX
    let ray_dirs: Vec<Vector3> = points
        .iter()
        .map(|&p| hash::take_one_unit(p))
        .collect();

    let c_opt = cuda.signed_crossing_rays(&sphere, &points, &ray_dirs);
    let c_base = cuda.signed_crossing_rays_baseline(&sphere, &points, &ray_dirs);
    let c_ptx = cuda.signed_crossing_rays_ptx(&sphere, &points, &ray_dirs);

    assert_eq!(c_opt, c_base, "SM86 optimized ray crossings differ from baseline!");
    assert_eq!(c_ptx, c_base, "Driver PTX ray crossings differ from baseline!");

    let in_opt = cuda.inside(&sphere, &points);
    let in_base = cuda.inside_baseline(&sphere, &points);
    let in_ptx = cuda.inside_ptx(&sphere, &points);

    assert_eq!(in_opt, in_base, "SM86 inside flags differ from baseline!");
    assert_eq!(in_ptx, in_base, "Driver PTX inside flags differ from baseline!");
}

#[cfg(feature = "cuda")]
#[test]
fn test_cuda_face_and_smooth_normals() {
    let cuda = CudaMeshAnalyzer::new();
    let sphere = common::shapes::sphere(Point3::origin(), 1.0, 16, 16);

    // 1. Face normals
    let face_normals = cuda.compute_face_normals(&sphere);
    let triangles = CudaMeshAnalyzer::extract_triangles(&sphere);
    assert_eq!(face_normals.len(), triangles.len());

    for (i, &tri) in triangles.iter().enumerate() {
        let p0 = Point3::new(tri.v0[0] as f64, tri.v0[1] as f64, tri.v0[2] as f64);
        let p1 = Point3::new(tri.v1[0] as f64, tri.v1[1] as f64, tri.v1[2] as f64);
        let p2 = Point3::new(tri.v2[0] as f64, tri.v2[1] as f64, tri.v2[2] as f64);
        let cross = (p1 - p0).cross(p2 - p0);
        let cuda_nor = face_normals[i];
        if cross.magnitude2() < 1e-12 {
            assert_eq!(cuda_nor, Vector3::zero(), "Degenerate polar triangle should yield zero normal");
        } else {
            let expected_nor = cross.normalize();
            let diff = (cuda_nor - expected_nor).magnitude();
            assert!(diff < 1e-4, "Face normal mismatch at {}: expected {:?}, got {:?}", i, expected_nor, cuda_nor);
        }
    }

    // 2. Smooth normals on sphere (should point radially outwards)
    let smooth_normals = cuda.compute_smooth_normals(&sphere);
    let positions = sphere.positions();
    assert_eq!(smooth_normals.len(), positions.len());

    for (i, &pos) in positions.iter().enumerate() {
        let expected_radial = pos.to_vec().normalize();
        let cuda_nor = smooth_normals[i];
        let dot = cuda_nor.dot(expected_radial);
        assert!(dot > 0.98, "Smooth normal not radial at vertex #{}: dot={}", i, dot);
    }
}

#[cfg(feature = "cuda")]
#[test]
fn test_cuda_trimming_loop_include() {
    let cuda = CudaMeshAnalyzer::new();

    // Unit square boundary [0, 1]^2
    let segments = vec![
        [Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)],
        [Point2::new(1.0, 0.0), Point2::new(1.0, 1.0)],
        [Point2::new(1.0, 1.0), Point2::new(0.0, 1.0)],
        [Point2::new(0.0, 1.0), Point2::new(0.0, 0.0)],
    ];

    let query_points = vec![
        Point2::new(0.5, 0.5),   // inside
        Point2::new(0.2, 0.8),   // inside
        Point2::new(1.5, 0.5),   // outside
        Point2::new(-0.5, 0.5),  // outside
        Point2::new(0.5, 1.5),   // outside
        Point2::new(0.5, -0.5),  // outside
    ];

    let inside = cuda.trimming_loop_include(&segments, &query_points);
    assert_eq!(inside, vec![true, true, false, false, false, false]);
}

#[cfg(feature = "cuda")]
#[test]
fn test_cuda_collide_triangles() {
    let cuda = CudaMeshAnalyzer::new();
    let cube0 = create_unit_cube();

    // Overlapping cube translated by (0.5, 0.5, 0.5)
    let cube1 = PolygonMesh::new(
        StandardAttributes {
            positions: cube0.positions().iter().map(|&p| p + Vector3::new(0.5, 0.5, 0.5)).collect(),
            ..Default::default()
        },
        cube0.faces().clone(),
    );

    // Far cube translated by (10.0, 0.0, 0.0)
    let cube_far = PolygonMesh::new(
        StandardAttributes {
            positions: cube0.positions().iter().map(|&p| p + Vector3::new(10.0, 0.0, 0.0)).collect(),
            ..Default::default()
        },
        cube0.faces().clone(),
    );

    // Colliding cubes
    let hit = cuda.collide_with(&cube0, &cube1);
    assert!(hit.is_some(), "Overlapping cubes should collide!");

    let lines = cuda.collide_triangles(&cube0, &cube1);
    assert!(!lines.is_empty(), "Should extract interference lines for overlapping cubes!");

    // Non-colliding cubes
    let miss = cuda.collide_with(&cube0, &cube_far);
    assert!(miss.is_none(), "Disjoint cubes should not collide!");

    let no_lines = cuda.collide_triangles(&cube0, &cube_far);
    assert!(no_lines.is_empty(), "Disjoint cubes should have 0 interference lines!");
}

#[cfg(feature = "cuda")]
#[test]
fn test_cuda_evaluate_bspline_surface() {
    use truck_geometry::prelude::*;

    let cuda = CudaMeshAnalyzer::new();

    // Construct a bi-quadratic B-spline surface
    let u_knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let v_knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let u_degree = 2;
    let v_degree = 2;

    let ctrl_pts = vec![
        vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.5, 0.0, 1.0), Point3::new(1.0, 0.0, 0.0)],
        vec![Point3::new(0.0, 0.5, 1.0), Point3::new(0.5, 0.5, 2.0), Point3::new(1.0, 0.5, 1.0)],
        vec![Point3::new(0.0, 1.0, 0.0), Point3::new(0.5, 1.0, 1.0), Point3::new(1.0, 1.0, 0.0)],
    ];

    let bsp = BSplineSurface::new(
        (KnotVec::from(u_knots.clone()), KnotVec::from(v_knots.clone())),
        ctrl_pts.clone(),
    );

    let mut query_uv = Vec::new();
    for i in 0..10 {
        for j in 0..10 {
            query_uv.push(Point2::new(i as f64 / 9.0, j as f64 / 9.0));
        }
    }

    let (cuda_pts, cuda_nors) = cuda.evaluate_bspline_surface(
        &u_knots,
        &v_knots,
        u_degree,
        v_degree,
        &ctrl_pts,
        &query_uv,
    );

    assert_eq!(cuda_pts.len(), query_uv.len());
    assert_eq!(cuda_nors.len(), query_uv.len());

    for (k, &uv) in query_uv.iter().enumerate() {
        let cpu_pt = bsp.subs(uv.x, uv.y);
        let cpu_nor = bsp.normal(uv.x, uv.y);

        let pt_diff = (cuda_pts[k] - cpu_pt).magnitude();
        assert!(pt_diff < 1e-4, "Surface point mismatch at index {}: CPU={:?}, CUDA={:?}", k, cpu_pt, cuda_pts[k]);

        let nor_diff = (cuda_nors[k] - cpu_nor).magnitude();
        assert!(nor_diff < 1e-3, "Surface normal mismatch at index {}: CPU={:?}, CUDA={:?}", k, cpu_nor, cuda_nors[k]);
    }
}

#[cfg(feature = "cuda")]
#[test]
fn test_cuda_evaluate_nurbs_surface() {
    use truck_geometry::prelude::*;

    let cuda = CudaMeshAnalyzer::new();

    // Construct a bi-quadratic rational NURBS surface patch with varying weights
    let u_knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let v_knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let u_degree = 2;
    let v_degree = 2;

    let w = std::f64::consts::FRAC_1_SQRT_2;
    let ctrl_pts = vec![
        vec![
            Vector4::new(1.0, 0.0, 0.0, 1.0),
            Vector4::new(w, w * 0.5, 0.0, w),
            Vector4::new(0.0, 1.0, 0.0, 1.0),
        ],
        vec![
            Vector4::new(w, 0.0, w * 0.8, w),
            Vector4::new(0.5, 0.5, 1.2, 0.8),
            Vector4::new(0.0, w, w * 0.8, w),
        ],
        vec![
            Vector4::new(1.0, 0.0, 1.5, 1.0),
            Vector4::new(w, w * 0.5, 1.5 * w, w),
            Vector4::new(0.0, 1.0, 1.5, 1.0),
        ],
    ];

    let nurbs = NurbsSurface::new(BSplineSurface::new(
        (KnotVec::from(u_knots.clone()), KnotVec::from(v_knots.clone())),
        ctrl_pts.clone(),
    ));

    // Verify as_nurbs trait implementation
    let nurbs_data = nurbs.as_nurbs();
    assert!(nurbs_data.is_some());
    let (uk, vk, ud, vd, cp) = nurbs_data.unwrap();
    assert_eq!(uk, u_knots.as_slice());
    assert_eq!(vk, v_knots.as_slice());
    assert_eq!(ud, u_degree);
    assert_eq!(vd, v_degree);
    assert_eq!(cp.len(), 3);

    let mut query_uv = Vec::new();
    for i in 0..15 {
        for j in 0..15 {
            query_uv.push(Point2::new(i as f64 / 14.0, j as f64 / 14.0));
        }
    }

    let (cuda_pts, cuda_nors) = cuda.evaluate_nurbs_surface(
        &u_knots,
        &v_knots,
        u_degree,
        v_degree,
        &ctrl_pts,
        &query_uv,
    );

    assert_eq!(cuda_pts.len(), query_uv.len());
    assert_eq!(cuda_nors.len(), query_uv.len());

    for (k, &uv) in query_uv.iter().enumerate() {
        let cpu_pt = nurbs.subs(uv.x, uv.y);
        let cpu_nor = nurbs.normal(uv.x, uv.y);

        let pt_diff = (cuda_pts[k] - cpu_pt).magnitude();
        assert!(pt_diff < 1e-10, "NURBS point mismatch at {}: CPU={:?}, CUDA={:?}, diff={:e}", k, cpu_pt, cuda_pts[k], pt_diff);

        let nor_diff = (cuda_nors[k] - cpu_nor).magnitude();
        assert!(nor_diff < 1e-8, "NURBS normal mismatch at {}: CPU={:?}, CUDA={:?}, diff={:e}", k, cpu_nor, cuda_nors[k], nor_diff);
    }
}

#[cfg(feature = "gpu")]
#[test]
fn test_vulkan_evaluate_nurbs_surface() {
    use truck_geometry::prelude::*;

    let handler = init_vulkan_device();
    let vulkan = GpuMeshAnalyzer::new(&handler);

    let u_knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let v_knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let u_degree = 2;
    let v_degree = 2;

    let w = std::f64::consts::FRAC_1_SQRT_2;
    let ctrl_pts = vec![
        vec![
            Vector4::new(1.0, 0.0, 0.0, 1.0),
            Vector4::new(w, w * 0.5, 0.0, w),
            Vector4::new(0.0, 1.0, 0.0, 1.0),
        ],
        vec![
            Vector4::new(w, 0.0, w * 0.8, w),
            Vector4::new(0.5, 0.5, 1.2, 0.8),
            Vector4::new(0.0, w, w * 0.8, w),
        ],
        vec![
            Vector4::new(1.0, 0.0, 1.5, 1.0),
            Vector4::new(w, w * 0.5, 1.5 * w, w),
            Vector4::new(0.0, 1.0, 1.5, 1.0),
        ],
    ];

    let nurbs = NurbsSurface::new(BSplineSurface::new(
        (KnotVec::from(u_knots.clone()), KnotVec::from(v_knots.clone())),
        ctrl_pts.clone(),
    ));

    let mut query_uv = Vec::new();
    for i in 0..10 {
        for j in 0..10 {
            query_uv.push(Point2::new(i as f64 / 9.0, j as f64 / 9.0));
        }
    }

    let (vulkan_pts, vulkan_nors) = vulkan.evaluate_nurbs_surface(
        &u_knots,
        &v_knots,
        u_degree,
        v_degree,
        &ctrl_pts,
        &query_uv,
    );

    assert_eq!(vulkan_pts.len(), query_uv.len());
    assert_eq!(vulkan_nors.len(), query_uv.len());

    for (k, &uv) in query_uv.iter().enumerate() {
        let cpu_pt = nurbs.subs(uv.x, uv.y);
        let cpu_nor = nurbs.normal(uv.x, uv.y);

        let pt_diff = (vulkan_pts[k] - cpu_pt).magnitude();
        assert!(pt_diff < 1e-3, "Vulkan NURBS point mismatch at {}: CPU={:?}, Vulkan={:?}, diff={:e}", k, cpu_pt, vulkan_pts[k], pt_diff);

        let nor_diff = (vulkan_nors[k] - cpu_nor).magnitude();
        assert!(nor_diff < 1e-2, "Vulkan NURBS normal mismatch at {}: CPU={:?}, Vulkan={:?}, diff={:e}", k, cpu_nor, vulkan_nors[k], nor_diff);
    }
}

#[cfg(feature = "cuda")]
#[test]
fn test_cuda_nurbs_tessellation_hook() {
    use truck_meshalgo::tessellation::{MeshableShape, MeshedShape};
    use truck_modeling::*;
    const PI: cgmath::Rad<f64> = cgmath::Rad(std::f64::consts::PI);

    let v: Vertex = builder::vertex(Point3::new(1.0, 0.0, 0.0));
    let circle: Wire = builder::rsweep(&v, Point3::origin(), Vector3::unit_z(), PI * 2.0, 4);
    let cylinder: Shell = builder::tsweep(&circle, Vector3::new(0.0, 0.0, 2.0));

    // Verify at least one face has Surface::NurbsSurface and as_nurbs returns Some
    let mut has_nurbs = false;
    for face in cylinder.iter() {
        if face.oriented_surface().as_nurbs().is_some() {
            has_nurbs = true;
            break;
        }
    }
    assert!(has_nurbs, "Cylinder shell should contain rational NURBS surfaces");

    let mesh = cylinder.triangulation(0.01).to_polygon();
    assert!(!mesh.positions().is_empty());
    assert!(!mesh.faces().is_empty());

    // Verify all evaluated mesh vertices lie on the cylinder x^2 + y^2 = 1.0
    for pos in mesh.positions() {
        let r2 = pos.x * pos.x + pos.y * pos.y;
        assert!((r2 - 1.0).abs() < 1e-3, "Vertex {:?} not on unit cylinder (r2 = {})", pos, r2);
        assert!(pos.z >= -1e-4 && pos.z <= 2.0 + 1e-4);
    }
}

#[cfg(feature = "cuda")]
#[test]
fn test_vulkan_vs_cuda_advanced_candidates_parity() {
    let handler = init_vulkan_device();
    let vulkan = GpuMeshAnalyzer::new(&handler);
    let cuda = CudaMeshAnalyzer::new();

    let sphere = common::shapes::sphere(Point3::origin(), 1.0, 16, 16);

    // 1. Face Normals Parity
    let vulkan_nors = vulkan.compute_face_normals(&sphere);
    let cuda_nors = cuda.compute_face_normals(&sphere);
    assert_eq!(vulkan_nors.len(), cuda_nors.len());
    for i in 0..vulkan_nors.len() {
        let diff = (vulkan_nors[i] - cuda_nors[i]).magnitude();
        assert!(diff < 1e-4, "Face normal difference between Vulkan and CUDA at #{}: Vulkan={:?}, CUDA={:?}", i, vulkan_nors[i], cuda_nors[i]);
    }

    // 2. Trimming Loop Include Parity
    let segments = vec![
        [Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)],
        [Point2::new(1.0, 0.0), Point2::new(1.0, 1.0)],
        [Point2::new(1.0, 1.0), Point2::new(0.0, 1.0)],
        [Point2::new(0.0, 1.0), Point2::new(0.0, 0.0)],
    ];
    let query_points = vec![
        Point2::new(0.5, 0.5),
        Point2::new(0.2, 0.8),
        Point2::new(1.5, 0.5),
        Point2::new(-0.5, 0.5),
        Point2::new(0.5, 1.5),
        Point2::new(0.5, -0.5),
    ];
    let vulkan_inc = vulkan.trimming_loop_include(&segments, &query_points);
    let cuda_inc = cuda.trimming_loop_include(&segments, &query_points);
    assert_eq!(vulkan_inc, cuda_inc, "Trimming loop include flags mismatch between Vulkan and CUDA!");

    // 3. Collision Parity
    let cube0 = create_unit_cube();
    let cube1 = PolygonMesh::new(
        StandardAttributes {
            positions: cube0.positions().iter().map(|&p| p + Vector3::new(0.5, 0.5, 0.5)).collect(),
            ..Default::default()
        },
        cube0.faces().clone(),
    );
    let vulkan_hit = vulkan.collide_with(&cube0, &cube1);
    let cuda_hit = cuda.collide_with(&cube0, &cube1);
    assert_eq!(vulkan_hit.is_some(), cuda_hit.is_some(), "Collision hit presence differs between Vulkan and CUDA!");
}

#[cfg(feature = "cuda")]
#[test]
fn test_cuda_and_vulkan_mass_properties_parity() {
    let handler = init_vulkan_device();
    let vulkan = GpuMeshAnalyzer::new(&handler);
    let cuda = CudaMeshAnalyzer::new();

    // 1. Unit Cube Parity
    let cube = create_unit_cube();
    let cpu_cube_vol = cube.volume();
    let cpu_cube_cog = cube.center_of_gravity().to_point();

    let (cuda_cube_vol, cuda_cube_cog) = cuda.mass_properties(&cube);
    let (vulkan_cube_vol, vulkan_cube_cog) = vulkan.mass_properties(&cube);

    assert!((cpu_cube_vol - 1.0).abs() < 1e-6, "CPU cube volume should be 1.0");
    assert!((cuda_cube_vol - cpu_cube_vol).abs() < 1e-6, "CUDA cube volume parity mismatch: CUDA={}, CPU={}", cuda_cube_vol, cpu_cube_vol);
    assert!((vulkan_cube_vol - cpu_cube_vol).abs() < 1e-4, "Vulkan cube volume parity mismatch: Vulkan={}, CPU={}", vulkan_cube_vol, cpu_cube_vol);

    let cuda_cog_diff = (cuda_cube_cog - cpu_cube_cog).magnitude();
    let vulkan_cog_diff = (vulkan_cube_cog - cpu_cube_cog).magnitude();
    assert!(cuda_cog_diff < 1e-6, "CUDA cube CoG mismatch: CUDA={:?}, CPU={:?}", cuda_cube_cog, cpu_cube_cog);
    assert!(vulkan_cog_diff < 1e-4, "Vulkan cube CoG mismatch: Vulkan={:?}, CPU={:?}", vulkan_cube_cog, cpu_cube_cog);

    // 2. Regular Trihedron in Unit Sphere
    let a = 1.0 / f64::sqrt(3.0);
    let trihedron = PolygonMesh::new(
        StandardAttributes {
            positions: vec![
                Point3::new(-a, -a, -a),
                Point3::new(a, a, -a),
                Point3::new(a, -a, a),
                Point3::new(-a, a, a),
            ],
            ..Default::default()
        },
        Faces::from_iter(&[
            [0, 1, 2],
            [1, 3, 2],
            [1, 0, 3],
            [0, 2, 3],
        ]),
    );

    let cpu_tri_vol = trihedron.volume();
    let cuda_tri_vol = cuda.volume(&trihedron);
    let vulkan_tri_vol = vulkan.volume(&trihedron);

    assert!((cuda_tri_vol - cpu_tri_vol).abs() < 1e-5, "CUDA trihedron volume mismatch: CUDA={}, CPU={}", cuda_tri_vol, cpu_tri_vol);
    assert!((vulkan_tri_vol - cpu_tri_vol).abs() < 1e-4, "Vulkan trihedron volume mismatch: Vulkan={}, CPU={}", vulkan_tri_vol, cpu_tri_vol);

    // 3. Tessellated Sphere
    let sphere = common::shapes::sphere(Point3::origin(), 1.0, 32, 32);
    let cpu_sphere_vol = sphere.volume();
    let cpu_sphere_cog = sphere.center_of_gravity().to_point();

    let cuda_sphere_vol = cuda.volume(&sphere);
    let cuda_sphere_cog = cuda.center_of_gravity(&sphere);
    let vulkan_sphere_vol = vulkan.volume(&sphere);
    let vulkan_sphere_cog = vulkan.center_of_gravity(&sphere);

    assert!((cuda_sphere_vol - cpu_sphere_vol).abs() < 1e-4, "CUDA sphere volume mismatch: CUDA={}, CPU={}", cuda_sphere_vol, cpu_sphere_vol);
    assert!((vulkan_sphere_vol - cpu_sphere_vol).abs() < 1e-4, "Vulkan sphere volume mismatch: Vulkan={}, CPU={}", vulkan_sphere_vol, cpu_sphere_vol);
    assert!((cuda_sphere_cog - cpu_sphere_cog).magnitude() < 1e-4, "CUDA sphere CoG mismatch: CUDA={:?}, CPU={:?}", cuda_sphere_cog, cpu_sphere_cog);
    assert!((vulkan_sphere_cog - cpu_sphere_cog).magnitude() < 1e-4, "Vulkan sphere CoG mismatch: Vulkan={:?}, CPU={:?}", vulkan_sphere_cog, cpu_sphere_cog);
}

#[cfg(feature = "cuda")]
#[test]
fn test_cuda_and_vulkan_planar_slicing_parity() {
    let handler = init_vulkan_device();
    let vulkan = GpuMeshAnalyzer::new(&handler);
    let cuda = CudaMeshAnalyzer::new();

    let cube = create_unit_cube();
    let planes = vec![
        [0.0, 0.0, 1.0, -0.25], // horizontal Z = 0.25
        [0.0, 0.0, 1.0, -0.50], // horizontal Z = 0.50
        [0.0, 0.0, 1.0, -0.75], // horizontal Z = 0.75
        [1.0, 1.0, 0.0, -1.0],  // diagonal X + Y = 1.0
    ];

    let cpu_slices = slice_mesh_planes_cpu(&cube, &planes);
    let cuda_slices = cuda.slice_mesh_planes(&cube, &planes);
    let vulkan_slices = vulkan.slice_mesh_planes(&cube, &planes);

    assert_eq!(cuda_slices.len(), planes.len());
    assert_eq!(vulkan_slices.len(), planes.len());

    for k in 0..planes.len() {
        assert_eq!(
            cuda_slices[k].len(),
            cpu_slices[k].len(),
            "CUDA segment count mismatch on plane #{}: CUDA={}, CPU={}",
            k,
            cuda_slices[k].len(),
            cpu_slices[k].len()
        );
        assert_eq!(
            vulkan_slices[k].len(),
            cpu_slices[k].len(),
            "Vulkan segment count mismatch on plane #{}: Vulkan={}, CPU={}",
            k,
            vulkan_slices[k].len(),
            cpu_slices[k].len()
        );

        let pl = planes[k];
        for &(p1, p2) in &cuda_slices[k] {
            let mid = Point3::new((p1.x + p2.x) * 0.5, (p1.y + p2.y) * 0.5, (p1.z + p2.z) * 0.5);
            let dist = pl[0] * mid.x + pl[1] * mid.y + pl[2] * mid.z + pl[3];
            assert!(dist.abs() < 1e-4, "CUDA segment midpoint should lie on plane #{}: dist={}", k, dist);
        }
    }

    // Test loop chaining on Z = 0.5 slice
    let z50_segments = &cuda_slices[1];
    let loops = chain_segments_into_loops(z50_segments, 1e-4);
    assert_eq!(loops.len(), 1, "Cube horizontal slice should yield exactly 1 closed polygon loop");
    let loop0 = &loops[0];
    assert!(loop0.len() >= 4, "Square loop should have at least 4 vertices, got {}", loop0.len());

    // Verify all vertices lie at Z = 0.5
    for pt in loop0 {
        assert!((pt.z - 0.5).abs() < 1e-4, "Z-coordinate in loop should be 0.5, got {}", pt.z);
    }

    // Verify closed loop perimeter is ~4.0 for a 1x1 square
    let mut perimeter = 0.0;
    for i in 0..loop0.len() {
        let p_curr = loop0[i];
        let p_next = loop0[(i + 1) % loop0.len()];
        perimeter += (p_next - p_curr).magnitude();
    }
    assert!((perimeter - 4.0).abs() < 1e-3, "Perimeter of square slice should be ~4.0, got {}", perimeter);
}

#[cfg(feature = "cuda")]
#[test]
fn test_cuda_and_vulkan_point_cloud_conformance_parity() {
    let handler = init_vulkan_device();
    let vulkan = GpuMeshAnalyzer::new(&handler);
    let cuda = CudaMeshAnalyzer::new();

    // 1. Concentric Spheres: R = 10.0 and R = 11.0
    let sphere0 = common::shapes::sphere(Point3::origin(), 10.0, 30, 30);
    let sphere1 = common::shapes::sphere(Point3::origin(), 11.0, 30, 30);

    // With tol = 1.01 (> distance of 1.0): both should be true across CPU, CUDA, and Vulkan
    let cpu_clung_101 = sphere0.is_clung_to_by(sphere1.positions(), 1.01);
    let cuda_clung_101 = cuda.is_clung_to_by(&sphere0, sphere1.positions(), 1.01);
    let vulkan_clung_101 = vulkan.is_clung_to_by(&sphere0, sphere1.positions(), 1.01);

    assert!(cpu_clung_101, "CPU is_clung_to_by should be true for tol=1.01");
    assert_eq!(cuda_clung_101, cpu_clung_101, "CUDA is_clung_to_by parity mismatch at tol=1.01");
    assert_eq!(vulkan_clung_101, cpu_clung_101, "Vulkan is_clung_to_by parity mismatch at tol=1.01");

    // With tol = 0.99 (< distance of 1.0): both should be false across CPU, CUDA, and Vulkan
    let cpu_clung_099 = sphere0.is_clung_to_by(sphere1.positions(), 0.99);
    let cuda_clung_099 = cuda.is_clung_to_by(&sphere0, sphere1.positions(), 0.99);
    let vulkan_clung_099 = vulkan.is_clung_to_by(&sphere0, sphere1.positions(), 0.99);

    assert!(!cpu_clung_099, "CPU is_clung_to_by should be false for tol=0.99");
    assert_eq!(cuda_clung_099, cpu_clung_099, "CUDA is_clung_to_by parity mismatch at tol=0.99");
    assert_eq!(vulkan_clung_099, cpu_clung_099, "Vulkan is_clung_to_by parity mismatch at tol=0.99");

    // 2. Unit Cube Neighborhood Inclusion
    let cube = create_unit_cube();
    let inside_pts = vec![
        Point3::new(0.5, 0.5, 0.5),
        Point3::new(0.1, 0.1, 0.1),
        Point3::new(0.9, 0.9, 0.9),
    ];
    let outside_pts = vec![
        Point3::new(2.0, 2.0, 2.0),
        Point3::new(-1.0, 0.5, 0.5),
    ];

    let cpu_inc_inside = cube.neighborhood_include(&inside_pts, 0.1);
    let cuda_inc_inside = cuda.neighborhood_include(&cube, &inside_pts, 0.1);
    let vulkan_inc_inside = vulkan.neighborhood_include(&cube, &inside_pts, 0.1);

    assert_eq!(cuda_inc_inside, cpu_inc_inside, "CUDA neighborhood_include parity mismatch for inside points");
    assert_eq!(vulkan_inc_inside, cpu_inc_inside, "Vulkan neighborhood_include parity mismatch for inside points");

    let cpu_inc_outside = cube.neighborhood_include(&outside_pts, 0.1);
    let cuda_inc_outside = cuda.neighborhood_include(&cube, &outside_pts, 0.1);
    let vulkan_inc_outside = vulkan.neighborhood_include(&cube, &outside_pts, 0.1);

    assert_eq!(cuda_inc_outside, cpu_inc_outside, "CUDA neighborhood_include parity mismatch for outside points");
    assert_eq!(vulkan_inc_outside, cpu_inc_outside, "Vulkan neighborhood_include parity mismatch for outside points");
}

#[cfg(feature = "cuda")]
#[test]
fn test_cuda_and_vulkan_surface_intersection_seeds_parity() {
    use truck_geometry::prelude::*;

    let handler = init_vulkan_device();
    let vulkan = GpuMeshAnalyzer::new(&handler);
    let cuda = CudaMeshAnalyzer::new();

    let u_knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let v_knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];

    // Surface 1: Paraboloid with depression intersecting z = 0
    let ctrl_pts1 = vec![
        vec![Point3::new(-1.0, -1.0, 1.0), Point3::new(0.0, -1.0, 0.0), Point3::new(1.0, -1.0, 1.0)],
        vec![Point3::new(-1.0,  0.0, 0.0), Point3::new(0.0,  0.0, -1.0), Point3::new(1.0,  0.0, 0.0)],
        vec![Point3::new(-1.0,  1.0, 1.0), Point3::new(0.0,  1.0, 0.0), Point3::new(1.0,  1.0, 1.0)],
    ];
    let s1 = BSplineSurface::new(
        (KnotVec::from(u_knots.clone()), KnotVec::from(v_knots.clone())),
        ctrl_pts1,
    );

    // Surface 2: Flat plane at z = 0
    let ctrl_pts2 = vec![
        vec![Point3::new(-1.0, -1.0, 0.0), Point3::new(0.0, -1.0, 0.0), Point3::new(1.0, -1.0, 0.0)],
        vec![Point3::new(-1.0,  0.0, 0.0), Point3::new(0.0,  0.0, 0.0), Point3::new(1.0,  0.0, 0.0)],
        vec![Point3::new(-1.0,  1.0, 0.0), Point3::new(0.0,  1.0, 0.0), Point3::new(1.0,  1.0, 0.0)],
    ];
    let s2 = BSplineSurface::new(
        (KnotVec::from(u_knots.clone()), KnotVec::from(v_knots.clone())),
        ctrl_pts2,
    );

    let tol = 0.05;
    let cuda_seeds = cuda.find_surface_intersection_seeds(&s1, &s2, tol);
    let vulkan_seeds = vulkan.find_surface_intersection_seeds(&s1, &s2, tol);
    let dispatch_seeds = truck_meshalgo::dispatch::Dispatcher::global().find_surface_intersection_seeds(&s1, &s2, tol);

    assert!(!cuda_seeds.is_empty(), "CUDA should detect candidate intersection seeds");
    assert!(!vulkan_seeds.is_empty(), "Vulkan should detect candidate intersection seeds");
    assert!(!dispatch_seeds.is_empty(), "Dispatcher should detect candidate intersection seeds");

    // Verify geometric accuracy: distance between surfaces at each seed is small
    for &(p1, p2) in &cuda_seeds {
        let pt1 = s1.subs(p1.x, p1.y);
        let pt2 = s2.subs(p2.x, p2.y);
        let dist = pt1.distance(pt2);
        assert!(
            dist < 0.25,
            "CUDA seed distance between surfaces should be within grid resolution bound, got {}",
            dist
        );
    }

    for &(p1, p2) in &vulkan_seeds {
        let pt1 = s1.subs(p1.x, p1.y);
        let pt2 = s2.subs(p2.x, p2.y);
        let dist = pt1.distance(pt2);
        assert!(
            dist < 0.25,
            "Vulkan seed distance between surfaces should be within grid resolution bound, got {}",
            dist
        );
    }
}

#[cfg(feature = "cuda")]
#[test]
fn test_cuda_and_vulkan_extract_planes_parity() {
    let handler = init_vulkan_device();
    let vulkan = GpuMeshAnalyzer::new(&handler);
    let cuda = CudaMeshAnalyzer::new();

    // 1. Canonical mixed tri/quad mesh from Splitting unit test
    let mesh = PolygonMesh::new(
        StandardAttributes {
            positions: vec![
                Point3::new(-5.0, 0.0, -2.0),
                Point3::new(-5.0, 0.0, 0.0),
                Point3::new(0.0, 2.0, -2.0),
                Point3::new(0.0, 2.0, 0.0),
                Point3::new(0.0, 2.0, 2.0),
                Point3::new(5.0, 0.0, 0.0),
                Point3::new(5.0, 0.0, 2.0),
            ],
            normals: vec![
                Vector3::new(-2.0, 5.0, 0.0).normalize(),
                Vector3::new(0.0, 1.0, 0.0),
                Vector3::new(2.0, 5.0, 0.0).normalize(),
            ],
            ..Default::default()
        },
        Faces::from_iter(&[
            [
                (0, None, Some(0)),
                (1, None, Some(0)),
                (3, None, Some(1)),
                (2, None, Some(1)),
            ]
            .as_ref(),
            &[(1, None, Some(1)), (4, None, Some(1)), (3, None, Some(1))],
            &[(2, None, Some(1)), (3, None, Some(1)), (5, None, Some(2))],
            &[
                (3, None, Some(1)),
                (4, None, Some(1)),
                (6, None, Some(1)),
                (5, None, Some(1)),
            ],
        ]),
    );

    let tol = 1.0e-7;
    let (cuda_planes, cuda_others) = cuda.extract_planes(&mesh, tol);
    let (vulkan_planes, vulkan_others) = vulkan.extract_planes(&mesh, tol);
    let (dispatch_planes, dispatch_others) = truck_meshalgo::dispatch::Dispatcher::global().extract_planes(&mesh, tol);
    let (trait_planes, trait_others) = mesh.extract_planes(tol);

    assert_eq!(cuda_planes, vec![1, 2], "CUDA extract_planes planar indices parity mismatch");
    assert_eq!(cuda_others, vec![0, 3], "CUDA extract_planes remainder indices parity mismatch");

    assert_eq!(vulkan_planes, vec![1, 2], "Vulkan extract_planes planar indices parity mismatch");
    assert_eq!(vulkan_others, vec![0, 3], "Vulkan extract_planes remainder indices parity mismatch");

    assert_eq!(dispatch_planes, vec![1, 2], "Dispatcher extract_planes planar indices mismatch");
    assert_eq!(dispatch_others, vec![0, 3], "Dispatcher extract_planes remainder indices mismatch");

    assert_eq!(trait_planes, vec![1, 2], "Splitting::extract_planes dispatch mismatch");
    assert_eq!(trait_others, vec![0, 3], "Splitting::extract_planes dispatch mismatch");

    // 2. Larger procedural sphere mesh to verify scaling parity and warp-aggregated atomics
    let u_div = 32;
    let v_div = 16;
    let mut sphere = common::shapes::sphere(Point3::origin(), 1.0, u_div, v_div);
    sphere.add_naive_normals(true);

    let start_cpu = std::time::Instant::now();
    truck_meshalgo::dispatch::Dispatcher::global().set_override(Some(truck_meshalgo::dispatch::KernelTarget::CpuSingle));
    let (cpu_sp_planes, cpu_sp_others) = sphere.extract_planes(0.05);
    truck_meshalgo::dispatch::Dispatcher::global().reset_override();
    let dur_cpu = start_cpu.elapsed();

    let start_cuda = std::time::Instant::now();
    let (cuda_sp_planes, cuda_sp_others) = cuda.extract_planes(&sphere, 0.05);
    let dur_cuda = start_cuda.elapsed();

    let start_vulkan = std::time::Instant::now();
    let (vulkan_sp_planes, vulkan_sp_others) = vulkan.extract_planes(&sphere, 0.05);
    let dur_vulkan = start_vulkan.elapsed();

    println!(
        "extract_planes on sphere ({} faces): CPU={:?}, CUDA={:?}, Vulkan={:?}",
        sphere.faces().len(),
        dur_cpu,
        dur_cuda,
        dur_vulkan
    );

    assert_eq!(cuda_sp_planes, cpu_sp_planes, "CUDA sphere extract_planes planar parity mismatch");
    assert_eq!(cuda_sp_others, cpu_sp_others, "CUDA sphere extract_planes other parity mismatch");
    assert_eq!(vulkan_sp_planes, cpu_sp_planes, "Vulkan sphere extract_planes planar parity mismatch");
    assert_eq!(vulkan_sp_others, cpu_sp_others, "Vulkan sphere extract_planes other parity mismatch");
}

#[cfg(feature = "cuda")]
#[test]
fn test_cuda_and_vulkan_linear_subdivision_parity() {
    let handler = init_vulkan_device();
    let vulkan = GpuMeshAnalyzer::new(&handler);
    let cuda = CudaMeshAnalyzer::new();

    let cube = create_unit_cube();
    let orig_tri_count = GpuMeshAnalyzer::extract_triangles(&cube).len();
    assert_eq!(orig_tri_count, 12);

    let start_cuda = std::time::Instant::now();
    let cuda_sub = cuda.linear_subdivide(&cube);
    let dur_cuda = start_cuda.elapsed();

    let start_vulkan = std::time::Instant::now();
    let vulkan_sub = vulkan.linear_subdivide(&cube);
    let dur_vulkan = start_vulkan.elapsed();

    let start_dispatch = std::time::Instant::now();
    let dispatch_sub = truck_meshalgo::dispatch::Dispatcher::global().linear_subdivide(&cube);
    let dur_dispatch = start_dispatch.elapsed();

    let trait_sub = cube.linear_subdivide();

    println!(
        "linear_subdivide 12 triangles -> 48 triangles: CUDA={:?}, Vulkan={:?}, Dispatcher={:?}",
        dur_cuda,
        dur_vulkan,
        dur_dispatch
    );

    assert_eq!(cuda_sub.tri_faces().len(), orig_tri_count * 4);
    assert_eq!(vulkan_sub.tri_faces().len(), orig_tri_count * 4);
    assert_eq!(dispatch_sub.tri_faces().len(), orig_tri_count * 4);
    assert_eq!(trait_sub.tri_faces().len(), orig_tri_count * 4);

    // Verify geometric accuracy: volume of unit cube must remain 1.0
    let (cuda_vol, cuda_cog) = cuda.mass_properties(&cuda_sub);
    let (vulkan_vol, vulkan_cog) = vulkan.mass_properties(&vulkan_sub);
    assert!((cuda_vol - 1.0).abs() < 1e-4, "Subdivided cube volume should be 1.0, got {}", cuda_vol);
    assert!((vulkan_vol - 1.0).abs() < 1e-4, "Subdivided cube volume should be 1.0, got {}", vulkan_vol);
    assert!((cuda_cog.x - 0.5).abs() < 1e-4 && (cuda_cog.y - 0.5).abs() < 1e-4 && (cuda_cog.z - 0.5).abs() < 1e-4);
    assert!((vulkan_cog.x - 0.5).abs() < 1e-4 && (vulkan_cog.y - 0.5).abs() < 1e-4 && (vulkan_cog.z - 0.5).abs() < 1e-4);

    // Test subdivide_triangles directly
    let tris = GpuMeshAnalyzer::extract_triangles(&cube);
    let cuda_tris = cuda.subdivide_triangles(&tris);
    let vulkan_tris = vulkan.subdivide_triangles(&tris);
    let dispatch_tris = truck_meshalgo::dispatch::Dispatcher::global().subdivide_triangles(&tris);

    assert_eq!(cuda_tris.len(), tris.len() * 4);
    assert_eq!(vulkan_tris.len(), tris.len() * 4);
    assert_eq!(dispatch_tris.len(), tris.len() * 4);

    for i in 0..cuda_tris.len() {
        for v in 0..3 {
            let p_cuda = match v { 0 => cuda_tris[i].v0, 1 => cuda_tris[i].v1, _ => cuda_tris[i].v2 };
            let p_vulk = match v { 0 => vulkan_tris[i].v0, 1 => vulkan_tris[i].v1, _ => vulkan_tris[i].v2 };
            let p_disp = match v { 0 => dispatch_tris[i].v0, 1 => dispatch_tris[i].v1, _ => dispatch_tris[i].v2 };
            for c in 0..3 {
                assert!((p_cuda[c] - p_vulk[c]).abs() < 1e-5, "CUDA vs Vulkan triangle vertex parity mismatch at tri {} vert {}", i, v);
                assert!((p_cuda[c] - p_disp[c]).abs() < 1e-5, "CUDA vs Dispatcher triangle vertex parity mismatch at tri {} vert {}", i, v);
            }
        }
    }
}

#[cfg(feature = "cuda")]
#[test]
fn test_cuda_and_vulkan_nurbs_curve_parity() {
    use std::f64::consts::FRAC_1_SQRT_2;
    use truck_geometry::prelude::*;

    let handler = init_vulkan_device();
    let vulkan = GpuMeshAnalyzer::new(&handler);
    let cuda = CudaMeshAnalyzer::new();

    // Construct circular arc (unit 90-degree quadrant in XY plane)
    // Degree 2 NURBS curve with knot vector [0, 0, 0, 1, 1, 1]
    let knot_vec = KnotVec::bezier_knot(2);
    let w = FRAC_1_SQRT_2;
    let control_points = vec![
        Vector4::new(1.0, 0.0, 0.0, 1.0),
        Vector4::new(w, w, 0.0, w),
        Vector4::new(0.0, 1.0, 0.0, 1.0),
    ];

    let bsp = BSplineCurve::new(knot_vec, control_points);
    let nurbs_curve = NurbsCurve::new(bsp);

    // Test as_nurbs_curve extraction via ParametricCurve trait
    let (knots, degree, cps) = nurbs_curve.as_nurbs_curve().expect("as_nurbs_curve should succeed");
    assert_eq!(degree, 2);
    assert_eq!(knots.len(), 6);
    assert_eq!(cps.len(), 3);

    // Generate batch evaluation points t in [0.0, 1.0]
    let n_eval = 200;
    let t_points: Vec<f64> = (0..=n_eval)
        .map(|i| i as f64 / n_eval as f64)
        .collect();

    // CPU ground truth
    let mut cpu_pts = Vec::with_capacity(t_points.len());
    let mut cpu_ders = Vec::with_capacity(t_points.len());
    for &t in &t_points {
        cpu_pts.push(nurbs_curve.subs(t));
        cpu_ders.push(nurbs_curve.der(t));
    }

    // CUDA evaluation (FP64 bare-metal kernel)
    let start_cuda = std::time::Instant::now();
    let (cuda_pts, cuda_ders) = cuda.evaluate_nurbs_curve(knots, degree, cps, &t_points);
    let dur_cuda = start_cuda.elapsed();

    // Vulkan evaluation (FP32 wgpu compute shader)
    let start_vulkan = std::time::Instant::now();
    let (vulkan_pts, vulkan_ders) = vulkan.evaluate_nurbs_curve(knots, degree, cps, &t_points);
    let dur_vulkan = start_vulkan.elapsed();

    // Dispatcher evaluation
    let start_dispatch = std::time::Instant::now();
    let (dispatch_pts, dispatch_ders) =
        truck_meshalgo::dispatch::Dispatcher::global().evaluate_nurbs_curve(knots, degree, cps, &t_points);
    let dur_dispatch = start_dispatch.elapsed();

    println!(
        "evaluate_nurbs_curve {} samples: CUDA={:?}, Vulkan={:?}, Dispatcher={:?}",
        t_points.len(),
        dur_cuda,
        dur_vulkan,
        dur_dispatch
    );

    assert_eq!(cuda_pts.len(), t_points.len());
    assert_eq!(vulkan_pts.len(), t_points.len());
    assert_eq!(dispatch_pts.len(), t_points.len());

    for i in 0..t_points.len() {
        let p_cpu = cpu_pts[i];
        let d_cpu = cpu_ders[i];

        let p_cuda = cuda_pts[i];
        let d_cuda = cuda_ders[i];

        let p_vulk = vulkan_pts[i];
        let d_vulk = vulkan_ders[i];

        let p_disp = dispatch_pts[i];
        let d_disp = dispatch_ders[i];

        // CUDA FP64 parity against CPU ground truth: <= 1e-10
        assert!(
            p_cuda.distance(p_cpu) < 1e-10,
            "CUDA vs CPU point mismatch at sample {}: {:?} vs {:?}",
            i, p_cuda, p_cpu
        );
        assert!(
            (d_cuda - d_cpu).magnitude() < 1e-9,
            "CUDA vs CPU derivative mismatch at sample {}: {:?} vs {:?}",
            i, d_cuda, d_cpu
        );

        // Vulkan FP32 parity against CPU ground truth: <= 1e-4
        assert!(
            p_vulk.distance(p_cpu) < 1e-4,
            "Vulkan vs CPU point mismatch at sample {}: {:?} vs {:?}",
            i, p_vulk, p_cpu
        );
        assert!(
            (d_vulk - d_cpu).magnitude() < 1e-3,
            "Vulkan vs CPU derivative mismatch at sample {}: {:?} vs {:?}",
            i, d_vulk, d_cpu
        );

        // Dispatcher match
        assert!(
            p_disp.distance(p_cpu) < 1e-4,
            "Dispatcher vs CPU point mismatch at sample {}: {:?} vs {:?}",
            i, p_disp, p_cpu
        );
        assert!(
            (d_disp - d_cpu).magnitude() < 1e-3,
            "Dispatcher vs CPU derivative mismatch at sample {}: {:?} vs {:?}",
            i, d_disp, d_cpu
        );

        // Analytical geometric correctness: every point must lie on the unit circle x^2 + y^2 = 1.0, z = 0.0
        let radius = (p_cuda.x * p_cuda.x + p_cuda.y * p_cuda.y).sqrt();
        assert!((radius - 1.0).abs() < 1e-10, "Point must lie on unit circle, got radius {}", radius);
        assert!(p_cuda.z.abs() < 1e-10, "Z coordinate must be 0, got {}", p_cuda.z);

        // Tangent vector must be perpendicular to radius vector (dot product = 0)
        let dot_prod = (p_cuda - Point3::origin()).dot(d_cuda);
        assert!(dot_prod.abs() < 1e-9, "Circle tangent must be perpendicular to position, dot={}", dot_prod);
    }
}

#[cfg(feature = "cuda")]
#[test]
fn test_cuda_and_vulkan_raycast_parity() {
    use truck_meshalgo::analyzers::{raycast_mesh_cpu, raycast_mesh_rayon, RaycastMesh};

    let handler = init_vulkan_device();
    let vulkan = GpuMeshAnalyzer::new(&handler);
    let cuda = CudaMeshAnalyzer::new();

    let cube = create_unit_cube();

    // Construct deterministic rays targeting various faces and misses
    let mut rays = vec![
        // Front face: through z=0 towards +z
        (Point3::new(0.5, 0.5, -2.0), Vector3::new(0.0, 0.0, 1.0)),
        // Back face: through z=1 towards -z
        (Point3::new(0.5, 0.5, 3.0), Vector3::new(0.0, 0.0, -1.0)),
        // Left face: through x=0 towards +x
        (Point3::new(-2.0, 0.5, 0.5), Vector3::new(1.0, 0.0, 0.0)),
        // Right face: through x=1 towards -x
        (Point3::new(3.0, 0.5, 0.5), Vector3::new(-1.0, 0.0, 0.0)),
        // Bottom face: through y=0 towards +y
        (Point3::new(0.5, -2.0, 0.5), Vector3::new(0.0, 1.0, 0.0)),
        // Top face: through y=1 towards -y
        (Point3::new(0.5, 3.0, 0.5), Vector3::new(0.0, -1.0, 0.0)),
        // Definite miss (aimed away)
        (Point3::new(5.0, 5.0, 5.0), Vector3::new(1.0, 2.0, 3.0)),
        // Definite miss (parallel to cube face outside bounds)
        (Point3::new(2.0, 2.0, -1.0), Vector3::new(0.0, 0.0, 1.0)),
    ];

    // Add extra rays around the unit sphere aimed at origin
    for i in 0..100 {
        let theta = (i as f64) * 0.314;
        let phi = (i as f64) * 0.173;
        let origin = Point3::new(3.0 * theta.cos() * phi.sin(), 3.0 * theta.sin() * phi.sin(), 3.0 * phi.cos());
        let dir = Point3::new(0.5, 0.5, 0.5) - origin;
        rays.push((origin, dir));
    }

    // CPU single-thread ground truth
    let cpu_results = raycast_mesh_cpu(&cube, &rays);
    // Rayon CPU ground truth
    let rayon_results = raycast_mesh_rayon(&cube, &rays);

    // CUDA
    let start_cuda = std::time::Instant::now();
    let cuda_results = cuda.raycast_mesh(&cube, &rays);
    let dur_cuda = start_cuda.elapsed();

    // Vulkan
    let start_vulkan = std::time::Instant::now();
    let vulkan_results = vulkan.raycast_mesh(&cube, &rays);
    let dur_vulkan = start_vulkan.elapsed();

    // Dispatcher / Trait method
    let start_dispatch = std::time::Instant::now();
    let dispatch_results = cube.raycast(&rays);
    let dur_dispatch = start_dispatch.elapsed();

    println!(
        "raycast_mesh {} rays: CUDA={:?}, Vulkan={:?}, Dispatcher={:?}",
        rays.len(),
        dur_cuda,
        dur_vulkan,
        dur_dispatch
    );

    assert_eq!(cpu_results.len(), rays.len());
    assert_eq!(rayon_results.len(), rays.len());
    assert_eq!(cuda_results.len(), rays.len());
    assert_eq!(vulkan_results.len(), rays.len());
    assert_eq!(dispatch_results.len(), rays.len());

    let mut hit_count = 0;
    let mut miss_count = 0;

    for i in 0..rays.len() {
        let cpu_hit = &cpu_results[i];
        let rayon_hit = &rayon_results[i];
        let cuda_hit = &cuda_results[i];
        let vulkan_hit = &vulkan_results[i];
        let dispatch_hit = &dispatch_results[i];

        // Rayon parity with single-threaded CPU
        assert_eq!(cpu_hit.is_some(), rayon_hit.is_some(), "Rayon hit/miss mismatch at ray {}", i);

        // CUDA parity with CPU
        assert_eq!(
            cpu_hit.is_some(),
            cuda_hit.is_some(),
            "CUDA hit/miss mismatch at ray {}: CPU={:?}, CUDA={:?}",
            i, cpu_hit.is_some(), cuda_hit.is_some()
        );

        // Vulkan parity with CPU
        assert_eq!(
            cpu_hit.is_some(),
            vulkan_hit.is_some(),
            "Vulkan hit/miss mismatch at ray {}: CPU={:?}, Vulkan={:?}",
            i, cpu_hit.is_some(), vulkan_hit.is_some()
        );

        // Dispatch parity with CPU
        assert_eq!(
            cpu_hit.is_some(),
            dispatch_hit.is_some(),
            "Dispatcher hit/miss mismatch at ray {}: CPU={:?}, Dispatcher={:?}",
            i, cpu_hit.is_some(), dispatch_hit.is_some()
        );

        if let Some((cpu_t, cpu_p, cpu_n)) = cpu_hit {
            hit_count += 1;
            let (cuda_t, cuda_p, cuda_n) = cuda_hit.as_ref().unwrap();
            let (vulkan_t, vulkan_p, vulkan_n) = vulkan_hit.as_ref().unwrap();
            let (disp_t, disp_p, disp_n) = dispatch_hit.as_ref().unwrap();

            // Distance match
            assert!((cuda_t - cpu_t).abs() < 1e-4, "CUDA vs CPU t mismatch at ray {}: {} vs {}", i, cuda_t, cpu_t);
            assert!((vulkan_t - cpu_t).abs() < 1e-3, "Vulkan vs CPU t mismatch at ray {}: {} vs {}", i, vulkan_t, cpu_t);
            assert!((disp_t - cpu_t).abs() < 1e-3, "Dispatcher vs CPU t mismatch at ray {}: {} vs {}", i, disp_t, cpu_t);

            // Hit point match
            assert!(cuda_p.distance(*cpu_p) < 1e-4, "CUDA vs CPU hit point mismatch at ray {}", i);
            assert!(vulkan_p.distance(*cpu_p) < 1e-3, "Vulkan vs CPU hit point mismatch at ray {}", i);
            assert!(disp_p.distance(*cpu_p) < 1e-3, "Dispatcher vs CPU hit point mismatch at ray {}", i);

            // Normal match (should point outwards and dot product with ray direction should be <= 0)
            assert!((cuda_n - cpu_n).magnitude() < 1e-3, "CUDA vs CPU normal mismatch at ray {}: {:?} vs {:?}", i, cuda_n, cpu_n);
            assert!((vulkan_n - cpu_n).magnitude() < 1e-2, "Vulkan vs CPU normal mismatch at ray {}: {:?} vs {:?}", i, vulkan_n, cpu_n);
            assert!((disp_n - cpu_n).magnitude() < 1e-2, "Dispatcher vs CPU normal mismatch at ray {}: {:?} vs {:?}", i, disp_n, cpu_n);
        } else {
            miss_count += 1;
        }
    }

    assert!(hit_count > 0, "Should have hits");
    assert!(miss_count > 0, "Should have misses");
    println!("Raycast parity verified across {} rays: {} hits, {} misses.", rays.len(), hit_count, miss_count);
}

#[cfg(feature = "cuda")]
fn create_octahedron() -> PolygonMesh {
    PolygonMesh::new(
        StandardAttributes {
            positions: vec![
                Point3::new(1.0, 0.0, 0.0),
                Point3::new(-1.0, 0.0, 0.0),
                Point3::new(0.0, 1.0, 0.0),
                Point3::new(0.0, -1.0, 0.0),
                Point3::new(0.0, 0.0, 1.0),
                Point3::new(0.0, 0.0, -1.0),
            ],
            ..Default::default()
        },
        Faces::from_iter(vec![
            [0, 2, 4],
            [2, 1, 4],
            [1, 3, 4],
            [3, 0, 4],
            [2, 0, 5],
            [1, 2, 5],
            [3, 1, 5],
            [0, 3, 5],
        ]),
    )
}

#[cfg(feature = "cuda")]
fn create_open_quad_mesh() -> PolygonMesh {
    PolygonMesh::new(
        StandardAttributes {
            positions: vec![
                Point3::new(-1.0, 0.0, 0.0),
                Point3::new(0.0, 1.0, 1.0),
                Point3::new(0.0, 1.0, -1.0),
                Point3::new(1.0, 0.0, 0.0),
            ],
            ..Default::default()
        },
        Faces::from_iter(vec![
            [0, 1, 2],
            [3, 2, 1],
        ]),
    )
}

#[cfg(feature = "cuda")]
fn canonicalize_segments(mut segs: Vec<(Point3, Point3)>) -> Vec<(Point3, Point3)> {
    fn pt_key(p: Point3) -> (i64, i64, i64) {
        (
            (p.x * 1000.0).round() as i64,
            (p.y * 1000.0).round() as i64,
            (p.z * 1000.0).round() as i64,
        )
    }
    for seg in &mut segs {
        if pt_key(seg.0) > pt_key(seg.1) {
            std::mem::swap(&mut seg.0, &mut seg.1);
        }
    }
    segs.sort_by(|a, b| {
        pt_key(a.0)
            .cmp(&pt_key(b.0))
            .then_with(|| pt_key(a.1).cmp(&pt_key(b.1)))
    });
    segs
}

#[cfg(feature = "cuda")]
#[test]
fn test_cuda_and_vulkan_loop_subdivision_parity() {
    let handler = init_vulkan_device();
    let vulkan = GpuMeshAnalyzer::new(&handler);
    let cuda = CudaMeshAnalyzer::new();
    let dispatcher = Dispatcher::global();

    // 1. Closed Octahedron (6 vertices, 8 faces, 12 edges)
    let octa = create_octahedron();
    let cpu_octa = dispatcher.loop_subdivide(&octa);
    let cuda_octa = cuda.loop_subdivide(&octa);
    let vulk_octa = vulkan.loop_subdivide(&octa);
    let disp_octa = octa.loop_subdivide();

    assert_eq!(cpu_octa.positions().len(), 6 + 12);
    assert_eq!(cpu_octa.tri_faces().len(), 8 * 4);
    assert_eq!(cuda_octa.positions().len(), cpu_octa.positions().len());
    assert_eq!(vulk_octa.positions().len(), cpu_octa.positions().len());
    assert_eq!(disp_octa.positions().len(), cpu_octa.positions().len());
    assert_eq!(cuda_octa.tri_faces().len(), cpu_octa.tri_faces().len());
    assert_eq!(vulk_octa.tri_faces().len(), cpu_octa.tri_faces().len());
    assert_eq!(disp_octa.tri_faces().len(), cpu_octa.tri_faces().len());

    for i in 0..cpu_octa.positions().len() {
        let p_cpu = cpu_octa.positions()[i];
        let p_cuda = cuda_octa.positions()[i];
        let p_vulk = vulk_octa.positions()[i];
        let p_disp = disp_octa.positions()[i];

        assert!(
            p_cuda.distance(p_cpu) < 1e-10,
            "CUDA vs CPU octahedron pos mismatch at {}: {:?} vs {:?}",
            i, p_cuda, p_cpu
        );
        assert!(
            p_vulk.distance(p_cpu) < 1e-4,
            "Vulkan vs CPU octahedron pos mismatch at {}: {:?} vs {:?}",
            i, p_vulk, p_cpu
        );
        assert!(
            p_disp.distance(p_cpu) < 1e-4,
            "Dispatcher vs CPU octahedron pos mismatch at {}: {:?} vs {:?}",
            i, p_disp, p_cpu
        );
    }
    assert_eq!(cuda_octa.tri_faces(), cpu_octa.tri_faces());
    assert_eq!(vulk_octa.tri_faces(), cpu_octa.tri_faces());
    assert_eq!(disp_octa.tri_faces(), cpu_octa.tri_faces());

    // 2. Closed Triangulated Cube (8 vertices, 12 faces, 18 edges)
    let mut cube = create_unit_cube();
    cube.triangulate();
    let cpu_cube = dispatcher.loop_subdivide(&cube);
    let cuda_cube = cuda.loop_subdivide(&cube);
    let vulk_cube = vulkan.loop_subdivide(&cube);
    let disp_cube = cube.loop_subdivide();

    assert_eq!(cpu_cube.positions().len(), 8 + 18);
    assert_eq!(cpu_cube.tri_faces().len(), 12 * 4);
    assert_eq!(cuda_cube.positions().len(), cpu_cube.positions().len());
    assert_eq!(vulk_cube.positions().len(), cpu_cube.positions().len());
    assert_eq!(disp_cube.positions().len(), cpu_cube.positions().len());

    for i in 0..cpu_cube.positions().len() {
        let p_cpu = cpu_cube.positions()[i];
        let p_cuda = cuda_cube.positions()[i];
        let p_vulk = vulk_cube.positions()[i];
        let p_disp = disp_cube.positions()[i];

        assert!(
            p_cuda.distance(p_cpu) < 1e-10,
            "CUDA vs CPU cube pos mismatch at {}: {:?} vs {:?}",
            i, p_cuda, p_cpu
        );
        assert!(
            p_vulk.distance(p_cpu) < 1e-4,
            "Vulkan vs CPU cube pos mismatch at {}: {:?} vs {:?}",
            i, p_vulk, p_cpu
        );
        assert!(
            p_disp.distance(p_cpu) < 1e-4,
            "Dispatcher vs CPU cube pos mismatch at {}: {:?} vs {:?}",
            i, p_disp, p_cpu
        );
    }

    // 3. Open Quad (4 vertices, 2 faces, 5 edges: 4 boundary, 1 interior)
    let quad = create_open_quad_mesh();
    let cpu_quad = dispatcher.loop_subdivide(&quad);
    let cuda_quad = cuda.loop_subdivide(&quad);
    let vulk_quad = vulkan.loop_subdivide(&quad);
    let disp_quad = quad.loop_subdivide();

    assert_eq!(cpu_quad.positions().len(), 4 + 5);
    assert_eq!(cpu_quad.tri_faces().len(), 2 * 4);
    assert_eq!(cuda_quad.positions().len(), 9);
    assert_eq!(vulk_quad.positions().len(), 9);
    assert_eq!(disp_quad.positions().len(), 9);

    for i in 0..cpu_quad.positions().len() {
        let p_cpu = cpu_quad.positions()[i];
        let p_cuda = cuda_quad.positions()[i];
        let p_vulk = vulk_quad.positions()[i];
        let p_disp = disp_quad.positions()[i];

        assert!(
            p_cuda.distance(p_cpu) < 1e-10,
            "CUDA vs CPU open quad pos mismatch at {}: {:?} vs {:?}",
            i, p_cuda, p_cpu
        );
        assert!(
            p_vulk.distance(p_cpu) < 1e-4,
            "Vulkan vs CPU open quad pos mismatch at {}: {:?} vs {:?}",
            i, p_vulk, p_cpu
        );
        assert!(
            p_disp.distance(p_cpu) < 1e-4,
            "Dispatcher vs CPU open quad pos mismatch at {}: {:?} vs {:?}",
            i, p_disp, p_cpu
        );
    }

    // In-place subdivision parity
    let mut in_place_quad = quad.clone();
    in_place_quad.loop_subdivision();
    assert_eq!(in_place_quad.positions().len(), 9);
    for i in 0..9 {
        assert!(
            in_place_quad.positions()[i].distance(cpu_quad.positions()[i]) < 1e-4,
            "In-place loop_subdivision mismatch at {}: {:?} vs {:?}",
            i, in_place_quad.positions()[i], cpu_quad.positions()[i]
        );
    }

    println!("Loop subdivision parity verified across octahedron, cube, and open quad.");
}

#[cfg(feature = "cuda")]
#[test]
fn test_cuda_and_vulkan_feature_and_boundary_edges_parity() {
    let handler = init_vulkan_device();
    let vulkan = GpuMeshAnalyzer::new(&handler);
    let cuda = CudaMeshAnalyzer::new();
    let dispatcher = Dispatcher::global();

    // 1. Closed Triangulated Cube
    let mut cube = create_unit_cube();
    cube.triangulate();
    let crease_angle = std::f64::consts::FRAC_PI_4; // 45 degrees

    // Boundary edges on closed cube: 0
    let cpu_bounds = dispatcher.extract_boundary_edges(&cube);
    let cuda_bounds = cuda.extract_boundary_edges(&cube);
    let vulk_bounds = vulkan.extract_boundary_edges(&cube);
    let disp_bounds = cube.extract_boundary_edges();

    assert_eq!(cpu_bounds.len(), 0, "Cube is closed, 0 boundary edges expected");
    assert_eq!(cuda_bounds.len(), 0, "CUDA should detect 0 boundary edges on cube");
    assert_eq!(vulk_bounds.len(), 0, "Vulkan should detect 0 boundary edges on cube");
    assert_eq!(disp_bounds.len(), 0, "Dispatcher should detect 0 boundary edges on cube");

    // Crease feature edges on cube: exactly 12 cube edges (dihedral 90 deg > 45 deg)
    let cpu_creases = canonicalize_segments(dispatcher.extract_feature_edges(&cube, crease_angle));
    let cuda_creases = canonicalize_segments(cuda.extract_feature_edges(&cube, crease_angle));
    let vulk_creases = canonicalize_segments(vulkan.extract_feature_edges(&cube, crease_angle));
    let disp_creases = canonicalize_segments(cube.extract_feature_edges(crease_angle));

    assert_eq!(cpu_creases.len(), 12, "Cube has exactly 12 crease edges");
    assert_eq!(cuda_creases.len(), 12, "CUDA should find 12 crease edges on cube");
    assert_eq!(vulk_creases.len(), 12, "Vulkan should find 12 crease edges on cube");
    assert_eq!(disp_creases.len(), 12, "Dispatcher should find 12 crease edges on cube");

    for i in 0..12 {
        let (cpu_p0, cpu_p1) = cpu_creases[i];
        let (cuda_p0, cuda_p1) = cuda_creases[i];
        let (vulk_p0, vulk_p1) = vulk_creases[i];
        let (disp_p0, disp_p1) = disp_creases[i];

        assert!(
            cuda_p0.distance(cpu_p0) < 1e-6 && cuda_p1.distance(cpu_p1) < 1e-6,
            "CUDA vs CPU crease mismatch at segment {}: {:?} vs {:?}",
            i, (cuda_p0, cuda_p1), (cpu_p0, cpu_p1)
        );
        assert!(
            vulk_p0.distance(cpu_p0) < 1e-3 && vulk_p1.distance(cpu_p1) < 1e-3,
            "Vulkan vs CPU crease mismatch at segment {}: {:?} vs {:?}",
            i, (vulk_p0, vulk_p1), (cpu_p0, cpu_p1)
        );
        assert!(
            disp_p0.distance(cpu_p0) < 1e-3 && disp_p1.distance(cpu_p1) < 1e-3,
            "Dispatcher vs CPU crease mismatch at segment {}: {:?} vs {:?}",
            i, (disp_p0, disp_p1), (cpu_p0, cpu_p1)
        );
    }

    // 2. Open Quad Mesh (4 boundary edges, 1 interior crease edge with 90 deg dihedral)
    let quad = create_open_quad_mesh();

    // Boundary edges
    let cpu_q_bounds = canonicalize_segments(dispatcher.extract_boundary_edges(&quad));
    let cuda_q_bounds = canonicalize_segments(cuda.extract_boundary_edges(&quad));
    let vulk_q_bounds = canonicalize_segments(vulkan.extract_boundary_edges(&quad));
    let disp_q_bounds = canonicalize_segments(quad.extract_boundary_edges());

    assert_eq!(cpu_q_bounds.len(), 4, "Open quad has 4 boundary edges");
    assert_eq!(cuda_q_bounds.len(), 4, "CUDA open quad 4 boundary edges");
    assert_eq!(vulk_q_bounds.len(), 4, "Vulkan open quad 4 boundary edges");
    assert_eq!(disp_q_bounds.len(), 4, "Dispatcher open quad 4 boundary edges");

    for i in 0..4 {
        let (cpu_p0, cpu_p1) = cpu_q_bounds[i];
        let (cuda_p0, cuda_p1) = cuda_q_bounds[i];
        let (vulk_p0, vulk_p1) = vulk_q_bounds[i];
        let (disp_p0, disp_p1) = disp_q_bounds[i];

        assert!(
            cuda_p0.distance(cpu_p0) < 1e-6 && cuda_p1.distance(cpu_p1) < 1e-6,
            "CUDA boundary mismatch at segment {}: {:?} vs {:?}",
            i, (cuda_p0, cuda_p1), (cpu_p0, cpu_p1)
        );
        assert!(
            vulk_p0.distance(cpu_p0) < 1e-3 && vulk_p1.distance(cpu_p1) < 1e-3,
            "Vulkan boundary mismatch at segment {}: {:?} vs {:?}",
            i, (vulk_p0, vulk_p1), (cpu_p0, cpu_p1)
        );
        assert!(
            disp_p0.distance(cpu_p0) < 1e-3 && disp_p1.distance(cpu_p1) < 1e-3,
            "Dispatcher boundary mismatch at segment {}: {:?} vs {:?}",
            i, (disp_p0, disp_p1), (cpu_p0, cpu_p1)
        );
    }

    // Feature edges (4 boundary + 1 crease = 5 edges)
    let cpu_q_features = canonicalize_segments(dispatcher.extract_feature_edges(&quad, crease_angle));
    let cuda_q_features = canonicalize_segments(cuda.extract_feature_edges(&quad, crease_angle));
    let vulk_q_features = canonicalize_segments(vulkan.extract_feature_edges(&quad, crease_angle));
    let disp_q_features = canonicalize_segments(quad.extract_feature_edges(crease_angle));

    assert_eq!(cpu_q_features.len(), 5, "Open quad has 5 feature edges (4 bounds + 1 crease)");
    assert_eq!(cuda_q_features.len(), 5, "CUDA open quad 5 feature edges");
    assert_eq!(vulk_q_features.len(), 5, "Vulkan open quad 5 feature edges");
    assert_eq!(disp_q_features.len(), 5, "Dispatcher open quad 5 feature edges");

    for i in 0..5 {
        let (cpu_p0, cpu_p1) = cpu_q_features[i];
        let (cuda_p0, cuda_p1) = cuda_q_features[i];
        let (vulk_p0, vulk_p1) = vulk_q_features[i];
        let (disp_p0, disp_p1) = disp_q_features[i];

        assert!(
            cuda_p0.distance(cpu_p0) < 1e-6 && cuda_p1.distance(cpu_p1) < 1e-6,
            "CUDA feature mismatch at segment {}: {:?} vs {:?}",
            i, (cuda_p0, cuda_p1), (cpu_p0, cpu_p1)
        );
        assert!(
            vulk_p0.distance(cpu_p0) < 1e-3 && vulk_p1.distance(cpu_p1) < 1e-3,
            "Vulkan feature mismatch at segment {}: {:?} vs {:?}",
            i, (vulk_p0, vulk_p1), (cpu_p0, cpu_p1)
        );
        assert!(
            disp_p0.distance(cpu_p0) < 1e-3 && disp_p1.distance(cpu_p1) < 1e-3,
            "Dispatcher feature mismatch at segment {}: {:?} vs {:?}",
            i, (disp_p0, disp_p1), (cpu_p0, cpu_p1)
        );
    }

    println!("Feature edges and boundary parity verified across cube and open quad.");
}

#[cfg(feature = "cuda")]
#[test]
fn test_cuda_and_vulkan_qem_parity() {
    let dev = init_vulkan_device();
    let vulkan = GpuMeshAnalyzer::new(&dev);
    let cuda = CudaMeshAnalyzer::new();
    let dispatcher = Dispatcher::global();

    // Test on unit cube
    let cube = create_unit_cube();

    let cpu_costs = dispatcher.compute_edge_collapse_costs(&cube);
    let cuda_costs = cuda.compute_edge_collapse_costs(&cube);
    let vulk_costs = vulkan.compute_edge_collapse_costs(&cube);
    let disp_costs = cube.compute_edge_collapse_costs();

    assert_eq!(cpu_costs.len(), cuda_costs.len(), "CPU vs CUDA edge count match");
    assert_eq!(cpu_costs.len(), vulk_costs.len(), "CPU vs Vulkan edge count match");
    assert_eq!(cpu_costs.len(), disp_costs.len(), "CPU vs Dispatcher edge count match");
    assert!(!cpu_costs.is_empty(), "Mesh has candidate edges");

    for i in 0..cpu_costs.len() {
        let (cpu_edge, cpu_cost, cpu_tgt) = cpu_costs[i];
        let (cuda_edge, cuda_cost, cuda_tgt) = cuda_costs[i];
        let (vulk_edge, vulk_cost, vulk_tgt) = vulk_costs[i];
        let (disp_edge, disp_cost, disp_tgt) = disp_costs[i];

        assert_eq!(cpu_edge, cuda_edge, "Edge indices match CPU vs CUDA at {}", i);
        assert_eq!(cpu_edge, vulk_edge, "Edge indices match CPU vs Vulkan at {}", i);
        assert_eq!(cpu_edge, disp_edge, "Edge indices match CPU vs Dispatcher at {}", i);

        // CUDA vs CPU (FP64 parity)
        assert!(
            (cuda_cost - cpu_cost).abs() < 1e-6,
            "CUDA cost mismatch at edge {:?}: {} vs {}",
            cpu_edge, cuda_cost, cpu_cost
        );
        assert!(
            cuda_tgt.distance(cpu_tgt) < 1e-6,
            "CUDA target mismatch at edge {:?}: {:?} vs {:?}",
            cpu_edge, cuda_tgt, cpu_tgt
        );

        // Vulkan vs CPU (FP32 parity)
        assert!(
            (vulk_cost - cpu_cost).abs() < 1e-3,
            "Vulkan cost mismatch at edge {:?}: {} vs {}",
            cpu_edge, vulk_cost, cpu_cost
        );
        assert!(
            vulk_tgt.distance(cpu_tgt) < 1e-3,
            "Vulkan target mismatch at edge {:?}: {:?} vs {:?}",
            cpu_edge, vulk_tgt, cpu_tgt
        );

        // Dispatcher vs CPU
        assert!(
            (disp_cost - cpu_cost).abs() < 1e-3,
            "Dispatcher cost mismatch at edge {:?}: {} vs {}",
            cpu_edge, disp_cost, cpu_cost
        );
        assert!(
            disp_tgt.distance(cpu_tgt) < 1e-3,
            "Dispatcher target mismatch at edge {:?}: {:?} vs {:?}",
            cpu_edge, disp_tgt, cpu_tgt
        );
    }

    println!("QEM parity test passed across CPU, CUDA, and Vulkan.");
}

#[cfg(feature = "cuda")]
#[test]
fn test_cuda_and_vulkan_point_in_2d_loops_parity() {
    let dev = init_vulkan_device();
    let vulkan = GpuMeshAnalyzer::new(&dev);
    let cuda = CudaMeshAnalyzer::new();
    let dispatcher = Dispatcher::global();

    // Nested loops: Outer boundary (10x10) + Hole 1 (center 4x4) + Hole 2 (small 1x1)
    let outer_loop = vec![
        Point2::new(0.0, 0.0),
        Point2::new(10.0, 0.0),
        Point2::new(10.0, 10.0),
        Point2::new(0.0, 10.0),
    ];
    let hole1 = vec![
        Point2::new(3.0, 3.0),
        Point2::new(7.0, 3.0),
        Point2::new(7.0, 7.0),
        Point2::new(3.0, 7.0),
    ];
    let hole2 = vec![
        Point2::new(1.0, 1.0),
        Point2::new(2.0, 1.0),
        Point2::new(2.0, 2.0),
        Point2::new(1.0, 2.0),
    ];
    let loops = vec![outer_loop, hole1, hole2];

    // Diverse test points:
    let query_points = vec![
        // Completely outside outer box
        Point2::new(-1.0, 5.0),
        Point2::new(11.0, 5.0),
        Point2::new(5.0, -2.0),
        Point2::new(5.0, 12.0),
        // Inside outer box, outside holes (should be true)
        Point2::new(1.5, 5.0),
        Point2::new(8.5, 5.0),
        Point2::new(5.0, 1.5),
        Point2::new(5.0, 8.5),
        // Inside hole1 (should be false by parity rule)
        Point2::new(5.0, 5.0),
        Point2::new(4.0, 4.0),
        Point2::new(6.0, 6.0),
        // Inside hole2 (should be false by parity rule)
        Point2::new(1.5, 1.5),
        Point2::new(1.2, 1.8),
    ];

    let cpu_inside = dispatcher.point_in_2d_loops(&query_points, &loops);
    let cuda_inside = cuda.point_in_2d_loops(&query_points, &loops);
    let vulk_inside = vulkan.point_in_2d_loops(&query_points, &loops);

    assert_eq!(cpu_inside.len(), query_points.len());
    assert_eq!(cuda_inside.len(), query_points.len());
    assert_eq!(vulk_inside.len(), query_points.len());

    assert_eq!(cuda_inside, cpu_inside, "CUDA vs CPU point-in-2d-loops parity");
    assert_eq!(vulk_inside, cpu_inside, "Vulkan vs CPU point-in-2d-loops parity");

    // Semantic assertions:
    // Points 0..4 are outside outer box -> false
    for (i, &inside) in cpu_inside.iter().enumerate().take(4) {
        assert!(!inside, "Point {} should be outside", i);
    }
    // Points 4..8 are in solid domain -> true
    for (i, &inside) in cpu_inside.iter().enumerate().take(8).skip(4) {
        assert!(inside, "Point {} should be inside domain", i);
    }
    // Points 8..11 are in hole1 -> false
    for (i, &inside) in cpu_inside.iter().enumerate().take(11).skip(8) {
        assert!(!inside, "Point {} should be inside hole1 -> outside domain", i);
    }
    // Points 11..13 are in hole2 -> false
    for (i, &inside) in cpu_inside.iter().enumerate().take(13).skip(11) {
        assert!(!inside, "Point {} should be inside hole2 -> outside domain", i);
    }

    println!("2D multi-loop point-in-polygon parity verified across CPU, CUDA, and Vulkan.");
}

#[cfg(feature = "cuda")]
#[test]
fn test_cuda_and_vulkan_segment_mesh_distance_parity() {
    let dev = init_vulkan_device();
    let vulkan = GpuMeshAnalyzer::new(&dev);
    let cuda = CudaMeshAnalyzer::new();
    let dispatcher = Dispatcher::global();

    let cube = create_unit_cube();

    let segments = vec![
        // Penetrating segment through the cube interior -> distance 0.0
        (Point3::new(-0.5, 0.5, 0.5), Point3::new(1.5, 0.5, 0.5)),
        // Parallel segment 1.0 unit in front of +X face (x=1.0) -> distance 1.0
        (Point3::new(2.0, 0.2, 0.2), Point3::new(2.0, 0.8, 0.8)),
        // Parallel segment 2.0 units above +Z face (z=1.0) -> distance 2.0
        (Point3::new(0.2, 0.2, 3.0), Point3::new(0.8, 0.8, 3.0)),
        // Segment closest to edge along x=1, y=1: at (1.5, 1.5, 0.5) -> dist = sqrt(0.5^2 + 0.5^2) = sqrt(0.5) ~= 0.70710678
        (Point3::new(1.5, 1.5, 0.2), Point3::new(1.5, 1.5, 0.8)),
        // Segment closest to vertex (1, 1, 1): from (2, 2, 2) to (3, 3, 3) -> dist = sqrt(1+1+1) = sqrt(3) ~= 1.7320508
        (Point3::new(2.0, 2.0, 2.0), Point3::new(3.0, 3.0, 3.0)),
        // Skew cutter move passing close to the cube corner
        (Point3::new(-1.0, 1.2, 1.2), Point3::new(2.0, 1.2, 1.2)),
    ];

    let cpu_dists = dispatcher.segment_mesh_distances(&cube, &segments);
    let cuda_dists = cuda.segment_mesh_distances(&cube, &segments);
    let vulk_dists = vulkan.segment_mesh_distances(&cube, &segments);
    let disp_dists = cube.segment_mesh_distances(&segments);

    assert_eq!(cpu_dists.len(), segments.len());
    assert_eq!(cuda_dists.len(), segments.len());
    assert_eq!(vulk_dists.len(), segments.len());
    assert_eq!(disp_dists.len(), segments.len());

    // Check penetrating segment
    assert!(cpu_dists[0] < 1e-6, "Penetrating segment distance is 0.0");
    assert!(cuda_dists[0] < 1e-6, "CUDA penetrating segment distance is 0.0");
    assert!(vulk_dists[0] < 1e-3, "Vulkan penetrating segment distance is 0.0");

    // Check parallel face segment (dist = 1.0)
    assert!((cpu_dists[1] - 1.0).abs() < 1e-5, "CPU face clearance is 1.0");
    assert!((cuda_dists[1] - 1.0).abs() < 1e-5, "CUDA face clearance is 1.0");
    assert!((vulk_dists[1] - 1.0).abs() < 1e-3, "Vulkan face clearance is 1.0");

    for i in 0..segments.len() {
        assert!(
            (cuda_dists[i] - cpu_dists[i]).abs() < 1e-5,
            "CUDA clearance mismatch at segment {}: {} vs {}",
            i, cuda_dists[i], cpu_dists[i]
        );
        assert!(
            (vulk_dists[i] - cpu_dists[i]).abs() < 2e-3,
            "Vulkan clearance mismatch at segment {}: {} vs {}",
            i, vulk_dists[i], cpu_dists[i]
        );
        assert!(
            (disp_dists[i] - cpu_dists[i]).abs() < 2e-3,
            "Dispatcher clearance mismatch at segment {}: {} vs {}",
            i, disp_dists[i], cpu_dists[i]
        );
    }

    println!("3D segment toolpath clearance parity verified across CPU, CUDA, and Vulkan.");
}




