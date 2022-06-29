use super::*;
use truck_topology::shell::ShellCondition;

macro_rules! dir ( () => { concat!(env!("CARGO_MANIFEST_DIR"), "/../resources/shape/") });

const SHAPE_JSONS: [&str; 3] = [
    concat!(dir!(), "bottle.json"),
    concat!(dir!(), "punched-cube.json"),
    concat!(dir!(), "torus-punched-cube.json"),
];

fn read_jsons() -> Vec<Vec<u8>> {
    let closure = |json| std::fs::read(json).unwrap();
    SHAPE_JSONS.iter().map(closure).collect()
}

#[test]
fn solid_is_closed() {
    for (i, json) in read_jsons().into_iter().enumerate() {
        let solid: Solid = serde_json::from_reader(json.as_slice()).unwrap();
        let mut poly = solid.triangulation(0.02).unwrap().to_polygon();
        poly.put_together_same_attrs()
            .remove_degenerate_faces()
            .remove_unused_attrs();
        assert_eq!(
            poly.shell_condition(),
            ShellCondition::Closed,
            "not closed: file no. {}",
            i
        );
    }
}

#[test]
fn compare_occt_mesh() {
    let jsons = read_jsons();
    let solid: Solid = serde_json::from_slice(jsons[2].as_slice()).unwrap();
    let res = solid.triangulation(0.01).unwrap().to_polygon();
    let path = concat!(dir!(), "../obj/by_occt.obj");
    let ans = obj::read(std::fs::read(path).unwrap().as_slice()).unwrap();
    assert!(res.is_clung_to_by(ans.positions(), 0.05));
    assert!(ans.is_clung_to_by(res.positions(), 0.05));
}

#[test]
fn large_number_meshing() {
    let json = std::fs::read(concat!(dir!(), "large-torus.json")).unwrap();
    let torus: Solid = serde_json::from_slice(json.as_slice()).unwrap();
    let _ = torus.triangulation(1.0).unwrap().to_polygon();
}
