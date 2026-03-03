//! Spheres on cube corners -- boolean operations and fillet.
//!
//! Places 1/3-unit spheres on all 8 corners of a unit cube. Four spheres on
//! one tetrahedral diagonal are subtracted, the other four are unioned.
//! Finally, all edges are round-filleted.
//!
//! Outputs both JSON and STEP files.

use anyhow::Result;
use monstertruck_modeling::*;
use monstertruck_solid::{difference, or};
use monstertruck_step::out::{CompleteStepDisplay, StepModel};
use std::f64::consts::PI;

/// Create a sphere solid centered at `center` with the given `radius`.
fn sphere(center: Point3, radius: f64) -> Solid {
    let top = builder::vertex(Point3::new(0.0, radius, 0.0));
    let wire: Wire = builder::revolve(&top, Point3::origin(), Vector3::unit_x(), Rad(PI), 3);
    let shell = builder::cone(&wire, Vector3::unit_y(), Rad(7.0), 4);
    let s = Solid::new(vec![shell]);
    builder::translated(&s, center.to_vec())
}

fn main() -> Result<()> {
    let tol = 0.01;
    let r = 1.0 / 3.0;

    // Unit cube at origin.
    let v = builder::vertex(Point3::origin());
    let e = builder::extrude(&v, Vector3::unit_x());
    let f = builder::extrude(&e, Vector3::unit_y());
    let cube = builder::extrude(&f, Vector3::unit_z());

    // Tetrahedral group A -- subtract these four corners.
    let subtract = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
    ];

    // Tetrahedral group B -- union these four corners.
    let unite = [
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
    ];

    // Apply subtractions.
    let mut body = cube;
    for &center in &subtract {
        body = difference(&body, &sphere(center, r), tol)
            .ok_or_else(|| anyhow::anyhow!("difference failed at {center:?}"))?;
    }

    // Apply unions.
    for &center in &unite {
        body = or(&body, &sphere(center, r), tol)
            .ok_or_else(|| anyhow::anyhow!("union failed at {center:?}"))?;
    }

    // Fillet all edges.
    let mut shell = body.into_boundaries().pop().unwrap();
    let edges: Vec<_> = shell
        .iter()
        .flat_map(|face| face.edge_iter())
        .collect();
    let opts = FilletOptions::constant(0.05);
    fillet_edges(&mut shell, &edges, Some(&opts))?;

    let result = Solid::new(vec![shell]);

    // Write JSON.
    let json = serde_json::to_vec_pretty(&result)?;
    std::fs::write("filleted-spheres-cube.json", &json)?;

    // Write STEP.
    let compressed = result.compress();
    let step = CompleteStepDisplay::new(StepModel::from(&compressed), Default::default());
    std::fs::write("filleted-spheres-cube.step", step.to_string())?;

    println!("Wrote filleted-spheres-cube.json and filleted-spheres-cube.step");
    Ok(())
}
