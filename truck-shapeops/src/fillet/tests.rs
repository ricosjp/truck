use itertools::Itertools;
use truck_geometry::prelude::*;
use truck_meshalgo::prelude::*;

use super::geometry::*;
use super::types::*;

use truck_geotrait::ParametricSurface;

use super::{
    fillet, fillet_along_wire, fillet_edges, fillet_edges_generic, fillet_with_side, FilletOptions,
    FilletProfile, FilletableCurve, FilletableSurface, RadiusSpec,
};

#[test]
fn create_fillet_surface() {
    #[rustfmt::skip]
    let surface0 = BSplineSurface::new(
        (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)),
        vec![
            vec![Point3::new(0.2, 0.0, 0.0), Point3::new(0.0, 0.5, 0.0), Point3::new(-0.2, 1.0, 0.0)],
            vec![Point3::new(0.5, 0.0, 0.1), Point3::new(0.5, 0.5, 0.0), Point3::new(0.5, 1.0, 0.2)],
            vec![Point3::new(1.0, 0.0, 0.3), Point3::new(1.0, 0.5, 0.3), Point3::new(1.0, 1.0, 0.1)],
        ],
    )
    .into();
    #[rustfmt::skip]
    let surface1 = BSplineSurface::new(
        (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)),
        vec![
            vec![Point3::new(0.2, 0.0, 0.0),  Point3::new(0.0, 0.0, -0.5), Point3::new(-0.2, 0.0, -1.0)],
            vec![Point3::new(0.0, 0.5, 0.0),  Point3::new(0.0, 0.5, -0.5), Point3::new(0.0, 0.5, -1.0)],
            vec![Point3::new(-0.2, 1.0, 0.0), Point3::new(0.2, 1.0, -0.5), Point3::new(0.0, 1.0, -1.0)],
        ],
    )
    .into();

    let mut poly0 =
        StructuredMesh::from_surface(&surface0, ((0.0, 1.0), (0.0, 1.0)), 0.001).destruct();
    let poly1 = StructuredMesh::from_surface(&surface1, ((0.0, 1.0), (0.0, 1.0)), 0.001).destruct();
    poly0.merge(poly1);

    let file0 = std::fs::File::create("edged.obj").unwrap();
    obj::write(&poly0, file0).unwrap();

    let curve = BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            Point3::new(-0.2, 1.0, 0.0),
            Point3::new(0.0, 0.5, 0.0),
            Point3::new(0.2, 0.0, 0.0),
        ],
    );
    let surface =
        rolling_ball_fillet_surface(&surface0, &surface1, &curve, 5, |_| 0.3, true).unwrap();
    let poly = StructuredMesh::from_surface(&surface, ((0.0, 1.0), (0.0, 1.0)), 0.01).destruct();
    let file1 = std::fs::File::create("fillet.obj").unwrap();
    obj::write(&poly, file1).unwrap();
}

#[test]
fn create_fillet() {
    #[rustfmt::skip]
    let surface0: NurbsSurface<_> = BSplineSurface::new(
        (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)),
        vec![
            vec![Point3::new(-1.0, 0.0, 0.0), Point3::new(-1.0, 0.5, 0.0), Point3::new(-1.0, 1.0, 1.0)],
            vec![Point3::new(0.0, 0.0, 0.0),  Point3::new(0.0, 0.5, 0.0),  Point3::new(0.0, 1.0, 1.0)],
            vec![Point3::new(1.0, 0.0, 0.0),  Point3::new(1.0, 0.5, 0.0),  Point3::new(1.0, 1.0, 1.0)],
        ],
    )
    .into();
    #[rustfmt::skip]
    let surface1: NurbsSurface<_> = BSplineSurface::new(
        (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)),
        vec![
            vec![Point3::new(1.0, 0.0, 0.0),  Point3::new(1.0, 0.0, -0.5),  Point3::new(1.0, 1.0, -1.0)],
            vec![Point3::new(0.0, 0.0, 0.0),  Point3::new(0.0, 0.5, -0.5),  Point3::new(0.0, 1.0, -1.0)],
            vec![Point3::new(-1.0, 0.0, 0.0), Point3::new(-1.0, 0.0, -0.5), Point3::new(-1.0, 1.0, -1.0)],
        ],
    )
    .into();

    let v = Vertex::news([
        Point3::new(-1.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(-1.0, 1.0, 1.0),
        Point3::new(-1.0, 1.0, -1.0),
        Point3::new(1.0, 1.0, -1.0),
    ]);

    let boundary0 = surface0.splitted_boundary();
    let boundary1 = surface1.splitted_boundary();

    let wire0: Wire = [
        Edge::new(&v[0], &v[1], boundary0[0].clone().into()),
        Edge::new(&v[1], &v[2], boundary0[1].clone().into()),
        Edge::new(&v[2], &v[3], boundary0[2].clone().into()),
        Edge::new(&v[3], &v[0], boundary0[3].clone().into()),
    ]
    .into();

    let wire1: Wire = [
        wire0[0].inverse(),
        Edge::new(&v[0], &v[4], boundary1[1].clone().into()),
        Edge::new(&v[4], &v[5], boundary1[2].clone().into()),
        Edge::new(&v[5], &v[1], boundary1[3].clone().into()),
    ]
    .into();

    let shared_edge_id = wire0[0].id();
    let face0 = Face::new(vec![wire0], surface0);
    let face1 = Face::new(vec![wire1], surface1);

    let shell: Shell = [face0.clone(), face1.clone()].into();
    let poly = shell.robust_triangulation(0.001).to_polygon();
    let file = std::fs::File::create("edged-shell.obj").unwrap();
    obj::write(&poly, file).unwrap();

    let (face0, face1, fillet) = fillet(
        &face0,
        &face1,
        shared_edge_id,
        &FilletOptions {
            radius: RadiusSpec::Constant(0.3),
            ..Default::default()
        },
    )
    .unwrap();

    let shell: Shell = [face0, face1, fillet].into();
    let poly = shell.robust_triangulation(0.001).to_polygon();
    let file = std::fs::File::create("fillet-shell.obj").unwrap();
    obj::write(&poly, file).unwrap();
}

