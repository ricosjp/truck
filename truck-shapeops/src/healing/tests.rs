use super::*;
use std::f64::consts::PI;
use truck_geotrait::algo::DefaultSplitParams;
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

    let CompressedShell {
        vertices,
        edges,
        faces,
    } = &shell;
    assert_eq!(vertices.len(), 2);
    assert_near!(vertices[0], Point2::new(1.0, 0.0));
    assert_near!(vertices[1], Point2::new(-1.0, 0.0));

    assert_eq!(edges.len(), 2);
    assert_eq!(edges[0].vertices, (0, 1));
    assert_eq!(edges[1].vertices, (1, 0));

    assert_eq!(
        *faces,
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
    #[derive(
        Clone,
        Debug,
        ParametricCurve,
        BoundedCurve,
        ParameterDivision1D,
        Cut,
        SearchNearestParameterD1,
    )]
    enum Curve {
        Line(Line<Point3>),
        Arc(TrimmedCurve<Processor<UnitCircle<Point3>, Matrix4>>),
        #[allow(clippy::enum_variant_names)]
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
        surface,
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

    split_closed_faces(&mut shell, DefaultSplitParams::new(0.01), sp);

    assert!(Shell::extract(shell.clone()).is_ok());
    let CompressedShell {
        ref vertices,
        ref edges,
        ref mut faces,
    } = shell;
    assert_eq!(vertices.len(), 4);
    assert_eq!(edges.len(), 6);
    assert_eq!(edges[5].vertices, (3, 1));
    let curve0 = &edges[5].curve;
    let curve1 = Line(Point3::new(-1.0, 0.0, 1.0), Point3::new(-1.0, 0.0, 0.0));
    for i in 0..=10 {
        let t = i as f64 / 10.0;
        assert_near!(curve0.subs(t), curve1.subs(t));
    }
    assert_eq!(faces.len(), 2);
    let i = faces
        .iter_mut()
        .position(|face| {
            face.boundaries[0].contains(&CompressedEdgeIndex {
                index: 2,
                orientation: true,
            })
        })
        .unwrap();
    if i == 1 {
        faces.swap(0, 1);
    }
    let i = faces[0].boundaries[0]
        .iter()
        .position(|edge_index| {
            *edge_index
                == CompressedEdgeIndex {
                    index: 2,
                    orientation: true,
                }
        })
        .unwrap();
    faces[0].boundaries[0].rotate_left(i);
    let i = faces[1].boundaries[0]
        .iter()
        .position(|edge_index| {
            *edge_index
                == CompressedEdgeIndex {
                    index: 3,
                    orientation: false,
                }
        })
        .unwrap();
    faces[1].boundaries[0].rotate_left(i);

    assert_eq!(
        *faces,
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
                surface,
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
                surface,
                orientation: true,
            },
        ]
    );
}

