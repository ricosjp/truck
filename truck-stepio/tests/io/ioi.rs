use truck_meshalgo::prelude::*;
use truck_stepio::{out::*, r#in::*};
use truck_topology::shell::ShellCondition;

const STEP_DIRECTORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../resources/step/");

const STEP_FILES: &[&str] = &[
    "occt-cone.step",
    "occt-cube.step",
    "occt-cylinder.step",
    "abc-0000.step",
    "abc-0006.step",
    "abc-0008.step",
    "abc-0035.step",
];

#[test]
fn ioi() {
    STEP_FILES.iter().for_each(|file_name| {
        println!("{file_name}");
        let input = [STEP_DIRECTORY, file_name].concat();
        let step_string = std::fs::read_to_string(&input).unwrap();
        let table = Table::from_step(&step_string).unwrap();
        table.shell.values().cloned().for_each(|step_shell| {
            let cshell = table.to_compressed_shell(&step_shell).unwrap();
            let step_string =
                CompleteStepDisplay::new(StepModel::from(&cshell), Default::default()).to_string();
            println!("{step_string}");
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
                    "{file_name}"
                );
            })
        });
    });
}
