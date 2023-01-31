//! Modeling a torus by two sweeps.
//!
//! Generated json file can be visualized by `simple-shape-viewer`, an example of `truck-rendimpl`.

use truck_modeling::*;

fn modeling(radius0: f64, radius1: f64) -> Solid {
    let v = builder::vertex(Point3::new(radius0, 0.0, radius1));
    let w = builder::rsweep(
        &v,
        Point3::new(radius0, 0.0, 0.0),
        Vector3::unit_y(),
        Rad(7.0),
    );
    let shell = builder::rsweep(&w, Point3::origin(), Vector3::unit_z(), Rad(7.0));
    Solid::new(vec![shell])
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let (radius0, radius1, filename) = match args.len() {
        1 => (0.75, 0.25, "torus.json".to_string()),
        3 => (
            args[1].parse().expect("invalid input"),
            args[2].parse().expect("invalid input"),
            "torus.json".to_string(),
        ),
        4 => (
            args[1].parse().expect("invalid input"),
            args[2].parse().expect("invalid input"),
            args[3].clone(),
        ),
        _ => panic!("the number of arguments must be 1, 3 or 4"),
    };
    let torus = modeling(radius0, radius1);
    let json = serde_json::to_vec_pretty(&torus).unwrap();
    std::fs::write(filename, json).unwrap();
}
