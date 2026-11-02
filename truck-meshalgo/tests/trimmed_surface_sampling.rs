use std::f64::consts::PI;
use truck_geometry::prelude::*;
use truck_meshalgo::prelude::*;
use truck_topology::compress::*;

fn notched_cylinder(radius: f64, inverted: bool) -> PolygonMesh {
    let surface = RevolutedCurve::by_revolution(
        Line(Point3::new(radius, 0., 0.), Point3::new(radius, 0., 1.)),
        Point3::origin(),
        Vector3::unit_z(),
    );
    let length = 3.0 * radius;
    let uv = [
        Point2::new(0., 0.25),
        Point2::new(length / 2., 0.25),
        Point2::new(length / 2., 0.),
        Point2::new(length, 0.),
        Point2::new(length, PI),
        Point2::new(0., PI),
    ];
    let vertices = uv.iter().map(|p| surface.subs(p.x, p.y)).collect();
    let edges = (0..uv.len())
        .map(|i| {
            let j = (i + 1) % uv.len();
            CompressedEdge {
                vertices: (i, j),
                curve: PCurve::new(Line(uv[i], uv[j]), surface.clone()),
            }
        })
        .collect();
    let boundaries = vec![(0..uv.len())
        .map(|index| CompressedEdgeIndex {
            index,
            orientation: true,
        })
        .collect()];
    let shell = CompressedShell {
        vertices,
        edges,
        faces: vec![CompressedFace {
            surface,
            boundaries,
            orientation: !inverted,
        }],
    };
    shell.robust_triangulation(radius * 0.001).to_polygon()
}

fn assert_cylinder_fidelity(radius: f64, inverted: bool) {
    let mesh = notched_cylinder(radius, inverted);
    let mut area = 0.;
    let mut triangles = 0;
    for triangle in mesh.faces().triangle_iter() {
        let [a, b, c] = triangle.map(|v| mesh.positions()[v.pos]);
        let center = Point3::from_vec((a.to_vec() + b.to_vec() + c.to_vec()) / 3.);
        let radial_error = (center.x.hypot(center.y) - radius).abs();
        assert!(
            radial_error <= radius * 0.002,
            "triangle crosses the cylinder: {radial_error}; vertices={a:?}/{b:?}/{c:?}; uv={:?}",
            triangle.map(|v| mesh.uv_coords()[v.uv.unwrap()])
        );
        // The notch is outside the face even after refinement.
        assert!(center.z >= 1.5 * radius - 1e-8 || center.y.atan2(center.x) >= 0.25 - 1e-8);
        let normal = (b - a).cross(c - a);
        let radial_dot = normal.x * center.x + normal.y * center.y;
        assert!(if inverted {
            radial_dot >= 0.
        } else {
            radial_dot <= 0.
        });
        area += normal.magnitude() / 2.;
        triangles += 1;
    }
    let expected_area = radius * radius * (3. * PI - 0.375);
    assert!((area / expected_area - 1.).abs() < 0.005);
    assert!(triangles > 0 && triangles < 100_000);
}

#[test]
fn notched_cylinder_retains_surface_curvature() {
    assert_cylinder_fidelity(10., false);
}

#[test]
fn small_source_units_retain_surface_curvature() {
    assert_cylinder_fidelity(0.1, false);
}

#[test]
fn inverted_face_retains_curvature_and_orientation() {
    assert_cylinder_fidelity(10., true);
}