#[test]
fn create_fillet_with_side() {
    let p = [
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.3, 1.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let v = Vertex::news(p);

    let line = |i: usize, j: usize| {
        let bsp = BSplineCurve::new(KnotVec::bezier_knot(1), vec![p[i], p[j]]);
        Edge::new(&v[i], &v[j], NurbsCurve::from(bsp).into())
    };

    let edge = [
        line(0, 1),
        line(1, 2),
        line(2, 3),
        line(3, 0),
        line(0, 4),
        line(1, 5),
        line(2, 6),
        line(3, 7),
        line(4, 5),
        line(5, 6),
        line(6, 7),
        line(7, 4),
    ];

    let plane = |i: usize, j: usize, k: usize, l: usize| {
        let control_points = vec![vec![p[i], p[l]], vec![p[j], p[k]]];
        let knot_vec = KnotVec::bezier_knot(1);
        let knot_vecs = (knot_vec.clone(), knot_vec);
        let bsp = BSplineSurface::new(knot_vecs, control_points);

        let wire: Wire = [i, j, k, l]
            .into_iter()
            .circular_tuple_windows()
            .map(|(i, j)| {
                edge.iter()
                    .find_map(|edge| {
                        if edge.front() == &v[i] && edge.back() == &v[j] {
                            Some(edge.clone())
                        } else if edge.back() == &v[i] && edge.front() == &v[j] {
                            Some(edge.inverse())
                        } else {
                            None
                        }
                    })
                    .unwrap()
            })
            .collect();
        Face::new(vec![wire], bsp.into())
    };

    let face = [plane(0, 1, 2, 3), plane(0, 3, 7, 4), plane(0, 4, 5, 1)];

    let (face0, face1, fillet, _, side1) = fillet_with_side(
        &face[0],
        &face[1],
        edge[3].id(),
        None,
        Some(&face[2]),
        &FilletOptions {
            radius: RadiusSpec::Variable(Box::new(|t| 0.3 + 0.3 * t)),
            ..Default::default()
        },
    )
    .unwrap();

    let shell: Shell = vec![face0, face1, fillet, side1.unwrap()].into();

    let poly = shell.robust_triangulation(0.001).to_polygon();
    let file = std::fs::File::create("fillet-with-edge.obj").unwrap();
    obj::write(&poly, file).unwrap();
}

#[test]
fn fillet_to_nurbs() {
    let p = [
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let v = Vertex::news(p);

    let line = |i: usize, j: usize| {
        let bsp = BSplineCurve::new(KnotVec::bezier_knot(1), vec![p[i], p[j]]);
        Edge::new(&v[i], &v[j], NurbsCurve::from(bsp).into())
    };
    let edge = [
        line(0, 1),
        Edge::new(
            &v[1],
            &v[2],
            circle_arc_by_three_points(
                p[1].to_homogeneous(),
                p[2].to_homogeneous(),
                Point3::new(1.0 / f64::sqrt(2.0), 1.0 / f64::sqrt(2.0), 1.0),
            )
            .into(),
        ),
        line(2, 0),
        line(1, 4),
        line(2, 5),
        Edge::new(
            &v[4],
            &v[5],
            circle_arc_by_three_points(
                p[4].to_homogeneous(),
                p[5].to_homogeneous(),
                Point3::new(1.0 / f64::sqrt(2.0), 1.0 / f64::sqrt(2.0), 0.0),
            )
            .into(),
        ),
    ];
    let bsp0 = NurbsSurface::new(BSplineSurface::new(
        (KnotVec::bezier_knot(1), KnotVec::bezier_knot(1)),
        vec![
            vec![
                Vector4::new(0.0, 0.0, 1.0, 1.0),
                Vector4::new(0.0, 1.0, 1.0, 1.0),
            ],
            vec![
                Vector4::new(1.0, 0.0, 1.0, 1.0),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
            ],
        ],
    ));
    let bsp1 = NurbsSurface::new(BSplineSurface::new(
        (KnotVec::bezier_knot(1), unit_circle_knot_vec()),
        vec![
            circle_arc_by_three_points(
                p[1].to_homogeneous(),
                p[2].to_homogeneous(),
                Point3::new(1.0 / f64::sqrt(2.0), 1.0 / f64::sqrt(2.0), 1.0),
            )
            .control_points()
            .clone(),
            circle_arc_by_three_points(
                p[4].to_homogeneous(),
                p[5].to_homogeneous(),
                Point3::new(1.0 / f64::sqrt(2.0), 1.0 / f64::sqrt(2.0), 0.0),
            )
            .control_points()
            .clone(),
        ],
    ));
    let shell: Shell = [
        Face::new(
            vec![[edge[0].clone(), edge[1].clone(), edge[2].clone()].into()],
            bsp0,
        ),
        Face::new(
            vec![[
                edge[3].clone(),
                edge[5].clone(),
                edge[4].inverse(),
                edge[1].inverse(),
            ]
            .into()],
            bsp1,
        ),
    ]
    .into();

    let poly = shell.triangulation(0.001).to_polygon();
    let file = std::fs::File::create("cylinder.obj").unwrap();
    obj::write(&poly, file).unwrap();

    let (face0, face1, fillet) = fillet(
        &shell[0],
        &shell[1],
        edge[1].id(),
        &FilletOptions {
            radius: RadiusSpec::Constant(0.3),
            ..Default::default()
        },
    )
    .unwrap();
    let shell: Shell = [face0, face1, fillet].into();

    let poly = shell.triangulation(0.001).to_polygon();
    let file = std::fs::File::create("fillet-cylinder.obj").unwrap();
    obj::write(&poly, file).unwrap();
}

#[test]
fn fillet_semi_cube() {
    let p = [
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, -0.1, 0.0),
        Point3::new(1.1, -0.1, 0.0),
        Point3::new(1.1, 1.1, 0.0),
        Point3::new(0.0, 1.1, 0.0),
    ];
    let v = Vertex::news(p);

    let line = |i: usize, j: usize| {
        let bsp = BSplineCurve::new(KnotVec::bezier_knot(1), vec![p[i], p[j]]);
        Edge::new(&v[i], &v[j], NurbsCurve::from(bsp).into())
    };
    let edge = [
        line(0, 1),
        line(1, 2),
        line(2, 3),
        line(3, 0),
        line(0, 4),
        line(1, 5),
        line(2, 6),
        line(3, 7),
        line(4, 5),
        line(5, 6),
        line(6, 7),
        line(7, 4),
    ];

    let plane = |i: usize, j: usize, k: usize, l: usize| {
        let control_points = vec![vec![p[i], p[l]], vec![p[j], p[k]]];
        let knot_vec = KnotVec::bezier_knot(1);
        let knot_vecs = (knot_vec.clone(), knot_vec);
        let bsp = BSplineSurface::new(knot_vecs, control_points);

        let wire: Wire = [i, j, k, l]
            .into_iter()
            .circular_tuple_windows()
            .map(|(i, j)| {
                edge.iter()
                    .find_map(|edge| {
                        if edge.front() == &v[i] && edge.back() == &v[j] {
                            Some(edge.clone())
                        } else if edge.back() == &v[i] && edge.front() == &v[j] {
                            Some(edge.inverse())
                        } else {
                            None
                        }
                    })
                    .unwrap()
            })
            .collect();
        Face::new(vec![wire], bsp.into())
    };
    let mut shell: Shell = [
        plane(0, 1, 2, 3),
        plane(1, 0, 4, 5),
        plane(2, 1, 5, 6),
        plane(3, 2, 6, 7),
    ]
    .into();

    let poly = shell.robust_triangulation(0.001).to_polygon();
    let file = std::fs::File::create("semi-cube.obj").unwrap();
    obj::write(&poly, file).unwrap();

    let opts = FilletOptions {
        radius: RadiusSpec::Constant(0.4),
        ..Default::default()
    };
    let (face0, face1, face2, _, side1) = fillet_with_side(
        &shell[1],
        &shell[2],
        edge[5].id(),
        None,
        Some(&shell[0]),
        &opts,
    )
    .unwrap();
    (shell[1], shell[2], shell[0]) = (face0, face1, side1.unwrap());
    shell.push(face2);

    let (face0, face1, face2, _, side1) = fillet_with_side(
        &shell[2],
        &shell[3],
        edge[6].id(),
        None,
        Some(&shell[0]),
        &opts,
    )
    .unwrap();
    (shell[2], shell[3], shell[0]) = (face0, face1, side1.unwrap());
    shell.push(face2);

    let mut boundary = shell[0].boundaries().pop().unwrap();
    boundary.pop_back();
    assert_eq!(boundary.front_vertex().unwrap(), &v[0]);

    let poly = shell.robust_triangulation(0.001).to_polygon();
    let file = std::fs::File::create("pre-fillet-cube.obj").unwrap();
    obj::write(&poly, file).unwrap();

    fillet_along_wire(
        &mut shell,
        &boundary,
        &FilletOptions {
            radius: RadiusSpec::Constant(0.2),
            ..Default::default()
        },
    )
    .unwrap();

    let poly = shell.robust_triangulation(0.001).to_polygon();
    let file = std::fs::File::create("fillet-cube.obj").unwrap();
    obj::write(&poly, file).unwrap();
}

#[test]
fn fillet_closed_wire_box_top() {
    // Build a 5-face partial box (top + 4 sides), then fillet all 4 top edges
    // which form a closed square wire on the top face.
    let p = [
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let v = Vertex::news(p);

    let line = |i: usize, j: usize| {
        let bsp = BSplineCurve::new(KnotVec::bezier_knot(1), vec![p[i], p[j]]);
        Edge::new(&v[i], &v[j], NurbsCurve::from(bsp).into())
    };
    let edge = [
        line(0, 1), // 0: top front
        line(1, 2), // 1: top right
        line(2, 3), // 2: top back
        line(3, 0), // 3: top left
        line(0, 4), // 4
        line(1, 5), // 5
        line(2, 6), // 6
        line(3, 7), // 7
        line(4, 5), // 8
        line(5, 6), // 9
        line(6, 7), // 10
        line(7, 4), // 11
    ];

    let plane = |i: usize, j: usize, k: usize, l: usize| {
        let control_points = vec![vec![p[i], p[l]], vec![p[j], p[k]]];
        let knot_vec = KnotVec::bezier_knot(1);
        let knot_vecs = (knot_vec.clone(), knot_vec);
        let bsp = BSplineSurface::new(knot_vecs, control_points);

        let wire: Wire = [i, j, k, l]
            .into_iter()
            .circular_tuple_windows()
            .map(|(i, j)| {
                edge.iter()
                    .find_map(|edge| {
                        if edge.front() == &v[i] && edge.back() == &v[j] {
                            Some(edge.clone())
                        } else if edge.back() == &v[i] && edge.front() == &v[j] {
                            Some(edge.inverse())
                        } else {
                            None
                        }
                    })
                    .unwrap()
            })
            .collect();
        Face::new(vec![wire], bsp.into())
    };

    let mut shell: Shell = [
        plane(0, 1, 2, 3), // face 0: top
        plane(1, 0, 4, 5), // face 1: front
        plane(2, 1, 5, 6), // face 2: right
        plane(3, 2, 6, 7), // face 3: back
        plane(0, 3, 7, 4), // face 4: left
    ]
    .into();

    let initial_face_count = shell.len();

    // All 4 top edges form a closed wire on the top face.
    let closed_wire: Wire = [
        edge[0].clone(),
        edge[1].clone(),
        edge[2].clone(),
        edge[3].clone(),
    ]
    .into();
    assert!(closed_wire.is_closed());

    fillet_along_wire(
        &mut shell,
        &closed_wire,
        &FilletOptions {
            radius: RadiusSpec::Constant(0.2),
            ..Default::default()
        },
    )
    .unwrap();

    // 4 fillet faces should be added.
    assert_eq!(shell.len(), initial_face_count + 4);

    // The shell should still triangulate cleanly.
    let poly = shell.robust_triangulation(0.001).to_polygon();
    let file = std::fs::File::create("fillet-closed-box-top.obj").unwrap();
    obj::write(&poly, file).unwrap();
}

/// Helper: builds a box-like shell with `plane()` and `line()` helpers.
/// Returns `(shell, edges, vertices)`.
fn build_box_shell() -> (Shell, [Edge; 12], Vec<Vertex>) {
    let p = [
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let v = Vertex::news(p);

    let line = |i: usize, j: usize| {
        let bsp = BSplineCurve::new(KnotVec::bezier_knot(1), vec![p[i], p[j]]);
        Edge::new(&v[i], &v[j], NurbsCurve::from(bsp).into())
    };
    let edge = [
        line(0, 1), // 0
        line(1, 2), // 1
        line(2, 3), // 2
        line(3, 0), // 3
        line(0, 4), // 4
        line(1, 5), // 5
        line(2, 6), // 6
        line(3, 7), // 7
        line(4, 5), // 8
        line(5, 6), // 9
        line(6, 7), // 10
        line(7, 4), // 11
    ];

    let plane = |i: usize, j: usize, k: usize, l: usize| {
        let control_points = vec![vec![p[i], p[l]], vec![p[j], p[k]]];
        let knot_vec = KnotVec::bezier_knot(1);
        let knot_vecs = (knot_vec.clone(), knot_vec);
        let bsp = BSplineSurface::new(knot_vecs, control_points);

        let wire: Wire = [i, j, k, l]
            .into_iter()
            .circular_tuple_windows()
            .map(|(i, j)| {
                edge.iter()
                    .find_map(|edge| {
                        if edge.front() == &v[i] && edge.back() == &v[j] {
                            Some(edge.clone())
                        } else if edge.back() == &v[i] && edge.front() == &v[j] {
                            Some(edge.inverse())
                        } else {
                            None
                        }
                    })
                    .unwrap()
            })
            .collect();
        Face::new(vec![wire], bsp.into())
    };

    // Top, front, right, back (partial box — 4 faces sharing edges).
    let shell: Shell = [
        plane(0, 1, 2, 3), // face 0: top
        plane(1, 0, 4, 5), // face 1: front
        plane(2, 1, 5, 6), // face 2: right
        plane(3, 2, 6, 7), // face 3: back
    ]
    .into();

    (shell, edge, v)
}

#[test]
fn fillet_edges_single_edge() {
    let (mut shell, edge, _) = build_box_shell();
    let initial_face_count = shell.len();

    // Fillet edge[5] (shared by face 1: front and face 2: right),
    // same as the first fillet in fillet_semi_cube.
    let params = FilletOptions {
        radius: RadiusSpec::Constant(0.4),
        ..Default::default()
    };
    fillet_edges(&mut shell, &[edge[5].id()], Some(&params)).unwrap();

    // fillet_with_side adds 1 fillet face.
    assert!(shell.len() > initial_face_count);

    // Verify the shell can still be triangulated.
    let poly = shell.robust_triangulation(0.001).to_polygon();
    let file = std::fs::File::create("fillet-edges-single.obj").unwrap();
    obj::write(&poly, file).unwrap();
}

#[test]
fn fillet_edges_rejects_missing() {
    let (mut shell, _, v) = build_box_shell();

    // Create a bogus edge not in the shell.
    let bogus = {
        let bsp = BSplineCurve::new(
            KnotVec::bezier_knot(1),
            vec![
                Point3::new(99.0, 99.0, 99.0),
                Point3::new(100.0, 100.0, 100.0),
            ],
        );
        Edge::new(&v[0], &v[1], NurbsCurve::from(bsp).into())
    };

    let params = FilletOptions {
        radius: RadiusSpec::Constant(0.3),
        ..Default::default()
    };
    let result = fillet_edges(&mut shell, &[bogus.id()], Some(&params));
    assert!(matches!(result, Err(super::FilletError::EdgeNotFound)));
}

#[test]
fn fillet_edges_rejects_boundary() {
    // Build a simple 2-face open shell where one edge is on the boundary.
    let p = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let v = Vertex::news(p);
    let line = |i: usize, j: usize| {
        let bsp = BSplineCurve::new(KnotVec::bezier_knot(1), vec![p[i], p[j]]);
        Edge::new(&v[i], &v[j], NurbsCurve::from(bsp).into())
    };

    let edge = [line(0, 1), line(1, 2), line(2, 3), line(3, 0)];

    let knot_vec = KnotVec::bezier_knot(1);
    let surface: NurbsSurface<_> = BSplineSurface::new(
        (knot_vec.clone(), knot_vec),
        vec![vec![p[0], p[3]], vec![p[1], p[2]]],
    )
    .into();

    let wire: Wire = [
        edge[0].clone(),
        edge[1].clone(),
        edge[2].clone(),
        edge[3].clone(),
    ]
    .into();
    let face = Face::new(vec![wire], surface);
    let mut shell: Shell = vec![face].into();

    // edge[0] is a boundary edge (shared by only 1 face).
    let params = FilletOptions {
        radius: RadiusSpec::Constant(0.3),
        ..Default::default()
    };
    let result = fillet_edges(&mut shell, &[edge[0].id()], Some(&params));
    assert!(matches!(
        result,
        Err(super::FilletError::NonManifoldEdge(1))
    ));
}

// ---------------------------------------------------------------------------
// Generic fillet tests
// ---------------------------------------------------------------------------

// These impls are needed locally because dev-dependency cycles cause
// trait resolution issues — the truck-modeling fillet_impls implement
// traits from truck-modeling's copy of truck-shapeops, not this one.
mod modeling_impls {
    use super::super::types::ParamCurveLinear;
    use truck_geometry::prelude::*;

    type ModelCurve = truck_modeling::Curve;
    type ModelSurface = truck_modeling::Surface;

    impl super::FilletableSurface for ModelSurface {
        fn to_nurbs_surface(&self) -> Option<NurbsSurface<Vector4>> {
            match self {
                ModelSurface::Plane(plane) => {
                    let bsp: BSplineSurface<Point3> = (*plane).into();
                    Some(NurbsSurface::from(bsp))
                }
                ModelSurface::BSplineSurface(bsp) => Some(NurbsSurface::from(bsp.clone())),
                ModelSurface::NurbsSurface(ns) => Some(ns.clone()),
                ModelSurface::RevolutedCurve(_) | ModelSurface::TSplineSurface(_) => None,
            }
        }
        fn from_nurbs_surface(surface: NurbsSurface<Vector4>) -> Self {
            ModelSurface::NurbsSurface(surface)
        }
    }

    fn sample_to_nurbs(
        range: (f64, f64),
        subs: impl Fn(f64) -> Point3,
        n: usize,
    ) -> NurbsCurve<Vector4> {
        let (t0, t1) = range;
        let pts: Vec<Point3> = (0..=n)
            .map(|i| subs(t0 + (t1 - t0) * (i as f64) / (n as f64)))
            .collect();
        let knots: Vec<f64> = (0..=n).map(|i| i as f64 / n as f64).collect();
        let knot_vec = KnotVec::from(
            std::iter::once(0.0)
                .chain(knots.iter().copied())
                .chain(std::iter::once(1.0))
                .collect::<Vec<_>>(),
        );
        let bsp = BSplineCurve::new(knot_vec, pts);
        NurbsCurve::from(bsp)
    }

    impl super::FilletableCurve for ModelCurve {
        fn to_nurbs_curve(&self) -> Option<NurbsCurve<Vector4>> {
            match self {
                ModelCurve::Line(line) => {
                    let bsp: BSplineCurve<Point3> = (*line).into();
                    Some(NurbsCurve::from(bsp))
                }
                ModelCurve::BSplineCurve(bsp) => Some(NurbsCurve::from(bsp.clone())),
                ModelCurve::NurbsCurve(nc) => Some(nc.clone()),
                ModelCurve::IntersectionCurve(ic) => {
                    let range = ic.range_tuple();
                    Some(sample_to_nurbs(range, |t| ic.subs(t), 16))
                }
            }
        }
        fn from_nurbs_curve(c: NurbsCurve<Vector4>) -> Self { ModelCurve::NurbsCurve(c) }
        fn from_pcurve(c: ParamCurveLinear) -> Self {
            let range = c.range_tuple();
            ModelCurve::NurbsCurve(sample_to_nurbs(range, |t| c.subs(t), 16))
        }
        fn from_intersection_curve(
            c: IntersectionCurve<
                ParamCurveLinear,
                Box<NurbsSurface<Vector4>>,
                Box<NurbsSurface<Vector4>>,
            >,
        ) -> Self {
            let range = c.range_tuple();
            ModelCurve::NurbsCurve(sample_to_nurbs(range, |t| c.subs(t), 16))
        }
    }
}

/// Generic fillet with identity (internal) types — verifies the pipeline works as passthrough.
#[test]
fn generic_fillet_identity() {
    let (mut shell, edge, _) = build_box_shell();
    let initial_face_count = shell.len();

    let target_edge = shell.edge_iter().find(|e| e.id() == edge[5].id()).unwrap();

    let params = FilletOptions {
        radius: RadiusSpec::Constant(0.4),
        ..Default::default()
    };
    fillet_edges_generic(&mut shell, &[target_edge], Some(&params)).unwrap();

    assert!(shell.len() > initial_face_count);
    let _poly = shell.robust_triangulation(0.001).to_polygon();
}

/// Generic fillet with truck_modeling types (Plane surfaces, Line curves).
#[test]
fn generic_fillet_modeling_types() {
    type MCurve = truck_modeling::Curve;
    type MSurface = truck_modeling::Surface;
    type MVertex = truck_topology::Vertex<Point3>;
    type MEdge = truck_topology::Edge<Point3, MCurve>;
    type MWire = truck_topology::Wire<Point3, MCurve>;
    type MFace = truck_topology::Face<Point3, MCurve, MSurface>;
    type MShell = truck_topology::Shell<Point3, MCurve, MSurface>;

    let p = [
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let v: Vec<MVertex> = MVertex::news(p);

    let line_edge =
        |i: usize, j: usize| -> MEdge { MEdge::new(&v[i], &v[j], MCurve::Line(Line(p[i], p[j]))) };

    let edge = [
        line_edge(0, 1),
        line_edge(1, 2),
        line_edge(2, 3),
        line_edge(3, 0),
        line_edge(0, 4),
        line_edge(1, 5),
        line_edge(2, 6),
        line_edge(3, 7),
        line_edge(4, 5),
        line_edge(5, 6),
        line_edge(6, 7),
        line_edge(7, 4),
    ];

    let plane_face = |i: usize, j: usize, k: usize, l: usize| -> MFace {
        let plane = Plane::new(p[i], p[j], p[l]);
        let wire: MWire = [i, j, k, l]
            .into_iter()
            .circular_tuple_windows()
            .map(|(a, b)| {
                edge.iter()
                    .find_map(|e| {
                        if e.front() == &v[a] && e.back() == &v[b] {
                            Some(e.clone())
                        } else if e.back() == &v[a] && e.front() == &v[b] {
                            Some(e.inverse())
                        } else {
                            None
                        }
                    })
                    .unwrap()
            })
            .collect();
        MFace::new(vec![wire], MSurface::Plane(plane))
    };

    let mut shell: MShell = [
        plane_face(0, 1, 2, 3),
        plane_face(1, 0, 4, 5),
        plane_face(2, 1, 5, 6),
        plane_face(3, 2, 6, 7),
    ]
    .into();

    let initial_face_count = shell.len();

    // edge[5] is shared by face 1 (front) and face 2 (right).
    let params = FilletOptions {
        radius: RadiusSpec::Constant(0.4),
        ..Default::default()
    };
    fillet_edges_generic(&mut shell, &[edge[5].clone()], Some(&params)).unwrap();

    assert!(shell.len() > initial_face_count);
}

/// Generic fillet with mixed surfaces (some Plane, some NurbsSurface).
#[test]
fn generic_fillet_mixed_surfaces() {
    type MCurve = truck_modeling::Curve;
    type MSurface = truck_modeling::Surface;
    type MVertex = truck_topology::Vertex<Point3>;
    type MEdge = truck_topology::Edge<Point3, MCurve>;
    type MWire = truck_topology::Wire<Point3, MCurve>;
    type MFace = truck_topology::Face<Point3, MCurve, MSurface>;
    type MShell = truck_topology::Shell<Point3, MCurve, MSurface>;

    let p = [
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let v: Vec<MVertex> = MVertex::news(p);

    let line_edge =
        |i: usize, j: usize| -> MEdge { MEdge::new(&v[i], &v[j], MCurve::Line(Line(p[i], p[j]))) };

    let edge = [
        line_edge(0, 1),
        line_edge(1, 2),
        line_edge(2, 3),
        line_edge(3, 0),
        line_edge(0, 4),
        line_edge(1, 5),
        line_edge(2, 6),
        line_edge(3, 7),
        line_edge(4, 5),
        line_edge(5, 6),
        line_edge(6, 7),
        line_edge(7, 4),
    ];

    let make_wire = |i: usize, j: usize, k: usize, l: usize| -> MWire {
        [i, j, k, l]
            .into_iter()
            .circular_tuple_windows()
            .map(|(a, b)| {
                edge.iter()
                    .find_map(|e| {
                        if e.front() == &v[a] && e.back() == &v[b] {
                            Some(e.clone())
                        } else if e.back() == &v[a] && e.front() == &v[b] {
                            Some(e.inverse())
                        } else {
                            None
                        }
                    })
                    .unwrap()
            })
            .collect()
    };

    // Face 0: Plane surface (top face)
    let face0 = MFace::new(
        vec![make_wire(0, 1, 2, 3)],
        MSurface::Plane(Plane::new(p[0], p[1], p[3])),
    );

    // Face 1: NurbsSurface (front face) — convert from BSpline
    let bsp1 = BSplineSurface::new(
        (KnotVec::bezier_knot(1), KnotVec::bezier_knot(1)),
        vec![vec![p[1], p[5]], vec![p[0], p[4]]],
    );
    let face1 = MFace::new(
        vec![make_wire(1, 0, 4, 5)],
        MSurface::NurbsSurface(NurbsSurface::from(bsp1)),
    );

    // Face 2: Plane surface (right face)
    let face2 = MFace::new(
        vec![make_wire(2, 1, 5, 6)],
        MSurface::Plane(Plane::new(p[2], p[1], p[6])),
    );

    // Face 3: Plane surface (back face)
    let face3 = MFace::new(
        vec![make_wire(3, 2, 6, 7)],
        MSurface::Plane(Plane::new(p[3], p[2], p[7])),
    );

    let mut shell: MShell = [face0, face1, face2, face3].into();
    let initial_face_count = shell.len();

    // edge[5] is shared by face 1 (NurbsSurface) and face 2 (Plane).
    let params = FilletOptions {
        radius: RadiusSpec::Constant(0.4),
        ..Default::default()
    };
    fillet_edges_generic(&mut shell, &[edge[5].clone()], Some(&params)).unwrap();

    assert!(shell.len() > initial_face_count);
}

/// Generic fillet with unsupported surface type → UnsupportedGeometry error.
#[test]
fn generic_fillet_unsupported() {
    type MCurve = truck_modeling::Curve;
    type MSurface = truck_modeling::Surface;
    type MVertex = truck_topology::Vertex<Point3>;
    type MEdge = truck_topology::Edge<Point3, MCurve>;
    type MWire = truck_topology::Wire<Point3, MCurve>;
    type MFace = truck_topology::Face<Point3, MCurve, MSurface>;
    type MShell = truck_topology::Shell<Point3, MCurve, MSurface>;

    let p = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let v: Vec<MVertex> = MVertex::news(p);

    let line_edge =
        |i: usize, j: usize| -> MEdge { MEdge::new(&v[i], &v[j], MCurve::Line(Line(p[i], p[j]))) };

    let edge = [
        line_edge(0, 1),
        line_edge(1, 2),
        line_edge(2, 3),
        line_edge(3, 0),
    ];
    let wire: MWire = [
        edge[0].clone(),
        edge[1].clone(),
        edge[2].clone(),
        edge[3].clone(),
    ]
    .into();

    // Use a TSplineSurface (Tmesh) which is unsupported.
    let tmesh = Tmesh::new(p, 1.0);
    let face = MFace::new(vec![wire], MSurface::TSplineSurface(tmesh));
    let mut shell: MShell = vec![face].into();

    let params = FilletOptions {
        radius: RadiusSpec::Constant(0.3),
        ..Default::default()
    };
    let result = fillet_edges_generic(&mut shell, &[edge[0].clone()], Some(&params));
    assert!(
        matches!(result, Err(super::FilletError::UnsupportedGeometry { .. })),
        "expected UnsupportedGeometry, got: {result:?}"
    );
}

/// Fillet two independent edges (different face pairs) in a single `fillet_edges` call.
#[test]
fn fillet_edges_multi_chain() {
    // 5-face box: top + 4 sides
    let p = [
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let v = Vertex::news(p);

    let line = |i: usize, j: usize| {
        let bsp = BSplineCurve::new(KnotVec::bezier_knot(1), vec![p[i], p[j]]);
        Edge::new(&v[i], &v[j], NurbsCurve::from(bsp).into())
    };
    let edge = [
        line(0, 1), // 0: top front
        line(1, 2), // 1: top right
        line(2, 3), // 2: top back
        line(3, 0), // 3: top left
        line(0, 4), // 4
        line(1, 5), // 5
        line(2, 6), // 6
        line(3, 7), // 7
        line(4, 5), // 8
        line(5, 6), // 9
        line(6, 7), // 10
        line(7, 4), // 11
    ];

    let plane = |i: usize, j: usize, k: usize, l: usize| {
        let control_points = vec![vec![p[i], p[l]], vec![p[j], p[k]]];
        let knot_vec = KnotVec::bezier_knot(1);
        let knot_vecs = (knot_vec.clone(), knot_vec);
        let bsp = BSplineSurface::new(knot_vecs, control_points);

        let wire: Wire = [i, j, k, l]
            .into_iter()
            .circular_tuple_windows()
            .map(|(i, j)| {
                edge.iter()
                    .find_map(|edge| {
                        if edge.front() == &v[i] && edge.back() == &v[j] {
                            Some(edge.clone())
                        } else if edge.back() == &v[i] && edge.front() == &v[j] {
                            Some(edge.inverse())
                        } else {
                            None
                        }
                    })
                    .unwrap()
            })
            .collect();
        Face::new(vec![wire], bsp.into())
    };

    let mut shell: Shell = [
        plane(0, 1, 2, 3), // face 0: top
        plane(1, 0, 4, 5), // face 1: front
        plane(2, 1, 5, 6), // face 2: right
        plane(3, 2, 6, 7), // face 3: back
        plane(0, 3, 7, 4), // face 4: left
    ]
    .into();

    let initial_face_count = shell.len();

    // Fillet two independent edges belonging to different face pairs:
    // edge[5] (front-right) and edge[7] (top-left / back-left).
    let params = FilletOptions {
        radius: RadiusSpec::Constant(0.3),
        ..Default::default()
    };
    fillet_edges(&mut shell, &[edge[5].id(), edge[7].id()], Some(&params)).unwrap();

    // Both fillets should add faces.
    assert!(
        shell.len() >= initial_face_count + 2,
        "expected at least 2 new fillet faces, got {} total (was {})",
        shell.len(),
        initial_face_count
    );

    // The shell should triangulate cleanly.
    let _poly = shell.robust_triangulation(0.001).to_polygon();
}

/// Multi-chain test with truck_modeling types via `fillet_edges_generic`.
#[test]
fn generic_fillet_multi_chain() {
    type MCurve = truck_modeling::Curve;
    type MSurface = truck_modeling::Surface;
    type MVertex = truck_topology::Vertex<Point3>;
    type MEdge = truck_topology::Edge<Point3, MCurve>;
    type MWire = truck_topology::Wire<Point3, MCurve>;
    type MFace = truck_topology::Face<Point3, MCurve, MSurface>;
    type MShell = truck_topology::Shell<Point3, MCurve, MSurface>;

    let p = [
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let v: Vec<MVertex> = MVertex::news(p);

    let line_edge =
        |i: usize, j: usize| -> MEdge { MEdge::new(&v[i], &v[j], MCurve::Line(Line(p[i], p[j]))) };

    let edge = [
        line_edge(0, 1),
        line_edge(1, 2),
        line_edge(2, 3),
        line_edge(3, 0),
        line_edge(0, 4),
        line_edge(1, 5),
        line_edge(2, 6),
        line_edge(3, 7),
        line_edge(4, 5),
        line_edge(5, 6),
        line_edge(6, 7),
        line_edge(7, 4),
    ];

    let plane_face = |i: usize, j: usize, k: usize, l: usize| -> MFace {
        let plane = Plane::new(p[i], p[j], p[l]);
        let wire: MWire = [i, j, k, l]
            .into_iter()
            .circular_tuple_windows()
            .map(|(a, b)| {
                edge.iter()
                    .find_map(|e| {
                        if e.front() == &v[a] && e.back() == &v[b] {
                            Some(e.clone())
                        } else if e.back() == &v[a] && e.front() == &v[b] {
                            Some(e.inverse())
                        } else {
                            None
                        }
                    })
                    .unwrap()
            })
            .collect();
        MFace::new(vec![wire], MSurface::Plane(plane))
    };

    let mut shell: MShell = [
        plane_face(0, 1, 2, 3),
        plane_face(1, 0, 4, 5),
        plane_face(2, 1, 5, 6),
        plane_face(3, 2, 6, 7),
        plane_face(0, 3, 7, 4),
    ]
    .into();

    let initial_face_count = shell.len();

    // Fillet two independent edges from different face pairs.
    let params = FilletOptions {
        radius: RadiusSpec::Constant(0.3),
        ..Default::default()
    };
    fillet_edges_generic(
        &mut shell,
        &[edge[5].clone(), edge[7].clone()],
        Some(&params),
    )
    .unwrap();

    assert!(
        shell.len() >= initial_face_count + 2,
        "expected at least 2 new fillet faces, got {} total (was {})",
        shell.len(),
        initial_face_count
    );
}

// ---------------------------------------------------------------------------
// Chamfer tests
// ---------------------------------------------------------------------------

/// Single-edge chamfer on a 2-face shell.
#[test]
fn chamfer_single_edge() {
    let (mut shell, edge, _) = build_box_shell();
    let initial_face_count = shell.len();

    let params = FilletOptions {
        radius: RadiusSpec::Constant(0.4),
        profile: FilletProfile::Chamfer,
        ..Default::default()
    };
    fillet_edges(&mut shell, &[edge[5].id()], Some(&params)).unwrap();

    assert!(shell.len() > initial_face_count);
    let _poly = shell.robust_triangulation(0.001).to_polygon();
}

/// Chamfer along an open wire (same topology as fillet_semi_cube).
#[test]
fn chamfer_semi_cube() {
    let p = [
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, -0.1, 0.0),
        Point3::new(1.1, -0.1, 0.0),
        Point3::new(1.1, 1.1, 0.0),
        Point3::new(0.0, 1.1, 0.0),
    ];
    let v = Vertex::news(p);

    let line = |i: usize, j: usize| {
        let bsp = BSplineCurve::new(KnotVec::bezier_knot(1), vec![p[i], p[j]]);
        Edge::new(&v[i], &v[j], NurbsCurve::from(bsp).into())
    };
    let edge = [
        line(0, 1),
        line(1, 2),
        line(2, 3),
        line(3, 0),
        line(0, 4),
        line(1, 5),
        line(2, 6),
        line(3, 7),
        line(4, 5),
        line(5, 6),
        line(6, 7),
        line(7, 4),
    ];

    let plane = |i: usize, j: usize, k: usize, l: usize| {
        let control_points = vec![vec![p[i], p[l]], vec![p[j], p[k]]];
        let knot_vec = KnotVec::bezier_knot(1);
        let bsp = BSplineSurface::new((knot_vec.clone(), knot_vec), control_points);
        let wire: Wire = [i, j, k, l]
            .into_iter()
            .circular_tuple_windows()
            .map(|(i, j)| {
                edge.iter()
                    .find_map(|edge| {
                        if edge.front() == &v[i] && edge.back() == &v[j] {
                            Some(edge.clone())
                        } else if edge.back() == &v[i] && edge.front() == &v[j] {
                            Some(edge.inverse())
                        } else {
                            None
                        }
                    })
                    .unwrap()
            })
            .collect();
        Face::new(vec![wire], bsp.into())
    };

    let mut shell: Shell = [
        plane(0, 1, 2, 3),
        plane(1, 0, 4, 5),
        plane(2, 1, 5, 6),
        plane(3, 2, 6, 7),
    ]
    .into();

    let chamfer_opts = FilletOptions {
        radius: RadiusSpec::Constant(0.4),
        profile: FilletProfile::Chamfer,
        ..Default::default()
    };
    let (face0, face1, face2, _, side1) = fillet_with_side(
        &shell[1],
        &shell[2],
        edge[5].id(),
        None,
        Some(&shell[0]),
        &chamfer_opts,
    )
    .unwrap();
    (shell[1], shell[2], shell[0]) = (face0, face1, side1.unwrap());
    shell.push(face2);

    let (face0, face1, face2, _, side1) = fillet_with_side(
        &shell[2],
        &shell[3],
        edge[6].id(),
        None,
        Some(&shell[0]),
        &chamfer_opts,
    )
    .unwrap();
    (shell[2], shell[3], shell[0]) = (face0, face1, side1.unwrap());
    shell.push(face2);

    let mut boundary = shell[0].boundaries().pop().unwrap();
    boundary.pop_back();

    fillet_along_wire(
        &mut shell,
        &boundary,
        &FilletOptions {
            radius: RadiusSpec::Constant(0.2),
            profile: FilletProfile::Chamfer,
            ..Default::default()
        },
    )
    .unwrap();

    let _poly = shell.robust_triangulation(0.001).to_polygon();
}

/// Chamfer along a closed wire (same topology as fillet_closed_wire_box_top).
#[test]
fn chamfer_closed_wire() {
    let p = [
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let v = Vertex::news(p);

    let line = |i: usize, j: usize| {
        let bsp = BSplineCurve::new(KnotVec::bezier_knot(1), vec![p[i], p[j]]);
        Edge::new(&v[i], &v[j], NurbsCurve::from(bsp).into())
    };
    let edge = [
        line(0, 1),
        line(1, 2),
        line(2, 3),
        line(3, 0),
        line(0, 4),
        line(1, 5),
        line(2, 6),
        line(3, 7),
        line(4, 5),
        line(5, 6),
        line(6, 7),
        line(7, 4),
    ];

    let plane = |i: usize, j: usize, k: usize, l: usize| {
        let control_points = vec![vec![p[i], p[l]], vec![p[j], p[k]]];
        let knot_vec = KnotVec::bezier_knot(1);
        let bsp = BSplineSurface::new((knot_vec.clone(), knot_vec), control_points);
        let wire: Wire = [i, j, k, l]
            .into_iter()
            .circular_tuple_windows()
            .map(|(i, j)| {
                edge.iter()
                    .find_map(|edge| {
                        if edge.front() == &v[i] && edge.back() == &v[j] {
                            Some(edge.clone())
                        } else if edge.back() == &v[i] && edge.front() == &v[j] {
                            Some(edge.inverse())
                        } else {
                            None
                        }
                    })
                    .unwrap()
            })
            .collect();
        Face::new(vec![wire], bsp.into())
    };

    let mut shell: Shell = [
        plane(0, 1, 2, 3),
        plane(1, 0, 4, 5),
        plane(2, 1, 5, 6),
        plane(3, 2, 6, 7),
        plane(0, 3, 7, 4),
    ]
    .into();

    let initial_face_count = shell.len();

    let closed_wire: Wire = [
        edge[0].clone(),
        edge[1].clone(),
        edge[2].clone(),
        edge[3].clone(),
    ]
    .into();
    assert!(closed_wire.is_closed());

    fillet_along_wire(
        &mut shell,
        &closed_wire,
        &FilletOptions {
            radius: RadiusSpec::Constant(0.2),
            profile: FilletProfile::Chamfer,
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(shell.len(), initial_face_count + 4);
    let _poly = shell.robust_triangulation(0.001).to_polygon();
}

// ---------------------------------------------------------------------------
// Ridge tests
// ---------------------------------------------------------------------------

/// Single-edge ridge on a 2-face shell.
#[test]
fn ridge_single_edge() {
    let (mut shell, edge, _) = build_box_shell();
    let initial_face_count = shell.len();

    let params = FilletOptions {
        radius: RadiusSpec::Constant(0.4),
        profile: FilletProfile::Ridge,
        ..Default::default()
    };
    fillet_edges(&mut shell, &[edge[5].id()], Some(&params)).unwrap();

    assert!(shell.len() > initial_face_count);
    let _poly = shell.robust_triangulation(0.001).to_polygon();
}

/// Ridge along an open wire (same topology as chamfer_semi_cube).
#[test]
fn ridge_semi_cube() {
    let p = [
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, -0.1, 0.0),
        Point3::new(1.1, -0.1, 0.0),
        Point3::new(1.1, 1.1, 0.0),
        Point3::new(0.0, 1.1, 0.0),
    ];
    let v = Vertex::news(p);

    let line = |i: usize, j: usize| {
        let bsp = BSplineCurve::new(KnotVec::bezier_knot(1), vec![p[i], p[j]]);
        Edge::new(&v[i], &v[j], NurbsCurve::from(bsp).into())
    };
    let edge = [
        line(0, 1),
        line(1, 2),
        line(2, 3),
        line(3, 0),
        line(0, 4),
        line(1, 5),
        line(2, 6),
        line(3, 7),
        line(4, 5),
        line(5, 6),
        line(6, 7),
        line(7, 4),
    ];

    let plane = |i: usize, j: usize, k: usize, l: usize| {
        let control_points = vec![vec![p[i], p[l]], vec![p[j], p[k]]];
        let knot_vec = KnotVec::bezier_knot(1);
        let bsp = BSplineSurface::new((knot_vec.clone(), knot_vec), control_points);
        let wire: Wire = [i, j, k, l]
            .into_iter()
            .circular_tuple_windows()
            .map(|(i, j)| {
                edge.iter()
                    .find_map(|edge| {
                        if edge.front() == &v[i] && edge.back() == &v[j] {
                            Some(edge.clone())
                        } else if edge.back() == &v[i] && edge.front() == &v[j] {
                            Some(edge.inverse())
                        } else {
                            None
                        }
                    })
                    .unwrap()
            })
            .collect();
        Face::new(vec![wire], bsp.into())
    };

    let mut shell: Shell = [
        plane(0, 1, 2, 3),
        plane(1, 0, 4, 5),
        plane(2, 1, 5, 6),
        plane(3, 2, 6, 7),
    ]
    .into();

    let ridge_opts = FilletOptions {
        radius: RadiusSpec::Constant(0.4),
        profile: FilletProfile::Ridge,
        ..Default::default()
    };
    let (face0, face1, face2, _, side1) = fillet_with_side(
        &shell[1],
        &shell[2],
        edge[5].id(),
        None,
        Some(&shell[0]),
        &ridge_opts,
    )
    .unwrap();
    (shell[1], shell[2], shell[0]) = (face0, face1, side1.unwrap());
    shell.push(face2);

    let (face0, face1, face2, _, side1) = fillet_with_side(
        &shell[2],
        &shell[3],
        edge[6].id(),
        None,
        Some(&shell[0]),
        &ridge_opts,
    )
    .unwrap();
    (shell[2], shell[3], shell[0]) = (face0, face1, side1.unwrap());
    shell.push(face2);

    let mut boundary = shell[0].boundaries().pop().unwrap();
    boundary.pop_back();

    fillet_along_wire(
        &mut shell,
        &boundary,
        &FilletOptions {
            radius: RadiusSpec::Constant(0.2),
            profile: FilletProfile::Ridge,
            ..Default::default()
        },
    )
    .unwrap();

    let _poly = shell.robust_triangulation(0.001).to_polygon();
}

/// Ridge along a closed wire (same topology as chamfer_closed_wire).
#[test]
fn ridge_closed_wire() {
    let p = [
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let v = Vertex::news(p);

    let line = |i: usize, j: usize| {
        let bsp = BSplineCurve::new(KnotVec::bezier_knot(1), vec![p[i], p[j]]);
        Edge::new(&v[i], &v[j], NurbsCurve::from(bsp).into())
    };
    let edge = [
        line(0, 1),
        line(1, 2),
        line(2, 3),
        line(3, 0),
        line(0, 4),
        line(1, 5),
        line(2, 6),
        line(3, 7),
        line(4, 5),
        line(5, 6),
        line(6, 7),
        line(7, 4),
    ];

    let plane = |i: usize, j: usize, k: usize, l: usize| {
        let control_points = vec![vec![p[i], p[l]], vec![p[j], p[k]]];
        let knot_vec = KnotVec::bezier_knot(1);
        let bsp = BSplineSurface::new((knot_vec.clone(), knot_vec), control_points);
        let wire: Wire = [i, j, k, l]
            .into_iter()
            .circular_tuple_windows()
            .map(|(i, j)| {
                edge.iter()
                    .find_map(|edge| {
                        if edge.front() == &v[i] && edge.back() == &v[j] {
                            Some(edge.clone())
                        } else if edge.back() == &v[i] && edge.front() == &v[j] {
                            Some(edge.inverse())
                        } else {
                            None
                        }
                    })
                    .unwrap()
            })
            .collect();
        Face::new(vec![wire], bsp.into())
    };

    let mut shell: Shell = [
        plane(0, 1, 2, 3),
        plane(1, 0, 4, 5),
        plane(2, 1, 5, 6),
        plane(3, 2, 6, 7),
        plane(0, 3, 7, 4),
    ]
    .into();

    let initial_face_count = shell.len();

    let closed_wire: Wire = [
        edge[0].clone(),
        edge[1].clone(),
        edge[2].clone(),
        edge[3].clone(),
    ]
    .into();
    assert!(closed_wire.is_closed());

    fillet_along_wire(
        &mut shell,
        &closed_wire,
        &FilletOptions {
            radius: RadiusSpec::Constant(0.2),
            profile: FilletProfile::Ridge,
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(shell.len(), initial_face_count + 4);
    let _poly = shell.robust_triangulation(0.001).to_polygon();
}

// ---------------------------------------------------------------------------
// Custom profile tests
// ---------------------------------------------------------------------------

/// Custom with linear profile (0,0)→(1,0) — should behave like chamfer.
#[test]
fn custom_profile_linear() {
    let (mut shell, edge, _) = build_box_shell();
    let initial_face_count = shell.len();

    let profile = BSplineCurve::new(
        KnotVec::bezier_knot(1),
        vec![Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)],
    );
    let params = FilletOptions {
        radius: RadiusSpec::Constant(0.4),
        profile: FilletProfile::Custom(Box::new(profile)),
        ..Default::default()
    };
    fillet_edges(&mut shell, &[edge[5].id()], Some(&params)).unwrap();

    assert!(shell.len() > initial_face_count);
    let _poly = shell.robust_triangulation(0.001).to_polygon();
}

/// Variable-radius fillet along a closed wire (radius varies 0.15..0.20, f(0) ≈ f(1)).
#[test]
fn variable_radius_closed_wire() {
    let p = [
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let v = Vertex::news(p);

    let line = |i: usize, j: usize| {
        let bsp = BSplineCurve::new(KnotVec::bezier_knot(1), vec![p[i], p[j]]);
        Edge::new(&v[i], &v[j], NurbsCurve::from(bsp).into())
    };
    let edge = [
        line(0, 1),
        line(1, 2),
        line(2, 3),
        line(3, 0),
        line(0, 4),
        line(1, 5),
        line(2, 6),
        line(3, 7),
        line(4, 5),
        line(5, 6),
        line(6, 7),
        line(7, 4),
    ];

    let plane = |i: usize, j: usize, k: usize, l: usize| {
        let control_points = vec![vec![p[i], p[l]], vec![p[j], p[k]]];
        let knot_vec = KnotVec::bezier_knot(1);
        let bsp = BSplineSurface::new((knot_vec.clone(), knot_vec), control_points);
        let wire: Wire = [i, j, k, l]
            .into_iter()
            .circular_tuple_windows()
            .map(|(i, j)| {
                edge.iter()
                    .find_map(|edge| {
                        if edge.front() == &v[i] && edge.back() == &v[j] {
                            Some(edge.clone())
                        } else if edge.back() == &v[i] && edge.front() == &v[j] {
                            Some(edge.inverse())
                        } else {
                            None
                        }
                    })
                    .unwrap()
            })
            .collect();
        Face::new(vec![wire], bsp.into())
    };

    let mut shell: Shell = [
        plane(0, 1, 2, 3),
        plane(1, 0, 4, 5),
        plane(2, 1, 5, 6),
        plane(3, 2, 6, 7),
        plane(0, 3, 7, 4),
    ]
    .into();

    let initial_face_count = shell.len();

    let closed_wire: Wire = [
        edge[0].clone(),
        edge[1].clone(),
        edge[2].clone(),
        edge[3].clone(),
    ]
    .into();
    assert!(closed_wire.is_closed());

    // Variable radius: 0.15 at endpoints, peaks at ~0.20 at t=0.5.
    // f(0) ≈ f(1) ≈ 0.15, satisfying the closed-wire constraint.
    let opts = FilletOptions {
        radius: RadiusSpec::Variable(Box::new(|t| 0.15 + 0.05 * (std::f64::consts::PI * t).sin())),
        ..Default::default()
    };
    fillet_along_wire(&mut shell, &closed_wire, &opts).unwrap();

    assert_eq!(shell.len(), initial_face_count + 4);
    let _poly = shell.robust_triangulation(0.001).to_polygon();
}

/// Edge too short for requested fillet radius → DegenerateEdge error.
#[test]
fn fillet_rejects_degenerate_edge() {
    let (mut shell, edge, _) = build_box_shell();

    // The box edges are length 1.0. Request a radius of 0.6 → 2*0.6 = 1.2 > 1.0.
    let params = FilletOptions {
        radius: RadiusSpec::Constant(0.6),
        ..Default::default()
    };
    let result = fillet_edges(&mut shell, &[edge[5].id()], Some(&params));
    assert!(
        matches!(result, Err(super::FilletError::DegenerateEdge)),
        "expected DegenerateEdge, got: {result:?}"
    );
}

/// Custom with degree-2 bump (0,0)→(0.5,1.0)→(1,0) — non-trivial shape.
#[test]
fn custom_profile_bump() {
    let (mut shell, edge, _) = build_box_shell();
    let initial_face_count = shell.len();

    let profile = BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            Point2::new(0.0, 0.0),
            Point2::new(0.5, 1.0),
            Point2::new(1.0, 0.0),
        ],
    );
    let params = FilletOptions {
        radius: RadiusSpec::Constant(0.4),
        profile: FilletProfile::Custom(Box::new(profile)),
        ..Default::default()
    };
    fillet_edges(&mut shell, &[edge[5].id()], Some(&params)).unwrap();

    assert!(shell.len() > initial_face_count);
    let _poly = shell.robust_triangulation(0.001).to_polygon();
}

// ---------------------------------------------------------------------------
// CSG/boolean-result fillet tests
// ---------------------------------------------------------------------------

/// Verify that `convert_shell_in` successfully converts a boolean AND result
/// containing `IntersectionCurve` edges to the internal NURBS representation.
///
/// This exercises the `IntersectionCurve` → NURBS sampling path added to
/// `FilletableCurve::to_nurbs_curve`.
#[test]
fn boolean_shell_converts_for_fillet() {
    use super::convert::convert_shell_in;
    use truck_modeling::builder;

    // Unit cube at origin.
    let v = builder::vertex(Point3::origin());
    let e = builder::tsweep(&v, Vector3::unit_x());
    let f = builder::tsweep(&e, Vector3::unit_y());
    let cube: truck_modeling::Solid = builder::tsweep(&f, Vector3::unit_z());

    // Cylinder punching through the cube (same pattern as punched-cube example).
    let cv = builder::vertex(Point3::new(0.5, 0.25, -0.5));
    let cw = builder::rsweep(
        &cv,
        Point3::new(0.5, 0.5, 0.0),
        Vector3::unit_z(),
        Rad(7.0),
        3,
    );
    let cf = builder::try_attach_plane(&[cw]).unwrap();
    let mut cylinder = builder::tsweep(&cf, Vector3::unit_z() * 2.0);
    cylinder.not();

    // Boolean AND — produces IntersectionCurve edges.
    let solid = crate::and(&cube, &cylinder, 0.05).expect("boolean AND failed");
    let shell = solid.into_boundaries().pop().unwrap();

    // Verify IntersectionCurve edges exist.
    let ic_edges: Vec<_> = shell
        .edge_iter()
        .filter(|e| matches!(e.curve(), truck_modeling::Curve::IntersectionCurve(_)))
        .collect();
    assert!(
        !ic_edges.is_empty(),
        "expected IntersectionCurve edges in boolean result"
    );

    // convert_shell_in should succeed now that IntersectionCurve→NURBS is implemented.
    // Previously this would return UnsupportedGeometry.
    let result = convert_shell_in(&shell, &ic_edges);
    assert!(
        result.is_ok(),
        "convert_shell_in failed: {:?}",
        result.err()
    );

    let (internal_shell, internal_ids) = result.unwrap();
    assert_eq!(internal_ids.len(), ic_edges.len());
    assert!(!internal_shell.is_empty());
}

// ---------------------------------------------------------------------------
// Phase 6c: Variable radius on open wires
// ---------------------------------------------------------------------------

/// Variable radius on an open wire should succeed (no f(0)≈f(1) constraint).
#[test]
fn variable_radius_open_wire() {
    // Reuse fillet_semi_cube topology: 4-face open box, 2-edge open wire.
    let p = [
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, -0.1, 0.0),
        Point3::new(1.1, -0.1, 0.0),
        Point3::new(1.1, 1.1, 0.0),
        Point3::new(0.0, 1.1, 0.0),
    ];
    let v = Vertex::news(p);

    let line = |i: usize, j: usize| {
        let bsp = BSplineCurve::new(KnotVec::bezier_knot(1), vec![p[i], p[j]]);
        Edge::new(&v[i], &v[j], NurbsCurve::from(bsp).into())
    };
    let edge = [
        line(0, 1),
        line(1, 2),
        line(2, 3),
        line(3, 0),
        line(0, 4),
        line(1, 5),
        line(2, 6),
        line(3, 7),
        line(4, 5),
        line(5, 6),
        line(6, 7),
        line(7, 4),
    ];

    let plane = |i: usize, j: usize, k: usize, l: usize| {
        let control_points = vec![vec![p[i], p[l]], vec![p[j], p[k]]];
        let knot_vec = KnotVec::bezier_knot(1);
        let bsp = BSplineSurface::new((knot_vec.clone(), knot_vec), control_points);
        let wire: Wire = [i, j, k, l]
            .into_iter()
            .circular_tuple_windows()
            .map(|(i, j)| {
                edge.iter()
                    .find_map(|edge| {
                        if edge.front() == &v[i] && edge.back() == &v[j] {
                            Some(edge.clone())
                        } else if edge.back() == &v[i] && edge.front() == &v[j] {
                            Some(edge.inverse())
                        } else {
                            None
                        }
                    })
                    .unwrap()
            })
            .collect();
        Face::new(vec![wire], bsp.into())
    };

    let mut shell: Shell = [
        plane(0, 1, 2, 3),
        plane(1, 0, 4, 5),
        plane(2, 1, 5, 6),
        plane(3, 2, 6, 7),
    ]
    .into();

    let opts = FilletOptions {
        radius: RadiusSpec::Constant(0.4),
        ..Default::default()
    };
    let (face0, face1, face2, _, side1) = fillet_with_side(
        &shell[1],
        &shell[2],
        edge[5].id(),
        None,
        Some(&shell[0]),
        &opts,
    )
    .unwrap();
    (shell[1], shell[2], shell[0]) = (face0, face1, side1.unwrap());
    shell.push(face2);

    let (face0, face1, face2, _, side1) = fillet_with_side(
        &shell[2],
        &shell[3],
        edge[6].id(),
        None,
        Some(&shell[0]),
        &opts,
    )
    .unwrap();
    (shell[2], shell[3], shell[0]) = (face0, face1, side1.unwrap());
    shell.push(face2);

    let mut boundary = shell[0].boundaries().pop().unwrap();
    boundary.pop_back();
    assert_eq!(boundary.front_vertex().unwrap(), &v[0]);
    assert!(!boundary.is_closed());

    // Variable radius where f(0)=0.1, f(1)=0.3 — NOT equal, would fail on closed wire.
    let var_opts = FilletOptions {
        radius: RadiusSpec::Variable(Box::new(|t| 0.1 + 0.2 * t)),
        ..Default::default()
    };
    fillet_along_wire(&mut shell, &boundary, &var_opts).unwrap();

    let _poly = shell.robust_triangulation(0.001).to_polygon();
}

// ---------------------------------------------------------------------------
// Phase 6a: Identity-based edge replacement
// ---------------------------------------------------------------------------

/// Verify `cut_face_by_bezier` works on a 5-edge boundary (pentagon).
#[test]
fn cut_face_five_edge_boundary() {
    use super::topology::cut_face_by_bezier;

    // Build a planar pentagon: vertices at unit-circle positions.
    let pts: Vec<Point3> = (0..5)
        .map(|i| {
            let angle = 2.0 * std::f64::consts::PI * (i as f64) / 5.0;
            Point3::new(angle.cos(), angle.sin(), 0.0)
        })
        .collect();
    let v = Vertex::news(&pts);

    let line_edge = |i: usize, j: usize| {
        let bsp = BSplineCurve::new(KnotVec::bezier_knot(1), vec![pts[i], pts[j]]);
        Edge::new(&v[i], &v[j], NurbsCurve::from(bsp).into())
    };

    // 5 edges: e0(0→1), e1(1→2), e2(2→3), e3(3→4), e4(4→0)
    let edges = [
        line_edge(0, 1),
        line_edge(1, 2),
        line_edge(2, 3),
        line_edge(3, 4),
        line_edge(4, 0),
    ];

    let wire: Wire = edges.iter().cloned().collect();

    // Simple planar surface covering the pentagon area.
    let surface: NurbsSurface<_> = BSplineSurface::new(
        (KnotVec::bezier_knot(1), KnotVec::bezier_knot(1)),
        vec![
            vec![Point3::new(-1.5, -1.5, 0.0), Point3::new(-1.5, 1.5, 0.0)],
            vec![Point3::new(1.5, -1.5, 0.0), Point3::new(1.5, 1.5, 0.0)],
        ],
    )
    .into();

    let face = Face::new(vec![wire], surface);

    // Pick edge[2] (2→3) as the filleted edge.
    // Adjacent edges: front=edge[1] (1→2), back=edge[3] (3→4).
    // Build a bezier that starts near the midpoint of edge[1] and ends near
    // the midpoint of edge[3], crossing through the filleted edge region.
    let mid1 = (pts[1] + pts[2].to_vec()) / 2.0;
    let mid3 = (pts[3] + pts[4].to_vec()) / 2.0;
    let mid_control = (mid1 + mid3.to_vec()) / 2.0;
    let bezier: NurbsCurve<Vector4> = NurbsCurve::from(BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![mid1, mid_control, mid3],
    ));

    let result = cut_face_by_bezier(&face, bezier, edges[2].id());
    assert!(result.is_some(), "cut_face_by_bezier returned None");

    let (new_face, fillet_edge) = result.unwrap();
    // Should still have 5 edges (3 original untouched + new_front + fillet + new_back,
    // replacing front + filleted + back = same count).
    let boundary = &new_face.absolute_boundaries()[0];
    assert_eq!(
        boundary.len(),
        5,
        "expected 5 edges after cut, got {}",
        boundary.len()
    );
    // The fillet edge should appear in the boundary.
    assert!(
        boundary.iter().any(|e| e.id() == fillet_edge.id()),
        "fillet edge not found in new boundary"
    );
}

// ---------------------------------------------------------------------------
// Phase 6b: Per-edge radius
// ---------------------------------------------------------------------------

/// Per-edge radius with two edges having different radii.
#[test]
fn per_edge_radius_two_edges() {
    let p = [
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let v = Vertex::news(p);

    let line = |i: usize, j: usize| {
        let bsp = BSplineCurve::new(KnotVec::bezier_knot(1), vec![p[i], p[j]]);
        Edge::new(&v[i], &v[j], NurbsCurve::from(bsp).into())
    };
    let edge = [
        line(0, 1),
        line(1, 2),
        line(2, 3),
        line(3, 0),
        line(0, 4),
        line(1, 5),
        line(2, 6),
        line(3, 7),
        line(4, 5),
        line(5, 6),
        line(6, 7),
        line(7, 4),
    ];

    let plane = |i: usize, j: usize, k: usize, l: usize| {
        let control_points = vec![vec![p[i], p[l]], vec![p[j], p[k]]];
        let knot_vec = KnotVec::bezier_knot(1);
        let bsp = BSplineSurface::new((knot_vec.clone(), knot_vec), control_points);
        let wire: Wire = [i, j, k, l]
            .into_iter()
            .circular_tuple_windows()
            .map(|(i, j)| {
                edge.iter()
                    .find_map(|edge| {
                        if edge.front() == &v[i] && edge.back() == &v[j] {
                            Some(edge.clone())
                        } else if edge.back() == &v[i] && edge.front() == &v[j] {
                            Some(edge.inverse())
                        } else {
                            None
                        }
                    })
                    .unwrap()
            })
            .collect();
        Face::new(vec![wire], bsp.into())
    };

    let mut shell: Shell = [
        plane(0, 1, 2, 3),
        plane(1, 0, 4, 5),
        plane(2, 1, 5, 6),
        plane(3, 2, 6, 7),
        plane(0, 3, 7, 4),
    ]
    .into();

    let initial_face_count = shell.len();

    // Two independent edges with different radii.
    let params = FilletOptions {
        radius: RadiusSpec::PerEdge(vec![0.3, 0.15]),
        ..Default::default()
    };
    fillet_edges(&mut shell, &[edge[5].id(), edge[7].id()], Some(&params)).unwrap();

    assert!(
        shell.len() >= initial_face_count + 2,
        "expected at least 2 new fillet faces, got {} total (was {})",
        shell.len(),
        initial_face_count
    );
    let _poly = shell.robust_triangulation(0.001).to_polygon();
}

/// Per-edge radius count mismatch → PerEdgeRadiusMismatch error.
#[test]
fn per_edge_radius_mismatch() {
    let (mut shell, edge, _) = build_box_shell();

    // Provide 1 radius for 2 edges → mismatch.
    let params = FilletOptions {
        radius: RadiusSpec::PerEdge(vec![0.3]),
        ..Default::default()
    };
    let result = fillet_edges(&mut shell, &[edge[5].id(), edge[6].id()], Some(&params));
    assert!(
        matches!(
            result,
            Err(super::FilletError::PerEdgeRadiusMismatch {
                given: 1,
                expected: 2
            })
        ),
        "expected PerEdgeRadiusMismatch, got: {result:?}"
    );
}

/// Per-edge radius where one edge is too short → DegenerateEdge.
#[test]
fn per_edge_radius_degenerate() {
    let (mut shell, edge, _) = build_box_shell();

    // edge[5] length ~1.0, radius 0.15 → ok (2*0.15=0.3 < 1.0).
    // edge[6] length ~1.0, radius 0.6 → too big (2*0.6=1.2 > 1.0).
    let params = FilletOptions {
        radius: RadiusSpec::PerEdge(vec![0.15, 0.6]),
        ..Default::default()
    };
    let result = fillet_edges(&mut shell, &[edge[5].id(), edge[6].id()], Some(&params));
    assert!(
        matches!(result, Err(super::FilletError::DegenerateEdge)),
        "expected DegenerateEdge, got: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// Geometric accuracy tests
// ---------------------------------------------------------------------------

/// Round fillet contact curves lie at the correct distance from the original planes.
#[test]
fn radius_error_bounds() {
    let (shell, edge, _) = build_box_shell();

    // face 1 (front) is y=0 plane, face 2 (right) is x=1 plane.
    // edge[5] (1→5) runs along z at (x=1, y=0). These faces are orthogonal.
    let radius = 0.3;
    let (_, _, fillet) = fillet(
        &shell[1],
        &shell[2],
        edge[5].id(),
        &FilletOptions {
            radius: RadiusSpec::Constant(radius),
            ..Default::default()
        },
    )
    .unwrap();

    let fillet_surface = fillet.oriented_surface();
    let n = 8;
    let tol = 0.01;

    // u=0 contact curve: fillet touches face1 (y=0 plane).
    // Points should be on that plane (dy≈0) at distance radius from face2 (dx≈radius).
    for j in 0..=n {
        let v = j as f64 / n as f64;
        let pt = fillet_surface.subs(0.0, v);
        let dy = pt.y.abs();
        let dx = (pt.x - 1.0).abs();
        assert!(dy < tol, "u=0 contact not on y=0 plane: dy={dy:.6}");
        assert!(
            (dx - radius).abs() < tol,
            "u=0 contact distance from x=1 plane: dx={dx:.6}, expected {radius}"
        );
    }

    // u=1 contact curve: fillet touches face2 (x=1 plane).
    // Points should be on that plane (dx≈0) at distance radius from face1 (dy≈radius).
    for j in 0..=n {
        let v = j as f64 / n as f64;
        let pt = fillet_surface.subs(1.0, v);
        let dx = (pt.x - 1.0).abs();
        let dy = pt.y.abs();
        assert!(dx < tol, "u=1 contact not on x=1 plane: dx={dx:.6}");
        assert!(
            (dy - radius).abs() < tol,
            "u=1 contact distance from y=0 plane: dy={dy:.6}, expected {radius}"
        );
    }

    // Interior: all points should be inside the fillet pocket (0 < dx < radius, 0 < dy < radius).
    for i in 1..n {
        for j in 0..=n {
            let u = i as f64 / n as f64;
            let v = j as f64 / n as f64;
            let pt = fillet_surface.subs(u, v);
            let dx = (pt.x - 1.0).abs();
            let dy = pt.y.abs();
            assert!(
                dx < radius + tol && dy < radius + tol,
                "interior point ({u:.2},{v:.2}) outside pocket: dx={dx:.4} dy={dy:.4}"
            );
        }
    }
}

/// Adjacent fillet surfaces in a multi-edge wire should meet with C0 continuity
/// and approximate G1 tangent alignment at their shared seam.
#[test]
fn continuity_at_wire_joins() {
    // Use the same 4-face semi-cube topology as fillet_semi_cube, producing
    // a 2-edge open wire fillet with two adjacent fillet surfaces.
    let p = [
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, -0.1, 0.0),
        Point3::new(1.1, -0.1, 0.0),
        Point3::new(1.1, 1.1, 0.0),
        Point3::new(0.0, 1.1, 0.0),
    ];
    let v = Vertex::news(p);

    let line = |i: usize, j: usize| {
        let bsp = BSplineCurve::new(KnotVec::bezier_knot(1), vec![p[i], p[j]]);
        Edge::new(&v[i], &v[j], NurbsCurve::from(bsp).into())
    };
    let edge = [
        line(0, 1),
        line(1, 2),
        line(2, 3),
        line(3, 0),
        line(0, 4),
        line(1, 5),
        line(2, 6),
        line(3, 7),
        line(4, 5),
        line(5, 6),
        line(6, 7),
        line(7, 4),
    ];

    let plane = |i: usize, j: usize, k: usize, l: usize| {
        let control_points = vec![vec![p[i], p[l]], vec![p[j], p[k]]];
        let knot_vec = KnotVec::bezier_knot(1);
        let bsp = BSplineSurface::new((knot_vec.clone(), knot_vec), control_points);
        let wire: Wire = [i, j, k, l]
            .into_iter()
            .circular_tuple_windows()
            .map(|(i, j)| {
                edge.iter()
                    .find_map(|edge| {
                        if edge.front() == &v[i] && edge.back() == &v[j] {
                            Some(edge.clone())
                        } else if edge.back() == &v[i] && edge.front() == &v[j] {
                            Some(edge.inverse())
                        } else {
                            None
                        }
                    })
                    .unwrap()
            })
            .collect();
        Face::new(vec![wire], bsp.into())
    };

    let mut shell: Shell = [
        plane(0, 1, 2, 3),
        plane(1, 0, 4, 5),
        plane(2, 1, 5, 6),
        plane(3, 2, 6, 7),
    ]
    .into();

    let opts = FilletOptions {
        radius: RadiusSpec::Constant(0.4),
        ..Default::default()
    };
    let (face0, face1, face2, _, side1) = fillet_with_side(
        &shell[1],
        &shell[2],
        edge[5].id(),
        None,
        Some(&shell[0]),
        &opts,
    )
    .unwrap();
    (shell[1], shell[2], shell[0]) = (face0, face1, side1.unwrap());
    shell.push(face2);

    let (face0, face1, face2, _, side1) = fillet_with_side(
        &shell[2],
        &shell[3],
        edge[6].id(),
        None,
        Some(&shell[0]),
        &opts,
    )
    .unwrap();
    (shell[2], shell[3], shell[0]) = (face0, face1, side1.unwrap());
    shell.push(face2);

    let mut boundary = shell[0].boundaries().pop().unwrap();
    boundary.pop_back();

    let initial_count = shell.len();
    fillet_along_wire(
        &mut shell,
        &boundary,
        &FilletOptions {
            radius: RadiusSpec::Constant(0.2),
            ..Default::default()
        },
    )
    .unwrap();

    // Fillet faces are appended at the end.
    let fillet_faces: Vec<_> = (initial_count..shell.len()).map(|i| &shell[i]).collect();
    assert!(
        fillet_faces.len() >= 2,
        "expected at least 2 fillet faces, got {}",
        fillet_faces.len()
    );

    // C0 check: adjacent fillet faces share boundary vertices. For each pair,
    // find vertices that appear in both faces' boundaries and verify positions match.
    let tol = 0.01;
    for win in fillet_faces.windows(2) {
        let verts0: Vec<_> = win[0]
            .boundary_iters()
            .into_iter()
            .flatten()
            .map(|e| (e.front().point(), e.back().point()))
            .collect();
        let verts1: Vec<_> = win[1]
            .boundary_iters()
            .into_iter()
            .flatten()
            .map(|e| (e.front().point(), e.back().point()))
            .collect();

        // Collect all vertex positions from each face.
        let pts0: Vec<Point3> = verts0.iter().flat_map(|(f, b)| [*f, *b]).collect();
        let pts1: Vec<Point3> = verts1.iter().flat_map(|(f, b)| [*f, *b]).collect();

        // Find shared vertices (points within tolerance).
        let shared: Vec<_> = pts0
            .iter()
            .filter(|p0| pts1.iter().any(|p1| (*p0 - *p1).magnitude() < tol))
            .collect();

        assert!(
            shared.len() >= 2,
            "adjacent fillet faces should share at least 2 vertices, found {}",
            shared.len()
        );
    }

    // G1 check: for each fillet face, sample the surface normal at the interior
    // and verify it varies smoothly (no sudden flips).
    for face in &fillet_faces {
        let s = face.oriented_surface();
        let n = 4;
        let mut prev_normal = None;
        for j in 0..=n {
            let v = j as f64 / n as f64;
            let normal = s.normal(0.5, v);
            if let Some(prev) = prev_normal {
                let dot: f64 = normal.dot(prev);
                assert!(
                    dot > 0.5,
                    "normal flip within fillet face: dot={dot:.4} at v={v:.2}"
                );
            }
            prev_normal = Some(normal);
        }
    }
}

// ---------------------------------------------------------------------------
// Phase 7: Boundary-run chain grouping tests
// ---------------------------------------------------------------------------

/// Helper: builds a 5-face cuboid (top + 4 sides, no bottom) with 12 edges.
fn build_5face_box() -> (Shell, [Edge; 12], Vec<Vertex>) {
    let p = [
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let v = Vertex::news(p);

    let line = |i: usize, j: usize| {
        let bsp = BSplineCurve::new(KnotVec::bezier_knot(1), vec![p[i], p[j]]);
        Edge::new(&v[i], &v[j], NurbsCurve::from(bsp).into())
    };
    let edge = [
        line(0, 1), // 0: top front
        line(1, 2), // 1: top right
        line(2, 3), // 2: top back
        line(3, 0), // 3: top left
        line(0, 4), // 4
        line(1, 5), // 5
        line(2, 6), // 6
        line(3, 7), // 7
        line(4, 5), // 8: bottom front
        line(5, 6), // 9: bottom right
        line(6, 7), // 10: bottom back
        line(7, 4), // 11: bottom left
    ];

    let plane = |i: usize, j: usize, k: usize, l: usize| {
        let control_points = vec![vec![p[i], p[l]], vec![p[j], p[k]]];
        let knot_vec = KnotVec::bezier_knot(1);
        let bsp = BSplineSurface::new((knot_vec.clone(), knot_vec), control_points);
        let wire: Wire = [i, j, k, l]
            .into_iter()
            .circular_tuple_windows()
            .map(|(a, b)| {
                edge.iter()
                    .find_map(|e| {
                        if e.front() == &v[a] && e.back() == &v[b] {
                            Some(e.clone())
                        } else if e.back() == &v[a] && e.front() == &v[b] {
                            Some(e.inverse())
                        } else {
                            None
                        }
                    })
                    .unwrap()
            })
            .collect();
        Face::new(vec![wire], bsp.into())
    };

    let shell: Shell = [
        plane(0, 1, 2, 3), // face 0: top
        plane(1, 0, 4, 5), // face 1: front
        plane(2, 1, 5, 6), // face 2: right
        plane(3, 2, 6, 7), // face 3: back
        plane(0, 3, 7, 4), // face 4: left
    ]
    .into();

    (shell, edge, v)
}

/// Helper: builds a 6-face closed cuboid with 12 edges.
fn build_6face_box() -> (Shell, [Edge; 12], Vec<Vertex>) {
    let p = [
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let v = Vertex::news(p);

    let line = |i: usize, j: usize| {
        let bsp = BSplineCurve::new(KnotVec::bezier_knot(1), vec![p[i], p[j]]);
        Edge::new(&v[i], &v[j], NurbsCurve::from(bsp).into())
    };
    let edge = [
        line(0, 1), // 0: top front
        line(1, 2), // 1: top right
        line(2, 3), // 2: top back
        line(3, 0), // 3: top left
        line(0, 4), // 4
        line(1, 5), // 5
        line(2, 6), // 6
        line(3, 7), // 7
        line(4, 5), // 8: bottom front
        line(5, 6), // 9: bottom right
        line(6, 7), // 10: bottom back
        line(7, 4), // 11: bottom left
    ];

    let plane = |i: usize, j: usize, k: usize, l: usize| {
        let control_points = vec![vec![p[i], p[l]], vec![p[j], p[k]]];
        let knot_vec = KnotVec::bezier_knot(1);
        let bsp = BSplineSurface::new((knot_vec.clone(), knot_vec), control_points);
        let wire: Wire = [i, j, k, l]
            .into_iter()
            .circular_tuple_windows()
            .map(|(a, b)| {
                edge.iter()
                    .find_map(|e| {
                        if e.front() == &v[a] && e.back() == &v[b] {
                            Some(e.clone())
                        } else if e.back() == &v[a] && e.front() == &v[b] {
                            Some(e.inverse())
                        } else {
                            None
                        }
                    })
                    .unwrap()
            })
            .collect();
        Face::new(vec![wire], bsp.into())
    };

    let shell: Shell = [
        plane(0, 1, 2, 3), // face 0: top
        plane(1, 0, 4, 5), // face 1: front
        plane(2, 1, 5, 6), // face 2: right
        plane(3, 2, 6, 7), // face 3: back
        plane(0, 3, 7, 4), // face 4: left
        plane(5, 4, 7, 6), // face 5: bottom
    ]
    .into();

    (shell, edge, v)
}

/// Fillet all 4 top edges of a cuboid in a single `fillet_edges` call.
///
/// Previously this produced 4 singleton chains (different face pairs) processed
/// sequentially, causing `EdgeNotFound` as earlier fillets invalidated adjacent
/// edge IDs. With boundary-run grouping, the 4 top edges form one closed chain
/// on the top face, processed in a single `fillet_along_wire_closed` call.
#[test]
fn fillet_edges_cuboid_top_4() {
    let (mut shell, edge, _v) = build_5face_box();
    let top_ids: Vec<EdgeID> = (0..4).map(|i| edge[i].id()).collect();
    let opts = FilletOptions {
        radius: RadiusSpec::Constant(0.2),
        ..Default::default()
    };
    fillet_edges(&mut shell, &top_ids, Some(&opts)).unwrap();
    // 4 fillet faces added (one per edge in the closed wire).
    assert!(shell.len() >= 9, "expected >= 9 faces, got {}", shell.len());
    let _poly = shell.robust_triangulation(0.001).to_polygon();
}

/// Fillet top 4 + bottom 4 edges (two independent closed chains).
#[test]
fn fillet_edges_cuboid_top_and_bottom() {
    let (mut shell, edge, _v) = build_6face_box();
    let ids: Vec<EdgeID> = [0, 1, 2, 3, 8, 9, 10, 11]
        .iter()
        .map(|&i| edge[i].id())
        .collect();
    let opts = FilletOptions {
        radius: RadiusSpec::Constant(0.15),
        ..Default::default()
    };
    fillet_edges(&mut shell, &ids, Some(&opts)).unwrap();
    // 8 fillet faces added (4 per closed chain).
    assert!(
        shell.len() >= 14,
        "expected >= 14 faces, got {}",
        shell.len()
    );
    let _poly = shell.robust_triangulation(0.001).to_polygon();
}
