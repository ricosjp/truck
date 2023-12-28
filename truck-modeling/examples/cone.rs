//! Modeling cone.
//!
//! Generated json file can be visualized by `simple-shape-viewer`, an example of `truck-rendimpl`.

use truck_modeling::*;

fn main() {
    let v0 = builder::vertex(Point3::new(0.0, 0.5, 0.0));
    let v1 = builder::vertex(Point3::new(0.0, -0.5, 0.5));
    let v2 = builder::vertex(Point3::new(0.0, -0.5, 0.0));
    let wire: Wire = vec![builder::line(&v0, &v1), builder::line(&v1, &v2)].into();
    let shell = builder::cone(&wire, Vector3::unit_y(), Rad(7.0));
    let cone = Solid::new(vec![shell]);
    let json = serde_json::to_vec_pretty(&cone).unwrap();
    std::fs::write("cone.json", json).unwrap();
}
