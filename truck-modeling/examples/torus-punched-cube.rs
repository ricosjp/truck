//! A cube punched by a torus.

use std::f64::consts::PI;
use truck_modeling::*;

fn main() {
    let v = builder::vertex(Point3::origin());
    let e = builder::tsweep(&v, Vector3::unit_x());
    let f = builder::tsweep(&e, Vector3::unit_y());
    let s = builder::tsweep(&f, Vector3::unit_z());
    let mut shell = s.into_boundaries().pop().unwrap();
    let v = builder::vertex(Point3::new(0.5, 0.0, 0.25));
    let w = builder::rsweep(
        &v,
        Point3::new(0.5, 0.0, 0.5),
        -Vector3::unit_y(),
        Rad(2.0 * PI),
    );
    let torus = builder::rsweep(&w, Point3::origin(), Vector3::unit_z(), Rad(PI / 2.0));
    let bdds = torus.extract_boundaries();
    let face = shell
        .iter_mut()
        .find(|face| {
            let surface = face.oriented_surface();
            let normal = surface.normal(0.5, 0.5);
            normal.near(&-Vector3::unit_y())
        })
        .unwrap();
    let bdd = bdds
        .iter()
        .find(|wire| {
            let curve = wire[0].oriented_curve();
            let pt = curve.front();
            pt[1].near(&0.0)
        })
        .unwrap();
    face.add_boundary(bdd.inverse());
    let face = shell
        .iter_mut()
        .find(|face| {
            let surface = face.oriented_surface();
            let normal = surface.normal(0.5, 0.5);
            normal.near(&-Vector3::unit_x())
        })
        .unwrap();
    let bdd = bdds
        .iter()
        .find(|wire| {
            let curve = wire[0].oriented_curve();
            let pt = curve.front();
            pt[0].near(&0.0)
        })
        .unwrap();
    face.add_boundary(bdd.inverse());
    shell.extend(torus);
    let solid = Solid::new(vec![shell]);
    let json = serde_json::to_vec_pretty(&solid).unwrap();
    std::fs::write("torus-punched-cube.json", json).unwrap();
}
