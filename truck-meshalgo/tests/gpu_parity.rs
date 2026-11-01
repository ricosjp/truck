#![cfg(feature = "gpu")]

use truck_base::cgmath64::*;
use truck_meshalgo::prelude::*;
use truck_platform::wgpu::*;
use truck_platform::*;

mod common;

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


