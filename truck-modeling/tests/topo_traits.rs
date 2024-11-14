use std::collections::HashSet;
use std::f64::consts::PI;
use truck_geometry::prelude::*;
use truck_modeling::topo_traits::*;
use truck_topology::{shell::ShellCondition, *};

type Line = truck_geometry::prelude::Line<Point3>;
type Surface = BSplineSurface<Point3>;

fn point_mapping(p: &Point3) -> Point3 { p + Vector3::new(0.0, 1.0, 0.0) }

fn curve_mapping(line: &Line) -> Line { Line(point_mapping(&line.0), point_mapping(&line.1)) }

fn surface_mapping(surface: &Surface) -> Surface {
    surface.transformed(Matrix4::from_translation(Vector3::new(0.0, 1.0, 0.0)))
}

fn connect_points(p: &Point3, q: &Point3) -> Line { Line(*p, *q) }

fn connect_curves(line0: &Line, line1: &Line) -> Surface {
    Surface::new(
        (KnotVec::bezier_knot(1), KnotVec::bezier_knot(1)),
        vec![vec![line0.0, line1.0], vec![line0.1, line1.1]],
    )
}

fn sweep<T: Sweep<Point3, Line, Surface>>(elem: &T) -> T::Swept {
    elem.sweep(
        &point_mapping,
        &curve_mapping,
        &surface_mapping,
        &connect_points,
        &connect_curves,
    )
}

fn multi_sweep<T: MultiSweep<Point3, Line, Surface>>(elem: &T) -> T::Swept {
    elem.multi_sweep(
        &point_mapping,
        &curve_mapping,
        &surface_mapping,
        &connect_points,
        &connect_curves,
        2,
    )
}

fn consistent_line(edge: &Edge<Point3, Line>) -> bool {
    let p = edge.front().point();
    let q = edge.back().point();
    let line = edge.oriented_curve();
    p.near(&line.0) && q.near(&line.1)
}

fn face_top_normal(face: &Face<Point3, Line, Surface>) -> Vector3 {
    face.edge_iter()
        .map(|edge| {
            let vec0 = edge.front().point().to_vec();
            let vec1 = edge.back().point().to_vec();
            vec0.cross(vec1)
        })
        .sum::<Vector3>()
        / 2.0
}

fn test_face() -> Face<Point3, Line, Surface> {
    let p = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.25, 0.0, 0.25),
        Point3::new(0.25, 0.0, 0.75),
        Point3::new(0.75, 0.0, 0.75),
        Point3::new(0.75, 0.0, 0.25),
    ];
    let v = Vertex::news(p);
    let wire: Vec<Wire<_, _>> = vec![
        wire![
            Edge::new(&v[0], &v[1], Line(p[0], p[1])),
            Edge::new(&v[2], &v[1], Line(p[2], p[1])).inverse(),
            Edge::new(&v[2], &v[3], Line(p[2], p[3])),
            Edge::new(&v[3], &v[0], Line(p[3], p[0])),
        ],
        wire![
            Edge::new(&v[7], &v[4], Line(p[7], p[4])).inverse(),
            Edge::new(&v[7], &v[6], Line(p[7], p[6])),
            Edge::new(&v[5], &v[6], Line(p[5], p[6])).inverse(),
            Edge::new(&v[4], &v[5], Line(p[4], p[5])).inverse(),
        ],
    ];
    Face::new(wire, Plane::new(p[0], p[1], p[3]).into_bspline())
}

fn test_shell() -> Shell<Point3, Line, Surface> {
    let p = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(0.0, 0.0, 2.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.25, 0.0, 0.25),
        Point3::new(0.25, 0.0, 0.75),
        Point3::new(0.75, 0.0, 0.75),
        Point3::new(0.75, 0.0, 0.25),
    ];
    let v = Vertex::news(p);
    let e = [
        Edge::new(&v[0], &v[1], Line(p[0], p[1])),
        Edge::new(&v[1], &v[2], Line(p[1], p[2])),
        Edge::new(&v[3], &v[2], Line(p[3], p[2])),
        Edge::new(&v[3], &v[4], Line(p[3], p[4])),
        Edge::new(&v[4], &v[0], Line(p[4], p[0])),
        Edge::new(&v[1], &v[3], Line(p[1], p[3])),
        Edge::new(&v[5], &v[6], Line(p[5], p[6])),
        Edge::new(&v[6], &v[7], Line(p[6], p[7])),
        Edge::new(&v[7], &v[8], Line(p[7], p[8])),
        Edge::new(&v[8], &v[5], Line(p[8], p[5])),
    ];
    let w: Vec<Wire<_, _>> = vec![
        wire![e[0].clone(), e[5].clone(), e[3].clone(), e[4].clone()],
        wire![
            e[9].inverse(),
            e[8].inverse(),
            e[7].inverse(),
            e[6].inverse(),
        ],
        wire![e[5].clone(), e[2].clone(), e[1].inverse()],
    ];
    shell![
        Face::new(w[..2].to_vec(), Plane::new(p[0], p[1], p[4]).into_bspline()),
        Face::new(w[2..].to_vec(), Plane::new(p[1], p[3], p[2]).into_bspline()).inverse(),
    ]
}

