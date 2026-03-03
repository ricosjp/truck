# `monstertruck-core`

Core types and traits for linear algebra, curves, surfaces, and tolerances.

> Forked from [`truck-base`](https://crates.io/crates/truck-base) v0.5.0 by [ricosjp](https://github.com/ricosjp/truck).

## Quick Start

```rust
use monstertruck_core::{cgmath64::*, tolerance::Tolerance, bounding_box::BoundingBox};

let a = Point3::new(1.0, 2.0, 3.0);
let b = Point3::new(1.0 + 1e-7, 2.0, 3.0);
assert!(a.near(&b)); // within TOLERANCE (1e-6)

let bb: BoundingBox<Point3> = vec![
    Point3::new(0.0, 0.0, 0.0),
    Point3::new(1.0, 2.0, 3.0),
].into_iter().collect();
assert_eq!(bb.diagonal(), Vector3::new(1.0, 2.0, 3.0));
```

## License

Apache License 2.0
