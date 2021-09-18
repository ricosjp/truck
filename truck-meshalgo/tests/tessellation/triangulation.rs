use super::*;
use truck_topology::shell::ShellCondition;

const SHAPE_JSONS: [&'static [u8]; 3] = [
    include_bytes!("bottle.json"),
    include_bytes!("punched-cube.json"),
    include_bytes!("torus-punched-cube.json"),
];

#[test]
fn solid_is_closed() {
    for (i, json) in SHAPE_JSONS.iter().enumerate() {
        let solid = Solid::extract(serde_json::from_reader(*json).unwrap()).unwrap();
        let mut poly = solid.triangulation(0.02).unwrap().into_polygon();
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
    let solid = Solid::extract(serde_json::from_slice(SHAPE_JSONS[2]).unwrap()).unwrap();
    let res = solid.triangulation(0.01).unwrap().into_polygon();
    let ans = obj::read(include_bytes!("by_occt.obj").as_ref()).unwrap();
    assert!(res.is_clung_to_by(ans.positions(), 0.05));
    assert!(ans.is_clung_to_by(res.positions(), 0.05));
}

#[test]
fn large_number_meshing() {
    const RADIUS0: f64 = 500.0;
    const RADIUS1: f64 = 100.0;
    let vertex = builder::vertex(Point3::new(RADIUS0, 0.0, RADIUS1));
    let circle: Wire = builder::rsweep(
        &vertex,
        Point3::new(RADIUS0, 0.0, 0.0),
        Vector3::unit_y(),
        Rad(7.0),
    );
    let disk = builder::try_attach_plane(&vec![circle]).unwrap();
    let torus = builder::rsweep(
        &disk,
        Point3::origin(),
        Vector3::unit_z(),
        Rad(7.0),
    );
    let _ = torus.triangulation(1.0).unwrap().into_polygon();
}
