//! Integration tests for planar profile normalization and solid construction.

use monstertruck_modeling::*;

/// Helper: builds a CCW rectangular wire in the XY plane at z=0.
fn rect_wire(x0: f64, y0: f64, x1: f64, y1: f64) -> Wire {
    let v0 = builder::vertex(Point3::new(x0, y0, 0.0));
    let v1 = builder::vertex(Point3::new(x1, y0, 0.0));
    let v2 = builder::vertex(Point3::new(x1, y1, 0.0));
    let v3 = builder::vertex(Point3::new(x0, y1, 0.0));
    vec![
        builder::line(&v0, &v1),
        builder::line(&v1, &v2),
        builder::line(&v2, &v3),
        builder::line(&v3, &v0),
    ]
    .into()
}

/// Helper: builds a CW rectangular wire.
fn rect_wire_cw(x0: f64, y0: f64, x1: f64, y1: f64) -> Wire { rect_wire(x0, y0, x1, y1).inverse() }

/// Helper: builds a triangular wire in the XY plane.
fn triangle_wire(p0: Point3, p1: Point3, p2: Point3) -> Wire {
    let v0 = builder::vertex(p0);
    let v1 = builder::vertex(p1);
    let v2 = builder::vertex(p2);
    vec![
        builder::line(&v0, &v1),
        builder::line(&v1, &v2),
        builder::line(&v2, &v0),
    ]
    .into()
}

// -- Phase 0 fixture tests: nested holes --

#[test]
fn nested_holes_two_level() {
    let outer = rect_wire(-5.0, -5.0, 5.0, 5.0);
    let hole = rect_wire(-2.0, -2.0, 2.0, 2.0);
    let face: Face = profile::attach_plane_normalized(vec![outer, hole]).unwrap();
    assert_eq!(face.boundaries().len(), 2);
}

#[test]
fn nested_holes_four_corners() {
    let outer = rect_wire(-10.0, -10.0, 10.0, 10.0);
    let h1 = rect_wire(-9.0, -9.0, -6.0, -6.0);
    let h2 = rect_wire(6.0, -9.0, 9.0, -6.0);
    let h3 = rect_wire(-9.0, 6.0, -6.0, 9.0);
    let h4 = rect_wire(6.0, 6.0, 9.0, 9.0);
    let face: Face = profile::attach_plane_normalized(vec![outer, h1, h2, h3, h4]).unwrap();
    assert_eq!(face.boundaries().len(), 5);
}

// -- Phase 0 fixture tests: mixed CW/CCW inputs --

#[test]
fn all_ccw_auto_normalized() {
    // All wires given as CCW; holes should be flipped automatically.
    let outer = rect_wire(-3.0, -3.0, 3.0, 3.0);
    let hole = rect_wire(-1.0, -1.0, 1.0, 1.0);
    let face: Face = profile::attach_plane_normalized(vec![outer, hole]).unwrap();
    assert_eq!(face.boundaries().len(), 2);
}

#[test]
fn all_cw_auto_normalized() {
    // All wires given as CW; outer should be flipped, holes kept.
    let outer = rect_wire_cw(-3.0, -3.0, 3.0, 3.0);
    let hole = rect_wire_cw(-1.0, -1.0, 1.0, 1.0);
    let face: Face = profile::attach_plane_normalized(vec![outer, hole]).unwrap();
    assert_eq!(face.boundaries().len(), 2);
}

// -- Phase 0 fixture: near-degenerate tiny holes --

#[test]
fn tiny_hole_survives() {
    let outer = rect_wire(-100.0, -100.0, 100.0, 100.0);
    let tiny = rect_wire(-0.01, -0.01, 0.01, 0.01);
    let face: Face = profile::attach_plane_normalized(vec![outer, tiny]).unwrap();
    assert_eq!(face.boundaries().len(), 2);
}

// -- Phase 1: validation error tests --

#[test]
fn open_wire_error() {
    let v0 = builder::vertex(Point3::new(0.0, 0.0, 0.0));
    let v1 = builder::vertex(Point3::new(1.0, 0.0, 0.0));
    let v2 = builder::vertex(Point3::new(1.0, 1.0, 0.0));
    let wire: Wire = vec![builder::line(&v0, &v1), builder::line(&v1, &v2)].into();
    let result = profile::attach_plane_normalized::<Curve, Surface>(vec![wire]);
    assert!(result.is_err());
}

// -- Phase 5: solid from profile integration --

#[test]
fn solid_box_from_profile() {
    let outer = rect_wire(-1.0, -1.0, 1.0, 1.0);
    let solid = profile::solid_from_planar_profile::<Curve, Surface>(
        vec![outer],
        Vector3::new(0.0, 0.0, 2.0),
    )
    .unwrap();
    let shell = &solid.boundaries()[0];
    assert_eq!(shell.len(), 6);
    assert!(solid.is_geometric_consistent());
}

#[test]
fn solid_tube_from_profile_with_hole() {
    let outer = rect_wire(-2.0, -2.0, 2.0, 2.0);
    let hole = rect_wire(-1.0, -1.0, 1.0, 1.0);
    let solid = profile::solid_from_planar_profile::<Curve, Surface>(
        vec![outer, hole],
        Vector3::new(0.0, 0.0, 3.0),
    )
    .unwrap();
    let shell = &solid.boundaries()[0];
    // 2 caps + 4 outer sides + 4 inner sides = 10 faces.
    assert_eq!(shell.len(), 10);
    assert!(solid.is_geometric_consistent());
}

#[test]
fn solid_with_triangle_profile() {
    let tri = triangle_wire(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(2.0, 0.0, 0.0),
        Point3::new(1.0, 2.0, 0.0),
    );
    let solid = profile::solid_from_planar_profile::<Curve, Surface>(
        vec![tri],
        Vector3::new(0.0, 0.0, 1.0),
    )
    .unwrap();
    let shell = &solid.boundaries()[0];
    // Triangular prism: 2 triangles + 3 quads = 5 faces.
    assert_eq!(shell.len(), 5);
    assert!(solid.is_geometric_consistent());
}

#[test]
fn solid_diagonal_extrusion() {
    let outer = rect_wire(0.0, 0.0, 1.0, 1.0);
    let solid = profile::solid_from_planar_profile::<Curve, Surface>(
        vec![outer],
        Vector3::new(1.0, 1.0, 1.0),
    )
    .unwrap();
    assert!(solid.is_geometric_consistent());
}

// -- Non-XY planes --

#[test]
fn profile_on_yz_plane() {
    let v0 = builder::vertex(Point3::new(0.0, -1.0, -1.0));
    let v1 = builder::vertex(Point3::new(0.0, 1.0, -1.0));
    let v2 = builder::vertex(Point3::new(0.0, 1.0, 1.0));
    let v3 = builder::vertex(Point3::new(0.0, -1.0, 1.0));
    let wire: Wire = vec![
        builder::line(&v0, &v1),
        builder::line(&v1, &v2),
        builder::line(&v2, &v3),
        builder::line(&v3, &v0),
    ]
    .into();
    let face: Face = profile::attach_plane_normalized(vec![wire]).unwrap();
    assert_eq!(face.boundaries().len(), 1);
}
