# `monstertruck-modeling`

Integrated geometric and topological modeling algorithms.

> Forked from [`truck-modeling`](https://crates.io/crates/truck-modeling) v0.6.0 by [ricosjp](https://github.com/ricosjp/truck).

## Quick Start

```rust
use monstertruck_modeling::*;

// Build a unit cube by three successive extrusions: point → edge → face → solid
let v = builder::vertex(Point3::new(-0.5, -0.5, -0.5));
let e = builder::extrude(&v, Vector3::unit_x());
let f = builder::extrude(&e, Vector3::unit_y());
let cube: Solid = builder::extrude(&f, Vector3::unit_z());

assert_eq!(cube.boundaries()[0].len(), 6); // six faces
```

## License

Apache License 2.0
