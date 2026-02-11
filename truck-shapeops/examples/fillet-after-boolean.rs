//! Fillet an edge on a boolean (AND) result.
//!
//! Builds the same punched-cube as `punched-cube-shapeops`, then fillets
//! one of the remaining cube edges to demonstrate `fillet_edges_generic`
//! working on shells that contain `IntersectionCurve` edges.

use truck_modeling::*;

fn main() {
    // Unit cube at origin.
    let v = builder::vertex(Point3::origin());
    let e = builder::tsweep(&v, Vector3::unit_x());
    let f = builder::tsweep(&e, Vector3::unit_y());
    let cube: Solid = builder::tsweep(&f, Vector3::unit_z());

    // Cylinder through the cube (same as punched-cube-shapeops example).
    let v = builder::vertex(Point3::new(0.5, 0.25, -0.5));
    let w = builder::rsweep(
        &v,
        Point3::new(0.5, 0.5, 0.0),
        Vector3::unit_z(),
        Rad(7.0),
        4,
    );
    let f = builder::try_attach_plane(&[w]).unwrap();
    let mut cylinder = builder::tsweep(&f, Vector3::unit_z() * 2.0);
    cylinder.not();

    let and = truck_shapeops::and(&cube, &cylinder, 0.05).unwrap();
    let json = serde_json::to_vec_pretty(&and).unwrap();
    std::fs::write("fillet-after-boolean.json", json).unwrap();
    eprintln!("Wrote fillet-after-boolean.json");
}