#[test]
fn vertex_sweep() {
    let p = Point3::new(0.0, 0.0, 0.0);
    let q = point_mapping(&p);
    let edge = sweep(&Vertex::new(p));
    assert_near!(edge.front().point(), p);
    assert_near!(edge.back().point(), q);
    assert!(consistent_line(&edge));
}

#[test]
fn edge_sweep() {
    let p = Point3::new(0.0, 0.0, 0.0);
    let q = Point3::new(1.0, 0.0, 0.0);
    let edge = Edge::new(&Vertex::new(p), &Vertex::new(q), Line(p, q));

    let face = sweep(&edge);
    let surface = face.oriented_surface();
    assert_eq!(face.boundaries().len(), 1);
    assert_near!(face_top_normal(&face), Vector3::unit_z());
    assert_near!(surface.normal(0.5, 0.5), Vector3::unit_z());
    assert!(face.edge_iter().all(|edge| consistent_line(&edge)));

    let face = sweep(&edge.inverse());
    let surface = face.oriented_surface();
    assert_eq!(face.boundaries().len(), 1);
    assert_near!(face_top_normal(&face), -Vector3::unit_z());
    assert_near!(surface.normal(0.5, 0.5), -Vector3::unit_z());
    assert!(face.edge_iter().all(|edge| consistent_line(&edge)));
}

#[test]
fn wire_sweep() {
    let p = Point3::new(0.0, 0.0, 0.0);
    let q = Point3::new(1.0, 0.0, 0.0);
    let r = Point3::new(2.0, 0.0, 0.0);

    let v = Vertex::news([p, q, r]);
    let wire: Wire<_, _> = vec![
        Edge::new(&v[0], &v[1], Line(p, q)),
        Edge::new(&v[2], &v[1], Line(r, q)).inverse(),
    ]
    .into();
    let shell = sweep(&wire);
    assert_eq!(shell.len(), 2);
    assert!(shell.edge_iter().all(|edge| consistent_line(&edge)));
    let eset: HashSet<_> = shell.edge_iter().map(|e| e.id()).collect();
    assert_eq!(eset.len(), 7);

    shell.face_iter().for_each(|face| {
        assert_eq!(face.boundaries().len(), 1);
        assert_near!(face_top_normal(face), Vector3::unit_z());
        let surface = face.oriented_surface();
        let normal = surface.normal(0.5, 0.5);
        assert_near!(normal, Vector3::unit_z());
    });
}

#[test]
fn face_sweep() {
    let face = test_face();
    let solid = sweep(&face);
    assert_eq!(solid.boundaries().len(), 1);
    let shell = &solid.boundaries()[0];
    assert_eq!(shell.len(), 10);
    assert!(shell.edge_iter().all(|edge| consistent_line(&edge)));
    let o = Point3::new(0.5, 0.5, 0.5);
    shell.face_iter().for_each(|face| {
        let surface = face.oriented_surface();
        let vec = surface.subs(0.5, 0.5) - o;
        let normal = surface.normal(0.5, 0.5);

        let is_side_plane = vec.y.so_small();
        let is_inner_side_plane = vec.magnitude() < 0.3 && is_side_plane;

        let expected_num_boundaries = if is_side_plane { 1 } else { 2 };
        let expected_normal = vec * if is_inner_side_plane { -4.0 } else { 2.0 };
        let expected_area = match (is_side_plane, is_inner_side_plane) {
            (true, true) => 0.5,
            (true, false) => 1.0,
            (false, false) => 0.75,
            (false, true) => unreachable!(),
        };
        assert_eq!(face.boundaries().len(), expected_num_boundaries);
        assert_near!(normal, expected_normal);
        assert_near!(face_top_normal(face), expected_area * expected_normal);
    });

    let solid = sweep(&face.inverse());
    assert_eq!(solid.boundaries().len(), 1);
    let shell = &solid.boundaries()[0];
    assert_eq!(shell.len(), 10);
    assert!(shell.edge_iter().all(|edge| consistent_line(&edge)));
    let o = Point3::new(0.5, 0.5, 0.5);
    shell.face_iter().for_each(|face| {
        let surface = face.oriented_surface();
        let vec = surface.subs(0.5, 0.5) - o;
        let normal = surface.normal(0.5, 0.5);

        let is_side_plane = vec.y.so_small();
        let is_inner_side_plane = vec.magnitude() < 0.3 && is_side_plane;

        let expected_num_boundaries = if is_side_plane { 1 } else { 2 };
        let expected_normal = vec * if is_inner_side_plane { 4.0 } else { -2.0 };
        let expected_area = match (is_side_plane, is_inner_side_plane) {
            (true, true) => 0.5,
            (true, false) => 1.0,
            (false, false) => 0.75,
            (false, true) => unreachable!(),
        };
        assert_eq!(face.boundaries().len(), expected_num_boundaries);
        assert_near!(normal, expected_normal);
        assert_near!(face_top_normal(face), expected_area * expected_normal);
    });
}

