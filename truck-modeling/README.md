# truck-modeling

[![Crates.io](https://img.shields.io/crates/v/truck-modeling.svg)](https://crates.io/crates/truck-modeling) [![Docs.rs](https://docs.rs/truck-modeling/badge.svg)](https://docs.rs/truck-modeling)

Integrated modeling algorithms by geometry and topology

## Sample Codes

### bottle

Modeling a bottle.

This is a benchmark for comparison with Open CASCADE Technology.
We want to reproduce the bottle made in the [OCCT tutorial].
Filleting is supported via the `fillet` feature flag. Enable with `features = ["fillet"]`.

Generated json file can be visualized by `simple-shape-viewer`, an example of `truck-rendimpl`.

[OCCT tutorial]: https://dev.opencascade.org/doc/overview/html/occt__tutorial.html

### cone

Modeling cone.

Generated json file can be visualized by `simple-shape-viewer`, an example of `truck-rendimpl`.

### cube-in-cube

An example of the solid with several boundaries

### cube

Modeling a unit cube by three sweeps.

Generated json file can be visualized by `simple-shape-viewer`, an example of `truck-rendimpl`.

### cylinder

Modeling a cylinder by two sweeps.

Generated json file can be visualized by `simple-shape-viewer`, an example of `truck-rendimpl`.

### punched-cube

Modeling a unit cube with a hole through it.

Generated json file can be visualized by `simple-shape-viewer`, an example of `truck-rendimpl`.

### sphere

Modeling a sphere

Generated json file can be visualized by `simple-shape-viewer`, an example of `truck-rendimpl`.

### torus-punched-cube

A cube punched by a torus.

Generated json file can be visualized by `simple-shape-viewer`, an example of `truck-rendimpl`.

### torus

Modeling a torus by two sweeps.

Generated json file can be visualized by `simple-shape-viewer`, an example of `truck-rendimpl`.

### tsudumi

Modeling a one-leaf hyperboloid.

Generated json file can be visualized by `simple-shape-viewer`, an example of `truck-rendimpl`.
