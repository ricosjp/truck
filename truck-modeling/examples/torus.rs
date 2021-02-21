//! Modeling a torus by two sweeps.

use truck_modeling::*;

fn main() {
    let v = builder::vertex(Point3::new(0.5, 0.0, 0.0));
    let w = builder::rsweep(&v, Point3::new(0.75, 0.0, 0.0), Vector3::unit_y(), Rad(7.0));
    let shell = builder::rsweep(&w, Point3::origin(), Vector3::unit_z(), Rad(7.0));
    let torus = Solid::new(vec![shell]);
    let json = serde_json::to_vec_pretty(&torus.compress()).unwrap();
    std::fs::write("torus.json", &json).unwrap();
}
