//! Modeling a unit cube by three sweeps.

mod framework;
use framework::ShapeViewer;
use truck_modeling::*;

fn main() {
    let v0 = builder::vertex(Point3::new(0.0, 0.5, 0.0));
    let v1 = builder::vertex(Point3::new(0.0, -0.5, 0.5));
    let v2 = builder::vertex(Point3::new(0.0, -0.5, 0.0));
    let wire: Wire = vec![builder::line(&v0, &v1), builder::line(&v1, &v2)].into();
    let cone = builder::cone(&wire, Vector3::unit_y(), Rad(7.0));
    for edge in cone.face_iter().flat_map(Face::boundary_iters).flatten() {
        println!("{:?}", edge);
    }
    ShapeViewer::run(cone, 0.01);
}
