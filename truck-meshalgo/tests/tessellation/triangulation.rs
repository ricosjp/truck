use super::*;
use truck_topology::shell::ShellCondition;

const SHAPE_JSONS: [&'static [u8]; 3] = [
    include_bytes!("bottle.json"),
    include_bytes!("punched-cube.json"),
    include_bytes!("torus-punched-cube.json"),
];

#[test]
fn solid_is_closed() {
    for json in SHAPE_JSONS.iter() {
        let solid = Solid::extract(serde_json::from_reader(*json).unwrap()).unwrap();
        let mut poly = solid.triangulation(0.01).unwrap();
        poly.put_together_same_attrs()
            .remove_degenerate_faces()
            .remove_unused_attrs();
        assert_eq!(poly.shell_condition(), ShellCondition::Closed);
    }
}

#[test]
fn compare_occt_mesh() {
    let solid = Solid::extract(serde_json::from_slice(SHAPE_JSONS[2]).unwrap()).unwrap();
    let res = solid.triangulation(0.01).unwrap();
    let ans = obj::read(include_bytes!("by_occt.obj").as_ref()).unwrap();
    assert!(res.is_clung_to_by(ans.positions(), 0.05));
    assert!(ans.is_clung_to_by(res.positions(), 0.05));
}
