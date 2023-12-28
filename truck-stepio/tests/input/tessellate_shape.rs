use truck_meshalgo::prelude::*;
use truck_stepio::r#in::*;
use truck_topology::shell::ShellCondition;

const STEP_DIRECTORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../resources/step/");

const STEP_FILES: &[&str] = &[
    "occt-cone.step",
    "occt-cube.step",
    "occt-cylinder.step",
    "occt-sphere.step",
    "occt-torus.step",
    "abc-0000.step",
    "abc-0006.step",
    "abc-0035.step",
];

#[test]
fn tessellate_shape() {
    STEP_FILES.iter().for_each(|name| {
        let path = [STEP_DIRECTORY, name].concat();
        let step_string = std::fs::read_to_string(path).unwrap();
        let table = Table::from_step(&step_string).unwrap();
        table.shell.values().cloned().for_each(|step_shell| {
            let cshell = table.to_compressed_shell(&step_shell).unwrap();
            let mut poly = cshell.triangulation(0.01).to_polygon();
            poly.put_together_same_attrs().remove_degenerate_faces();
            assert_eq!(poly.shell_condition(), ShellCondition::Closed, "{name}");
        });
    });
}
