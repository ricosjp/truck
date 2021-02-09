//! Modeling a unit cube by three sweeps.

mod framework;
use framework::ShapeViewer;
use truck_modeling::*;

fn main() {
    let v = builder::vertex(Point3::new(-0.5, -0.5, -0.5));
    let e = builder::tsweep(&v, Vector3::unit_x());
    let f = builder::tsweep(&e, Vector3::unit_y());
    let cube = builder::tsweep(&f, Vector3::unit_z());
    ShapeViewer::run(cube, 0.1);
}
