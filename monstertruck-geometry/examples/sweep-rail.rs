//! Demonstrates `BsplineSurface::sweep_rail` — sweeping a profile along a
//! curved rail with tangent-aligned framing.

use monstertruck_geometry::prelude::*;

fn main() {
    // A curved rail (quadratic arc rising in z).
    let rail = BsplineCurve::new(
        KnotVector::bezier_knot(2),
        vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(3.0, 4.0, 0.0),
            Point3::new(6.0, 0.0, 0.0),
        ],
    );

    // A small square profile centered at the rail origin.
    let profile = BsplineCurve::new(
        KnotVector::bezier_knot(1),
        vec![Point3::new(-0.5, 0.0, -0.5), Point3::new(0.5, 0.0, -0.5)],
    );

    let surface = BsplineSurface::sweep_rail(profile, &rail, 15);

    println!("Sweep rail surface:");
    println!("  u-degree = {}", surface.udegree());
    println!("  v-degree = {}", surface.vdegree());

    // Trace the leading edge (u=0) along the rail.
    println!("\nLeading edge (u=0) along rail:");
    for i in 0..=5 {
        let v = i as f64 / 5.0;
        let pt = surface.subs(0.0, v);
        println!("  v={:.1}: ({:.2}, {:.2}, {:.2})", v, pt.x, pt.y, pt.z);
    }
}