#[test]
fn test_split_closed_face_cylinder_with_hole() {
    type Surface = RevolutedCurve<Line<Point3>>;
    #[derive(
        Clone,
        Debug,
        ParametricCurve,
        BoundedCurve,
        ParameterDivision1D,
        Cut,
        SearchNearestParameterD1,
    )]
    enum ParameterCurve {
        Line(Line<Point2>),
        Arc(TrimmedCurve<Processor<UnitCircle<Point2>, Matrix3>>),
    }
    #[derive(
        Clone,
        Debug,
        ParametricCurve,
        BoundedCurve,
        ParameterDivision1D,
        Cut,
        SearchNearestParameterD1,
    )]
    enum Curve {
        Line(Line<Point3>),
        Arc(TrimmedCurve<Processor<UnitCircle<Point3>, Matrix4>>),
        #[allow(clippy::enum_variant_names)]
        PCurve(PCurve<ParameterCurve, Surface>),
    }
    impl From<PCurve<Line<Point2>, Surface>> for Curve {
        fn from(value: PCurve<Line<Point2>, Surface>) -> Self {
            let (line, surface) = value.decompose();
            Self::PCurve(PCurve::new(ParameterCurve::Line(line), surface))
        }
    }

    let vertices = vec![
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(-1.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(-1.0, 0.0, 1.0),
        Point3::new(-1.0, 0.0, 0.25),
        Point3::new(-1.0, 0.0, 0.75),
    ];

    let surface = RevolutedCurve::by_revolution(
        Line(vertices[2], vertices[0]),
        Point3::origin(),
        Vector3::unit_z(),
    );

    let translate = Matrix4::from_translation(Vector3::unit_z());
    let transform = Matrix3::from_translation(Vector2::new(0.5, PI)) * Matrix3::from_scale(0.25);
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
        CompressedEdge {
            vertices: (4, 5),
            curve: Curve::PCurve(PCurve::new(
                ParameterCurve::Arc(TrimmedCurve::new(
                    Processor::new(UnitCircle::new()).transformed(transform),
                    (0.0, PI),
                )),
                surface,
            )),
        },
        CompressedEdge {
            vertices: (5, 4),
            curve: Curve::PCurve(PCurve::new(
                ParameterCurve::Arc(TrimmedCurve::new(
                    Processor::new(UnitCircle::new()).transformed(transform),
                    (PI, 2.0 * PI),
                )),
                surface,
            )),
        },
    ];
    let faces = vec![Face {
        surface,
        boundaries: vec![
            vec![
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
            ],
            vec![
                CompressedEdgeIndex {
                    index: 6,
                    orientation: false,
                },
                CompressedEdgeIndex {
                    index: 5,
                    orientation: false,
                },
            ],
        ],
        orientation: true,
    }];
    let mut shell = CompressedShell {
        vertices,
        edges,
        faces,
    };

    assert!(Shell::extract(shell.clone()).is_err());
    split_closed_faces(&mut shell, DefaultSplitParams::new(0.01), sp);
    assert!(Shell::extract(shell.clone()).is_ok());

    let CompressedShell {
        ref vertices,
        ref edges,
        ref mut faces,
    } = shell;
    assert_eq!(vertices.len(), 6);
    assert_eq!(edges.len(), 9);
    assert_eq!(edges[7].vertices, (3, 5));
    let curve0 = &edges[7].curve;
    let curve1 = Line(Point3::new(-1.0, 0.0, 1.0), Point3::new(-1.0, 0.0, 0.75));
    for i in 0..=10 {
        let t = i as f64 / 10.0;
        assert_near!(curve0.subs(t), curve1.subs(t));
    }
    assert_eq!(edges[8].vertices, (4, 1));
    let curve0 = &edges[8].curve;
    let curve1 = Line(Point3::new(-1.0, 0.0, 0.25), Point3::new(-1.0, 0.0, 0.0));
    for i in 0..=10 {
        let t = i as f64 / 10.0;
        assert_near!(curve0.subs(t), curve1.subs(t));
    }
    assert_eq!(faces.len(), 2);
    let i = faces
        .iter_mut()
        .position(|face| {
            face.boundaries[0].contains(&CompressedEdgeIndex {
                index: 2,
                orientation: true,
            })
        })
        .unwrap();
    if i == 1 {
        faces.swap(0, 1);
    }
    let i = faces[0].boundaries[0]
        .iter()
        .position(|edge_index| {
            *edge_index
                == CompressedEdgeIndex {
                    index: 2,
                    orientation: true,
                }
        })
        .unwrap();
    faces[0].boundaries[0].rotate_left(i);
    let i = faces[1].boundaries[0]
        .iter()
        .position(|edge_index| {
            *edge_index
                == CompressedEdgeIndex {
                    index: 3,
                    orientation: false,
                }
        })
        .unwrap();
    faces[1].boundaries[0].rotate_left(i);

    assert_eq!(
        *faces,
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
                        index: 7,
                        orientation: true,
                    },
                    CompressedEdgeIndex {
                        index: 5,
                        orientation: false,
                    },
                    CompressedEdgeIndex {
                        index: 8,
                        orientation: true,
                    },
                    CompressedEdgeIndex {
                        index: 1,
                        orientation: true,
                    }
                ]],
                surface,
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
                        index: 8,
                        orientation: false,
                    },
                    CompressedEdgeIndex {
                        index: 6,
                        orientation: false,
                    },
                    CompressedEdgeIndex {
                        index: 7,
                        orientation: false,
                    },
                ]],
                surface,
                orientation: true,
            },
        ]
    );
}

