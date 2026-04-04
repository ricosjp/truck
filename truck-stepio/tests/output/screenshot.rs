use truck_modeling::*;
use truck_stepio::out::*;

#[test]
fn default_shell() {
    let cshell = Shell::new().compress();
    let header: StepHeaderDescriptor = Default::default();
    let time_stamp = header.time_stamp.clone();
    let design = StepDesign::from_model(StepModel::from(&cshell));
    let step_string = StepDisplay::new(header, design).to_string();
    assert_eq!(
        step_string,
        format!(
            "ISO-10303-21;
HEADER;
FILE_DESCRIPTION(('Shape Data from truck'), '2;1');
FILE_NAME('', '{time_stamp}', (''), (''), 'truck', '', '');
FILE_SCHEMA(('ISO-10303-203'));
ENDSEC;
DATA;
#1 = APPLICATION_CONTEXT('generated shape data');
#2 = (
    GEOMETRIC_REPRESENTATION_CONTEXT(3)
    GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#6))
    GLOBAL_UNIT_ASSIGNED_CONTEXT((#3, #4, #5))
    REPRESENTATION_CONTEXT('Context #1', '3D Context with UNIT and UNCERTAINTY')
);
#3 = ( LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.));
#4 = ( NAMED_UNIT(*) PLANE_ANGLE_UNIT() SI_UNIT($,.RADIAN.) );
#5 = ( NAMED_UNIT(*) SI_UNIT($,.STERADIAN.) SOLID_ANGLE_UNIT() );
#6 = UNCERTAINTY_MEASURE_WITH_UNIT(1.0E-6, #3, 'distance_accuracy_value', 'confusion accuracy');
#7 = AXIS2_PLACEMENT_3D('', #8, #9, #10);
#8 = CARTESIAN_POINT('', (0.0, 0.0, 0.0));
#9 = DIRECTION('', (0.0, 0.0, 1.0));
#10 = DIRECTION('', (1.0, 0.0, 0.0));
#11 = SHAPE_DEFINITION_REPRESENTATION(#12, #18);
#12 = PRODUCT_DEFINITION_SHAPE('', '', #13);
#13 = PRODUCT_DEFINITION('design', '', #14, #16);
#14 = PRODUCT_DEFINITION_FORMATION('', '', #15);
#15 = PRODUCT('', '', '', (#17));
#16 = DESIGN_CONTEXT('', #1, 'design');
#17 = MECHANICAL_CONTEXT('', #1, 'mechanical');
#18 = SHAPE_REPRESENTATION('', (#7, #19), #2);
#19 = SHELL_BASED_SURFACE_MODEL('', (#20));
#20 = OPEN_SHELL('', ());
ENDSEC;
END-ISO-10303-21;\n"
        )
    );
}

#[test]
fn cube() {
    let json = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../resources/shape/cube.json"
    ))
    .unwrap();
    let csolid: CompressedSolid = serde_json::from_reader(json.as_slice()).unwrap();
    let header: StepHeaderDescriptor = Default::default();
    let time_stamp = header.time_stamp.clone();
    let design = StepDesign::from_model(StepModel::from(&csolid));
    let step_string = StepDisplay::new(header, design).to_string();
    assert_eq!(
        step_string,
        format!(
            "ISO-10303-21;
HEADER;
FILE_DESCRIPTION(('Shape Data from truck'), '2;1');
FILE_NAME('', '{time_stamp}', (''), (''), 'truck', '', '');
FILE_SCHEMA(('ISO-10303-203'));
ENDSEC;
DATA;
#1 = APPLICATION_CONTEXT('generated shape data');
#2 = (
    GEOMETRIC_REPRESENTATION_CONTEXT(3)
    GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#6))
    GLOBAL_UNIT_ASSIGNED_CONTEXT((#3, #4, #5))
    REPRESENTATION_CONTEXT('Context #1', '3D Context with UNIT and UNCERTAINTY')
);
#3 = ( LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.));
#4 = ( NAMED_UNIT(*) PLANE_ANGLE_UNIT() SI_UNIT($,.RADIAN.) );
#5 = ( NAMED_UNIT(*) SI_UNIT($,.STERADIAN.) SOLID_ANGLE_UNIT() );
#6 = UNCERTAINTY_MEASURE_WITH_UNIT(1.0E-6, #3, 'distance_accuracy_value', 'confusion accuracy');
#7 = AXIS2_PLACEMENT_3D('', #8, #9, #10);
#8 = CARTESIAN_POINT('', (0.0, 0.0, 0.0));
#9 = DIRECTION('', (0.0, 0.0, 1.0));
#10 = DIRECTION('', (1.0, 0.0, 0.0));
#11 = SHAPE_DEFINITION_REPRESENTATION(#12, #18);
#12 = PRODUCT_DEFINITION_SHAPE('', '', #13);
#13 = PRODUCT_DEFINITION('design', '', #14, #16);
#14 = PRODUCT_DEFINITION_FORMATION('', '', #15);
#15 = PRODUCT('', '', '', (#17));
#16 = DESIGN_CONTEXT('', #1, 'design');
#17 = MECHANICAL_CONTEXT('', #1, 'mechanical');
#18 = SHAPE_REPRESENTATION('', (#7, #19), #2);
#19 = MANIFOLD_SOLID_BREP('', #20);
#20 = CLOSED_SHELL('', (#21, #28, #35, #42, #49, #56));
#21 = FACE_SURFACE('', (#22), #83, .F.);
#22 = FACE_BOUND('', #23, .F.);
#23 = EDGE_LOOP('', (#24, #25, #26, #27));
#24 = ORIENTED_EDGE('', *, *, #63, .T.);
#25 = ORIENTED_EDGE('', *, *, #64, .T.);
#26 = ORIENTED_EDGE('', *, *, #65, .F.);
#27 = ORIENTED_EDGE('', *, *, #66, .F.);
#28 = FACE_SURFACE('', (#29), #88, .T.);
#29 = FACE_BOUND('', #30, .T.);
#30 = EDGE_LOOP('', (#31, #32, #33, #34));
#31 = ORIENTED_EDGE('', *, *, #63, .T.);
#32 = ORIENTED_EDGE('', *, *, #67, .T.);
#33 = ORIENTED_EDGE('', *, *, #68, .F.);
#34 = ORIENTED_EDGE('', *, *, #69, .F.);
#35 = FACE_SURFACE('', (#36), #93, .T.);
#36 = FACE_BOUND('', #37, .T.);
#37 = EDGE_LOOP('', (#38, #39, #40, #41));
#38 = ORIENTED_EDGE('', *, *, #64, .T.);
#39 = ORIENTED_EDGE('', *, *, #70, .T.);
#40 = ORIENTED_EDGE('', *, *, #71, .F.);
#41 = ORIENTED_EDGE('', *, *, #67, .F.);
#42 = FACE_SURFACE('', (#43), #98, .F.);
#43 = FACE_BOUND('', #44, .F.);
#44 = EDGE_LOOP('', (#45, #46, #47, #48));
#45 = ORIENTED_EDGE('', *, *, #70, .T.);
#46 = ORIENTED_EDGE('', *, *, #72, .F.);
#47 = ORIENTED_EDGE('', *, *, #73, .F.);
#48 = ORIENTED_EDGE('', *, *, #65, .T.);
#49 = FACE_SURFACE('', (#50), #103, .F.);
#50 = FACE_BOUND('', #51, .F.);
#51 = EDGE_LOOP('', (#52, #53, #54, #55));
#52 = ORIENTED_EDGE('', *, *, #73, .T.);
#53 = ORIENTED_EDGE('', *, *, #74, .F.);
#54 = ORIENTED_EDGE('', *, *, #69, .F.);
#55 = ORIENTED_EDGE('', *, *, #66, .T.);
#56 = FACE_SURFACE('', (#57), #108, .T.);
#57 = FACE_BOUND('', #58, .T.);
#58 = EDGE_LOOP('', (#59, #60, #61, #62));
#59 = ORIENTED_EDGE('', *, *, #68, .T.);
#60 = ORIENTED_EDGE('', *, *, #71, .T.);
#61 = ORIENTED_EDGE('', *, *, #72, .F.);
#62 = ORIENTED_EDGE('', *, *, #74, .F.);
#63 = EDGE_CURVE('', #75, #76, #113, .T.);
#64 = EDGE_CURVE('', #76, #77, #117, .T.);
#65 = EDGE_CURVE('', #78, #77, #121, .T.);
#66 = EDGE_CURVE('', #75, #78, #125, .T.);
#67 = EDGE_CURVE('', #76, #79, #129, .T.);
#68 = EDGE_CURVE('', #80, #79, #133, .T.);
#69 = EDGE_CURVE('', #75, #80, #137, .T.);
#70 = EDGE_CURVE('', #77, #81, #141, .T.);
#71 = EDGE_CURVE('', #79, #81, #145, .T.);
#72 = EDGE_CURVE('', #82, #81, #149, .T.);
#73 = EDGE_CURVE('', #78, #82, #153, .T.);
#74 = EDGE_CURVE('', #80, #82, #157, .T.);
#75 = VERTEX_POINT('', #161);
#76 = VERTEX_POINT('', #162);
#77 = VERTEX_POINT('', #163);
#78 = VERTEX_POINT('', #164);
#79 = VERTEX_POINT('', #165);
#80 = VERTEX_POINT('', #166);
#81 = VERTEX_POINT('', #167);
#82 = VERTEX_POINT('', #168);
#83 = PLANE('', #84);
#84 = AXIS2_PLACEMENT_3D('', #85, #86, #87);
#85 = CARTESIAN_POINT('', (-0.5, -0.5, -0.5));
#86 = DIRECTION('', (0.0, 0.0, 1.0));
#87 = DIRECTION('', (1.0, 0.0, 0.0));
#88 = PLANE('', #89);
#89 = AXIS2_PLACEMENT_3D('', #90, #91, #92);
#90 = CARTESIAN_POINT('', (-0.5, -0.5, -0.5));
#91 = DIRECTION('', (0.0, -1.0, 0.0));
#92 = DIRECTION('', (1.0, 0.0, 0.0));
#93 = PLANE('', #94);
#94 = AXIS2_PLACEMENT_3D('', #95, #96, #97);
#95 = CARTESIAN_POINT('', (0.5, -0.5, -0.5));
#96 = DIRECTION('', (1.0, 0.0, 0.0));
#97 = DIRECTION('', (0.0, 1.0, 0.0));
#98 = PLANE('', #99);
#99 = AXIS2_PLACEMENT_3D('', #100, #101, #102);
#100 = CARTESIAN_POINT('', (-0.5, 0.5, -0.5));
#101 = DIRECTION('', (0.0, -1.0, 0.0));
#102 = DIRECTION('', (1.0, 0.0, 0.0));
#103 = PLANE('', #104);
#104 = AXIS2_PLACEMENT_3D('', #105, #106, #107);
#105 = CARTESIAN_POINT('', (-0.5, -0.5, -0.5));
#106 = DIRECTION('', (1.0, 0.0, 0.0));
#107 = DIRECTION('', (0.0, 1.0, 0.0));
#108 = PLANE('', #109);
#109 = AXIS2_PLACEMENT_3D('', #110, #111, #112);
#110 = CARTESIAN_POINT('', (-0.5, -0.5, 0.5));
#111 = DIRECTION('', (0.0, 0.0, 1.0));
#112 = DIRECTION('', (1.0, 0.0, 0.0));
#113 = LINE('', #114, #115);
#114 = CARTESIAN_POINT('', (-0.5, -0.5, -0.5));
#115 = VECTOR('', #116, 1.0);
#116 = DIRECTION('', (1.0, 0.0, 0.0));
#117 = LINE('', #118, #119);
#118 = CARTESIAN_POINT('', (0.5, -0.5, -0.5));
#119 = VECTOR('', #120, 1.0);
#120 = DIRECTION('', (0.0, 1.0, 0.0));
#121 = LINE('', #122, #123);
#122 = CARTESIAN_POINT('', (-0.5, 0.5, -0.5));
#123 = VECTOR('', #124, 1.0);
#124 = DIRECTION('', (1.0, 0.0, 0.0));
#125 = LINE('', #126, #127);
#126 = CARTESIAN_POINT('', (-0.5, -0.5, -0.5));
#127 = VECTOR('', #128, 1.0);
#128 = DIRECTION('', (0.0, 1.0, 0.0));
#129 = LINE('', #130, #131);
#130 = CARTESIAN_POINT('', (0.5, -0.5, -0.5));
#131 = VECTOR('', #132, 1.0);
#132 = DIRECTION('', (0.0, 0.0, 1.0));
#133 = LINE('', #134, #135);
#134 = CARTESIAN_POINT('', (-0.5, -0.5, 0.5));
#135 = VECTOR('', #136, 1.0);
#136 = DIRECTION('', (1.0, 0.0, 0.0));
#137 = LINE('', #138, #139);
#138 = CARTESIAN_POINT('', (-0.5, -0.5, -0.5));
#139 = VECTOR('', #140, 1.0);
#140 = DIRECTION('', (0.0, 0.0, 1.0));
#141 = LINE('', #142, #143);
#142 = CARTESIAN_POINT('', (0.5, 0.5, -0.5));
#143 = VECTOR('', #144, 1.0);
#144 = DIRECTION('', (0.0, 0.0, 1.0));
#145 = LINE('', #146, #147);
#146 = CARTESIAN_POINT('', (0.5, -0.5, 0.5));
#147 = VECTOR('', #148, 1.0);
#148 = DIRECTION('', (0.0, 1.0, 0.0));
#149 = LINE('', #150, #151);
#150 = CARTESIAN_POINT('', (-0.5, 0.5, 0.5));
#151 = VECTOR('', #152, 1.0);
#152 = DIRECTION('', (1.0, 0.0, 0.0));
#153 = LINE('', #154, #155);
#154 = CARTESIAN_POINT('', (-0.5, 0.5, -0.5));
#155 = VECTOR('', #156, 1.0);
#156 = DIRECTION('', (0.0, 0.0, 1.0));
#157 = LINE('', #158, #159);
#158 = CARTESIAN_POINT('', (-0.5, -0.5, 0.5));
#159 = VECTOR('', #160, 1.0);
#160 = DIRECTION('', (0.0, 1.0, 0.0));
#161 = CARTESIAN_POINT('', (-0.5, -0.5, -0.5));
#162 = CARTESIAN_POINT('', (0.5, -0.5, -0.5));
#163 = CARTESIAN_POINT('', (0.5, 0.5, -0.5));
#164 = CARTESIAN_POINT('', (-0.5, 0.5, -0.5));
#165 = CARTESIAN_POINT('', (0.5, -0.5, 0.5));
#166 = CARTESIAN_POINT('', (-0.5, -0.5, 0.5));
#167 = CARTESIAN_POINT('', (0.5, 0.5, 0.5));
#168 = CARTESIAN_POINT('', (-0.5, 0.5, 0.5));
ENDSEC;
END-ISO-10303-21;\n"
        ),
    );
}

