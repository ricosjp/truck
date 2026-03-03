# `monstertruck-gpu`

Graphics utility crate built on wgpu.

> Forked from [`truck-platform`](https://crates.io/crates/truck-platform) v0.6.0 by [ricosjp](https://github.com/ricosjp/truck).

## Quick Start

```rust
use std::f64::consts::TAU;
use monstertruck_core::cgmath64::*;
use monstertruck_gpu::*;

let handler = pollster::block_on(DeviceHandler::default_device());
let scene = Scene::new(handler, &SceneDescriptor {
    studio: StudioConfig {
        camera: Camera {
            matrix: Matrix4::look_at_rh(
                Point3::new(1.0, 1.0, 1.0),
                Point3::origin(),
                Vector3::unit_y(),
            ).invert().unwrap(),
            method: ProjectionMethod::perspective(Rad(TAU / 8.0)),
            near_clip: 0.1,
            far_clip: 100.0,
        },
        ..Default::default()
    },
    ..Default::default()
});
```

## License

Apache License 2.0
