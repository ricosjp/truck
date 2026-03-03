# `monstertruck`

**M**ultifarious **O**mnificence, **N**omenclature **S**tandardized, **T**erminology **E**nhanced & **R**efactored **Truck** – a **Ru**st **C**ad **K**ernel.

Meta-crate that re-exports all `monstertruck-*` sub-crates via feature flags.
Enable only what you need, or use `full` for everything.

## Quick Start

```toml
[dependencies]
monstertruck = { version = "0.1", features = ["full"] }
```

### Example – spheres on cube corners with booleans and fillet

Place 1/3-unit spheres on all 8 corners of a unit cube. Subtract four along
one tetrahedral diagonal, union the other four, then round-fillet all edges.
See [`examples/filleted-spheres-cube.rs`](examples/filleted-spheres-cube.rs)
for the full runnable version with STEP export.

```rust
use monstertruck_modeling::*;
use monstertruck_solid::{difference, fillet_edges, or, FilletOptions};
use std::f64::consts::PI;

fn sphere(center: Point3, radius: f64) -> Solid {
    let top = builder::vertex(Point3::new(0.0, radius, 0.0));
    let wire: Wire = builder::revolve(&top, Point3::origin(), Vector3::unit_x(), Rad(PI), 3);
    let shell = builder::cone(&wire, Vector3::unit_y(), Rad(7.0), 4);
    builder::translated(&Solid::new(vec![shell]), center.to_vec())
}

fn main() -> anyhow::Result<()> {
    let tol = 0.01;
    let r = 1.0 / 3.0;

    // Unit cube at origin.
    let v = builder::vertex(Point3::origin());
    let cube = builder::extrude(
        &builder::extrude(&builder::extrude(&v, Vector3::unit_x()), Vector3::unit_y()),
        Vector3::unit_z(),
    );

    // Tetrahedral group A -- subtract.
    let subtract = [
        Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 0.0),
        Point3::new(1.0, 0.0, 1.0), Point3::new(0.0, 1.0, 1.0),
    ];
    // Tetrahedral group B -- union.
    let unite = [
        Point3::new(1.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, 1.0), Point3::new(1.0, 1.0, 1.0),
    ];

    let mut body = cube;
    for &c in &subtract {
        body = difference(&body, &sphere(c, r), tol)
            .ok_or_else(|| anyhow::anyhow!("difference failed at {c:?}"))?;
    }
    for &c in &unite {
        body = or(&body, &sphere(c, r), tol)
            .ok_or_else(|| anyhow::anyhow!("union failed at {c:?}"))?;
    }

    // Fillet all edges.
    let mut shell = body.into_boundaries().pop().unwrap();
    let edge_ids: Vec<_> = shell
        .iter()
        .flat_map(|face| face.edge_iter())
        .map(|e| e.id())
        .collect();
    fillet_edges(&mut shell, &edge_ids, Some(&FilletOptions::constant(0.05)))?;

    let result = Solid::new(vec![shell]);
    std::fs::write("output.json", serde_json::to_vec_pretty(&result)?)?;
    Ok(())
}
```

## Features

| Feature    | Crate                   | Includes               |
| ---------- | ----------------------- | ---------------------- |
| _(always)_ | `monstertruck-core`     |                        |
| `traits`   | `monstertruck-traits`   |                        |
| `derive`   | `monstertruck-derive`   |                        |
| `geometry` | `monstertruck-geometry` | `traits`               |
| `topology` | `monstertruck-topology` |                        |
| `mesh`     | `monstertruck-mesh`     | `traits`               |
| `modeling` | `monstertruck-modeling` | `geometry`, `topology` |
| `meshing`  | `monstertruck-meshing`  | `mesh`, `modeling`     |
| `solid`    | `monstertruck-solid`    | `modeling`             |
| `assembly` | `monstertruck-assembly` |                        |
| `step`     | `monstertruck-step`     | `modeling`             |
| `gpu`      | `monstertruck-gpu`      |                        |
| `render`   | `monstertruck-render`   | `gpu`, `mesh`          |

### Bundles

| Bundle    | Features                                                               |
| --------- | ---------------------------------------------------------------------- |
| `default` | `modeling`, `meshing`                                                  |
| `full`    | `modeling`, `meshing`, `solid`, `assembly`, `step`, `render`, `derive` |

## Re-exported Modules

Each enabled feature exposes a top-level module:

```rust
monstertruck::core        // always available
monstertruck::modeling    // with "modeling"
monstertruck::meshing     // with "meshing"
monstertruck::geometry    // with "geometry"
// ... etc.
```

## License

Apache License 2.0