#[test]
fn shell_sweep() {
    let shell = test_shell();
    let res_sweep = sweep(&shell);
    assert_eq!(res_sweep.len(), 1);
    let solid = res_sweep[0].as_ref().unwrap();
    assert_eq!(solid.boundaries().len(), 1);
    let shell = &solid.boundaries()[0];
    assert_eq!(shell.len(), 13);

    assert!(shell.edge_iter().all(|edge| consistent_line(&edge)));
    shell.face_iter().for_each(|face| {
        let surface = face.oriented_surface();
        let p = surface.subs(0.5, 0.5);
        let normal = surface.normal(0.5, 0.5);

        let is_left_side = p.z < 1.0;
        let is_side_plane = p.y.near(&0.5);
        let is_inner_plane =
            is_left_side && is_side_plane && Vector2::new(p.z - 0.5, p.x - 0.5).magnitude() < 0.3;
        let is_diagonal_plane = p.near(&Point3::new(0.5, 0.5, 1.5));

        let expected_normal = match (
            is_left_side,
            is_side_plane,
            is_inner_plane,
            is_diagonal_plane,
        ) {
            (_, false, true, _) => unreachable!(),
            (true, _, _, true) => unreachable!(),
            (false, _, true, _) => unreachable!(),
            (_, false, false, _) => Vector3::new(0.0, p.y - 0.5, 0.0) * 2.0,
            (true, true, false, false) => (p - Point3::new(0.5, 0.5, 0.5)) * 2.0,
            (true, true, true, false) => -(p - Point3::new(0.5, 0.5, 0.5)) * 4.0,
            (false, true, false, true) => Vector3::new(1.0, 0.0, 1.0).normalize(),
            (false, true, false, false) => Vector3::new(-1.0, 0.0, 0.0),
        };
        let expected_area = match (
            is_left_side,
            is_side_plane,
            is_inner_plane,
            is_diagonal_plane,
        ) {
            (_, false, true, _) => unreachable!(),
            (true, _, _, true) => unreachable!(),
            (false, _, true, _) => unreachable!(),
            (true, false, false, _) => 0.75,
            (false, false, false, _) => 0.5,
            (_, true, false, false) => 1.0,
            (true, true, true, false) => 0.5,
            (false, true, false, true) => f64::sqrt(2.0),
        };
        let expected_num_boundaries = if is_left_side && !is_side_plane { 2 } else { 1 };

        assert_eq!(face.boundaries().len(), expected_num_boundaries);
        assert_near!(normal, expected_normal);
        assert_near!(face_top_normal(face), expected_area * expected_normal);
    });
}

#[test]
fn vertex_multi_sweep() {
    let p = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, 2.0, 0.0),
    ];
    let wire = multi_sweep(&Vertex::new(p[0]));
    assert_eq!(wire.len(), 2);
    assert!(wire.vertex_iter().zip(p).all(|(v, p)| v.point().near(&p)));
    assert_eq!(wire[0].back(), wire[1].front());
    assert!(wire.edge_iter().all(consistent_line));
}