#[test]
fn test_split_closed_face_cylinder_with_rotated_hole() {
    #[derive(
        Clone,
        Debug,
        ParametricCurve,
        BoundedCurve,
        ParameterDivision1D,
        Cut,
        SearchNearestParameterD1,
    )]
    enum ParameterCurve {
        Line(Line<Point2>),
        Arc(TrimmedCurve<Processor<UnitCircle<Point2>, Matrix3>>),
    }
    #[derive(
        Clone,
        Debug,
        ParametricCurve,
        BoundedCurve,
        ParameterDivision1D,
        Cut,
        SearchNearestParameterD1,
    )]
    enum Curve {
        Line(Line<Point3>),
        Arc(TrimmedCurve<Processor<UnitCircle<Point3>, Matrix4>>),
        PCurve(PCurve<ParameterCurve, Surface>),
    }
    impl From<PCurve<Line<Point2>, Surface>> for Curve {
        fn from(value: PCurve<Line<Point2>, Surface>) -> Self {
            let (line, surface) = value.decompose();
            Self::PCurve(PCurve::new(ParameterCurve::Line(line), surface))
        }
    }
    type Surface = RevolutedCurve<Line<Point3>>;

    let surface = RevolutedCurve::by_revolution(
        Line(Point3::new(1.0, 0.0, 1.0), Point3::new(1.0, 0.0, 0.0)),
        Point3::origin(),
        Vector3::unit_z(),
    );

    let vertices = vec![
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(-1.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(-1.0, 0.0, 1.0),
        surface.subs(0.5, PI + 0.25),
        surface.subs(0.5, PI - 0.25),
    ];

    let translate = Matrix4::from_translation(Vector3::unit_z());
    let transform = Matrix3::from_translation(Vector2::new(0.5, PI)) * Matrix3::from_scale(0.25);
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
        CompressedEdge {
            vertices: (4, 5),
            curve: Curve::PCurve(PCurve::new(
                ParameterCurve::Arc(TrimmedCurve::new(
                    Processor::new(UnitCircle::new()).transformed(transform),
                    (0.5 * PI, 1.5 * PI),
                )),
                surface,
            )),
        },
        CompressedEdge {
            vertices: (5, 4),
            curve: Curve::PCurve(PCurve::new(
                ParameterCurve::Arc(TrimmedCurve::new(
                    Processor::new(UnitCircle::new()).transformed(transform),
                    (1.5 * PI, 2.5 * PI),
                )),
                surface,
            )),
        },
    ];
    let faces = vec![Face {
        surface,
        boundaries: vec![
            vec![
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
            ],
            vec![
                CompressedEdgeIndex {
                    index: 6,
                    orientation: false,
                },
                CompressedEdgeIndex {
                    index: 5,
                    orientation: false,
                },
            ],
        ],
        orientation: true,
    }];
    let mut shell = CompressedShell {
        vertices,
        edges,
        faces,
    };

    assert!(Shell::extract(shell.clone()).is_err());
    split_closed_faces(&mut shell, DefaultSplitParams::new(0.01), sp);
    assert!(Shell::extract(shell.clone()).is_ok());

    let CompressedShell {
        ref vertices,
        ref edges,
        ref mut faces,
    } = shell;
    assert_eq!(vertices.len(), 8);
    assert_eq!(edges.len(), 11);
    assert_eq!(edges[5].vertices, (4, 7));
    assert_eq!(edges[6].vertices, (5, 6));
    assert_eq!(edges[7].vertices, (6, 4));
    assert_eq!(edges[8].vertices, (7, 5));
    assert_eq!(edges[9].vertices, (3, 7));
    let curve0 = &edges[9].curve;
    let curve1 = Line(Point3::new(-1.0, 0.0, 1.0), Point3::new(-1.0, 0.0, 0.75));
    for i in 0..=10 {
        let t = i as f64 / 10.0;
        assert_near!(curve0.subs(t), curve1.subs(t));
    }
    assert_eq!(edges[10].vertices, (6, 1));
    let curve0 = &edges[10].curve;
    let curve1 = Line(Point3::new(-1.0, 0.0, 0.25), Point3::new(-1.0, 0.0, 0.0));
    for i in 0..=10 {
        let t = i as f64 / 10.0;
        assert_near!(curve0.subs(t), curve1.subs(t));
    }
    assert_eq!(faces.len(), 2);
    let i = faces
        .iter_mut()
        .position(|face| {
            face.boundaries[0].contains(&CompressedEdgeIndex {
                index: 2,
                orientation: true,
            })
        })
        .unwrap();
    if i == 1 {
        faces.swap(0, 1);
    }
    let i = faces[0].boundaries[0]
        .iter()
        .position(|edge_index| {
            *edge_index
                == CompressedEdgeIndex {
                    index: 2,
                    orientation: true,
                }
        })
        .unwrap();
    faces[0].boundaries[0].rotate_left(i);
    let i = faces[1].boundaries[0]
        .iter()
        .position(|edge_index| {
            *edge_index
                == CompressedEdgeIndex {
                    index: 3,
                    orientation: false,
                }
        })
        .unwrap();
    faces[1].boundaries[0].rotate_left(i);

    assert_eq!(
        *faces,
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
                        index: 9,
                        orientation: true,
                    },
                    CompressedEdgeIndex {
                        index: 5,
                        orientation: false,
                    },
                    CompressedEdgeIndex {
                        index: 7,
                        orientation: false,
                    },
                    CompressedEdgeIndex {
                        index: 10,
                        orientation: true,
                    },
                    CompressedEdgeIndex {
                        index: 1,
                        orientation: true,
                    }
                ]],
                surface,
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
                        index: 10,
                        orientation: false,
                    },
                    CompressedEdgeIndex {
                        index: 6,
                        orientation: false,
                    },
                    CompressedEdgeIndex {
                        index: 8,
                        orientation: false,
                    },
                    CompressedEdgeIndex {
                        index: 9,
                        orientation: false,
                    },
                ]],
                surface,
                orientation: true,
            },
        ]
    );
}

