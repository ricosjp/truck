# `monstertruck-wasm`

WebAssembly/JavaScript bindings for monstertruck.

> Forked from [`truck-js`](https://crates.io/crates/truck-js) v0.2.0 by [ricosjp](https://github.com/ricosjp/truck).

## Quick Start

```js
import init, { vertex, line, extrude } from "monstertruck-wasm";

await init();

// Build a cube by successive extrusions
const v = vertex(0, 0, 0);
const edge = extrude(v.upcast(), [1, 0, 0]);
const face = extrude(edge, [0, 1, 0]);
const solid = extrude(face, [0, 0, 1]);

// Tessellate and get vertex buffer
const mesh = solid.into_solid().to_polygon(0.01);
```

## License

Apache License 2.0