#[test]
fn edge_multi_sweep() {
    let p = Point3::new(0.0, 0.0, 0.0);
    let q = Point3::new(1.0, 0.0, 0.0);
    let edge = Edge::new(&Vertex::new(p), &Vertex::new(q), Line(p, q));

    let shell = multi_sweep(&edge);
    assert_eq!(shell.len(), 2);
    assert!(shell.edge_iter().all(|e| consistent_line(&e)));
    let eset: HashSet<_> = shell.edge_iter().map(|e| e.id()).collect();
    assert_eq!(eset.len(), 7);
    shell.face_iter().for_each(|face| {
        let surface = face.oriented_surface();
        assert_eq!(face.boundaries().len(), 1);
        assert_near!(face_top_normal(face), Vector3::unit_z());
        assert_near!(surface.normal(0.5, 0.5), Vector3::unit_z());
    });

    let shell = multi_sweep(&edge.inverse());
    assert_eq!(shell.len(), 2);
    assert!(shell.edge_iter().all(|e| consistent_line(&e)));
    let eset: HashSet<_> = shell.edge_iter().map(|e| e.id()).collect();
    assert_eq!(eset.len(), 7);
    shell.face_iter().for_each(|face| {
        let surface = face.oriented_surface();
        assert_eq!(face.boundaries().len(), 1);
        assert_near!(face_top_normal(face), -Vector3::unit_z());
        assert_near!(surface.normal(0.5, 0.5), -Vector3::unit_z());
    });
}

#[test]
fn wire_multi_sweep() {
    let p = Point3::new(0.0, 0.0, 0.0);
    let q = Point3::new(1.0, 0.0, 0.0);
    let r = Point3::new(2.0, 0.0, 0.0);

    let v = Vertex::news([p, q, r]);
    let wire: Wire<_, _> = vec![
        Edge::new(&v[0], &v[1], Line(p, q)),
        Edge::new(&v[2], &v[1], Line(r, q)).inverse(),
    ]
    .into();
    let shell = multi_sweep(&wire);
    assert_eq!(shell.len(), 4);
    assert!(shell.edge_iter().all(|e| consistent_line(&e)));
    let eset: HashSet<_> = shell.edge_iter().map(|e| e.id()).collect();
    assert_eq!(eset.len(), 12);

    shell.face_iter().for_each(|face| {
        assert_eq!(face.boundaries().len(), 1);
        assert_near!(face_top_normal(face), Vector3::unit_z());
        let surface = face.oriented_surface();
        let normal = surface.normal(0.5, 0.5);
        assert_near!(normal, Vector3::unit_z());
    });
}

#[test]
fn face_multi_sweep() {
    let face = test_face();
    let solid = multi_sweep(&face);
    assert_eq!(solid.boundaries().len(), 1);
    let shell = &solid.boundaries()[0];
    assert_eq!(shell.len(), 18);
    assert!(shell.edge_iter().all(|edge| consistent_line(&edge)));
    shell.face_iter().for_each(|face| {
        let surface = face.oriented_surface();
        let p = surface.subs(0.5, 0.5);
        let y = if p.y < 1.0 { 0.5 } else { 1.5 };
        let vec = p - Point3::new(0.5, y, 0.5);
        let normal = surface.normal(0.5, 0.5);

        let is_side_plane = vec.y.so_small();
        let is_inner_side_plane = vec.magnitude() < 0.3 && is_side_plane;

        let expected_num_boundaries = if is_side_plane { 1 } else { 2 };
        let expected_normal = vec * if is_inner_side_plane { -4.0 } else { 2.0 };
        let expected_area = match (is_side_plane, is_inner_side_plane) {
            (true, true) => 0.5,
            (true, false) => 1.0,
            (false, false) => 0.75,
            (false, true) => unreachable!(),
        };
        assert_eq!(face.boundaries().len(), expected_num_boundaries);
        assert_near!(normal, expected_normal);
        assert_near!(face_top_normal(face), expected_area * expected_normal);
    });

    let solid = multi_sweep(&face.inverse());
    assert_eq!(solid.boundaries().len(), 1);
    let shell = &solid.boundaries()[0];
    assert_eq!(shell.len(), 18);
    assert!(shell.edge_iter().all(|edge| consistent_line(&edge)));
    shell.face_iter().for_each(|face| {
        let surface = face.oriented_surface();
        let p = surface.subs(0.5, 0.5);
        let y = if p.y < 1.0 { 0.5 } else { 1.5 };
        let vec = p - Point3::new(0.5, y, 0.5);
        let normal = surface.normal(0.5, 0.5);

        let is_side_plane = vec.y.so_small();
        let is_inner_side_plane = vec.magnitude() < 0.3 && is_side_plane;

        let expected_num_boundaries = if is_side_plane { 1 } else { 2 };
        let expected_normal = vec * if is_inner_side_plane { 4.0 } else { -2.0 };
        let expected_area = match (is_side_plane, is_inner_side_plane) {
            (true, true) => 0.5,
            (true, false) => 1.0,
            (false, false) => 0.75,
            (false, true) => unreachable!(),
        };
        assert_eq!(face.boundaries().len(), expected_num_boundaries);
        assert_near!(normal, expected_normal);
        assert_near!(face_top_normal(face), expected_area * expected_normal);
    });
}

