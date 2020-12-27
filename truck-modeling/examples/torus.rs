//! Modeling a torus by two sweeps.

mod framework;
use framework::ShapeViewer;
use truck_modeling::*;

fn main() {
    let v = builder::vertex(Point3::new(0.5, 0.0, 0.0));
    let w = builder::rsweep(&v, Point3::new(0.75, 0.0, 0.0), Vector3::unit_y(), Rad(7.0));
    let torus = builder::rsweep(&w, Point3::origin(), Vector3::unit_z(), Rad(7.0));
    ShapeViewer::run(torus);
}