#[test]
fn too_simple_cylinder() {
    #[derive(
        Clone,
        Debug,
        ParametricCurve,
        BoundedCurve,
        ParameterDivision1D,
        Cut,
        SearchNearestParameterD1,
    )]
    enum Curve {
        Arc(TrimmedCurve<Processor<UnitCircle<Point3>, Matrix4>>),
        PCurve(PCurve<Line<Point2>, Surface>),
    }
    impl From<PCurve<Line<Point2>, Surface>> for Curve {
        fn from(value: PCurve<Line<Point2>, Surface>) -> Self { Curve::PCurve(value) }
    }
    type Surface = RevolutedCurve<Line<Point3>>;

    let vertices = vec![Point3::new(1.0, 0.0, 0.0), Point3::new(1.0, 0.0, 1.0)];

    let translation = Matrix4::from_translation(Vector3::unit_z());
    let circle0 = TrimmedCurve::new(Processor::new(UnitCircle::new()), (0.0, 2.0 * PI));
    let circle1 = TrimmedCurve::new(
        Processor::new(UnitCircle::new()).transformed(translation),
        (0.0, 2.0 * PI),
    );
    let edges = vec![
        CompressedEdge {
            vertices: (0, 0),
            curve: Curve::Arc(circle0),
        },
        CompressedEdge {
            vertices: (1, 1),
            curve: Curve::Arc(circle1),
        },
    ];

    let surface = RevolutedCurve::by_revolution(
        Line(vertices[1], vertices[0]),
        Point3::origin(),
        Vector3::unit_z(),
    );
    let faces = vec![CompressedFace {
        boundaries: vec![
            vec![CompressedEdgeIndex {
                index: 0,
                orientation: true,
            }],
            vec![CompressedEdgeIndex {
                index: 1,
                orientation: false,
            }],
        ],
        surface,
        orientation: true,
    }];

    let mut shell = CompressedShell {
        vertices,
        edges,
        faces,
    };

    assert!(Shell::extract(shell.clone()).is_err());
    split_closed_edges(&mut shell);
    split_closed_faces(&mut shell, DefaultSplitParams::new(0.01), sp);
    assert!(Shell::extract(shell.clone()).is_ok());

    let CompressedShell {
        ref vertices,
        ref edges,
        ref mut faces,
    } = shell;

    assert_eq!(vertices.len(), 4);
    assert_near!(vertices[2], Point3::new(-1.0, 0.0, 0.0));
    assert_near!(vertices[3], Point3::new(-1.0, 0.0, 1.0));

    assert_eq!(edges.len(), 6);
    assert_eq!(edges[0].vertices, (0, 2));
    assert_eq!(edges[1].vertices, (1, 3));
    assert_eq!(edges[2].vertices, (2, 0));
    assert_eq!(edges[3].vertices, (3, 1));
    assert_eq!(edges[4].vertices, (0, 1));
    assert_eq!(edges[5].vertices, (2, 3));

    assert_eq!(faces.len(), 2);
    let i = faces
        .iter_mut()
        .position(|face| {
            face.boundaries[0].contains(&CompressedEdgeIndex {
                index: 0,
                orientation: true,
            })
        })
        .unwrap();
    if i == 1 {
        faces.swap(0, 1);
    }
    let i = faces[0].boundaries[0]
        .iter()
        .position(|edge_index| {
            *edge_index
                == CompressedEdgeIndex {
                    index: 0,
                    orientation: true,
                }
        })
        .unwrap();
    faces[0].boundaries[0].rotate_left(i);
    let i = faces[1].boundaries[0]
        .iter()
        .position(|edge_index| {
            *edge_index
                == CompressedEdgeIndex {
                    index: 2,
                    orientation: true,
                }
        })
        .unwrap();
    faces[1].boundaries[0].rotate_left(i);

    assert_eq!(
        shell.faces,
        vec![
            Face {
                boundaries: vec![vec![
                    CompressedEdgeIndex {
                        index: 0,
                        orientation: true,
                    },
                    CompressedEdgeIndex {
                        index: 5,
                        orientation: true,
                    },
                    CompressedEdgeIndex {
                        index: 1,
                        orientation: false,
                    },
                    CompressedEdgeIndex {
                        index: 4,
                        orientation: false,
                    },
                ]],
                surface,
                orientation: true,
            },
            Face {
                boundaries: vec![vec![
                    CompressedEdgeIndex {
                        index: 2,
                        orientation: true,
                    },
                    CompressedEdgeIndex {
                        index: 4,
                        orientation: true,
                    },
                    CompressedEdgeIndex {
                        index: 3,
                        orientation: false,
                    },
                    CompressedEdgeIndex {
                        index: 5,
                        orientation: false,
                    },
                ]],
                surface,
                orientation: true,
            },
        ]
    );
}