#[test]
fn shell_multi_sweep() {
    let shell = test_shell();
    let res_sweep = multi_sweep(&shell);
    assert_eq!(res_sweep.len(), 1);
    let solid = res_sweep[0].as_ref().unwrap();
    assert_eq!(solid.boundaries().len(), 1);
    let shell = &solid.boundaries()[0];
    assert_eq!(shell.len(), 22);

    assert!(shell.edge_iter().all(|edge| consistent_line(&edge)));
    shell.face_iter().for_each(|face| {
        let surface = face.oriented_surface();
        let p = surface.subs(0.5, 0.5);
        let normal = surface.normal(0.5, 0.5);

        let is_left_side = p.z < 1.0;
        let is_side_plane = p.y.near(&0.5) || p.y.near(&1.5);
        let is_inner_plane =
            is_left_side && is_side_plane && Vector2::new(p.z - 0.5, p.x - 0.5).magnitude() < 0.3;
        let is_diagonal_plane = is_side_plane && Point2::new(p.z, p.x).near(&Point2::new(1.5, 0.5));

        let expected_normal = match (
            is_left_side,
            is_side_plane,
            is_inner_plane,
            is_diagonal_plane,
        ) {
            (_, false, true, _) => unreachable!(),
            (true, _, _, true) => unreachable!(),
            (false, _, true, _) => unreachable!(),
            (_, false, false, _) => Vector3::new(0.0, p.y - 1.0, 0.0),
            (true, true, false, false) => Vector3::new(p.x - 0.5, 0.0, p.z - 0.5) * 2.0,
            (true, true, true, false) => -Vector3::new(p.x - 0.5, 0.0, p.z - 0.5) * 4.0,
            (false, true, false, true) => Vector3::new(1.0, 0.0, 1.0).normalize(),
            (false, true, false, false) => Vector3::new(-1.0, 0.0, 0.0),
        };
        let expected_area = match (
            is_left_side,
            is_side_plane,
            is_inner_plane,
            is_diagonal_plane,
        ) {
            (_, false, true, _) => unreachable!(),
            (true, _, _, true) => unreachable!(),
            (false, _, true, _) => unreachable!(),
            (true, false, false, _) => 0.75,
            (false, false, false, _) => 0.5,
            (_, true, false, false) => 1.0,
            (true, true, true, false) => 0.5,
            (false, true, false, true) => f64::sqrt(2.0),
        };
        let expected_num_boundaries = if is_left_side && !is_side_plane { 2 } else { 1 };

        assert_eq!(face.boundaries().len(), expected_num_boundaries);
        assert_near!(normal, expected_normal);
        assert_near!(face_top_normal(face), expected_area * expected_normal);
    });
}

fn circle_point_mapping(p: &Point3) -> Point3 {
    Matrix4::from_angle_y(Rad(PI / 2.0)).transform_point(*p)
}

fn circle_curve_mapping(line: &Line) -> Line {
    Line(circle_point_mapping(&line.0), circle_point_mapping(&line.1))
}

fn circle_surface_mapping(surface: &Surface) -> Surface {
    surface.transformed(Matrix4::from_angle_y(Rad(PI / 2.0)))
}

fn closed_sweep<T: ClosedSweep<Point3, Line, Surface>>(elem: &T) -> T::Swept {
    elem.closed_sweep(
        &circle_point_mapping,
        &circle_curve_mapping,
        &circle_surface_mapping,
        &connect_points,
        &connect_curves,
        4,
    )
}

#[test]
fn vertex_closed_sweep() {
    let p = [
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 0.0, -1.0),
        Point3::new(-1.0, 0.0, 0.0),
    ];
    let wire = closed_sweep(&Vertex::new(p[0]));
    assert_eq!(wire.len(), 4);
    assert!(wire.is_closed() && wire.is_simple());
    assert!(wire.edge_iter().all(consistent_line));
    assert_eq!(wire.vertex_iter().count(), 4);
    assert!(wire.vertex_iter().zip(p).all(|(v, p)| v.point().near(&p)));
}

