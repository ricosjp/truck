//! Demonstrates `solid_from_planar_profile` with a simple rectangular outline.
//!
//! This produces the same result as `builder::extrude` on a square face, but
//! uses the profile normalization pipeline which automatically handles
//! winding direction.

use monstertruck_modeling::*;

fn main() {
    let v0 = builder::vertex(Point3::new(-1.0, -1.0, 0.0));
    let v1 = builder::vertex(Point3::new(1.0, -1.0, 0.0));
    let v2 = builder::vertex(Point3::new(1.0, 1.0, 0.0));
    let v3 = builder::vertex(Point3::new(-1.0, 1.0, 0.0));
    let wire: Wire = vec![
        builder::line(&v0, &v1),
        builder::line(&v1, &v2),
        builder::line(&v2, &v3),
        builder::line(&v3, &v0),
    ]
    .into();

    let solid: Solid =
        profile::solid_from_planar_profile(vec![wire], Vector3::new(0.0, 0.0, 2.0)).unwrap();

    assert!(solid.is_geometric_consistent());
    assert_eq!(solid.boundaries()[0].len(), 6);

    let json = serde_json::to_vec_pretty(&solid).unwrap();
    std::fs::write("profile-box.json", json).unwrap();
}
