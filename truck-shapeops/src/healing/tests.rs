use super::*;
use std::f64::consts::PI;
use truck_topology::Shell;

#[test]
fn test_split_closed_edges() {
    let vertices = vec![Point2::new(1.0, 0.0)];
    let curve = TrimmedCurve::new(UnitCircle::<Point2>::new(), (0.0, 2.0 * PI));
    let edges = vec![CompressedEdge {
        vertices: (0, 0),
        curve,
    }];
    let faces = vec![
        CompressedFace {
            surface: (),
            orientation: true,
            boundaries: vec![vec![CompressedEdgeIndex {
                index: 0,
                orientation: true,
            }]],
        },
        CompressedFace {
            surface: (),
            orientation: false,
            boundaries: vec![vec![CompressedEdgeIndex {
                index: 0,
                orientation: false,
            }]],
        },
    ];
    let mut shell = CompressedShell {
        vertices,
        edges,
        faces,
    };
    assert!(Shell::extract(shell.clone()).is_err());

    split_closed_edges(&mut shell);
    assert!(Shell::extract(shell.clone()).is_ok());
    assert_eq!(shell.vertices.len(), 2);
    assert_near!(shell.vertices[0], Point2::new(1.0, 0.0));
    assert_near!(shell.vertices[1], Point2::new(-1.0, 0.0));

    assert_eq!(shell.edges.len(), 2);
    assert_eq!(shell.edges[0].vertices, (0, 1));
    assert_eq!(shell.edges[1].vertices, (1, 0));

    assert_eq!(
        shell.faces,
        vec![
            CompressedFace {
                surface: (),
                orientation: true,
                boundaries: vec![vec![
                    CompressedEdgeIndex {
                        index: 0,
                        orientation: true,
                    },
                    CompressedEdgeIndex {
                        index: 1,
                        orientation: true,
                    }
                ]],
            },
            CompressedFace {
                surface: (),
                orientation: false,
                boundaries: vec![vec![
                    CompressedEdgeIndex {
                        index: 1,
                        orientation: false,
                    },
                    CompressedEdgeIndex {
                        index: 0,
                        orientation: false,
                    }
                ]],
            },
        ]
    );
}

#[test]
fn test_split_closed_face_simple_cylinder_case() {
    type Surface = RevolutedCurve<Line<Point3>>;
    #[derive(Clone, Debug, ParametricCurve, BoundedCurve, ParameterDivision1D)]
    enum Curve {
        Line(Line<Point3>),
        Arc(TrimmedCurve<Processor<UnitCircle<Point3>, Matrix4>>),
        PCurve(PCurve<Line<Point2>, Surface>),
    }
    impl From<PCurve<Line<Point2>, Surface>> for Curve {
        fn from(value: PCurve<Line<Point2>, Surface>) -> Self { Self::PCurve(value) }
    }

    let vertices = vec![
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(-1.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(-1.0, 0.0, 1.0),
    ];

    let translate = Matrix4::from_translation(Vector3::unit_z());
    let edges = vec![
        CompressedEdge {
            vertices: (0, 1),
            curve: Curve::Arc(TrimmedCurve::new(
                Processor::new(UnitCircle::new()),
                (0.0, PI),
            )),
        },
        CompressedEdge {
            vertices: (1, 0),
            curve: Curve::Arc(TrimmedCurve::new(
                Processor::new(UnitCircle::new()),
                (PI, 2.0 * PI),
            )),
        },
        CompressedEdge {
            vertices: (0, 2),
            curve: Curve::Line(Line(vertices[0], vertices[2])),
        },
        CompressedEdge {
            vertices: (2, 3),
            curve: Curve::Arc(TrimmedCurve::new(
                Processor::new(UnitCircle::new()).transformed(translate),
                (0.0, PI),
            )),
        },
        CompressedEdge {
            vertices: (3, 2),
            curve: Curve::Arc(TrimmedCurve::new(
                Processor::new(UnitCircle::new()).transformed(translate),
                (PI, 2.0 * PI),
            )),
        },
    ];
    let surface = RevolutedCurve::by_revolution(
        Line(vertices[2], vertices[0]),
        Point3::origin(),
        Vector3::unit_z(),
    );
    let faces = vec![Face {
        surface: surface.clone(),
        boundaries: vec![vec![
            CompressedEdgeIndex {
                index: 1,
                orientation: true,
            },
            CompressedEdgeIndex {
                index: 2,
                orientation: true,
            },
            CompressedEdgeIndex {
                index: 4,
                orientation: false,
            },
            CompressedEdgeIndex {
                index: 3,
                orientation: false,
            },
            CompressedEdgeIndex {
                index: 2,
                orientation: false,
            },
            CompressedEdgeIndex {
                index: 0,
                orientation: true,
            },
        ]],
        orientation: true,
    }];
    let mut shell = CompressedShell {
        vertices,
        edges,
        faces,
    };

    assert!(Shell::extract(shell.clone()).is_err());

    fn sp(surface: &Surface, p: Point3, hint: Option<(f64, f64)>) -> Option<(f64, f64)> {
        surface.search_parameter(p, hint, 10)
    }
    split_closed_faces(&mut shell, 0.01, sp);
    assert!(Shell::extract(shell.clone()).is_ok());
    assert_eq!(shell.vertices.len(), 4);
    assert_eq!(shell.edges.len(), 6);
    assert_eq!(shell.edges[5].vertices, (3, 1));
    let curve0 = &shell.edges[5].curve;
    let curve1 = Line(Point3::new(-1.0, 0.0, 1.0), Point3::new(-1.0, 0.0, 0.0));
    for i in 0..=10 {
        let t = i as f64 / 10.0;
        assert_near!(curve0.subs(t), curve1.subs(t));
    }
    assert_eq!(
        shell.faces,
        vec![
            Face {
                boundaries: vec![vec![
                    CompressedEdgeIndex {
                        index: 2,
                        orientation: true,
                    },
                    CompressedEdgeIndex {
                        index: 4,
                        orientation: false,
                    },
                    CompressedEdgeIndex {
                        index: 5,
                        orientation: true,
                    },
                    CompressedEdgeIndex {
                        index: 1,
                        orientation: true,
                    }
                ]],
                surface: surface.clone(),
                orientation: true,
            },
            Face {
                boundaries: vec![vec![
                    CompressedEdgeIndex {
                        index: 3,
                        orientation: false,
                    },
                    CompressedEdgeIndex {
                        index: 2,
                        orientation: false,
                    },
                    CompressedEdgeIndex {
                        index: 0,
                        orientation: true,
                    },
                    CompressedEdgeIndex {
                        index: 5,
                        orientation: false,
                    },
                ]],
                surface: surface.clone(),
                orientation: true,
            },
        ]
    );
}
