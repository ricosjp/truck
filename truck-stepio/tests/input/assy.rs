use truck_stepio::r#in::*;

const STEP_DIRECTORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../resources/step/");

#[test]
fn occt_assy() {
    let step_string = std::fs::read_to_string([STEP_DIRECTORY, "occt-assy.step"].concat()).unwrap();
    let table = Table::from_step(&step_string).unwrap();
    let assy = table.step_assy().unwrap();
    assert_eq!(assy.len(), 3);
    let top = assy.top_nodes().next().unwrap();
    let paths = assy.maximal_paths_iter(top.index()).collect::<Vec<_>>();
    assert_eq!(paths.len(), 5);
}
