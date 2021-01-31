//! Modeling a bottle.
//! 
//! This is a technical indicator for comparing with Open CASCADE Technology, a great senior.
//! We want to reproduce the bottle made in the [OCCT tutorial].
//! Now, one cannot make a fillet or run boolean operations by truck.
//! So, the bottle made by this script is not completed.
//! 
//! [OCCT tutorial]: https://dev.opencascade.org/doc/overview/html/occt__tutorial.html

mod framework;
use framework::ShapeViewer;
use std::f64::consts::PI;
use truck_modeling::*;

fn bottle(width: f64, height: f64, thickness: f64) -> Solid {
    let vertex0 = builder::vertex(Point3::new(-width / 2.0, 0.0, thickness / 4.0));
    let vertex1 = builder::vertex(Point3::new(width / 2.0, 0.0, thickness / 4.0));
    let transit = Point3::new(0.0, 0.0, thickness / 2.0);
    let arc0 = builder::circle_arc(&vertex0, &vertex1, transit);
    let arc1 = builder::rotated(&arc0, Point3::origin(), Vector3::unit_y(), Rad(PI));
    let bottom = builder::homotopy(&arc0, &arc1.inverse());
    builder::tsweep(&bottom, Vector3::new(0.0, height, 0.0))
    //bottom.inverse()
}

fn main() {
    let bottle = bottle(2.0, 2.0, 2.0);
    //let bottle = builder::translated(&bottle, Vector3::new(0.0, -1.0, 0.0));
    let bottle = bottle.into_boundaries().pop().unwrap();
    for face in &bottle {
        println!("{:?}", face.oriented_surface());
    }
    ShapeViewer::run(bottle);
    //ShapeViewer::run(Shell::from(vec![bottle[0].clone()]));
}
