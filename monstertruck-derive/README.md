# `monstertruck-derive`

Derive macros for geometric traits. Re-exported by `monstertruck-traits` (feature `"derive"`).

> Forked from [`truck-derivers`](https://crates.io/crates/truck-derivers) v0.1.0 by [ricosjp](https://github.com/ricosjp/truck).

## Quick Start

```rust
use monstertruck_traits::prelude::*;

/// An enum of curve types -- derive macros delegate trait methods
/// to the inner type via match arms.
#[derive(Clone, ParametricCurve, BoundedCurve)]
pub enum MyCurve {
    Line(Line<Point3>),
    Nurbs(NurbsCurve<Vector4>),
}

let curve: MyCurve = MyCurve::Line(/* ... */);
let pt = curve.evaluate(0.5); // dispatches to Line::evaluate
```

Users do not need to depend on this crate directly:

```toml
monstertruck-traits = { version = "0.1", features = ["derive"] }
```

## License

Apache License 2.0
