//! Modeling a bottle.
//!
//! This is a technical indicator for comparing with Open CASCADE Technology, a great senior.
//! We want to reproduce the bottle made in the [OCCT tutorial].
//!
//! When the `fillet` feature is enabled, the body edges are filleted just like
//! the OCCT tutorial (`BRepFilletAPI_MakeFillet` at radius = thickness / 12).
//!
//! Generated json file can be visualized by `simple-shape-viewer`, an example of `truck-rendimpl`.
//!
//! ```bash
//! cargo run -p truck-modeling --features fillet --example bottle
//! ```
//!
//! [OCCT tutorial]: https://dev.opencascade.org/doc/overview/html/occt__tutorial.html

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
    let circle = builder::rsweep(&vertex, Point3::origin(), Vector3::unit_y(), Rad(7.0), 2);
    let disk = builder::try_attach_plane(&[circle]).unwrap();
    let solid = builder::tsweep(&disk, Vector3::new(0.0, height, 0.0));
    solid.into_boundaries().pop().unwrap()
}

fn grue_body_neck(body: &mut Shell, neck: Shell) {
    let body_seiling = body.last_mut().unwrap();
    let wire = neck[0].boundaries()[0].clone();
    body_seiling.add_boundary(wire);
    body.extend(neck.into_iter().skip(1));
}

/// Collect the vertical `Line` edges of a body shell (those spanning the full height).
#[cfg(feature = "fillet")]
fn vertical_line_edges(shell: &Shell, height: f64) -> Vec<Edge> {
    let mut edge_face_count = std::collections::HashMap::new();
    for face in shell.face_iter() {
        for wire in face.boundaries() {
            for edge in wire.edge_iter() {
                *edge_face_count.entry(edge.id()).or_insert(0u32) += 1;
            }
        }
    }
    let mut seen = std::collections::HashSet::new();
    shell
        .edge_iter()
        .filter(|e| seen.insert(e.id()))
        .filter(|e| edge_face_count[&e.id()] == 2)
        .filter(|e| {
            matches!(e.oriented_curve(), Curve::Line(_)) && {
                let p0 = e.front().point();
                let p1 = e.back().point();
                (p0.y.abs() < 0.01 && (p1.y - height).abs() < 0.01)
                    || (p1.y.abs() < 0.01 && (p0.y - height).abs() < 0.01)
            }
        })
        .collect()
}

fn bottle(height: f64, width: f64, thickness: f64) -> Solid {
    let mut body = body_shell(0.0, height, width, thickness);

    // Fillet the body's vertical edges (Line edges spanning the full height)
    // matching the OCCT tutorial's BRepFilletAPI_MakeFillet at thickness / 12.
    #[cfg(feature = "fillet")]
    {
        let edges = vertical_line_edges(&body, height);
        let opts = FilletOptions {
            radius: RadiusSpec::Constant(thickness / 12.0),
            ..Default::default()
        };
        fillet_edges(&mut body, &edges, Some(&opts)).expect("body fillet");
    }

    let neck = cylinder(height, height / 10.0, thickness / 4.0);
    grue_body_neck(&mut body, neck);

    let eps = height / 50.0;
    let mut inner_body = body_shell(
        eps,
        height - 2.0 * eps,
        width - 2.0 * eps,
        thickness - 2.0 * eps,
    );

    #[cfg(feature = "fillet")]
    {
        let edges = vertical_line_edges(&inner_body, height - 2.0 * eps);
        let opts = FilletOptions {
            radius: RadiusSpec::Constant((thickness - 2.0 * eps) / 12.0),
            ..Default::default()
        };
        fillet_edges(&mut inner_body, &edges, Some(&opts)).expect("inner body fillet");
    }

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
    let json = serde_json::to_vec_pretty(&bottle).unwrap();
    std::fs::write("bottle.json", json).unwrap();
}
