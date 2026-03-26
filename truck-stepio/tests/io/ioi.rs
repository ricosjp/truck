const STEP_DIRECTORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../resources/step/");

const STEP_SHAPE_FILES: &[&str] = &[
    "occt-cone.step",
    "occt-cube.step",
    "occt-cylinder.step",
    "occt-sphere.step",
    "occt-torus.step",
    "abc-0000.step",
    "abc-0006.step",
    "abc-0008.step",
    "abc-0035.step",
];

const STEP_ASSY_FILE: &str = "occt-assy.step";

#[test]
fn ioi() {
    use truck_meshalgo::prelude::*;
    use truck_stepio::{out::*, r#in::*};
    use truck_topology::shell::ShellCondition;

    STEP_SHAPE_FILES.iter().for_each(|file_name| {
        let input = [STEP_DIRECTORY, file_name].concat();
        let step_string = std::fs::read_to_string(input).unwrap();
        let table = Table::from_step(&step_string).unwrap();
        table.shell.values().cloned().for_each(|step_shell| {
            let cshell = table.to_compressed_shell(&step_shell).unwrap();
            let design = StepDesign::from_model(StepModel::from(&cshell));
            let step_string = StepDisplay::new(Default::default(), design).to_string();
            let table = Table::from_step(&step_string).unwrap();
            table.shell.values().cloned().for_each(|step_shell| {
                let cshell = table.to_compressed_shell(&step_shell).unwrap();
                let bdb = cshell.triangulation(0.01).to_polygon().bounding_box();
                let diag = bdb.max() - bdb.min();
                let r = diag.x.min(diag.y).min(diag.z);
                let mut poly = cshell.triangulation(0.01 * r).to_polygon();
                poly.put_together_same_attrs(TOLERANCE * 50.0)
                    .remove_degenerate_faces();
                assert_eq!(
                    poly.shell_condition(),
                    ShellCondition::Closed,
                    "mesh is not closed: {file_name}",
                );
            })
        });
    });
}

#[test]
fn assy_ioi() {
    use convert::ProductShape;
    use truck_assembly::assy::{EdgeEntity, NodeEntity};
    use truck_stepio::{
        out::*,
        r#in::{step_geometry::*, *},
    };

    let input = [STEP_DIRECTORY, STEP_ASSY_FILE].concat();
    let step_string = std::fs::read_to_string(input).unwrap();
    let table = Table::from_step(&step_string).unwrap();
    let step_assy = table.step_assy().unwrap();

    let assy = step_assy.map(
        |NodeEntity { shape, attrs }| {
            let shape = shape.iter().find_map(|shape| match shape {
                ProductShape::Solid(solid) => Some(StepModel::from(solid)),
                _ => None,
            });
            NodeEntity {
                shape,
                attrs: attrs.clone(),
            }
        },
        |EdgeEntity { matrix, attrs }| {
            let matrix = Matrix4::try_from(matrix).unwrap();
            EdgeEntity {
                matrix: Matrix4::try_from(matrix).unwrap(),
                attrs: attrs.clone(),
            }
        },
    );
    let design = StepDesign::new(assy);
    let re_step_string = StepDisplay::new(Default::default(), design).to_string();

    let re_table = Table::from_step(&re_step_string).unwrap();
    let re_step_assy = re_table.step_assy().unwrap();

    assert_eq!(step_assy.len(), re_step_assy.len());
    for node0 in step_assy.all_nodes() {
        let node1 = re_step_assy
            .all_nodes()
            .find(|node| node0.attrs() == node.attrs())
            .unwrap();
        assert_eq!(node0.edges().len(), node1.edges().len());
    }
}
