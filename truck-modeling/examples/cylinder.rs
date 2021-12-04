//! Modeling a cylinder by two sweeps.
//!
//! Generated json file can be visualized by `simple-shape-viewer`, an example of `truck-rendimpl`.

use truck_modeling::*;

fn cylinder(height: f64, radius: f64) -> Solid {
    let vertex = builder::vertex(Point3::new(0.0, -height / 2.0, radius));
    let circle = builder::rsweep(&vertex, Point3::origin(), Vector3::unit_y(), Rad(7.0));
    let disk = builder::try_attach_plane(&vec![circle]).unwrap();
    let solid = builder::tsweep(&disk, Vector3::new(0.0, height, 0.0));
    solid
}

fn main() {
    let cylinder = cylinder(1.0, 0.5);
    let json = serde_json::to_vec_pretty(&cylinder.compress()).unwrap();
    std::fs::write("cylinder.json", &json).unwrap();
}
