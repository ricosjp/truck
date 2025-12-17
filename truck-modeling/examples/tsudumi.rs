//! Modeling a one-leaf hyperboloid.
//!
//! Generated json file can be visualized by `simple-shape-viewer`, an example of `truck-rendimpl`.

use truck_modeling::*;

fn main() {
    let v0 = builder::vertex(Point3::new(1.0, 1.0, 0.0));
    let v1 = builder::vertex(Point3::new(0.0, -1.0, 1.0));
    let line = builder::line(&v0, &v1);
    let mut shell = builder::rsweep(&line, Point3::origin(), Vector3::unit_y(), Rad(7.0), 2);
    let wires = shell.extract_boundaries();
    shell.push(
        builder::try_attach_plane(&[wires[0].clone()])
            .unwrap()
            .inverse(),
    );
    shell.push(
        builder::try_attach_plane(&[wires[1].clone()])
            .unwrap()
            .inverse(),
    );
    let solid = Solid::new(vec![shell]);
    let json = serde_json::to_vec_pretty(&solid).unwrap();
    std::fs::write("tsudumi.json", json).unwrap();
}
