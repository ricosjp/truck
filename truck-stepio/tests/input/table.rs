use ruststep::{
    ast::{DataSection, Name},
    primitive::Logical,
    tables::PlaceHolder,
};
use std::{collections::HashMap, str::FromStr};
use truck_stepio::r#in::*;

#[test]
fn read() {
    let data_section = DataSection::from_str(
        "DATA;
#1 = CARTESIAN_POINT('Point', (0.1, 0.2, 0.3));
#2 = DIRECTION('Dir', (1.0, 2.0, 3.0));
#3 = VECTOR('Vector', #2, 2.0);
#4 = PLACEMENT('Placement', #1);
#5 = AXIS1_PLACEMENT('Axis1Placement_0', #1, $);
#6 = AXIS1_PLACEMENT('Axis1Placement_1', #1, #2);
#7 = AXIS2_PLACEMENT_2D('Axis2Placement2d_0', #1, $);
#8 = AXIS2_PLACEMENT_2D('Axis2Placement2d_1', #1, #2);
#9 = AXIS2_PLACEMENT_3D('Axis2Placement3d_0', #1, $, $);
#10 = AXIS2_PLACEMENT_3D('Axis2Placement3d_1', #1, #2, $);
#11 = AXIS2_PLACEMENT_3D('Axis2Placement3d_2', #1, $, #2);
#12 = AXIS2_PLACEMENT_3D('Axis2Placement3d_3', #1, #2, #2);

#13 = LINE('Line', #1, #3);
#14 = POLYLINE('Polyline', (#1, #1, #1, #1, #1, #1));
#15 = B_SPLINE_CURVE_WITH_KNOTS('BSplineCurveWithKnots', 2, (#1, #1, #1, #1, #1), .UNSPECIFIED.,
    .U., .U., (3, 1, 3), (0.0, 0.5, 1.0), .UNSPECIFIED.);
#16 = BEZIER_CURVE('BezierCurve', 2, (#1, #1, #1, #1, #1), .UNSPECIFIED., .U., .U.);
#17 = QUASI_UNIFORM_CURVE('QuasiUniformCurve', 2, (#1, #1, #1, #1, #1), .UNSPECIFIED., .U., .U.);
#18 = UNIFORM_CURVE('UniformCurve', 2, (#1, #1, #1, #1, #1), .UNSPECIFIED., .U., .U.);
#19 = (
    BOUNDED_CURVE()
    B_SPLINE_CURVE(2, (#1, #1, #1, #1, #1), .UNSPECIFIED., .U., .U.)
    B_SPLINE_CURVE_WITH_KNOTS((3, 1, 3), (0.0, 0.5, 1.0), .UNSPECIFIED.)
    CURVE()
    GEOMETRIC_REPRESENTATION_ITEM()
    RATIONAL_B_SPLINE_CURVE((1.0, 2.0, 3.0, 4.0, 5.0))
    REPRESENTATION_ITEM('RationalBSplineCurve')
);
#20 = CIRCLE('Circle', #7, 10.0);
#21 = PLANE('Plane', #9);
#22 = B_SPLINE_SURFACE_WITH_KNOTS(
    'BSplineSurfaceWithKnots', 2, 2, ((#1, #1, #1), (#1, #1, #1), (#1, #1, #1)), .UNSPECIFIED., .U., .U., .U.,
    (3, 3), (3, 3), (0.0, 1.0), (0.0, 1.0), .UNSPECIFIED.
);
#23 = SURFACE_OF_REVOLUTION('SurfaceOfRevolution', #20, #5);
#24 = SPHERICAL_SURFACE('SphericalSurface', #9, 5.0);
#25 = CYLINDRICAL_SURFACE('CylindricalSurface', #9, 5.0);
#26 = UNIFORM_SURFACE('UniformSurface', 2, 2, ((#1, #1, #1), (#1, #1, #1), (#1, #1, #1)), .UNSPECIFIED., .U., .U., .U.);
#27 = QUASI_UNIFORM_SURFACE('QuasiUniformSurface', 2, 2, ((#1, #1, #1), (#1, #1, #1), (#1, #1, #1)), .UNSPECIFIED., .U., .U., .U.);
#28 = BEZIER_SURFACE('BezierSurface', 2, 2, ((#1, #1, #1), (#1, #1, #1), (#1, #1, #1)), .UNSPECIFIED., .U., .U., .U.);
#29 = TOROIDAL_SURFACE('ToroidalSurface', #9, 5.0, 2.0);
#30 = CONICAL_SURFACE('ConicalSurface', #9, 5.0, 0.5);
#31 = SURFACE_OF_LINEAR_EXTRUSION('SurfaceOfLinearExtrusion', #20, #3);

#33 = ( GEOMETRIC_REPRESENTATION_CONTEXT(2) PARAMETRIC_REPRESENTATION_CONTEXT() REPRESENTATION_CONTEXT('2D SPACE','') );
#34 = DEFINITIONAL_REPRESENTATION('DefinitionalRepresentation', (#13), #32);
#35 = PCURVE('Pcurve', #24, #33);

#36 = SURFACE_CURVE('SurfaceCurve', #20, (#21, #29), .CURVE_3D.);

#100 = VERTEX_POINT('VertexPoint', #1);
#101 = EDGE_CURVE('EdgeCurve', #100, #100, #13, .T.);
#102 = ORIENTED_EDGE('OrientedEdge', *, *, #101, .F.);
#103 = EDGE_LOOP('EdgeLoop', (#101, #102));
#104 = FACE_BOUND('FaceBound', #103, .T.);
#105 = FACE_OUTER_BOUND('FaceOuterBound', #103, .F.);
#106 = FACE_SURFACE('FaceSurface', (#104, #105), #21, .T.);
#107 = ADVANCED_FACE('AdvancedFace', (#104, #105), #21, .T.);
#108 = ORIENTED_FACE('OrientedFace', *, #106, .F.);
#109 = OPEN_SHELL('OpenShell', (#107, #108));
#110 = CLOSED_SHELL('ClosedShell', (#107, #108));
#111 = ORIENTED_OPEN_SHELL('OrientedOpenShell', *, #109, .F.);
#112 = ORIENTED_CLOSED_SHELL('OrientedClosedShell', *, #110, .T.);

#999 = HOGE('Dummy', #110, 3);
ENDSEC;
",
    )
    .unwrap();
    let table = Table::from_data_section(&data_section);
    let ans_table = Table {
        cartesian_point: HashMap::from_iter(vec![(
            1,
            CartesianPointHolder {
                label: "Point".to_string(),
                coordinates: vec![0.1, 0.2, 0.3],
            },
        )]),
        direction: HashMap::from_iter(vec![(
            2,
            DirectionHolder {
                label: "Dir".to_string(),
                direction_ratios: vec![1.0, 2.0, 3.0],
            },
        )]),
        vector: HashMap::from_iter(vec![(
            3,
            VectorHolder {
                label: "Vector".to_string(),
                orientation: PlaceHolder::Ref(Name::Entity(2)),
                magnitude: 2.0,
            },
        )]),
        placement: HashMap::from_iter(vec![(
            4,
            PlacementHolder {
                label: "Placement".to_string(),
                location: PlaceHolder::Ref(Name::Entity(1)),
            },
        )]),
        axis1_placement: HashMap::from_iter(vec![
            (
                5,
                Axis1PlacementHolder {
                    label: "Axis1Placement_0".to_string(),
                    location: PlaceHolder::Ref(Name::Entity(1)),
                    direction: None,
                },
            ),
            (
                6,
                Axis1PlacementHolder {
                    label: "Axis1Placement_1".to_string(),
                    location: PlaceHolder::Ref(Name::Entity(1)),
                    direction: Some(PlaceHolder::Ref(Name::Entity(2))),
                },
            ),
        ]),
        axis2_placement_2d: HashMap::from_iter(vec![
            (
                7,
                Axis2Placement2dHolder {
                    label: "Axis2Placement2d_0".to_string(),
                    location: PlaceHolder::Ref(Name::Entity(1)),
                    ref_direction: None,
                },
            ),
            (
                8,
                Axis2Placement2dHolder {
                    label: "Axis2Placement2d_1".to_string(),
                    location: PlaceHolder::Ref(Name::Entity(1)),
                    ref_direction: Some(PlaceHolder::Ref(Name::Entity(2))),
                },
            ),
        ]),
        axis2_placement_3d: HashMap::from_iter(vec![
            (
                9,
                Axis2Placement3dHolder {
                    label: "Axis2Placement3d_0".to_string(),
                    location: PlaceHolder::Ref(Name::Entity(1)),
                    axis: None,
                    ref_direction: None,
                },
            ),
            (
                10,
                Axis2Placement3dHolder {
                    label: "Axis2Placement3d_1".to_string(),
                    location: PlaceHolder::Ref(Name::Entity(1)),
                    axis: Some(PlaceHolder::Ref(Name::Entity(2))),
                    ref_direction: None,
                },
            ),
            (
                11,
                Axis2Placement3dHolder {
                    label: "Axis2Placement3d_2".to_string(),
                    location: PlaceHolder::Ref(Name::Entity(1)),
                    axis: None,
                    ref_direction: Some(PlaceHolder::Ref(Name::Entity(2))),
                },
            ),
            (
                12,
                Axis2Placement3dHolder {
                    label: "Axis2Placement3d_3".to_string(),
                    location: PlaceHolder::Ref(Name::Entity(1)),
                    axis: Some(PlaceHolder::Ref(Name::Entity(2))),
                    ref_direction: Some(PlaceHolder::Ref(Name::Entity(2))),
                },
            ),
        ]),
        line: HashMap::from_iter(vec![(
            13,
            LineHolder {
                label: "Line".to_string(),
                pnt: PlaceHolder::Ref(Name::Entity(1)),
                dir: PlaceHolder::Ref(Name::Entity(3)),
            },
        )]),
        polyline: HashMap::from_iter(vec![(
            14,
            PolylineHolder {
                label: "Polyline".to_string(),
                points: vec![
                    PlaceHolder::Ref(Name::Entity(1)),
                    PlaceHolder::Ref(Name::Entity(1)),
                    PlaceHolder::Ref(Name::Entity(1)),
                    PlaceHolder::Ref(Name::Entity(1)),
                    PlaceHolder::Ref(Name::Entity(1)),
                    PlaceHolder::Ref(Name::Entity(1)),
                ],
            },
        )]),
        b_spline_curve_with_knots: HashMap::from_iter(vec![(
            15,
            BSplineCurveWithKnotsHolder {
                label: "BSplineCurveWithKnots".to_string(),
                degree: 2,
                control_points_list: vec![
                    PlaceHolder::Ref(Name::Entity(1)),
                    PlaceHolder::Ref(Name::Entity(1)),
                    PlaceHolder::Ref(Name::Entity(1)),
                    PlaceHolder::Ref(Name::Entity(1)),
                    PlaceHolder::Ref(Name::Entity(1)),
                ],
                curve_form: BSplineCurveForm::Unspecified,
                closed_curve: Logical::Unknown,
                self_intersect: Logical::Unknown,
                knot_multiplicities: vec![3, 1, 3],
                knots: vec![0.0, 0.5, 1.0],
                knot_spec: KnotType::Unspecified,
            },
        )]),
        bezier_curve: HashMap::from_iter(vec![(
            16,
            BezierCurveHolder {
                label: "BezierCurve".to_string(),
                degree: 2,
                control_points_list: vec![
                    PlaceHolder::Ref(Name::Entity(1)),
                    PlaceHolder::Ref(Name::Entity(1)),
                    PlaceHolder::Ref(Name::Entity(1)),
                    PlaceHolder::Ref(Name::Entity(1)),
                    PlaceHolder::Ref(Name::Entity(1)),
                ],
                curve_form: BSplineCurveForm::Unspecified,
                closed_curve: Logical::Unknown,
                self_intersect: Logical::Unknown,
            },
        )]),
        quasi_uniform_curve: HashMap::from_iter(vec![(
            17,
            QuasiUniformCurveHolder {
                label: "QuasiUniformCurve".to_string(),
                degree: 2,
                control_points_list: vec![
                    PlaceHolder::Ref(Name::Entity(1)),
                    PlaceHolder::Ref(Name::Entity(1)),
                    PlaceHolder::Ref(Name::Entity(1)),
                    PlaceHolder::Ref(Name::Entity(1)),
                    PlaceHolder::Ref(Name::Entity(1)),
                ],
                curve_form: BSplineCurveForm::Unspecified,
                closed_curve: Logical::Unknown,
                self_intersect: Logical::Unknown,
            },
        )]),
        uniform_curve: HashMap::from_iter(vec![(
            18,
            UniformCurveHolder {
                label: "UniformCurve".to_string(),
                degree: 2,
                control_points_list: vec![
                    PlaceHolder::Ref(Name::Entity(1)),
                    PlaceHolder::Ref(Name::Entity(1)),
                    PlaceHolder::Ref(Name::Entity(1)),
                    PlaceHolder::Ref(Name::Entity(1)),
                    PlaceHolder::Ref(Name::Entity(1)),
                ],
                curve_form: BSplineCurveForm::Unspecified,
                closed_curve: Logical::Unknown,
                self_intersect: Logical::Unknown,
            },
        )]),
        rational_b_spline_curve: HashMap::from_iter(vec![(
            19,
            RationalBSplineCurveHolder {
                non_rational_b_spline_curve: PlaceHolder::Owned(
                    NonRationalBSplineCurveHolder::BSplineCurveWithKnots(
                        BSplineCurveWithKnotsHolder {
                            label: "RationalBSplineCurve".to_string(),
                            degree: 2,
                            control_points_list: vec![
                                PlaceHolder::Ref(Name::Entity(1)),
                                PlaceHolder::Ref(Name::Entity(1)),
                                PlaceHolder::Ref(Name::Entity(1)),
                                PlaceHolder::Ref(Name::Entity(1)),
                                PlaceHolder::Ref(Name::Entity(1)),
                            ],
                            curve_form: BSplineCurveForm::Unspecified,
                            closed_curve: Logical::Unknown,
                            self_intersect: Logical::Unknown,
                            knot_multiplicities: vec![3, 1, 3],
                            knots: vec![0.0, 0.5, 1.0],
                            knot_spec: KnotType::Unspecified,
                        },
                    ),
                ),
                weights_data: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            },
        )]),
        circle: HashMap::from_iter(vec![(
            20,
            CircleHolder {
                label: "Circle".to_string(),
                position: PlaceHolder::Ref(Name::Entity(7)),
                radius: 10.0,
            },
        )]),
        definitional_representation: HashMap::from_iter(vec![(
            34,
            DefinitionalRepresentationHolder {
                label: "DefinitionalRepresentation".to_string(),
                representation_item: vec![PlaceHolder::Ref(Name::Entity(13))],
                contex_of_items: PlaceHolder::Ref(Name::Entity(32)),
            }
        )]),
        pcurve: HashMap::from_iter(vec![(
            35,
            PcurveHolder {
                label: "Pcurve".to_string(),
                basis_surface: PlaceHolder::Ref(Name::Entity(24)),
                reference_to_curve: PlaceHolder::Ref(Name::Entity(33)),
            }
        )]),
        surface_curve: HashMap::from_iter(vec![(
            36,
            SurfaceCurveHolder {
                label: "SurfaceCurve".to_string(),
                curve_3d: PlaceHolder::Ref(Name::Entity(20)),
                associated_geometry: vec![
                    PlaceHolder::Ref(Name::Entity(21)),
                    PlaceHolder::Ref(Name::Entity(29)),
                ],
                master_representation: PreferredSurfaceCurveRepresentation::Curve3D
            }
        )]),
        plane: HashMap::from_iter(vec![(
            21,
            PlaneHolder {
                label: "Plane".to_string(),
                position: PlaceHolder::Ref(Name::Entity(9)),
            },
        )]),
        b_spline_surface_with_knots: HashMap::from_iter(vec![(
            22,
            BSplineSurfaceWithKnotsHolder {
                label: "BSplineSurfaceWithKnots".to_string(),
                u_degree: 2,
                v_degree: 2,
                control_points_list: vec![
                    vec![
                        PlaceHolder::Ref(Name::Entity(1)),
                        PlaceHolder::Ref(Name::Entity(1)),
                        PlaceHolder::Ref(Name::Entity(1)),
                    ],
                    vec![
                        PlaceHolder::Ref(Name::Entity(1)),
                        PlaceHolder::Ref(Name::Entity(1)),
                        PlaceHolder::Ref(Name::Entity(1)),
                    ],
                    vec![
                        PlaceHolder::Ref(Name::Entity(1)),
                        PlaceHolder::Ref(Name::Entity(1)),
                        PlaceHolder::Ref(Name::Entity(1)),
                    ],
                ],
                surface_form: BSplineSurfaceForm::Unspecified,
                u_closed: Logical::Unknown,
                v_closed: Logical::Unknown,
                self_intersect: Logical::Unknown,
                u_multiplicities: vec![3, 3],
                v_multiplicities: vec![3, 3],
                u_knots: vec![0.0, 1.0],
                v_knots: vec![0.0, 1.0],
                knot_spec: KnotType::Unspecified,
            },
        )]),
        surface_of_revolution: HashMap::from_iter(vec![(
            23,
            SurfaceOfRevolutionHolder {
                label: "SurfaceOfRevolution".to_string(),
                swept_curve: PlaceHolder::Ref(Name::Entity(20)),
                axis_position: PlaceHolder::Ref(Name::Entity(5)),
            },
        )]),
        spherical_surface: HashMap::from_iter(vec![(
            24,
            SphericalSurfaceHolder {
                label: "SphericalSurface".to_string(),
                position: PlaceHolder::Ref(Name::Entity(9)),
                radius: 5.0,
            },
        )]),
        cylindrical_surface: HashMap::from_iter(vec![(
            25,
            CylindricalSurfaceHolder {
                label: "CylindricalSurface".to_string(),
                position: PlaceHolder::Ref(Name::Entity(9)),
                radius: 5.0,
            },
        )]),
        uniform_surface: HashMap::from_iter(vec![(
            26,
            UniformSurfaceHolder {
                label: "UniformSurface".to_string(),
                u_degree: 2,
                v_degree: 2,
                control_points_list: vec![
                    vec![
                        PlaceHolder::Ref(Name::Entity(1)),
                        PlaceHolder::Ref(Name::Entity(1)),
                        PlaceHolder::Ref(Name::Entity(1)),
                    ],
                    vec![
                        PlaceHolder::Ref(Name::Entity(1)),
                        PlaceHolder::Ref(Name::Entity(1)),
                        PlaceHolder::Ref(Name::Entity(1)),
                    ],
                    vec![
                        PlaceHolder::Ref(Name::Entity(1)),
                        PlaceHolder::Ref(Name::Entity(1)),
                        PlaceHolder::Ref(Name::Entity(1)),
                    ],
                ],
                surface_form: BSplineSurfaceForm::Unspecified,
                u_closed: Logical::Unknown,
                v_closed: Logical::Unknown,
                self_intersect: Logical::Unknown,
            },
        )]),
        quasi_uniform_surface: HashMap::from_iter(vec![(
            27,
            QuasiUniformSurfaceHolder {
                label: "QuasiUniformSurface".to_string(),
                u_degree: 2,
                v_degree: 2,
                control_points_list: vec![
                    vec![
                        PlaceHolder::Ref(Name::Entity(1)),
                        PlaceHolder::Ref(Name::Entity(1)),
                        PlaceHolder::Ref(Name::Entity(1)),
                    ],
                    vec![
                        PlaceHolder::Ref(Name::Entity(1)),
                        PlaceHolder::Ref(Name::Entity(1)),
                        PlaceHolder::Ref(Name::Entity(1)),
                    ],
                    vec![
                        PlaceHolder::Ref(Name::Entity(1)),
                        PlaceHolder::Ref(Name::Entity(1)),
                        PlaceHolder::Ref(Name::Entity(1)),
                    ],
                ],
                surface_form: BSplineSurfaceForm::Unspecified,
                u_closed: Logical::Unknown,
                v_closed: Logical::Unknown,
                self_intersect: Logical::Unknown,
            },
        )]),
        bezier_surface: HashMap::from_iter(vec![(
            28,
            BezierSurfaceHolder {
                label: "BezierSurface".to_string(),
                u_degree: 2,
                v_degree: 2,
                control_points_list: vec![
                    vec![
                        PlaceHolder::Ref(Name::Entity(1)),
                        PlaceHolder::Ref(Name::Entity(1)),
                        PlaceHolder::Ref(Name::Entity(1)),
                    ],
                    vec![
                        PlaceHolder::Ref(Name::Entity(1)),
                        PlaceHolder::Ref(Name::Entity(1)),
                        PlaceHolder::Ref(Name::Entity(1)),
                    ],
                    vec![
                        PlaceHolder::Ref(Name::Entity(1)),
                        PlaceHolder::Ref(Name::Entity(1)),
                        PlaceHolder::Ref(Name::Entity(1)),
                    ],
                ],
                surface_form: BSplineSurfaceForm::Unspecified,
                u_closed: Logical::Unknown,
                v_closed: Logical::Unknown,
                self_intersect: Logical::Unknown,
            },
        )]),
        toroidal_surface: HashMap::from_iter(vec![(
            29,
            ToroidalSurfaceHolder {
                label: "ToroidalSurface".to_string(),
                position: PlaceHolder::Ref(Name::Entity(9)),
                major_radius: 5.0,
                minor_radius: 2.0,
            },
        )]),
        conical_surface: HashMap::from_iter(vec![(
            30,
            ConicalSurfaceHolder {
                label: "ConicalSurface".to_string(),
                position: PlaceHolder::Ref(Name::Entity(9)),
                radius: 5.0,
                semi_angle: 0.5,
            }
        )]),
        surface_of_linear_extrusion: HashMap::from_iter(vec![(
            31,
            SurfaceOfLinearExtrusionHolder {
                label: "SurfaceOfLinearExtrusion".to_string(),
                swept_curve: PlaceHolder::Ref(Name::Entity(20)),
                extrusion_axis: PlaceHolder::Ref(Name::Entity(3)),
            },
        )]),

        vertex_point: HashMap::from_iter(vec![(
            100,
            VertexPointHolder {
                label: "VertexPoint".to_string(),
                vertex_geometry: PlaceHolder::Ref(Name::Entity(1)),
            },
        )]),
        edge_curve: HashMap::from_iter(vec![(
            101,
            EdgeCurveHolder {
                label: "EdgeCurve".to_string(),
                edge_start: PlaceHolder::Ref(Name::Entity(100)),
                edge_end: PlaceHolder::Ref(Name::Entity(100)),
                edge_geometry: PlaceHolder::Ref(Name::Entity(13)),
                same_sense: true,
            },
        )]),
        oriented_edge: HashMap::from_iter(vec![(
            102,
            OrientedEdgeHolder {
                label: "OrientedEdge".to_string(),
                edge_element: PlaceHolder::Ref(Name::Entity(101)),
                orientation: false,
            },
        )]),
        edge_loop: HashMap::from_iter(vec![(
            103,
            EdgeLoopHolder {
                label: "EdgeLoop".to_string(),
                edge_list: vec![
                    PlaceHolder::Ref(Name::Entity(101)),
                    PlaceHolder::Ref(Name::Entity(102)),
                ],
            },
        )]),
        face_bound: HashMap::from_iter(vec![
            (
                104,
                FaceBoundHolder {
                    label: "FaceBound".to_string(),
                    bound: PlaceHolder::Ref(Name::Entity(103)),
                    orientation: true,
                },
            ),
            (
                105,
                FaceBoundHolder {
                    label: "FaceOuterBound".to_string(),
                    bound: PlaceHolder::Ref(Name::Entity(103)),
                    orientation: false,
                },
            ),
        ]),
        face_surface: HashMap::from_iter(vec![
            (
                106,
                FaceSurfaceHolder {
                    label: "FaceSurface".to_string(),
                    bounds: vec![
                        PlaceHolder::Ref(Name::Entity(104)),
                        PlaceHolder::Ref(Name::Entity(105)),
                    ],
                    face_geometry: PlaceHolder::Ref(Name::Entity(21)),
                    same_sense: true,
                },
            ),
            (
                107,
                FaceSurfaceHolder {
                    label: "AdvancedFace".to_string(),
                    bounds: vec![
                        PlaceHolder::Ref(Name::Entity(104)),
                        PlaceHolder::Ref(Name::Entity(105)),
                    ],
                    face_geometry: PlaceHolder::Ref(Name::Entity(21)),
                    same_sense: true,
                },
            ),
        ]),
        oriented_face: HashMap::from_iter(vec![(
            108,
            OrientedFaceHolder {
                label: "OrientedFace".to_string(),
                face_element: PlaceHolder::Ref(Name::Entity(106)),
                orientation: false,
            },
        )]),
        shell: HashMap::from_iter(vec![
            (
                109,
                ShellHolder {
                    label: "OpenShell".to_string(),
                    cfs_faces: vec![
                        PlaceHolder::Ref(Name::Entity(107)),
                        PlaceHolder::Ref(Name::Entity(108)),
                    ],
                },
            ),
            (
                110,
                ShellHolder {
                    label: "ClosedShell".to_string(),
                    cfs_faces: vec![
                        PlaceHolder::Ref(Name::Entity(107)),
                        PlaceHolder::Ref(Name::Entity(108)),
                    ],
                },
            ),
        ]),
        oriented_shell: HashMap::from_iter(vec![
            (
                111,
                OrientedShellHolder {
                    label: "OrientedOpenShell".to_string(),
                    shell_element: PlaceHolder::Ref(Name::Entity(109)),
                    orientation: false,
                },
            ),
            (
                112,
                OrientedShellHolder {
                    label: "OrientedClosedShell".to_string(),
                    shell_element: PlaceHolder::Ref(Name::Entity(110)),
                    orientation: true,
                },
            ),
        ]),
        dummy: HashMap::from_iter(vec![
            (
                999,
                DummyHolder {
                    record: "Record { name: \"HOGE\", parameter: List([String(\"Dummy\"), Ref(Entity(110)), Integer(3)]) }".to_string(),
                    is_simple: true,
                }
            ),
            (
                33,
                DummyHolder {
                    record: "[Record { name: \"GEOMETRIC_REPRESENTATION_CONTEXT\", parameter: List([Integer(2)]) }, \
Record { name: \"PARAMETRIC_REPRESENTATION_CONTEXT\", parameter: List([]) }, \
Record { name: \"REPRESENTATION_CONTEXT\", parameter: List([String(\"2D SPACE\"), String(\"\")]) }]".to_string(),
                    is_simple: false,
                }
            )
        ]),
        ..Default::default()
    };
    assert_eq!(table, ans_table);
}
