# monstertruck-modeling

[![Crates.io](https://img.shields.io/crates/v/monstertruck-modeling.svg)](https://crates.io/crates/monstertruck-modeling) [![Docs.rs](https://docs.rs/monstertruck-modeling/badge.svg)](https://docs.rs/monstertruck-modeling)

Integrated modeling algorithms by geometry and topology

## Sample Codes

### bottle

Modeling a bottle.

This is a benchmark for comparison with Open CASCADE Technology.
We want to reproduce the bottle made in the [OCCT tutorial].
When the `fillet` feature is enabled, the body edges are filleted just like
the OCCT tutorial (`BRepFilletAPI_MakeFillet` at radius = thickness / 12).

```bash
cargo run -p monstertruck-modeling --features fillet --example bottle
```

Generated json file can be visualized by `simple-shape-viewer`, an example of `monstertruck-render`.

[OCCT tutorial]: https://dev.opencascade.org/doc/overview/html/occt__tutorial.html

### cone

Modeling cone.

Generated json file can be visualized by `simple-shape-viewer`, an example of `monstertruck-render`.

### cube-in-cube

An example of the solid with several boundaries

### cube

Modeling a unit cube by three sweeps.

Generated json file can be visualized by `simple-shape-viewer`, an example of `monstertruck-render`.

### cylinder

Modeling a cylinder by two sweeps.

Generated json file can be visualized by `simple-shape-viewer`, an example of `monstertruck-render`.

### punched-cube

Modeling a unit cube with a hole through it.

Generated json file can be visualized by `simple-shape-viewer`, an example of `monstertruck-render`.

### sphere

Modeling a sphere

Generated json file can be visualized by `simple-shape-viewer`, an example of `monstertruck-render`.

### torus-punched-cube

A cube punched by a torus.

Generated json file can be visualized by `simple-shape-viewer`, an example of `monstertruck-render`.

### torus

Modeling a torus by two sweeps.

Generated json file can be visualized by `simple-shape-viewer`, an example of `monstertruck-render`.

### tsudumi

Modeling a one-leaf hyperboloid.

Generated json file can be visualized by `simple-shape-viewer`, an example of `monstertruck-render`.
