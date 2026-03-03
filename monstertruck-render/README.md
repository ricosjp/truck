# `monstertruck-render`

Shape and polygon mesh visualization built on `monstertruck-gpu`.

> Forked from [`truck-rendimpl`](https://crates.io/crates/truck-rendimpl) v0.6.0 by [ricosjp](https://github.com/ricosjp/truck).

## Quick Start

```rust
use monstertruck_gpu::*;
use monstertruck_render::*;

let handler = pollster::block_on(DeviceHandler::default_device());
let mut scene = Scene::new(handler, &Default::default());

// Create render instances from a mesh
let creator = scene.instance_creator();
let instance: PolygonInstance = creator.create_instance(
    &mesh,
    &PolygonState {
        material: Material {
            albedo: Vector4::new(0.8, 0.2, 0.2, 1.0),
            roughness: 0.3,
            ..Default::default()
        },
        ..Default::default()
    },
);
scene.add_object(&instance);
```

## License

Apache License 2.0