#[test]
fn custom_header() {
    let cshell = Shell::new().compress();
    let header = StepHeaderDescriptor {
        file_name: "AMAZING_PRODUCT.step".to_string(),
        time_stamp: chrono::Utc::now().naive_local().to_string(),
        authors: vec![
            "Tensai".to_string(),
            "Genius".to_string(),
            "Bokusama".to_string(),
        ],
        organization: vec![
            "Great Awesome Co. Ltd.".to_string(),
            "Univ. Clever Genius".to_string(),
            "Senkai".to_string(),
        ],
        organization_system: "Kusanagi Sword".to_string(),
        authorization: "IT'S ME!".to_string(),
    };
    let time_stamp = header.time_stamp.clone();
    let design = StepDesign::from_model(StepModel::from(&cshell));
    let step_string = StepDisplay::new(header, design).to_string();
    assert_eq!(
        step_string,
        format!(
            "ISO-10303-21;
HEADER;
FILE_DESCRIPTION(('Shape Data from truck'), '2;1');
FILE_NAME('AMAZING_PRODUCT.step', '{time_stamp}', ('Tensai', 'Genius', 'Bokusama'), ('Great Awesome Co. Ltd.', 'Univ. Clever Genius', 'Senkai'), 'truck', 'Kusanagi Sword', 'IT'S ME!');
FILE_SCHEMA(('ISO-10303-203'));
ENDSEC;
DATA;
#1 = APPLICATION_CONTEXT('generated shape data');
#2 = (
    GEOMETRIC_REPRESENTATION_CONTEXT(3)
    GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#6))
    GLOBAL_UNIT_ASSIGNED_CONTEXT((#3, #4, #5))
    REPRESENTATION_CONTEXT('Context #1', '3D Context with UNIT and UNCERTAINTY')
);
#3 = ( LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.));
#4 = ( NAMED_UNIT(*) PLANE_ANGLE_UNIT() SI_UNIT($,.RADIAN.) );
#5 = ( NAMED_UNIT(*) SI_UNIT($,.STERADIAN.) SOLID_ANGLE_UNIT() );
#6 = UNCERTAINTY_MEASURE_WITH_UNIT(1.0E-6, #3, 'distance_accuracy_value', 'confusion accuracy');
#7 = AXIS2_PLACEMENT_3D('', #8, #9, #10);
#8 = CARTESIAN_POINT('', (0.0, 0.0, 0.0));
#9 = DIRECTION('', (0.0, 0.0, 1.0));
#10 = DIRECTION('', (1.0, 0.0, 0.0));
#11 = SHAPE_DEFINITION_REPRESENTATION(#12, #18);
#12 = PRODUCT_DEFINITION_SHAPE('', '', #13);
#13 = PRODUCT_DEFINITION('design', '', #14, #16);
#14 = PRODUCT_DEFINITION_FORMATION('', '', #15);
#15 = PRODUCT('', '', '', (#17));
#16 = DESIGN_CONTEXT('', #1, 'design');
#17 = MECHANICAL_CONTEXT('', #1, 'mechanical');
#18 = SHAPE_REPRESENTATION('', (#7, #19), #2);
#19 = SHELL_BASED_SURFACE_MODEL('', (#20));
#20 = OPEN_SHELL('', ());
ENDSEC;
END-ISO-10303-21;\n"
        )
    );
}

#[test]
fn koch() {
    use std::f64::consts::PI;
    use truck_assembly::assy::*;
    use truck_stepio::common::PartAttrs;

    let b = BoundingBox::from_iter([Point3::new(0.1, -0.05, -0.05), Point3::new(0.9, 0.05, 0.05)]);
    let bar = primitive::cuboid::<Curve, Surface>(b).compress();
    let model = StepModel::from(&bar);

    let mut assy = Assembly::new();
    let nodes = assy.create_nodes([
        NodeEntity {
            shape: Some(model),
            attrs: PartAttrs {
                id: "node-0".to_string(),
                name: "base node".to_string(),
                description: "node with model".to_string(),
            },
        },
        NodeEntity {
            shape: None,
            attrs: PartAttrs {
                id: "node-1".to_string(),
                name: "Step 1".to_string(),
                description: "Step 1".to_string(),
            },
        },
        NodeEntity {
            shape: None,
            attrs: PartAttrs {
                id: "node-2".to_string(),
                name: "Step 2".to_string(),
                description: "Step 2".to_string(),
            },
        },
        NodeEntity {
            shape: None,
            attrs: PartAttrs {
                id: "node-3".to_string(),
                name: "Step 3".to_string(),
                description: "Step 3".to_string(),
            },
        },
    ]);

    let mut a = 1.0;
    for i in 0..3 {
        let next = i + 1;
        assy.create_edge(
            nodes[next],
            nodes[i],
            EdgeEntity {
                matrix: Matrix4::identity(),
                attrs: PartAttrs {
                    id: format!("edge-{next}-{i}-0"),
                    name: format!("Assy Step {next}-0"),
                    description: format!("0-edge from node-{next} to node-{i}"),
                },
            },
        );
        assy.create_edge(
            nodes[i + 1],
            nodes[i],
            EdgeEntity {
                matrix: Matrix4::from_translation(Vector3::new(a, 0.0, 0.0))
                    * Matrix4::from_angle_z(Rad(PI / 3.0)),
                attrs: PartAttrs {
                    id: format!("edge-{next}-{i}-1"),
                    name: format!("Assy Step {next}-1"),
                    description: format!("1-edge from node-{next} to node-{i}"),
                },
            },
        );
        assy.create_edge(
            nodes[i + 1],
            nodes[i],
            EdgeEntity {
                matrix: Matrix4::from_translation(Vector3::new(
                    a * 1.5,
                    a * f64::sqrt(3.0) / 2.0,
                    0.0,
                )) * Matrix4::from_angle_z(Rad(-PI / 3.0)),
                attrs: PartAttrs {
                    id: format!("edge-{next}-{i}-2"),
                    name: format!("Assy Step {next}-2"),
                    description: format!("2-edge from node-{next} to node-{i}"),
                },
            },
        );
        assy.create_edge(
            nodes[i + 1],
            nodes[i],
            EdgeEntity {
                matrix: Matrix4::from_translation(Vector3::new(a * 2.0, 0.0, 0.0)),
                attrs: PartAttrs {
                    id: format!("edge-{next}-{i}-3"),
                    name: format!("Assy Step {next}-3"),
                    description: format!("3-edge node from node-{next} to node-{i}"),
                },
            },
        );
        a *= 3.0;
    }

    let header = StepHeaderDescriptor::default();
    let time_stamp = header.time_stamp.clone();
    let step_string = StepDisplay::new(header, StepDesign::new(assy)).to_string();
    assert_eq!(
        step_string,
        format!(
            "ISO-10303-21;
HEADER;
FILE_DESCRIPTION(('Shape Data from truck'), '2;1');
FILE_NAME('', '{time_stamp}', (''), (''), 'truck', '', '');
FILE_SCHEMA(('ISO-10303-203'));
ENDSEC;
DATA;
#1 = APPLICATION_CONTEXT('generated shape data');
#2 = (
    GEOMETRIC_REPRESENTATION_CONTEXT(3)
    GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#6))
    GLOBAL_UNIT_ASSIGNED_CONTEXT((#3, #4, #5))
    REPRESENTATION_CONTEXT('Context #1', '3D Context with UNIT and UNCERTAINTY')
);
#3 = ( LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.));
#4 = ( NAMED_UNIT(*) PLANE_ANGLE_UNIT() SI_UNIT($,.RADIAN.) );
#5 = ( NAMED_UNIT(*) SI_UNIT($,.STERADIAN.) SOLID_ANGLE_UNIT() );
#6 = UNCERTAINTY_MEASURE_WITH_UNIT(1.0E-6, #3, 'distance_accuracy_value', 'confusion accuracy');
#7 = AXIS2_PLACEMENT_3D('', #8, #9, #10);
#8 = CARTESIAN_POINT('', (0.0, 0.0, 0.0));
#9 = DIRECTION('', (0.0, 0.0, 1.0));
#10 = DIRECTION('', (1.0, 0.0, 0.0));
#11 = SHAPE_DEFINITION_REPRESENTATION(#12, #18);
#12 = PRODUCT_DEFINITION_SHAPE('', '', #13);
#13 = PRODUCT_DEFINITION('design', '', #14, #16);
#14 = PRODUCT_DEFINITION_FORMATION('', '', #15);
#15 = PRODUCT('node-0', 'base node', 'node with model', (#17));
#16 = DESIGN_CONTEXT('', #1, 'design');
#17 = MECHANICAL_CONTEXT('', #1, 'mechanical');
#18 = SHAPE_REPRESENTATION('', (#7, #19), #2);
#19 = MANIFOLD_SOLID_BREP('', #20);
#20 = CLOSED_SHELL('', (#21, #28, #35, #42, #49, #56));
#21 = FACE_SURFACE('', (#22), #83, .T.);
#22 = FACE_BOUND('', #23, .T.);
#23 = EDGE_LOOP('', (#24, #25, #26, #27));
#24 = ORIENTED_EDGE('', *, *, #63, .F.);
#25 = ORIENTED_EDGE('', *, *, #64, .F.);
#26 = ORIENTED_EDGE('', *, *, #65, .F.);
#27 = ORIENTED_EDGE('', *, *, #66, .F.);
#28 = FACE_SURFACE('', (#29), #88, .T.);
#29 = FACE_BOUND('', #30, .T.);
#30 = EDGE_LOOP('', (#31, #32, #33, #34));
#31 = ORIENTED_EDGE('', *, *, #66, .T.);
#32 = ORIENTED_EDGE('', *, *, #67, .T.);
#33 = ORIENTED_EDGE('', *, *, #68, .F.);
#34 = ORIENTED_EDGE('', *, *, #69, .F.);
#35 = FACE_SURFACE('', (#36), #93, .T.);
#36 = FACE_BOUND('', #37, .T.);
#37 = EDGE_LOOP('', (#38, #39, #40, #41));
#38 = ORIENTED_EDGE('', *, *, #65, .T.);
#39 = ORIENTED_EDGE('', *, *, #70, .T.);
#40 = ORIENTED_EDGE('', *, *, #71, .F.);
#41 = ORIENTED_EDGE('', *, *, #67, .F.);
#42 = FACE_SURFACE('', (#43), #98, .T.);
#43 = FACE_BOUND('', #44, .T.);
#44 = EDGE_LOOP('', (#45, #46, #47, #48));
#45 = ORIENTED_EDGE('', *, *, #64, .T.);
#46 = ORIENTED_EDGE('', *, *, #72, .T.);
#47 = ORIENTED_EDGE('', *, *, #73, .F.);
#48 = ORIENTED_EDGE('', *, *, #70, .F.);
#49 = FACE_SURFACE('', (#50), #103, .T.);
#50 = FACE_BOUND('', #51, .T.);
#51 = EDGE_LOOP('', (#52, #53, #54, #55));
#52 = ORIENTED_EDGE('', *, *, #63, .T.);
#53 = ORIENTED_EDGE('', *, *, #69, .T.);
#54 = ORIENTED_EDGE('', *, *, #74, .F.);
#55 = ORIENTED_EDGE('', *, *, #72, .F.);
#56 = FACE_SURFACE('', (#57), #108, .T.);
#57 = FACE_BOUND('', #58, .T.);
#58 = EDGE_LOOP('', (#59, #60, #61, #62));
#59 = ORIENTED_EDGE('', *, *, #68, .T.);
#60 = ORIENTED_EDGE('', *, *, #71, .T.);
#61 = ORIENTED_EDGE('', *, *, #73, .T.);
#62 = ORIENTED_EDGE('', *, *, #74, .T.);
#63 = EDGE_CURVE('', #75, #76, #113, .T.);
#64 = EDGE_CURVE('', #77, #75, #117, .T.);
#65 = EDGE_CURVE('', #78, #77, #121, .T.);
#66 = EDGE_CURVE('', #76, #78, #125, .T.);
#67 = EDGE_CURVE('', #78, #79, #129, .T.);
#68 = EDGE_CURVE('', #80, #79, #133, .T.);
#69 = EDGE_CURVE('', #76, #80, #137, .T.);
#70 = EDGE_CURVE('', #77, #81, #141, .T.);
#71 = EDGE_CURVE('', #79, #81, #145, .T.);
#72 = EDGE_CURVE('', #75, #82, #149, .T.);
#73 = EDGE_CURVE('', #81, #82, #153, .T.);
#74 = EDGE_CURVE('', #82, #80, #157, .T.);
#75 = VERTEX_POINT('', #161);
#76 = VERTEX_POINT('', #162);
#77 = VERTEX_POINT('', #163);
#78 = VERTEX_POINT('', #164);
#79 = VERTEX_POINT('', #165);
#80 = VERTEX_POINT('', #166);
#81 = VERTEX_POINT('', #167);
#82 = VERTEX_POINT('', #168);
#83 = PLANE('', #84);
#84 = AXIS2_PLACEMENT_3D('', #85, #86, #87);
#85 = CARTESIAN_POINT('', (0.1, -0.05, -0.05));
#86 = DIRECTION('', (0.0, 0.0, -1.0));
#87 = DIRECTION('', (0.0, 1.0, 0.0));
#88 = PLANE('', #89);
#89 = AXIS2_PLACEMENT_3D('', #90, #91, #92);
#90 = CARTESIAN_POINT('', (0.1, -0.05, -0.05));
#91 = DIRECTION('', (0.0, -1.0, 0.0));
#92 = DIRECTION('', (1.0, 0.0, 0.0));
#93 = PLANE('', #94);
#94 = AXIS2_PLACEMENT_3D('', #95, #96, #97);
#95 = CARTESIAN_POINT('', (0.9, -0.05, -0.05));
#96 = DIRECTION('', (1.0, 0.0, 0.0));
#97 = DIRECTION('', (0.0, 1.0, 0.0));
#98 = PLANE('', #99);
#99 = AXIS2_PLACEMENT_3D('', #100, #101, #102);
#100 = CARTESIAN_POINT('', (0.9, 0.05, -0.05));
#101 = DIRECTION('', (0.0, 1.0, -0.0));
#102 = DIRECTION('', (-1.0, 0.0, 0.0));
#103 = PLANE('', #104);
#104 = AXIS2_PLACEMENT_3D('', #105, #106, #107);
#105 = CARTESIAN_POINT('', (0.1, 0.05, -0.05));
#106 = DIRECTION('', (-1.0, 0.0, 0.0));
#107 = DIRECTION('', (0.0, -0.7071067811865475, 0.7071067811865475));
#108 = PLANE('', #109);
#109 = AXIS2_PLACEMENT_3D('', #110, #111, #112);
#110 = CARTESIAN_POINT('', (0.1, -0.05, 0.05));
#111 = DIRECTION('', (0.0, 0.0, 1.0));
#112 = DIRECTION('', (1.0, 0.0, 0.0));
#113 = LINE('', #114, #115);
#114 = CARTESIAN_POINT('', (0.1, 0.05, -0.05));
#115 = VECTOR('', #116, 0.1);
#116 = DIRECTION('', (0.0, -1.0, 0.0));
#117 = LINE('', #118, #119);
#118 = CARTESIAN_POINT('', (0.9, 0.05, -0.05));
#119 = VECTOR('', #120, 0.8);
#120 = DIRECTION('', (-1.0, 0.0, 0.0));
#121 = LINE('', #122, #123);
#122 = CARTESIAN_POINT('', (0.9, -0.05, -0.05));
#123 = VECTOR('', #124, 0.1);
#124 = DIRECTION('', (0.0, 1.0, 0.0));
#125 = LINE('', #126, #127);
#126 = CARTESIAN_POINT('', (0.1, -0.05, -0.05));
#127 = VECTOR('', #128, 0.8);
#128 = DIRECTION('', (1.0, 0.0, 0.0));
#129 = LINE('', #130, #131);
#130 = CARTESIAN_POINT('', (0.9, -0.05, -0.05));
#131 = VECTOR('', #132, 0.1);
#132 = DIRECTION('', (0.0, 0.0, 1.0));
#133 = LINE('', #134, #135);
#134 = CARTESIAN_POINT('', (0.1, -0.05, 0.05));
#135 = VECTOR('', #136, 0.8);
#136 = DIRECTION('', (1.0, 0.0, 0.0));
#137 = LINE('', #138, #139);
#138 = CARTESIAN_POINT('', (0.1, -0.05, -0.05));
#139 = VECTOR('', #140, 0.1);
#140 = DIRECTION('', (0.0, 0.0, 1.0));
#141 = LINE('', #142, #143);
#142 = CARTESIAN_POINT('', (0.9, 0.05, -0.05));
#143 = VECTOR('', #144, 0.1);
#144 = DIRECTION('', (0.0, 0.0, 1.0));
#145 = LINE('', #146, #147);
#146 = CARTESIAN_POINT('', (0.9, -0.05, 0.05));
#147 = VECTOR('', #148, 0.1);
#148 = DIRECTION('', (0.0, 1.0, 0.0));
#149 = LINE('', #150, #151);
#150 = CARTESIAN_POINT('', (0.1, 0.05, -0.05));
#151 = VECTOR('', #152, 0.1);
#152 = DIRECTION('', (0.0, 0.0, 1.0));
#153 = LINE('', #154, #155);
#154 = CARTESIAN_POINT('', (0.9, 0.05, 0.05));
#155 = VECTOR('', #156, 0.8);
#156 = DIRECTION('', (-1.0, 0.0, 0.0));
#157 = LINE('', #158, #159);
#158 = CARTESIAN_POINT('', (0.1, 0.05, 0.05));
#159 = VECTOR('', #160, 0.1);
#160 = DIRECTION('', (0.0, -1.0, 0.0));
#161 = CARTESIAN_POINT('', (0.1, 0.05, -0.05));
#162 = CARTESIAN_POINT('', (0.1, -0.05, -0.05));
#163 = CARTESIAN_POINT('', (0.9, 0.05, -0.05));
#164 = CARTESIAN_POINT('', (0.9, -0.05, -0.05));
#165 = CARTESIAN_POINT('', (0.9, -0.05, 0.05));
#166 = CARTESIAN_POINT('', (0.1, -0.05, 0.05));
#167 = CARTESIAN_POINT('', (0.9, 0.05, 0.05));
#168 = CARTESIAN_POINT('', (0.1, 0.05, 0.05));
#169 = SHAPE_DEFINITION_REPRESENTATION(#170, #176);
#170 = PRODUCT_DEFINITION_SHAPE('', '', #171);
#171 = PRODUCT_DEFINITION('design', '', #172, #174);
#172 = PRODUCT_DEFINITION_FORMATION('', '', #173);
#173 = PRODUCT('node-1', 'Step 1', 'Step 1', (#175));
#174 = DESIGN_CONTEXT('', #1, 'design');
#175 = MECHANICAL_CONTEXT('', #1, 'mechanical');
#176 = SHAPE_REPRESENTATION('', (#7, #177, #181, #185, #189), #2);
#177 = AXIS2_PLACEMENT_3D('', #178, #179, #180);
#178 = CARTESIAN_POINT('', (0.0, 0.0, 0.0));
#179 = DIRECTION('', (0.0, 0.0, 1.0));
#180 = DIRECTION('', (1.0, 0.0, 0.0));
#181 = AXIS2_PLACEMENT_3D('', #182, #183, #184);
#182 = CARTESIAN_POINT('', (1.0, 0.0, 0.0));
#183 = DIRECTION('', (0.0, 0.0, 1.0));
#184 = DIRECTION('', (0.5000000000000001, 0.8660254037844386, 0.0));
#185 = AXIS2_PLACEMENT_3D('', #186, #187, #188);
#186 = CARTESIAN_POINT('', (1.5, 0.8660254037844386, 0.0));
#187 = DIRECTION('', (0.0, 0.0, 1.0));
#188 = DIRECTION('', (0.5000000000000001, -0.8660254037844386, 0.0));
#189 = AXIS2_PLACEMENT_3D('', #190, #191, #192);
#190 = CARTESIAN_POINT('', (2.0, 0.0, 0.0));
#191 = DIRECTION('', (0.0, 0.0, 1.0));
#192 = DIRECTION('', (1.0, 0.0, 0.0));
#193 = SHAPE_DEFINITION_REPRESENTATION(#194, #200);
#194 = PRODUCT_DEFINITION_SHAPE('', '', #195);
#195 = PRODUCT_DEFINITION('design', '', #196, #198);
#196 = PRODUCT_DEFINITION_FORMATION('', '', #197);
#197 = PRODUCT('node-2', 'Step 2', 'Step 2', (#199));
#198 = DESIGN_CONTEXT('', #1, 'design');
#199 = MECHANICAL_CONTEXT('', #1, 'mechanical');
#200 = SHAPE_REPRESENTATION('', (#7, #201, #205, #209, #213), #2);
#201 = AXIS2_PLACEMENT_3D('', #202, #203, #204);
#202 = CARTESIAN_POINT('', (0.0, 0.0, 0.0));
#203 = DIRECTION('', (0.0, 0.0, 1.0));
#204 = DIRECTION('', (1.0, 0.0, 0.0));
#205 = AXIS2_PLACEMENT_3D('', #206, #207, #208);
#206 = CARTESIAN_POINT('', (3.0, 0.0, 0.0));
#207 = DIRECTION('', (0.0, 0.0, 1.0));
#208 = DIRECTION('', (0.5000000000000001, 0.8660254037844386, 0.0));
#209 = AXIS2_PLACEMENT_3D('', #210, #211, #212);
#210 = CARTESIAN_POINT('', (4.5, 2.598076211353316, 0.0));
#211 = DIRECTION('', (0.0, 0.0, 1.0));
#212 = DIRECTION('', (0.5000000000000001, -0.8660254037844386, 0.0));
#213 = AXIS2_PLACEMENT_3D('', #214, #215, #216);
#214 = CARTESIAN_POINT('', (6.0, 0.0, 0.0));
#215 = DIRECTION('', (0.0, 0.0, 1.0));
#216 = DIRECTION('', (1.0, 0.0, 0.0));
#217 = SHAPE_DEFINITION_REPRESENTATION(#218, #224);
#218 = PRODUCT_DEFINITION_SHAPE('', '', #219);
#219 = PRODUCT_DEFINITION('design', '', #220, #222);
#220 = PRODUCT_DEFINITION_FORMATION('', '', #221);
#221 = PRODUCT('node-3', 'Step 3', 'Step 3', (#223));
#222 = DESIGN_CONTEXT('', #1, 'design');
#223 = MECHANICAL_CONTEXT('', #1, 'mechanical');
#224 = SHAPE_REPRESENTATION('', (#7, #225, #229, #233, #237), #2);
#225 = AXIS2_PLACEMENT_3D('', #226, #227, #228);
#226 = CARTESIAN_POINT('', (0.0, 0.0, 0.0));
#227 = DIRECTION('', (0.0, 0.0, 1.0));
#228 = DIRECTION('', (1.0, 0.0, 0.0));
#229 = AXIS2_PLACEMENT_3D('', #230, #231, #232);
#230 = CARTESIAN_POINT('', (9.0, 0.0, 0.0));
#231 = DIRECTION('', (0.0, 0.0, 1.0));
#232 = DIRECTION('', (0.5000000000000001, 0.8660254037844386, 0.0));
#233 = AXIS2_PLACEMENT_3D('', #234, #235, #236);
#234 = CARTESIAN_POINT('', (13.5, 7.794228634059947, 0.0));
#235 = DIRECTION('', (0.0, 0.0, 1.0));
#236 = DIRECTION('', (0.5000000000000001, -0.8660254037844386, 0.0));
#237 = AXIS2_PLACEMENT_3D('', #238, #239, #240);
#238 = CARTESIAN_POINT('', (18.0, 0.0, 0.0));
#239 = DIRECTION('', (0.0, 0.0, 1.0));
#240 = DIRECTION('', (1.0, 0.0, 0.0));
#241 = CONTEXT_DEPENDENT_SHAPE_REPRESENTATION(#242, #244);
#242 = (
    REPRESENTATION_RELATIONSHIP('', '', #176, #18)
    REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION(#243)
    SHAPE_REPRESENTATION_RELATIONSHIP()
);
#243 = ITEM_DEFINED_TRANSFORMATION('', '', #7, #177);
#244 = PRODUCT_DEFINITION_SHAPE('', '', #245);
#245 = NEXT_ASSEMBLY_USAGE_OCCURRENCE('edge-1-0-0', 'Assy Step 1-0', '0-edge from node-1 to node-0', #171, #13, $);
#246 = CONTEXT_DEPENDENT_SHAPE_REPRESENTATION(#247, #249);
#247 = (
    REPRESENTATION_RELATIONSHIP('', '', #176, #18)
    REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION(#248)
    SHAPE_REPRESENTATION_RELATIONSHIP()
);
#248 = ITEM_DEFINED_TRANSFORMATION('', '', #7, #181);
#249 = PRODUCT_DEFINITION_SHAPE('', '', #250);
#250 = NEXT_ASSEMBLY_USAGE_OCCURRENCE('edge-1-0-1', 'Assy Step 1-1', '1-edge from node-1 to node-0', #171, #13, $);
#251 = CONTEXT_DEPENDENT_SHAPE_REPRESENTATION(#252, #254);
#252 = (
    REPRESENTATION_RELATIONSHIP('', '', #176, #18)
    REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION(#253)
    SHAPE_REPRESENTATION_RELATIONSHIP()
);
#253 = ITEM_DEFINED_TRANSFORMATION('', '', #7, #185);
#254 = PRODUCT_DEFINITION_SHAPE('', '', #255);
#255 = NEXT_ASSEMBLY_USAGE_OCCURRENCE('edge-1-0-2', 'Assy Step 1-2', '2-edge from node-1 to node-0', #171, #13, $);
#256 = CONTEXT_DEPENDENT_SHAPE_REPRESENTATION(#257, #259);
#257 = (
    REPRESENTATION_RELATIONSHIP('', '', #176, #18)
    REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION(#258)
    SHAPE_REPRESENTATION_RELATIONSHIP()
);
#258 = ITEM_DEFINED_TRANSFORMATION('', '', #7, #189);
#259 = PRODUCT_DEFINITION_SHAPE('', '', #260);
#260 = NEXT_ASSEMBLY_USAGE_OCCURRENCE('edge-1-0-3', 'Assy Step 1-3', '3-edge node from node-1 to node-0', #171, #13, $);
#261 = CONTEXT_DEPENDENT_SHAPE_REPRESENTATION(#262, #264);
#262 = (
    REPRESENTATION_RELATIONSHIP('', '', #200, #176)
    REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION(#263)
    SHAPE_REPRESENTATION_RELATIONSHIP()
);
#263 = ITEM_DEFINED_TRANSFORMATION('', '', #7, #201);
#264 = PRODUCT_DEFINITION_SHAPE('', '', #265);
#265 = NEXT_ASSEMBLY_USAGE_OCCURRENCE('edge-2-1-0', 'Assy Step 2-0', '0-edge from node-2 to node-1', #195, #171, $);
#266 = CONTEXT_DEPENDENT_SHAPE_REPRESENTATION(#267, #269);
#267 = (
    REPRESENTATION_RELATIONSHIP('', '', #200, #176)
    REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION(#268)
    SHAPE_REPRESENTATION_RELATIONSHIP()
);
#268 = ITEM_DEFINED_TRANSFORMATION('', '', #7, #205);
#269 = PRODUCT_DEFINITION_SHAPE('', '', #270);
#270 = NEXT_ASSEMBLY_USAGE_OCCURRENCE('edge-2-1-1', 'Assy Step 2-1', '1-edge from node-2 to node-1', #195, #171, $);
#271 = CONTEXT_DEPENDENT_SHAPE_REPRESENTATION(#272, #274);
#272 = (
    REPRESENTATION_RELATIONSHIP('', '', #200, #176)
    REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION(#273)
    SHAPE_REPRESENTATION_RELATIONSHIP()
);
#273 = ITEM_DEFINED_TRANSFORMATION('', '', #7, #209);
#274 = PRODUCT_DEFINITION_SHAPE('', '', #275);
#275 = NEXT_ASSEMBLY_USAGE_OCCURRENCE('edge-2-1-2', 'Assy Step 2-2', '2-edge from node-2 to node-1', #195, #171, $);
#276 = CONTEXT_DEPENDENT_SHAPE_REPRESENTATION(#277, #279);
#277 = (
    REPRESENTATION_RELATIONSHIP('', '', #200, #176)
    REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION(#278)
    SHAPE_REPRESENTATION_RELATIONSHIP()
);
#278 = ITEM_DEFINED_TRANSFORMATION('', '', #7, #213);
#279 = PRODUCT_DEFINITION_SHAPE('', '', #280);
#280 = NEXT_ASSEMBLY_USAGE_OCCURRENCE('edge-2-1-3', 'Assy Step 2-3', '3-edge node from node-2 to node-1', #195, #171, $);
#281 = CONTEXT_DEPENDENT_SHAPE_REPRESENTATION(#282, #284);
#282 = (
    REPRESENTATION_RELATIONSHIP('', '', #224, #200)
    REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION(#283)
    SHAPE_REPRESENTATION_RELATIONSHIP()
);
#283 = ITEM_DEFINED_TRANSFORMATION('', '', #7, #225);
#284 = PRODUCT_DEFINITION_SHAPE('', '', #285);
#285 = NEXT_ASSEMBLY_USAGE_OCCURRENCE('edge-3-2-0', 'Assy Step 3-0', '0-edge from node-3 to node-2', #219, #195, $);
#286 = CONTEXT_DEPENDENT_SHAPE_REPRESENTATION(#287, #289);
#287 = (
    REPRESENTATION_RELATIONSHIP('', '', #224, #200)
    REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION(#288)
    SHAPE_REPRESENTATION_RELATIONSHIP()
);
#288 = ITEM_DEFINED_TRANSFORMATION('', '', #7, #229);
#289 = PRODUCT_DEFINITION_SHAPE('', '', #290);
#290 = NEXT_ASSEMBLY_USAGE_OCCURRENCE('edge-3-2-1', 'Assy Step 3-1', '1-edge from node-3 to node-2', #219, #195, $);
#291 = CONTEXT_DEPENDENT_SHAPE_REPRESENTATION(#292, #294);
#292 = (
    REPRESENTATION_RELATIONSHIP('', '', #224, #200)
    REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION(#293)
    SHAPE_REPRESENTATION_RELATIONSHIP()
);
#293 = ITEM_DEFINED_TRANSFORMATION('', '', #7, #233);
#294 = PRODUCT_DEFINITION_SHAPE('', '', #295);
#295 = NEXT_ASSEMBLY_USAGE_OCCURRENCE('edge-3-2-2', 'Assy Step 3-2', '2-edge from node-3 to node-2', #219, #195, $);
#296 = CONTEXT_DEPENDENT_SHAPE_REPRESENTATION(#297, #299);
#297 = (
    REPRESENTATION_RELATIONSHIP('', '', #224, #200)
    REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION(#298)
    SHAPE_REPRESENTATION_RELATIONSHIP()
);
#298 = ITEM_DEFINED_TRANSFORMATION('', '', #7, #237);
#299 = PRODUCT_DEFINITION_SHAPE('', '', #300);
#300 = NEXT_ASSEMBLY_USAGE_OCCURRENCE('edge-3-2-3', 'Assy Step 3-3', '3-edge node from node-3 to node-2', #219, #195, $);
ENDSEC;
END-ISO-10303-21;
"
        ),
    )
}