#[test]
fn edge_closed_sweep() {
    let p = [Point3::new(0.0, 1.0, 1.0), Point3::new(0.0, 0.0, 1.0)];
    let edge = Edge::new(&Vertex::new(p[0]), &Vertex::new(p[1]), Line(p[0], p[1]));

    let shell = closed_sweep(&edge);
    assert_eq!(shell.len(), 4);
    let vset: HashSet<_> = shell.vertex_iter().map(|v| v.id()).collect();
    assert_eq!(vset.len(), 8);
    let eset: HashSet<_> = shell.edge_iter().map(|e| e.id()).collect();
    assert_eq!(eset.len(), 12);
    assert!(shell.edge_iter().all(|e| consistent_line(&e)));
    assert_eq!(shell.extract_boundaries().len(), 2);
    assert_eq!(shell.shell_condition(), ShellCondition::Oriented);
    shell.face_iter().for_each(|face| {
        let surface = face.oriented_surface();
        let p = surface.subs(0.5, 0.5);
        let vec = Vector3::new(p.x, 0.0, p.z);
        let normal = surface.normal(0.5, 0.5);
        assert_near!(vec * f64::sqrt(2.0), normal);
        assert_near!(face_top_normal(face), f64::sqrt(2.0) * normal);
    });

    let shell = closed_sweep(&edge.inverse());
    assert_eq!(shell.len(), 4);
    let vset: HashSet<_> = shell.vertex_iter().map(|v| v.id()).collect();
    assert_eq!(vset.len(), 8);
    let eset: HashSet<_> = shell.edge_iter().map(|e| e.id()).collect();
    assert_eq!(eset.len(), 12);
    assert!(shell.edge_iter().all(|e| consistent_line(&e)));
    assert_eq!(shell.extract_boundaries().len(), 2);
    assert_eq!(shell.shell_condition(), ShellCondition::Oriented);
    shell.face_iter().for_each(|face| {
        let surface = face.oriented_surface();
        let p = surface.subs(0.5, 0.5);
        let vec = Vector3::new(p.x, 0.0, p.z);
        let normal = surface.normal(0.5, 0.5);
        assert_near!(-vec * f64::sqrt(2.0), normal);
        assert_near!(face_top_normal(face), f64::sqrt(2.0) * normal);
    });
}

#[test]
fn wire_closed_sweep() {
    let p = [
        Point3::new(0.0, 2.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, 0.0, 1.0),
    ];
    let v = Vertex::news(p);
    let wire: Wire<_, _> = vec![
        Edge::new(&v[0], &v[1], Line(p[0], p[1])),
        Edge::new(&v[2], &v[1], Line(p[2], p[1])).inverse(),
    ]
    .into();

    let shell = closed_sweep(&wire);
    assert_eq!(shell.len(), 8);
    let vset: HashSet<_> = shell.vertex_iter().map(|v| v.id()).collect();
    assert_eq!(vset.len(), 12);
    let eset: HashSet<_> = shell.edge_iter().map(|e| e.id()).collect();
    assert_eq!(eset.len(), 20);
    assert!(shell.edge_iter().all(|e| consistent_line(&e)));
    assert_eq!(shell.extract_boundaries().len(), 2);
    assert_eq!(shell.shell_condition(), ShellCondition::Oriented);
    shell.face_iter().for_each(|face| {
        let surface = face.oriented_surface();
        let p = surface.subs(0.5, 0.5);
        let vec = Vector3::new(p.x, 0.0, p.z);
        let normal = surface.normal(0.5, 0.5);
        assert_near!(vec * f64::sqrt(2.0), normal);
        assert_near!(face_top_normal(face), f64::sqrt(2.0) * normal);
    });
}

