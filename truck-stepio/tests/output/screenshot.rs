use truck_modeling::*;
use truck_stepio::out::*;

#[test]
fn default_shell() {
    let cshell = Shell::new().compress();
    let header: StepHeaderDescriptor = Default::default();
    let time_stamp = header.time_stamp.clone();
    let design = StepDesign::<_, Matrix4>::from_model(StepModel::from(&cshell));
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
#7 = SHAPE_DEFINITION_REPRESENTATION(#8, #14);
#8 = PRODUCT_DEFINITION_SHAPE('', '', #9);
#9 = PRODUCT_DEFINITION('design', '', #10, #12);
#10 = PRODUCT_DEFINITION_FORMATION('', '', #11);
#11 = PRODUCT('', '', '', (#13));
#12 = DESIGN_CONTEXT('', #1, 'design');
#13 = MECHANICAL_CONTEXT('', #1, 'mechanical');
#14 = SHAPE_REPRESENTATION('', (#15), #2);
#15 = SHELL_BASED_SURFACE_MODEL('', (#16));
#16 = OPEN_SHELL('', ());
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
    let design = StepDesign::<_, Matrix4>::from_model(StepModel::from(&csolid));
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
#7 = SHAPE_DEFINITION_REPRESENTATION(#8, #14);
#8 = PRODUCT_DEFINITION_SHAPE('', '', #9);
#9 = PRODUCT_DEFINITION('design', '', #10, #12);
#10 = PRODUCT_DEFINITION_FORMATION('', '', #11);
#11 = PRODUCT('', '', '', (#13));
#12 = DESIGN_CONTEXT('', #1, 'design');
#13 = MECHANICAL_CONTEXT('', #1, 'mechanical');
#14 = SHAPE_REPRESENTATION('', (#15), #2);
#15 = MANIFOLD_SOLID_BREP('', #16);
#16 = CLOSED_SHELL('', (#17, #24, #31, #38, #45, #52));
#17 = FACE_SURFACE('', (#18), #79, .F.);
#18 = FACE_BOUND('', #19, .F.);
#19 = EDGE_LOOP('', (#20, #21, #22, #23));
#20 = ORIENTED_EDGE('', *, *, #59, .T.);
#21 = ORIENTED_EDGE('', *, *, #60, .T.);
#22 = ORIENTED_EDGE('', *, *, #61, .F.);
#23 = ORIENTED_EDGE('', *, *, #62, .F.);
#24 = FACE_SURFACE('', (#25), #84, .T.);
#25 = FACE_BOUND('', #26, .T.);
#26 = EDGE_LOOP('', (#27, #28, #29, #30));
#27 = ORIENTED_EDGE('', *, *, #59, .T.);
#28 = ORIENTED_EDGE('', *, *, #63, .T.);
#29 = ORIENTED_EDGE('', *, *, #64, .F.);
#30 = ORIENTED_EDGE('', *, *, #65, .F.);
#31 = FACE_SURFACE('', (#32), #89, .T.);
#32 = FACE_BOUND('', #33, .T.);
#33 = EDGE_LOOP('', (#34, #35, #36, #37));
#34 = ORIENTED_EDGE('', *, *, #60, .T.);
#35 = ORIENTED_EDGE('', *, *, #66, .T.);
#36 = ORIENTED_EDGE('', *, *, #67, .F.);
#37 = ORIENTED_EDGE('', *, *, #63, .F.);
#38 = FACE_SURFACE('', (#39), #94, .F.);
#39 = FACE_BOUND('', #40, .F.);
#40 = EDGE_LOOP('', (#41, #42, #43, #44));
#41 = ORIENTED_EDGE('', *, *, #66, .T.);
#42 = ORIENTED_EDGE('', *, *, #68, .F.);
#43 = ORIENTED_EDGE('', *, *, #69, .F.);
#44 = ORIENTED_EDGE('', *, *, #61, .T.);
#45 = FACE_SURFACE('', (#46), #99, .F.);
#46 = FACE_BOUND('', #47, .F.);
#47 = EDGE_LOOP('', (#48, #49, #50, #51));
#48 = ORIENTED_EDGE('', *, *, #69, .T.);
#49 = ORIENTED_EDGE('', *, *, #70, .F.);
#50 = ORIENTED_EDGE('', *, *, #65, .F.);
#51 = ORIENTED_EDGE('', *, *, #62, .T.);
#52 = FACE_SURFACE('', (#53), #104, .T.);
#53 = FACE_BOUND('', #54, .T.);
#54 = EDGE_LOOP('', (#55, #56, #57, #58));
#55 = ORIENTED_EDGE('', *, *, #64, .T.);
#56 = ORIENTED_EDGE('', *, *, #67, .T.);
#57 = ORIENTED_EDGE('', *, *, #68, .F.);
#58 = ORIENTED_EDGE('', *, *, #70, .F.);
#59 = EDGE_CURVE('', #71, #72, #109, .T.);
#60 = EDGE_CURVE('', #72, #73, #113, .T.);
#61 = EDGE_CURVE('', #74, #73, #117, .T.);
#62 = EDGE_CURVE('', #71, #74, #121, .T.);
#63 = EDGE_CURVE('', #72, #75, #125, .T.);
#64 = EDGE_CURVE('', #76, #75, #129, .T.);
#65 = EDGE_CURVE('', #71, #76, #133, .T.);
#66 = EDGE_CURVE('', #73, #77, #137, .T.);
#67 = EDGE_CURVE('', #75, #77, #141, .T.);
#68 = EDGE_CURVE('', #78, #77, #145, .T.);
#69 = EDGE_CURVE('', #74, #78, #149, .T.);
#70 = EDGE_CURVE('', #76, #78, #153, .T.);
#71 = VERTEX_POINT('', #157);
#72 = VERTEX_POINT('', #158);
#73 = VERTEX_POINT('', #159);
#74 = VERTEX_POINT('', #160);
#75 = VERTEX_POINT('', #161);
#76 = VERTEX_POINT('', #162);
#77 = VERTEX_POINT('', #163);
#78 = VERTEX_POINT('', #164);
#79 = PLANE('', #80);
#80 = AXIS2_PLACEMENT_3D('', #81, #82, #83);
#81 = CARTESIAN_POINT('', (-0.5, -0.5, -0.5));
#82 = DIRECTION('', (0.0, 0.0, 1.0));
#83 = DIRECTION('', (1.0, 0.0, 0.0));
#84 = PLANE('', #85);
#85 = AXIS2_PLACEMENT_3D('', #86, #87, #88);
#86 = CARTESIAN_POINT('', (-0.5, -0.5, -0.5));
#87 = DIRECTION('', (0.0, -1.0, 0.0));
#88 = DIRECTION('', (1.0, 0.0, 0.0));
#89 = PLANE('', #90);
#90 = AXIS2_PLACEMENT_3D('', #91, #92, #93);
#91 = CARTESIAN_POINT('', (0.5, -0.5, -0.5));
#92 = DIRECTION('', (1.0, 0.0, 0.0));
#93 = DIRECTION('', (0.0, 1.0, 0.0));
#94 = PLANE('', #95);
#95 = AXIS2_PLACEMENT_3D('', #96, #97, #98);
#96 = CARTESIAN_POINT('', (-0.5, 0.5, -0.5));
#97 = DIRECTION('', (0.0, -1.0, 0.0));
#98 = DIRECTION('', (1.0, 0.0, 0.0));
#99 = PLANE('', #100);
#100 = AXIS2_PLACEMENT_3D('', #101, #102, #103);
#101 = CARTESIAN_POINT('', (-0.5, -0.5, -0.5));
#102 = DIRECTION('', (1.0, 0.0, 0.0));
#103 = DIRECTION('', (0.0, 1.0, 0.0));
#104 = PLANE('', #105);
#105 = AXIS2_PLACEMENT_3D('', #106, #107, #108);
#106 = CARTESIAN_POINT('', (-0.5, -0.5, 0.5));
#107 = DIRECTION('', (0.0, 0.0, 1.0));
#108 = DIRECTION('', (1.0, 0.0, 0.0));
#109 = LINE('', #110, #111);
#110 = CARTESIAN_POINT('', (-0.5, -0.5, -0.5));
#111 = VECTOR('', #112, 1.0);
#112 = DIRECTION('', (1.0, 0.0, 0.0));
#113 = LINE('', #114, #115);
#114 = CARTESIAN_POINT('', (0.5, -0.5, -0.5));
#115 = VECTOR('', #116, 1.0);
#116 = DIRECTION('', (0.0, 1.0, 0.0));
#117 = LINE('', #118, #119);
#118 = CARTESIAN_POINT('', (-0.5, 0.5, -0.5));
#119 = VECTOR('', #120, 1.0);
#120 = DIRECTION('', (1.0, 0.0, 0.0));
#121 = LINE('', #122, #123);
#122 = CARTESIAN_POINT('', (-0.5, -0.5, -0.5));
#123 = VECTOR('', #124, 1.0);
#124 = DIRECTION('', (0.0, 1.0, 0.0));
#125 = LINE('', #126, #127);
#126 = CARTESIAN_POINT('', (0.5, -0.5, -0.5));
#127 = VECTOR('', #128, 1.0);
#128 = DIRECTION('', (0.0, 0.0, 1.0));
#129 = LINE('', #130, #131);
#130 = CARTESIAN_POINT('', (-0.5, -0.5, 0.5));
#131 = VECTOR('', #132, 1.0);
#132 = DIRECTION('', (1.0, 0.0, 0.0));
#133 = LINE('', #134, #135);
#134 = CARTESIAN_POINT('', (-0.5, -0.5, -0.5));
#135 = VECTOR('', #136, 1.0);
#136 = DIRECTION('', (0.0, 0.0, 1.0));
#137 = LINE('', #138, #139);
#138 = CARTESIAN_POINT('', (0.5, 0.5, -0.5));
#139 = VECTOR('', #140, 1.0);
#140 = DIRECTION('', (0.0, 0.0, 1.0));
#141 = LINE('', #142, #143);
#142 = CARTESIAN_POINT('', (0.5, -0.5, 0.5));
#143 = VECTOR('', #144, 1.0);
#144 = DIRECTION('', (0.0, 1.0, 0.0));
#145 = LINE('', #146, #147);
#146 = CARTESIAN_POINT('', (-0.5, 0.5, 0.5));
#147 = VECTOR('', #148, 1.0);
#148 = DIRECTION('', (1.0, 0.0, 0.0));
#149 = LINE('', #150, #151);
#150 = CARTESIAN_POINT('', (-0.5, 0.5, -0.5));
#151 = VECTOR('', #152, 1.0);
#152 = DIRECTION('', (0.0, 0.0, 1.0));
#153 = LINE('', #154, #155);
#154 = CARTESIAN_POINT('', (-0.5, -0.5, 0.5));
#155 = VECTOR('', #156, 1.0);
#156 = DIRECTION('', (0.0, 1.0, 0.0));
#157 = CARTESIAN_POINT('', (-0.5, -0.5, -0.5));
#158 = CARTESIAN_POINT('', (0.5, -0.5, -0.5));
#159 = CARTESIAN_POINT('', (0.5, 0.5, -0.5));
#160 = CARTESIAN_POINT('', (-0.5, 0.5, -0.5));
#161 = CARTESIAN_POINT('', (0.5, -0.5, 0.5));
#162 = CARTESIAN_POINT('', (-0.5, -0.5, 0.5));
#163 = CARTESIAN_POINT('', (0.5, 0.5, 0.5));
#164 = CARTESIAN_POINT('', (-0.5, 0.5, 0.5));
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
    let design = StepDesign::<_, Matrix4>::from_model(StepModel::from(&cshell));
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
#7 = SHAPE_DEFINITION_REPRESENTATION(#8, #14);
#8 = PRODUCT_DEFINITION_SHAPE('', '', #9);
#9 = PRODUCT_DEFINITION('design', '', #10, #12);
#10 = PRODUCT_DEFINITION_FORMATION('', '', #11);
#11 = PRODUCT('', '', '', (#13));
#12 = DESIGN_CONTEXT('', #1, 'design');
#13 = MECHANICAL_CONTEXT('', #1, 'mechanical');
#14 = SHAPE_REPRESENTATION('', (#15), #2);
#15 = SHELL_BASED_SURFACE_MODEL('', (#16));
#16 = OPEN_SHELL('', ());
ENDSEC;
END-ISO-10303-21;\n"
        )
    );
}
