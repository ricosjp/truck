# truck - Truck is a RUst Cad Kernel

## Overview

`truck` is an open-source shape processing kernel written in Rust.
The broad concept consists of the following three elements.

- Trendy Tools
- Traditional Arts
- Theseus' ship

### Trendy Tools

- We target next-generation CAD using modern tools: Rust and WebGPU.
- Advanced optimizations with Rust and WebGPU maximize the performance of each crate.

### Traditional Arts

- We break away from legacy systems by re-implementing B-rep with NURBS and T-splines using the tools above.
- Safe implementation using Rust eliminates crashes in CPU-bound processes.
- Cargo's extensive maintenance features ensure thorough continuous integration.

### Theseus' ship

- We are modularizing into smaller crates that can be replaced, like [the Ship of Theseus](https://en.wikipedia.org/wiki/Ship_of_Theseus).
- Based on lessons learned, we have forgone monolithic optimization in favor of designing a collection of individually optimized crates.
- Since unexpected growth is inevitable, we manage it through small, composable modules.

## License

Apache License 2.0

## Usage

### How to Run Examples

All examples are located under the examples directory in each crate.
These examples use the default syntax for running examples, as found in the [Cargo](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#examples) documentation.

At first, let's run the following example!

```bash
git submodule update --init
cargo run --example rotate-objects
```

## crates

### truck-base [![Crates.io](https://img.shields.io/crates/v/truck-base.svg)](https://crates.io/crates/truck-base) [![Docs.rs](https://docs.rs/truck-base/badge.svg)](https://docs.rs/truck-base)

basic structs and traits: importing cgmath, curve and surface traits, tolerance, etc...

### truck-geotrait [![Crates.io](https://img.shields.io/crates/v/truck-geotrait.svg)](https://crates.io/crates/truck-geotrait) [![Docs.rs](https://docs.rs/truck-geotrait/badge.svg)](https://docs.rs/truck-geotrait)

Defines geometric traits: `ParametricCurve`, `ParametricSurface`, and so on.

### truck-geometry [![Crates.io](https://img.shields.io/crates/v/truck-geometry.svg)](https://crates.io/crates/truck-geometry) [![Docs.rs](https://docs.rs/truck-geometry/badge.svg)](https://docs.rs/truck-geometry)

Geometrical structs: knot vector, B-spline, NURBS, and T-spline

### truck-topology [![Crates.io](https://img.shields.io/crates/v/truck-topology.svg)](https://crates.io/crates/truck-topology) [![Docs.rs](https://docs.rs/truck-topology/badge.svg)](https://docs.rs/truck-topology)

topological structs: vertex, edge, wire, face, shell, and solid

### truck-polymesh [![Crates.io](https://img.shields.io/crates/v/truck-polymesh.svg)](https://crates.io/crates/truck-polymesh) [![Docs.rs](https://docs.rs/truck-polymesh/badge.svg)](https://docs.rs/truck-polymesh)

defines polygon data structure and some algorithms handling mesh, including meshing the shapes

### truck-meshalgo [![Crates.io](https://img.shields.io/crates/v/truck-meshalgo.svg)](https://crates.io/crates/truck-meshalgo) [![Docs.rs](https://docs.rs/truck-meshalgo/badge.svg)](https://docs.rs/truck-meshalgo)

Mesh algorithms, including tessellation of shapes.

### truck-modeling [![Crates.io](https://img.shields.io/crates/v/truck-modeling.svg)](https://crates.io/crates/truck-modeling) [![Docs.rs](https://docs.rs/truck-modeling/badge.svg)](https://docs.rs/truck-modeling)

integrated modeling algorithms by geometry and topology

### truck-shapeops [![Crates.io](https://img.shields.io/crates/v/truck-shapeops.svg)](https://crates.io/crates/truck-shapeops) [![Docs.rs](https://docs.rs/truck-shapeops/badge.svg)](https://docs.rs/truck-shapeops)

Provides boolean operations on solids

### truck-platform [![Crates.io](https://img.shields.io/crates/v/truck-platform.svg)](https://crates.io/crates/truck-platform) [![Docs.rs](https://docs.rs/truck-platform/badge.svg)](https://docs.rs/truck-platform)

graphic utility library based on wgpu

### truck-rendimpl [![Crates.io](https://img.shields.io/crates/v/truck-rendimpl.svg)](https://crates.io/crates/truck-rendimpl) [![Docs.rs](https://docs.rs/truck-rendimpl/badge.svg)](https://docs.rs/truck-rendimpl)

visualization of shape and polygon mesh based on platform

### truck-js

JavaScript wrapper for `truck`.

![dependencies](./dependencies.svg)

## Tutorials

There are some learning resources for using `truck` v0.6.x series.

- [truck-tutorial](https://ricos.gitlab.io/truck-tutorial/v0.6/)
- [truck-tutorial-ja](https://ricos.gitlab.io/truck-tutorial-ja/v0.6/) (Japanese version)
- [truck-tutorial-code](https://github.com/ricosjp/truck-tutorial-code/tree/v0.6) (pre-created sample code)