#[test]
fn face_closed_sweep() {
    let p = [
        Point3::new(0.0, 0.0, 2.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, 2.0),
        Point3::new(0.0, 0.25, 1.75),
        Point3::new(0.0, 0.25, 1.25),
        Point3::new(0.0, 0.75, 1.25),
        Point3::new(0.0, 0.75, 1.75),
    ];
    let v = Vertex::news(p);
    let face = Face::new(
        vec![
            wire![
                Edge::new(&v[0], &v[1], Line(p[0], p[1])),
                Edge::new(&v[2], &v[1], Line(p[2], p[1])).inverse(),
                Edge::new(&v[2], &v[3], Line(p[2], p[3])),
                Edge::new(&v[3], &v[0], Line(p[3], p[0])),
            ],
            wire![
                Edge::new(&v[7], &v[6], Line(p[7], p[6])),
                Edge::new(&v[6], &v[5], Line(p[6], p[5])),
                Edge::new(&v[4], &v[5], Line(p[4], p[5])).inverse(),
                Edge::new(&v[4], &v[7], Line(p[4], p[7])),
            ],
        ],
        Plane::new(p[0], p[1], p[3]).into_bspline(),
    );

    let solid = closed_sweep(&face);
    assert_eq!(solid.boundaries().len(), 2);
    let shell = &solid.boundaries()[0];
    assert_eq!(shell.len(), 16);
    let vset: HashSet<_> = shell.vertex_iter().map(|v| v.id()).collect();
    assert_eq!(vset.len(), 16);
    let eset: HashSet<_> = shell.edge_iter().map(|e| e.id()).collect();
    assert_eq!(eset.len(), 32);
    assert!(shell.edge_iter().all(|e| consistent_line(&e)));
    shell.face_iter().for_each(|face| {
        let surface = face.oriented_surface();
        let p = surface.subs(0.5, 0.5);
        let normal = surface.normal(0.5, 0.5);
        let vec = Vector3::new(p.x, 0.0, p.z);
        let (expected_normal, expected_area) = match (p.y.near(&0.5), vec.magnitude() > 1.0) {
            (true, true) => (vec / f64::sqrt(2.0), 2.0 * f64::sqrt(2.0)),
            (true, false) => (-vec * f64::sqrt(2.0), f64::sqrt(2.0)),
            (false, _) => (Vector3::unit_y() * (p.y - 0.5) * 2.0, 1.5),
        };
        assert_near!(normal, expected_normal);
        assert_near!(face_top_normal(face), expected_normal * expected_area);
    });
    let shell = &solid.boundaries()[1];
    assert_eq!(shell.len(), 16);
    let vset: HashSet<_> = shell.vertex_iter().map(|v| v.id()).collect();
    assert_eq!(vset.len(), 16);
    let eset: HashSet<_> = shell.edge_iter().map(|e| e.id()).collect();
    assert_eq!(eset.len(), 32);
    assert!(shell.edge_iter().all(|e| consistent_line(&e)));
    shell.face_iter().for_each(|face| {
        let surface = face.oriented_surface();
        let p = surface.subs(0.5, 0.5);
        let normal = surface.normal(0.5, 0.5);
        let vec = Vector3::new(p.x, 0.0, p.z);
        let (expected_normal, expected_area) = match (p.y.near(&0.5), vec.magnitude() > 1.0) {
            (true, true) => (
                -vec / (f64::sqrt(2.0) * 7.0 / 8.0),
                f64::sqrt(2.0) * 7.0 / 8.0,
            ),
            (true, false) => (
                vec / (f64::sqrt(2.0) * 5.0 / 8.0),
                f64::sqrt(2.0) * 5.0 / 8.0,
            ),
            (false, _) => (Vector3::unit_y() * (0.5 - p.y) * 4.0, 0.75),
        };
        assert_near!(normal, expected_normal);
        assert_near!(face_top_normal(face), expected_normal * expected_area);
    });

    let solid = closed_sweep(&face.inverse());
    assert_eq!(solid.boundaries().len(), 2);
    let shell = &solid.boundaries()[0];
    assert_eq!(shell.len(), 16);
    let vset: HashSet<_> = shell.vertex_iter().map(|v| v.id()).collect();
    assert_eq!(vset.len(), 16);
    let eset: HashSet<_> = shell.edge_iter().map(|e| e.id()).collect();
    assert_eq!(eset.len(), 32);
    assert!(shell.edge_iter().all(|e| consistent_line(&e)));
    shell.face_iter().for_each(|face| {
        let surface = face.oriented_surface();
        let p = surface.subs(0.5, 0.5);
        let normal = surface.normal(0.5, 0.5);
        let vec = Vector3::new(p.x, 0.0, p.z);
        let (expected_normal, expected_area) = match (p.y.near(&0.5), vec.magnitude() > 1.0) {
            (true, true) => (-vec / f64::sqrt(2.0), 2.0 * f64::sqrt(2.0)),
            (true, false) => (vec * f64::sqrt(2.0), f64::sqrt(2.0)),
            (false, _) => (Vector3::unit_y() * (0.5 - p.y) * 2.0, 1.5),
        };
        assert_near!(normal, expected_normal);
        assert_near!(face_top_normal(face), expected_normal * expected_area);
    });
    let shell = &solid.boundaries()[1];
    assert_eq!(shell.len(), 16);
    let vset: HashSet<_> = shell.vertex_iter().map(|v| v.id()).collect();
    assert_eq!(vset.len(), 16);
    let eset: HashSet<_> = shell.edge_iter().map(|e| e.id()).collect();
    assert_eq!(eset.len(), 32);
    assert!(shell.edge_iter().all(|e| consistent_line(&e)));
    shell.face_iter().for_each(|face| {
        let surface = face.oriented_surface();
        let p = surface.subs(0.5, 0.5);
        let normal = surface.normal(0.5, 0.5);
        let vec = Vector3::new(p.x, 0.0, p.z);
        let (expected_normal, expected_area) = match (p.y.near(&0.5), vec.magnitude() > 1.0) {
            (true, true) => (
                vec / (f64::sqrt(2.0) * 7.0 / 8.0),
                f64::sqrt(2.0) * 7.0 / 8.0,
            ),
            (true, false) => (
                -vec / (f64::sqrt(2.0) * 5.0 / 8.0),
                f64::sqrt(2.0) * 5.0 / 8.0,
            ),
            (false, _) => (Vector3::unit_y() * (p.y - 0.5) * 4.0, 0.75),
        };
        assert_near!(normal, expected_normal);
        assert_near!(face_top_normal(face), expected_normal * expected_area);
    });
}

