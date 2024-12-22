//! Modeling a unit cube with a hole through it.
//!
//! Generated json file can be visualized by `simple-shape-viewer`, an example of `truck-rendimpl`.

use truck_modeling::*;

fn main() {
    let v = builder::vertex(Point3::new(-0.5, -0.5, -0.5));
    let edge = builder::tsweep(&v, Vector3::unit_x());
    let mut face = builder::tsweep(&edge, Vector3::unit_y());
    let v = builder::vertex(Point3::new(0.2, 0.0, -0.5));
    let edge0 = builder::tsweep(&v, Vector3::new(-0.2, 0.2, 0.0));
    let wire1 = builder::rsweep(
        edge0.back(),
        Point3::origin(),
        Vector3::unit_z(),
        Rad(std::f64::consts::PI / 2.0),
    );
    let edge2 = builder::tsweep(wire1.back_vertex().unwrap(), Vector3::new(0.2, -0.2, 0.0));
    let mut wire3 = builder::rsweep(
        edge2.back(),
        Point3::origin(),
        Vector3::unit_z(),
        Rad(std::f64::consts::PI / 2.0),
    );
    let back_edge = wire3.pop_back().unwrap();
    let tmp = Edge::new(back_edge.front(), edge0.front(), back_edge.curve());
    wire3.push_back(tmp);
    let mut wire = Wire::from_iter(
        std::iter::once(edge0)
            .chain(wire1)
            .chain(std::iter::once(edge2))
            .chain(wire3),
    );
    wire.invert();
    face.add_boundary(wire);
    let shape: Solid = builder::tsweep(&face, Vector3::unit_z());
    let json = serde_json::to_vec_pretty(&shape).unwrap();
    std::fs::write("punched-cube.json", json).unwrap();
}
