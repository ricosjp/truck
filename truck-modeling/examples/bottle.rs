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

fn body_shell(bottom: f64, height: f64, width: f64, thickness: f64) -> Shell {
    let vertex0 = builder::vertex(Point3::new(-width / 2.0, bottom, thickness / 4.0));
    let vertex1 = builder::vertex(Point3::new(width / 2.0, bottom, thickness / 4.0));
    let transit = Point3::new(0.0, bottom, thickness / 2.0);
    let arc0 = builder::circle_arc(&vertex0, &vertex1, transit);
    let arc1 = builder::rotated(&arc0, Point3::origin(), Vector3::unit_y(), Rad(PI));
    let face = builder::homotopy(&arc0, &arc1.inverse());
    let solid = builder::tsweep(&face, Vector3::new(0.0, height, 0.0));
    solid.into_boundaries().pop().unwrap()
}

fn cylinder(bottom: f64, height: f64, radius: f64) -> Shell {
    let vertex = builder::vertex(Point3::new(0.0, bottom, radius));
    let circle = builder::rsweep(&vertex, Point3::origin(), Vector3::unit_y(), Rad(7.0));
    let disk = builder::try_attach_plane(&vec![circle]).unwrap();
    let solid = builder::tsweep(&disk, Vector3::new(0.0, height, 0.0));
    solid.into_boundaries().pop().unwrap()
}

fn grue_body_neck(body: &mut Shell, neck: Shell) {
    let body_seiling = body.last_mut().unwrap();
    let wire = neck[0].boundaries()[0].clone();
    body_seiling.add_boundary(wire);
    body.extend(neck.into_iter().skip(1));
}

fn bottle(height: f64, width: f64, thickness: f64) -> Solid {
    let mut body = body_shell(0.0, height, width, thickness);
    let neck = cylinder(height, height / 10.0, thickness / 4.0);
    grue_body_neck(&mut body, neck);

    let eps = height / 50.0;
    let mut inner_body = body_shell(
        eps,
        height - 2.0 * eps,
        width - 2.0 * eps,
        thickness - 2.0 * eps,
    );
    let inner_neck = cylinder(height - eps, height / 10.0 + eps, thickness / 4.0 - eps);
    grue_body_neck(&mut inner_body, inner_neck);

    let inner_hat = inner_body.pop().unwrap();
    let wire = inner_hat.into_boundaries()[0].inverse();
    body.last_mut().unwrap().add_boundary(wire);
    body.extend(inner_body.into_iter().map(|face| face.inverse()));
    
    Solid::new(vec![body])
}

fn main() {
    let bottle = bottle(1.4, 1.0, 0.6);
    let bottle = builder::translated(&bottle, Vector3::new(0.0, -0.7, 0.0));
    ShapeViewer::run(bottle, 0.005);
}