#[test]
fn shell_closed_sweep() {
    let p = [
        Point3::new(0.0, 0.0, 3.0),
        Point3::new(0.0, 0.0, 2.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, 2.0),
        Point3::new(0.0, 0.25, 1.75),
        Point3::new(0.0, 0.25, 1.25),
        Point3::new(0.0, 0.75, 1.25),
        Point3::new(0.0, 0.75, 1.75),
    ];
    let v = Vertex::news(p);
    let e = [
        Edge::new(&v[0], &v[1], Line(p[0], p[1])),
        Edge::new(&v[1], &v[2], Line(p[1], p[2])),
        Edge::new(&v[3], &v[2], Line(p[3], p[2])).inverse(),
        Edge::new(&v[3], &v[4], Line(p[3], p[4])),
        Edge::new(&v[0], &v[4], Line(p[0], p[4])).inverse(),
        Edge::new(&v[1], &v[4], Line(p[1], p[4])),
        Edge::new(&v[8], &v[7], Line(p[8], p[7])),
        Edge::new(&v[7], &v[6], Line(p[7], p[6])),
        Edge::new(&v[6], &v[5], Line(p[6], p[5])),
        Edge::new(&v[8], &v[5], Line(p[8], p[5])).inverse(),
    ];
    let shell: Shell<_, _, _> = vec![
        Face::new(
            vec![
                wire![e[1].clone(), e[2].clone(), e[3].clone(), e[5].inverse()],
                wire![e[6].clone(), e[7].clone(), e[8].clone(), e[9].clone()],
            ],
            Plane::new(p[1], p[2], p[4]).into_bspline(),
        ),
        Face::new(
            vec![vec![e[4].inverse(), e[5].inverse(), e[0].inverse()].into()],
            Plane::new(p[1], p[0], p[4]).into_bspline(),
        )
        .inverse(),
    ]
    .into();
    let solids = closed_sweep(&shell);
    assert_eq!(solids.len(), 1);
    let solid = solids[0].as_ref().unwrap();
    assert_eq!(solid.boundaries().len(), 2);
    let shell = &solid.boundaries()[0];
    assert_eq!(shell.len(), 20);
    let vset: HashSet<_> = shell.vertex_iter().map(|v| v.id()).collect();
    assert_eq!(vset.len(), 20);
    let eset: HashSet<_> = shell.edge_iter().map(|e| e.id()).collect();
    assert_eq!(eset.len(), 40);
    assert!(shell.edge_iter().all(|e| consistent_line(&e)));
    shell.face_iter().for_each(|face| {
        let surface = face.oriented_surface();
        let p = surface.subs(0.5, 0.5);
        let normal = surface.normal(0.5, 0.5);

        let vec = Vector3::new(p.x, 0.0, p.z);
        let (expected_normal, expected_area) =
            match (p.y.near(&0.5), vec.magnitude() < f64::sqrt(2.0)) {
                (true, true) => (-vec * f64::sqrt(2.0), f64::sqrt(2.0)),
                (true, false) => {
                    let unit_r = vec / 2.5 * f64::sqrt(2.0);
                    let normal = (unit_r * f64::sqrt(2.0) + Vector3::unit_y()) / f64::sqrt(3.0);
                    (normal, 2.5 * f64::sqrt(3.0))
                }
                (false, _) => (
                    Vector3::new(0.0, (p.y - 0.5) * 2.0, 0.0),
                    vec.magnitude() * f64::sqrt(2.0),
                ),
            };
        assert_near!(normal, expected_normal);
        assert_near!(face_top_normal(face), expected_area * expected_normal);
    });
}
