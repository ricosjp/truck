//! Modeling a unit cube by three sweeps.
//! 
//! Generated json file can be visualized by `simple-shape-viewer`, an example of `truck-rendimpl`.

use truck_modeling::*;

fn main() {
    let v = builder::vertex(Point3::new(-0.5, -0.5, -0.5));
    let e = builder::tsweep(&v, Vector3::unit_x());
    let f = builder::tsweep(&e, Vector3::unit_y());
    let cube = builder::tsweep(&f, Vector3::unit_z());
    let json = serde_json::to_vec_pretty(&cube.compress()).unwrap();
    std::fs::write("cube.json", &json).unwrap();
}
