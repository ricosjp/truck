use std::f64::consts::PI;
///! Modeling a sphere
use truck_modeling::*;

fn main() {
    let v0 = builder::vertex(Point3::new(0.0, 0.5, 0.0));
    let wire: Wire = builder::rsweep(&v0, Point3::origin(), Vector3::unit_x(), Rad(PI));
    let shell = builder::cone(&wire, Vector3::unit_y(), Rad(7.0));
    let sphere = Solid::new(vec![shell]);
    let json = serde_json::to_vec_pretty(&sphere).unwrap();
    std::fs::write("sphere.json", json).unwrap();
}
