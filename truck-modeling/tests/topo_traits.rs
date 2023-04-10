use std::collections::HashSet;
use truck_geometry::*;
use truck_modeling::topo_traits::*;
use truck_topology::*;

type Line = truck_geometry::Line<Point3>;
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
    let boundaries = face.boundaries();
    assert_eq!(boundaries.len(), 1);
    let wire = &boundaries[0];
    assert_near!(face_top_normal(&face), Vector3::unit_z());
    wire.iter().for_each(|edge| assert!(consistent_line(edge)));
    let surface = face.oriented_surface();
    let normal = surface.normal(0.5, 0.5);
    assert_near!(normal, Vector3::unit_z());

    let face = sweep(&edge.inverse());
    let boundaries = face.boundaries();
    assert_eq!(boundaries.len(), 1);
    let wire = &boundaries[0];
    assert_near!(face_top_normal(&face), -Vector3::unit_z());
    wire.iter().for_each(|edge| assert!(consistent_line(edge)));
    let surface = face.oriented_surface();
    let normal = surface.normal(0.5, 0.5);
    assert_near!(normal, -Vector3::unit_z());
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

    let eset: HashSet<_> = shell.edge_iter().map(|e| e.id()).collect();
    assert_eq!(eset.len(), 7);

    let boundaries = shell[0].boundaries();
    assert_eq!(boundaries.len(), 1);
    let wire = &boundaries[0];
    assert_near!(face_top_normal(&shell[0]), Vector3::unit_z());
    wire.iter().for_each(|edge| assert!(consistent_line(edge)));
    let surface = shell[0].oriented_surface();
    let normal = surface.normal(0.5, 0.5);
    assert_near!(normal, Vector3::unit_z());
    let boundaries = shell[1].boundaries();
    assert_eq!(boundaries.len(), 1);
    let wire = &boundaries[0];
    assert_near!(face_top_normal(&shell[1]), Vector3::unit_z());
    wire.iter().for_each(|edge| assert!(consistent_line(edge)));
    let surface = shell[0].oriented_surface();
    let normal = surface.normal(0.5, 0.5);
    assert_near!(normal, Vector3::unit_z());
}

#[test]
fn face_sweep() {
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
        vec![
            Edge::new(&v[0], &v[1], Line(p[0], p[1])),
            Edge::new(&v[2], &v[1], Line(p[2], p[1])).inverse(),
            Edge::new(&v[2], &v[3], Line(p[2], p[3])),
            Edge::new(&v[3], &v[0], Line(p[3], p[0])),
        ]
        .into(),
        vec![
            Edge::new(&v[7], &v[4], Line(p[7], p[4])).inverse(),
            Edge::new(&v[7], &v[6], Line(p[7], p[6])),
            Edge::new(&v[5], &v[6], Line(p[5], p[6])).inverse(),
            Edge::new(&v[4], &v[5], Line(p[4], p[5])).inverse(),
        ]
        .into(),
    ];
    let face = Face::new(wire, Plane::new(p[0], p[1], p[3]).into_bspline());

    let solid = sweep(&face);
    assert_eq!(solid.boundaries().len(), 1);
    let shell = &solid.boundaries()[0];
    assert_eq!(shell.len(), 10);
    let o = Point3::new(0.5, 0.5, 0.5);
    shell.face_iter().for_each(|face| {
        let surface = face.oriented_surface();
        let vec = surface.subs(0.5, 0.5) - o;
        let normal = surface.normal(0.5, 0.5);
        let boundaries = face.boundaries();

        let is_side_plane = vec.y.so_small();
        let is_inner_side_plane = vec.magnitude() < 0.3 && is_side_plane;

        let expected_normal = vec * if is_inner_side_plane { -4.0 } else { 2.0 };
        assert_near!(expected_normal, normal);
        let expected_num_boundaries = if is_side_plane { 1 } else { 2 };
        assert_eq!(boundaries.len(), expected_num_boundaries);
        let expected_area = normal
            * match (is_side_plane, is_inner_side_plane) {
                (true, true) => 0.5,
                (true, false) => 1.0,
                (false, false) => 0.75,
                (false, true) => unreachable!(),
            };
        assert_near!(face_top_normal(face), expected_area);
        face.edge_iter()
            .for_each(|edge| assert!(consistent_line(&edge)));
    });

    let solid = sweep(&face.inverse());
    assert_eq!(solid.boundaries().len(), 1);
    let shell = &solid.boundaries()[0];
    assert_eq!(shell.len(), 10);
    shell
        .edge_iter()
        .for_each(|edge| assert!(consistent_line(&edge)));
    let o = Point3::new(0.5, 0.5, 0.5);
    shell.face_iter().for_each(|face| {
        let surface = face.oriented_surface();
        let vec = surface.subs(0.5, 0.5) - o;
        let normal = surface.normal(0.5, 0.5);
        let boundaries = face.boundaries();

        let is_side_plane = vec.y.so_small();
        let is_inner_side_plane = vec.magnitude() < 0.3 && is_side_plane;

        let expected_normal = vec * if is_inner_side_plane { 4.0 } else { -2.0 };
        assert_near!(expected_normal, normal);
        let expected_num_boundaries = if is_side_plane { 1 } else { 2 };
        assert_eq!(boundaries.len(), expected_num_boundaries);
        let expected_area = normal
            * match (is_side_plane, is_inner_side_plane) {
                (true, true) => 0.5,
                (true, false) => 1.0,
                (false, false) => 0.75,
                (false, true) => unreachable!(),
            };
        assert_near!(face_top_normal(face), expected_area);
    });
}

#[test]
fn shell_test() {
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
        vec![e[0].clone(), e[5].clone(), e[3].clone(), e[4].clone()].into(),
        vec![
            e[9].inverse(),
            e[8].inverse(),
            e[7].inverse(),
            e[6].inverse(),
        ]
        .into(),
        vec![e[5].clone(), e[2].clone(), e[1].inverse()].into(),
    ];
    let shell: Shell<_, _, _> = vec![
        Face::new(w[..2].to_vec(), Plane::new(p[0], p[1], p[4]).into_bspline()),
        Face::new(w[2..].to_vec(), Plane::new(p[1], p[3], p[2]).into_bspline()).inverse(),
    ]
    .into();

    let res_sweep = sweep(&shell);
    assert_eq!(res_sweep.len(), 1);
    let solid = res_sweep[0].as_ref().unwrap();
    assert_eq!(solid.boundaries().len(), 1);
    let shell = &solid.boundaries()[0];
    assert_eq!(shell.len(), 13);

    shell
        .edge_iter()
        .for_each(|edge| assert!(consistent_line(&edge)));
    shell.face_iter().for_each(|face| {
        let surface = face.oriented_surface();
        let p = surface.subs(0.5, 0.5);
        let normal = surface.normal(0.5, 0.5);

        let is_left_side = p.z < 1.0;
        let is_side_plane = p.y.near(&0.5);
        let is_inner_plane =
            is_left_side && is_side_plane && (p - Point3::new(0.5, 0.5, 0.5)).magnitude() < 0.3;
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