#[test]
fn double_closed_boundary_cylinder() {
    #[derive(
        Clone,
        Debug,
        ParametricCurve,
        BoundedCurve,
        ParameterDivision1D,
        Cut,
        SearchNearestParameterD1,
    )]
    enum ParameterCurve {
        Line(Line<Point2>),
        Arc(TrimmedCurve<Processor<UnitCircle<Point2>, Matrix3>>),
    }
    #[derive(
        Clone,
        Debug,
        ParametricCurve,
        BoundedCurve,
        ParameterDivision1D,
        Cut,
        SearchNearestParameterD1,
    )]
    enum Curve {
        Line(Line<Point3>),
        Arc(TrimmedCurve<Processor<UnitCircle<Point3>, Matrix4>>),
        #[allow(clippy::enum_variant_names)]
        PCurve(PCurve<ParameterCurve, Surface>),
    }
    impl From<PCurve<Line<Point2>, Surface>> for Curve {
        fn from(value: PCurve<Line<Point2>, Surface>) -> Self {
            let (line, surface) = value.decompose();
            Self::PCurve(PCurve::new(ParameterCurve::Line(line), surface))
        }
    }
    type Surface = RevolutedCurve<Line<Point3>>;

    let vertices = vec![
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 0.25),
        Point3::new(-1.0, 0.0, 0.25),
    ];
    let surface = RevolutedCurve::by_revolution(
        Line(vertices[1], vertices[0]),
        Point3::origin(),
        Vector3::unit_z(),
    );
    let edges = vec![
        CompressedEdge {
            vertices: (0, 0),
            curve: Curve::Arc(TrimmedCurve::new(
                Processor::new(UnitCircle::new()),
                (0.0, 2.0 * PI),
            )),
        },
        CompressedEdge {
            vertices: (1, 1),
            curve: Curve::Arc(TrimmedCurve::new(
                Processor::new(UnitCircle::new())
                    .transformed(Matrix4::from_translation(Vector3::unit_z())),
                (0.0, 2.0 * PI),
            )),
        },
        CompressedEdge {
            vertices: (2, 2),
            curve: Curve::PCurve(PCurve::new(
                ParameterCurve::Arc(TrimmedCurve::new(
                    Processor::new(UnitCircle::new()).transformed(
                        Matrix3::from_translation(Vector2::new(0.5, 0.0))
                            * Matrix3::from_scale(0.25),
                    ),
                    (0.0, 2.0 * PI),
                )),
                surface,
            )),
        },
        CompressedEdge {
            vertices: (3, 3),
            curve: Curve::PCurve(PCurve::new(
                ParameterCurve::Arc(TrimmedCurve::new(
                    Processor::new(UnitCircle::new()).transformed(
                        Matrix3::from_translation(Vector2::new(0.5, PI))
                            * Matrix3::from_scale(0.25),
                    ),
                    (0.0, 2.0 * PI),
                )),
                surface,
            )),
        },
    ];
    let faces = vec![CompressedFace {
        boundaries: vec![
            vec![CompressedEdgeIndex {
                index: 0,
                orientation: true,
            }],
            vec![CompressedEdgeIndex {
                index: 1,
                orientation: false,
            }],
            vec![CompressedEdgeIndex {
                index: 2,
                orientation: false,
            }],
            vec![CompressedEdgeIndex {
                index: 3,
                orientation: false,
            }],
        ],
        surface,
        orientation: true,
    }];
    let mut shell = CompressedShell {
        vertices,
        edges,
        faces,
    };

    assert!(Shell::extract(shell.clone()).is_err());
    split_closed_edges(&mut shell);
    split_closed_faces(&mut shell, DefaultSplitParams::new(0.05), sp);
    assert!(Shell::extract(shell.clone()).is_ok());

    let CompressedShell {
        ref vertices,
        ref edges,
        ref mut faces,
    } = shell;

    assert_eq!(vertices.len(), 8);
    assert_near!(vertices[0], Point3::new(1.0, 0.0, 0.0));
    assert_near!(vertices[1], Point3::new(1.0, 0.0, 1.0));
    assert_near!(vertices[2], Point3::new(1.0, 0.0, 0.25));
    assert_near!(vertices[3], Point3::new(-1.0, 0.0, 0.25));
    assert_near!(vertices[4], Point3::new(-1.0, 0.0, 0.0));
    assert_near!(vertices[5], Point3::new(-1.0, 0.0, 1.0));
    assert_near!(vertices[6], Point3::new(1.0, 0.0, 0.75));
    assert_near!(vertices[7], Point3::new(-1.0, 0.0, 0.75));

    assert_eq!(edges.len(), 12);
    assert_eq!(edges[0].vertices, (0, 4));
    assert_eq!(edges[1].vertices, (1, 5));
    assert_eq!(edges[2].vertices, (2, 6));
    assert_eq!(edges[3].vertices, (3, 7));
    assert_eq!(edges[4].vertices, (4, 0));
    assert_eq!(edges[5].vertices, (5, 1));
    assert_eq!(edges[6].vertices, (6, 2));
    assert_eq!(edges[7].vertices, (7, 3));
    assert_eq!(edges[8].vertices, (0, 2));
    assert_eq!(edges[9].vertices, (6, 1));
    assert_eq!(edges[10].vertices, (4, 3));
    assert_eq!(edges[11].vertices, (7, 5));

    let i = faces
        .iter_mut()
        .position(|face| {
            face.boundaries[0].contains(&CompressedEdgeIndex {
                index: 0,
                orientation: true,
            })
        })
        .unwrap();
    if i == 1 {
        faces.swap(0, 1);
    }
    let i = faces[0].boundaries[0]
        .iter()
        .position(|edge_index| {
            *edge_index
                == CompressedEdgeIndex {
                    index: 0,
                    orientation: true,
                }
        })
        .unwrap();
    faces[0].boundaries[0].rotate_left(i);
    let i = faces[1].boundaries[0]
        .iter()
        .position(|edge_index| {
            *edge_index
                == CompressedEdgeIndex {
                    index: 4,
                    orientation: true,
                }
        })
        .unwrap();
    faces[1].boundaries[0].rotate_left(i);

    assert_eq!(
        *faces,
        vec![
            CompressedFace {
                boundaries: vec![vec![
                    CompressedEdgeIndex {
                        index: 0,
                        orientation: true,
                    },
                    CompressedEdgeIndex {
                        index: 10,
                        orientation: true,
                    },
                    CompressedEdgeIndex {
                        index: 7,
                        orientation: false,
                    },
                    CompressedEdgeIndex {
                        index: 11,
                        orientation: true,
                    },
                    CompressedEdgeIndex {
                        index: 1,
                        orientation: false,
                    },
                    CompressedEdgeIndex {
                        index: 9,
                        orientation: false,
                    },
                    CompressedEdgeIndex {
                        index: 2,
                        orientation: false,
                    },
                    CompressedEdgeIndex {
                        index: 8,
                        orientation: false,
                    },
                ]],
                surface,
                orientation: true,
            },
            CompressedFace {
                boundaries: vec![vec![
                    CompressedEdgeIndex {
                        index: 4,
                        orientation: true,
                    },
                    CompressedEdgeIndex {
                        index: 8,
                        orientation: true,
                    },
                    CompressedEdgeIndex {
                        index: 6,
                        orientation: false,
                    },
                    CompressedEdgeIndex {
                        index: 9,
                        orientation: true,
                    },
                    CompressedEdgeIndex {
                        index: 5,
                        orientation: false,
                    },
                    CompressedEdgeIndex {
                        index: 11,
                        orientation: false,
                    },
                    CompressedEdgeIndex {
                        index: 3,
                        orientation: false,
                    },
                    CompressedEdgeIndex {
                        index: 10,
                        orientation: false,
                    },
                ]],
                surface,
                orientation: true,
            },
        ]
    );
}

