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
