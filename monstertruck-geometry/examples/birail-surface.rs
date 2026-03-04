//! Demonstrates `BsplineSurface::birail1` and `birail2` — creating surfaces
//! by sweeping profiles along two rail curves.

use monstertruck_geometry::prelude::*;

fn main() {
    // Two diverging rails.
    let rail1 = BsplineCurve::new(
        KnotVector::bezier_knot(2),
        vec![
            Point3::new(-1.0, 0.0, 0.0),
            Point3::new(-1.5, 2.0, 0.0),
            Point3::new(-2.0, 4.0, 0.0),
        ],
    );
    let rail2 = BsplineCurve::new(
        KnotVector::bezier_knot(2),
        vec![
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(1.5, 2.0, 0.0),
            Point3::new(2.0, 4.0, 0.0),
        ],
    );

    // Birail1: a single arc profile swept along both rails.
    let profile = BsplineCurve::new(
        KnotVector::bezier_knot(2),
        vec![
            Point3::new(-1.0, 0.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(1.0, 0.0, 0.0),
        ],
    );

    let surf1 = BsplineSurface::birail1(profile, &rail1, &rail2, 10);
    println!("Birail1 surface (single profile, diverging rails):");
    println!(
        "  u-degree = {}, v-degree = {}",
        surf1.udegree(),
        surf1.vdegree()
    );
    println!("  corner (0,0) = {:?}", surf1.subs(0.0, 0.0));
    println!("  corner (1,0) = {:?}", surf1.subs(1.0, 0.0));
    println!("  corner (0,1) = {:?}", surf1.subs(0.0, 1.0));
    println!("  corner (1,1) = {:?}", surf1.subs(1.0, 1.0));

    // Birail2: two different profiles blended along the rails.
    let profile_start = BsplineCurve::new(
        KnotVector::bezier_knot(2),
        vec![
            Point3::new(-1.0, 0.0, 0.0),
            Point3::new(0.0, 0.0, 0.5),
            Point3::new(1.0, 0.0, 0.0),
        ],
    );
    let profile_end = BsplineCurve::new(
        KnotVector::bezier_knot(2),
        vec![
            Point3::new(-2.0, 4.0, 0.0),
            Point3::new(0.0, 4.0, 2.0),
            Point3::new(2.0, 4.0, 0.0),
        ],
    );

    let surf2 = BsplineSurface::birail2(profile_start, profile_end, &rail1, &rail2, 10);
    println!("\nBirail2 surface (two profiles, diverging rails):");
    println!(
        "  u-degree = {}, v-degree = {}",
        surf2.udegree(),
        surf2.vdegree()
    );
    println!("  midpoint = {:?}", surf2.subs(0.5, 0.5));
}
