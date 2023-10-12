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
];

#[test]
fn tessellate_shape() {
    STEP_FILES.iter().for_each(|name| {
        let path = [STEP_DIRECTORY, name].concat();
        let step_string = std::fs::read_to_string(path).unwrap();
        let table = Table::from_step(&step_string).unwrap();
        let step_shells = table.shell.values().cloned().collect::<Vec<_>>();
        assert_eq!(step_shells.len(), 1);
        let step_shell = step_shells.into_iter().next().unwrap();
        let cshell = table.to_compressed_shell(&step_shell).unwrap();
        let mut poly = cshell.triangulation(0.05).to_polygon();
        poly.put_together_same_attrs().remove_degenerate_faces();
        assert_eq!(poly.shell_condition(), ShellCondition::Closed);
    });
}
