use super::*;
use truck_topology::shell::ShellCondition;

const SHAPE_JSONS: [&'static [u8]; 2] = [
    include_bytes!("bottle.json"),
    include_bytes!("punched-cube.json"),
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
