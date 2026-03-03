# `monstertruck-geometry`

Geometric primitives: knot vectors, B-splines, NURBS, and T-splines.

> Forked from [`truck-geometry`](https://crates.io/crates/truck-geometry) v0.5.0 by [ricosjp](https://github.com/ricosjp/truck).

## Quick Start

```rust
use monstertruck_geometry::prelude::*;

// A quadratic Bézier curve from (0,0,0) through (1,1,0) to (2,0,0)
let knot_vec = KnotVector::bezier_knot(2);
let ctrl_pts = vec![
    Point3::new(0.0, 0.0, 0.0),
    Point3::new(1.0, 1.0, 0.0),
    Point3::new(2.0, 0.0, 0.0),
];
let curve = BsplineCurve::new(knot_vec, ctrl_pts);

let mid = curve.evaluate(0.5);    // Point3 at parameter t=0.5
let tan = curve.derivative(0.5);  // tangent vector
let (t0, t1) = curve.range_tuple(); // (0.0, 1.0)
```

## License

Apache License 2.0