#[test]
fn many_closed_boundary_cylinder() {
    #[derive(
        Clone,
        Debug,
        ParametricCurve,
        BoundedCurve,
        ParameterDivision1D,
        Cut,
        SearchNearestParameterD1,
    )]
    enum ParameterCurve {
        Line(Line<Point2>),
        Arc(TrimmedCurve<Processor<UnitCircle<Point2>, Matrix3>>),
    }
    #[derive(
        Clone,
        Debug,
        ParametricCurve,
        BoundedCurve,
        ParameterDivision1D,
        Cut,
        SearchNearestParameterD1,
    )]
    enum Curve {
        Line(Line<Point3>),
        Arc(TrimmedCurve<Processor<UnitCircle<Point3>, Matrix4>>),
        PCurve(PCurve<ParameterCurve, Surface>),
    }
    impl From<PCurve<Line<Point2>, Surface>> for Curve {
        fn from(value: PCurve<Line<Point2>, Surface>) -> Self {
            let (line, surface) = value.decompose();
            Self::PCurve(PCurve::new(ParameterCurve::Line(line), surface))
        }
    }
    type Surface = RevolutedCurve<Line<Point3>>;

    const NUM_OF_CIRCLES: usize = 10;

    let vertices = {
        let mut vertices = vec![Point3::new(1.0, 0.0, 0.0), Point3::new(1.0, 0.0, 1.0)];
        vertices.extend((0..NUM_OF_CIRCLES).map(|i| {
            let t = 2.0 * PI * i as f64 / NUM_OF_CIRCLES as f64;
            Point3::new(f64::cos(t), f64::sin(t), 0.4)
        }));
        vertices
    };
    let surface = RevolutedCurve::by_revolution(
        Line(vertices[1], vertices[0]),
        Point3::origin(),
        Vector3::unit_z(),
    );
    let edges = {
        let mut edges = vec![
            CompressedEdge {
                vertices: (0, 0),
                curve: Curve::Arc(TrimmedCurve::new(
                    Processor::new(UnitCircle::new()),
                    (0.0, 2.0 * PI),
                )),
            },
            CompressedEdge {
                vertices: (1, 1),
                curve: Curve::Arc(TrimmedCurve::new(
                    Processor::new(UnitCircle::new())
                        .transformed(Matrix4::from_translation(Vector3::unit_z())),
                    (0.0, 2.0 * PI),
                )),
            },
        ];
        edges.extend((0..NUM_OF_CIRCLES).map(|i| {
            let t = 2.0 * PI * i as f64 / NUM_OF_CIRCLES as f64;
            CompressedEdge {
                vertices: (2 + i, 2 + i),
                curve: Curve::PCurve(PCurve::new(
                    ParameterCurve::Arc(TrimmedCurve::new(
                        Processor::new(UnitCircle::new()).transformed(
                            Matrix3::from_translation(Vector2::new(0.5, t))
                                * Matrix3::from_scale(0.1),
                        ),
                        (0.0, 2.0 * PI),
                    )),
                    surface,
                )),
            }
        }));
        edges
    };
    let mut boundaries = vec![vec![CompressedEdgeIndex {
        index: 0,
        orientation: true,
    }]];
    boundaries.extend((0..=NUM_OF_CIRCLES).map(|i| {
        vec![CompressedEdgeIndex {
            index: 1 + i,
            orientation: false,
        }]
    }));
    let faces = vec![CompressedFace {
        boundaries,
        surface,
        orientation: true,
    }];
    let mut shell = CompressedShell {
        vertices,
        edges,
        faces,
    };

    assert!(Shell::extract(shell.clone()).is_err());
    split_closed_edges(&mut shell);
    split_closed_faces(&mut shell, DefaultSplitParams::new(0.05), sp);
    assert!(Shell::extract(shell.clone()).is_ok());

    let CompressedShell {
        ref vertices,
        ref edges,
        ref mut faces,
    } = shell;

    assert_eq!(vertices.len(), (2 + NUM_OF_CIRCLES) * 2);
    assert_near!(vertices[0], Point3::new(1.0, 0.0, 0.0));
    assert_near!(vertices[1], Point3::new(1.0, 0.0, 1.0));
    assert_near!(vertices[NUM_OF_CIRCLES + 2], Point3::new(-1.0, 0.0, 0.0));
    assert_near!(vertices[NUM_OF_CIRCLES + 3], Point3::new(-1.0, 0.0, 1.0));
    (0..NUM_OF_CIRCLES).for_each(|i| {
        let t = 2.0 * PI * i as f64 / NUM_OF_CIRCLES as f64;
        assert_near!(vertices[2 + i], Point3::new(f64::cos(t), f64::sin(t), 0.4));
        assert_near!(
            vertices[4 + i + NUM_OF_CIRCLES],
            Point3::new(f64::cos(t), f64::sin(t), 0.6)
        );
    });

    assert_eq!(edges.len(), 8 + NUM_OF_CIRCLES * 2);
    (0..NUM_OF_CIRCLES + 2).for_each(|i| {
        let j = i + 2 + NUM_OF_CIRCLES;
        assert_eq!(edges[i].vertices, (i, j));
        assert_eq!(edges[j].vertices, (j, i));
    });
    let i = 4 + NUM_OF_CIRCLES * 2;
    assert_eq!(edges[i].vertices, (0, 2));
    assert_eq!(edges[i + 1].vertices, (4 + NUM_OF_CIRCLES, 1));
    assert_eq!(
        edges[i + 2].vertices,
        (2 + NUM_OF_CIRCLES, 2 + NUM_OF_CIRCLES / 2)
    );
    assert_eq!(
        edges[i + 3].vertices,
        (4 + NUM_OF_CIRCLES * 3 / 2, 3 + NUM_OF_CIRCLES)
    );

    let i = faces
        .iter_mut()
        .position(|face| {
            face.boundaries[0].contains(&CompressedEdgeIndex {
                index: 0,
                orientation: true,
            })
        })
        .unwrap();
    if i == 1 {
        faces.swap(0, 1);
    }

    assert_eq!(faces[0].boundaries.len(), NUM_OF_CIRCLES / 2);
    assert_eq!(faces[1].boundaries.len(), NUM_OF_CIRCLES / 2);

    (1..NUM_OF_CIRCLES / 2).for_each(|i| {
        (0..=1).for_each(|fid| {
            let eid = faces[fid].boundaries[i][0].index;
            let p = vertices[edges[eid].vertices.0];
            assert!(f64::signum(p.y) == f64::signum(f64::powi(-1.0, fid as i32)));
        });
    });
}

fn sp<S>(surface: &S, p: Point3, hint: Option<(f64, f64)>) -> Option<(f64, f64)>
where S: SearchParameter<D2, Point = Point3> {
    surface.search_parameter(p, hint, 10)
}

#[test]
#[cfg(feature = "step-test")]
fn step_import() {
    use truck_stepio::r#in::*;
    const STEP_DIRECTORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../resources/step/");
    const STEP_FILES: &[&str] = &[
        "occt-cylinder.step",
        "occt-cone.step",
        "abc-0006.step",
        "abc-0008.step",
        "abc-0035.step",
    ];

    STEP_FILES.iter().for_each(|file_name| {
        println!("{file_name}");
        let path = [STEP_DIRECTORY, file_name].concat();
        let step_string = std::fs::read_to_string(path).unwrap();
        let table = Table::from_step(&step_string).unwrap();
        table.shell.values().cloned().for_each(|step_shell| {
            let mut cshell = table.to_compressed_shell(&step_shell).unwrap();
            cshell.robust_split_closed_edges_and_faces(DefaultSplitParams::new(0.05));
            truck_topology::Shell::extract(cshell).unwrap();
        });
    });
}
