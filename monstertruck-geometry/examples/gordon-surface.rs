//! Demonstrates `BsplineSurface::gordon` — creating a surface from two
//! families of curves using the boolean sum formula.

use monstertruck_geometry::prelude::*;

fn main() {
    // A 3x3 grid of intersection points on a curved surface.
    let grid: Vec<Vec<Point3>> = (0..3)
        .map(|i| {
            let y = i as f64;
            (0..3)
                .map(|j| {
                    let x = j as f64;
                    let z = (x * 0.5).sin() * (y * 0.5).cos();
                    Point3::new(x, y, z)
                })
                .collect()
        })
        .collect();

    // U-curves: connect grid points along the x-direction (j varies).
    let u_curves: Vec<BsplineCurve<Point3>> = grid
        .iter()
        .map(|row| BsplineCurve::new(KnotVector::bezier_knot(2), row.clone()))
        .collect();

    // V-curves: connect grid points along the y-direction (i varies).
    let v_curves: Vec<BsplineCurve<Point3>> = (0..3)
        .map(|j| {
            let pts: Vec<Point3> = (0..3).map(|i| grid[i][j]).collect();
            BsplineCurve::new(KnotVector::bezier_knot(2), pts)
        })
        .collect();

    let surface = BsplineSurface::gordon(u_curves, v_curves, &grid);

    println!("Gordon surface from 3x3 curve network:");
    println!("  u-degree = {}", surface.udegree());
    println!("  v-degree = {}", surface.vdegree());

    // Verify the surface interpolates the grid points.
    println!("\nGrid point verification (should match input):");
    grid.iter().enumerate().for_each(|(i, row)| {
        row.iter().enumerate().for_each(|(j, &expected)| {
            let u = j as f64 / 2.0;
            let v = i as f64 / 2.0;
            let pt = surface.subs(u, v);
            let err = ((pt.x - expected.x).powi(2)
                + (pt.y - expected.y).powi(2)
                + (pt.z - expected.z).powi(2))
            .sqrt();
            println!(
                "  ({u:.1}, {v:.1}): ({:.3}, {:.3}, {:.3})  expected ({:.3}, {:.3}, {:.3})  err={err:.2e}",
                pt.x, pt.y, pt.z, expected.x, expected.y, expected.z,
            );
        });
    });
}
