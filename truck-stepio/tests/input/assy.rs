use truck_assembly::assy::*;
use truck_stepio::r#in::*;

const STEP_DIRECTORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../resources/step/");

#[test]
fn occt_assy() {
    let step_string = std::fs::read_to_string([STEP_DIRECTORY, "occt-assy.step"].concat()).unwrap();
    let table = Table::from_step(&step_string).unwrap();
    let assy = Assembly::new();
    table.step_assy(&assy).unwrap();
    let top = assy.top_nodes()[0];
    let paths = top.maximul_paths_iter().collect::<Vec<_>>();
    assert_eq!(paths.len(), 5);
}
