//! Demonstrates `BsplineSurface::skin` — creating a lofted surface through
//! multiple section curves.

use monstertruck_geometry::prelude::*;

fn main() {
    // Three quadratic arcs at different y-positions.
    let sections: Vec<BsplineCurve<Point3>> = (0..3)
        .map(|i| {
            let y = i as f64 * 2.0;
            BsplineCurve::new(
                KnotVector::bezier_knot(2),
                vec![
                    Point3::new(-1.0, y, 0.0),
                    Point3::new(0.0, y, 1.0),
                    Point3::new(1.0, y, 0.0),
                ],
            )
        })
        .collect();

    let surface = BsplineSurface::skin(sections);

    // Verify corners.
    let p00 = surface.subs(0.0, 0.0);
    let p10 = surface.subs(1.0, 0.0);
    let p01 = surface.subs(0.0, 1.0);
    let p11 = surface.subs(1.0, 1.0);
    println!("Skin surface corners:");
    println!("  (0,0) = ({:.2}, {:.2}, {:.2})", p00.x, p00.y, p00.z);
    println!("  (1,0) = ({:.2}, {:.2}, {:.2})", p10.x, p10.y, p10.z);
    println!("  (0,1) = ({:.2}, {:.2}, {:.2})", p01.x, p01.y, p01.z);
    println!("  (1,1) = ({:.2}, {:.2}, {:.2})", p11.x, p11.y, p11.z);

    // Evaluate along the apex of the arcs (u=0.5).
    println!("\nApex line (u=0.5):");
    for i in 0..=4 {
        let v = i as f64 / 4.0;
        let pt = surface.subs(0.5, v);
        println!("  v={:.2}: ({:.2}, {:.2}, {:.2})", v, pt.x, pt.y, pt.z);
    }
}
