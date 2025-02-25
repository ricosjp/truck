//! A cube punched by a torus.
//!
//! Generated json file can be visualized by `simple-shape-viewer`, an example of `truck-rendimpl`.

use std::f64::consts::PI;
use truck_modeling::*;

fn cube_shell() -> Shell {
    let v = builder::vertex(Point3::origin());
    let e = builder::tsweep(&v, Vector3::unit_x());
    let f = builder::tsweep(&e, Vector3::unit_y());
    let s = builder::tsweep(&f, Vector3::unit_z());
    s.into_boundaries().pop().unwrap()
}

fn torus_shell() -> Shell {
    let v = builder::vertex(Point3::new(0.5, 0.0, 0.25));
    let w = builder::rsweep(
        &v,
        Point3::new(0.5, 0.0, 0.5),
        -Vector3::unit_y(),
        Rad(2.0 * PI),
    );
    builder::rsweep(&w, Point3::origin(), Vector3::unit_z(), Rad(PI / 2.0))
}

fn find_cube_face(cube_shell: &mut Shell, normal: Vector3) -> Option<&mut Face> {
    cube_shell.iter_mut().find(|face| {
        let surface = face.oriented_surface();
        surface.normal(0.5, 0.5).near(&normal)
    })
}

fn find_torus_boundary(bdds: &[Wire], idx: usize) -> Option<&Wire> {
    bdds.iter()
        .find(|wire| wire[0].front().point()[idx].near(&0.0))
}

fn main() {
    let mut shell = cube_shell();
    let torus = torus_shell();
    let bdds = torus.extract_boundaries();
    let face = find_cube_face(&mut shell, -Vector3::unit_y()).unwrap();
    let bdd = find_torus_boundary(&bdds, 1).unwrap();
    face.add_boundary(bdd.inverse());
    let face = find_cube_face(&mut shell, -Vector3::unit_x()).unwrap();
    let bdd = find_torus_boundary(&bdds, 0).unwrap();
    face.add_boundary(bdd.inverse());
    shell.extend(torus);
    let solid = Solid::new(vec![shell]);
    let json = serde_json::to_vec_pretty(&solid).unwrap();
    std::fs::write("torus-punched-cube.json", json).unwrap();
}
