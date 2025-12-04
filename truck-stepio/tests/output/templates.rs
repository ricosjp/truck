use truck_modeling::*;
use truck_stepio::out::*;

#[test]
fn default_shell_template() {
    let cshell = Shell::new().compress();
    let header: StepHeaderDescriptor = Default::default();
    let time_stamp = header.time_stamp.clone();
    let step_string = CompleteStepDisplay::new(StepModel::from(&cshell), header).to_string();
    assert_eq!(
        step_string,
        format!(
            "ISO-10303-21;
HEADER;
FILE_DESCRIPTION(('Shape Data from Truck'), '2;1');
FILE_NAME('', '{time_stamp}', (''), (''), 'truck', '', '');
FILE_SCHEMA(('ISO-10303-042'));
ENDSEC;
DATA;
#1 = APPLICATION_PROTOCOL_DEFINITION('international standard', 'automotive_design', 2000, #2);
#2 = APPLICATION_CONTEXT('core data for automotive mechanical design processes');
#3 = SHAPE_DEFINITION_REPRESENTATION(#4, #10);
#4 = PRODUCT_DEFINITION_SHAPE('','', #5);
#5 = PRODUCT_DEFINITION('design','', #6, #9);
#6 = PRODUCT_DEFINITION_FORMATION('','', #7);
#7 = PRODUCT('','','', (#8));
#8 = PRODUCT_CONTEXT('', #2, 'mechanical');
#9 = PRODUCT_DEFINITION_CONTEXT('part definition', #2, 'design');
#10 = SHAPE_REPRESENTATION('', (#16), #11);
#11 = (
    GEOMETRIC_REPRESENTATION_CONTEXT(3)
    GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#15))
    GLOBAL_UNIT_ASSIGNED_CONTEXT((#12, #13, #14))
    REPRESENTATION_CONTEXT('Context #1', '3D Context with UNIT and UNCERTAINTY')
);
#12 = ( LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.) );
#13 = ( NAMED_UNIT(*) PLANE_ANGLE_UNIT() SI_UNIT($,.RADIAN.) );
#14 = ( NAMED_UNIT(*) SI_UNIT($,.STERADIAN.) SOLID_ANGLE_UNIT() );
#15 = UNCERTAINTY_MEASURE_WITH_UNIT(LENGTH_MEASURE(1.0E-6), #12, 'distance_accuracy_value','confusion accuracy');
#16 = SHELL_BASED_SURFACE_MODEL('', (#17));
#17 = OPEN_SHELL('', ());
ENDSEC;
END-ISO-10303-21;\n"
        )
    );
}

#[test]
fn default_solid_template() {
    use std::io::{BufRead, BufReader};
    let json = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../resources/shape/cube.json"
    ));
    let csolid: CompressedSolid = serde_json::from_reader(json.as_slice()).unwrap();
    let header: StepHeaderDescriptor = Default::default();
    let time_stamp = header.time_stamp.clone();
    let step_string = CompleteStepDisplay::new(StepModel::from(&csolid), header).to_string();
    let lines = BufReader::new(step_string.as_bytes())
        .lines()
        .map(|line| line.unwrap())
        .collect::<Vec<_>>();
    let string = lines[0..28].iter().fold(String::new(), |s, l| s + l + "\n");
    assert_eq!(
        string,
        format!("ISO-10303-21;
HEADER;
FILE_DESCRIPTION(('Shape Data from Truck'), '2;1');
FILE_NAME('', '{time_stamp}', (''), (''), 'truck', '', '');
FILE_SCHEMA(('ISO-10303-042'));
ENDSEC;
DATA;
#1 = APPLICATION_PROTOCOL_DEFINITION('international standard', 'automotive_design', 2000, #2);
#2 = APPLICATION_CONTEXT('core data for automotive mechanical design processes');
#3 = SHAPE_DEFINITION_REPRESENTATION(#4, #10);
#4 = PRODUCT_DEFINITION_SHAPE('','', #5);
#5 = PRODUCT_DEFINITION('design','', #6, #9);
#6 = PRODUCT_DEFINITION_FORMATION('','', #7);
#7 = PRODUCT('','','', (#8));
#8 = PRODUCT_CONTEXT('', #2, 'mechanical');
#9 = PRODUCT_DEFINITION_CONTEXT('part definition', #2, 'design');
#10 = SHAPE_REPRESENTATION('', (#16), #11);
#11 = (
    GEOMETRIC_REPRESENTATION_CONTEXT(3)
    GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#15))
    GLOBAL_UNIT_ASSIGNED_CONTEXT((#12, #13, #14))
    REPRESENTATION_CONTEXT('Context #1', '3D Context with UNIT and UNCERTAINTY')
);
#12 = ( LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.) );
#13 = ( NAMED_UNIT(*) PLANE_ANGLE_UNIT() SI_UNIT($,.RADIAN.) );
#14 = ( NAMED_UNIT(*) SI_UNIT($,.STERADIAN.) SOLID_ANGLE_UNIT() );
#15 = UNCERTAINTY_MEASURE_WITH_UNIT(LENGTH_MEASURE(1.0E-6), #12, 'distance_accuracy_value','confusion accuracy');
#16 = MANIFOLD_SOLID_BREP('', #17);\n")
    );

    let len = lines.len();
    let string = lines[len - 2..]
        .iter()
        .fold(String::new(), |s, l| s + l + "\n");
    assert_eq!(&string, "ENDSEC;\nEND-ISO-10303-21;\n");
}

#[test]
fn custom_header_template() {
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
    let step_string = CompleteStepDisplay::new(StepModel::from(&cshell), header).to_string();
    assert_eq!(
        step_string,
        format!(
            "ISO-10303-21;
HEADER;
FILE_DESCRIPTION(('Shape Data from Truck'), '2;1');
FILE_NAME('AMAZING_PRODUCT.step', '{time_stamp}', ('Tensai', 'Genius', 'Bokusama'), \
('Great Awesome Co. Ltd.', 'Univ. Clever Genius', 'Senkai'), 'truck', 'Kusanagi Sword', 'IT'S ME!');
FILE_SCHEMA(('ISO-10303-042'));
ENDSEC;
DATA;
#1 = APPLICATION_PROTOCOL_DEFINITION('international standard', 'automotive_design', 2000, #2);
#2 = APPLICATION_CONTEXT('core data for automotive mechanical design processes');
#3 = SHAPE_DEFINITION_REPRESENTATION(#4, #10);
#4 = PRODUCT_DEFINITION_SHAPE('','', #5);
#5 = PRODUCT_DEFINITION('design','', #6, #9);
#6 = PRODUCT_DEFINITION_FORMATION('','', #7);
#7 = PRODUCT('','','', (#8));
#8 = PRODUCT_CONTEXT('', #2, 'mechanical');
#9 = PRODUCT_DEFINITION_CONTEXT('part definition', #2, 'design');
#10 = SHAPE_REPRESENTATION('', (#16), #11);
#11 = (
    GEOMETRIC_REPRESENTATION_CONTEXT(3)
    GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#15))
    GLOBAL_UNIT_ASSIGNED_CONTEXT((#12, #13, #14))
    REPRESENTATION_CONTEXT('Context #1', '3D Context with UNIT and UNCERTAINTY')
);
#12 = ( LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.) );
#13 = ( NAMED_UNIT(*) PLANE_ANGLE_UNIT() SI_UNIT($,.RADIAN.) );
#14 = ( NAMED_UNIT(*) SI_UNIT($,.STERADIAN.) SOLID_ANGLE_UNIT() );
#15 = UNCERTAINTY_MEASURE_WITH_UNIT(LENGTH_MEASURE(1.0E-6), #12, 'distance_accuracy_value','confusion accuracy');
#16 = SHELL_BASED_SURFACE_MODEL('', (#17));
#17 = OPEN_SHELL('', ());
ENDSEC;
END-ISO-10303-21;\n"
        )
    );
}
