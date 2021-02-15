//! Modeling a unit cube by three sweeps.

mod framework;
use framework::ShapeViewer;
use truck_modeling::*;

fn main() {
    let v0 = builder::vertex(Point3::new(0.0, 1.0, 0.0));
    let v1 = builder::vertex(Point3::new(0.0, 0.0, 1.0));
    //let edge = builder::line(&v0, &v1);
    let edge = builder::circle_arc(&v0, &v1, Point3::new(0.0, 1.0, 1.0) / f64::sqrt(2.0));
    ShapeViewer::run(builder::rsweep(&edge, Point3::origin(), Vector3::unit_y(), Rad(7.0)), 0.01);
}