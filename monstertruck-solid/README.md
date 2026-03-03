# `monstertruck-solid`

Boolean operations, fillets, and shape healing for solids.

> Forked from [`truck-shapeops`](https://crates.io/crates/truck-shapeops) v0.4.0 by [ricosjp](https://github.com/ricosjp/truck).

## Quick Start

```rust
use monstertruck_modeling::*;
use monstertruck_solid::or;

// Build two overlapping cubes and compute their union
let v = builder::vertex(Point3::origin());
let cube_a: Solid = builder::extrude(
    &builder::extrude(&builder::extrude(&v, Vector3::unit_x()), Vector3::unit_y()),
    Vector3::unit_z(),
);
let cube_b = builder::translated(&cube_a, Vector3::new(0.5, 0.5, 0.0));

let union: Option<Solid> = or(&cube_a, &cube_b, 0.05);
```

## License

Apache License 2.0
