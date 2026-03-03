# truck - Truck is a RUst Cad Kernel

## Overview

`truck` is an open-source shape processing kernel by Rust.  
The broad concept consists of the following three elements.

- Trendy Tools
- Traditional Arts
- Theseus' ship

### Trendy Tools

- We are targeting the next generation of market share using developmental tools: Rust and WebGPU.
- Advanced optimizations using Rust and WebGPU maximize the performance of each crate.

### Traditional Arts

- We will break away from the legacy by re-implementing the B-rep with NURBS in the above trendy tools.
- Safe implementation using Rust to eliminate core dumped for CPU-derived processes.
- Cargo's extensive maintenance features ensure thorough continuous integration.

### Theseus' ship

- We are modularizing into smaller crates that can be replaced, like [the Ship of Theseus](https://en.wikipedia.org/wiki/Ship_of_Theseus).
- Based on the many lessons learned in the past, we have given up on overall optimization as a single application, and design as a collection of individual optimized crates.
- Since unexpected expansions are bound to occur, we deal with uncontrolled expansions in the form of small modules.

## License

Apache License 2.0

## Usage

### How to Run Examples

All examples are located under the examples directory in each crates.  
These examples use the default syntax for running examples, as found in the [Cargo](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#examples) documentation.

At first, let's run the following example!

```bash
git submodule update --init
cargo run --example rotate-objects
```

## crates

### monstertruck-core [![Crates.io](https://img.shields.io/crates/v/monstertruck-core.svg)](https://crates.io/crates/monstertruck-core) [![Docs.rs](https://docs.rs/monstertruck-core/badge.svg)](https://docs.rs/monstertruck-core)

basic structs and traits: importing cgmath, curve and surface traits, tolerance, etc...

### monstertruck-traits [![Crates.io](https://img.shields.io/crates/v/monstertruck-traits.svg)](https://crates.io/crates/monstertruck-traits) [![Docs.rs](https://docs.rs/monstertruck-traits/badge.svg)](https://docs.rs/monstertruck-traits)

Defines geometric traits: `ParametricCurve`, `ParametricSurface`, and so on.

### monstertruck-geometry [![Crates.io](https://img.shields.io/crates/v/monstertruck-geometry.svg)](https://crates.io/crates/monstertruck-geometry) [![Docs.rs](https://docs.rs/monstertruck-geometry/badge.svg)](https://docs.rs/monstertruck-geometry)

geometrical structs: knot vector, B-spline and NURBS

### monstertruck-topology [![Crates.io](https://img.shields.io/crates/v/monstertruck-topology.svg)](https://crates.io/crates/monstertruck-topology) [![Docs.rs](https://docs.rs/monstertruck-topology/badge.svg)](https://docs.rs/monstertruck-topology)

topological structs: vertex, edge, wire, face, shell, and solid

### monstertruck-mesh [![Crates.io](https://img.shields.io/crates/v/monstertruck-mesh.svg)](https://crates.io/crates/monstertruck-mesh) [![Docs.rs](https://docs.rs/monstertruck-mesh/badge.svg)](https://docs.rs/monstertruck-mesh)

defines polygon data structure and some algorithms handling mesh, including meshing the shapes

### monstertruck-meshing [![Crates.io](https://img.shields.io/crates/v/monstertruck-meshing.svg)](https://crates.io/crates/monstertruck-meshing) [![Docs.rs](https://docs.rs/monstertruck-meshing/badge.svg)](https://docs.rs/monstertruck-meshing)

Mesh algorighms, include tessellations of the shape.

### monstertruck-modeling [![Crates.io](https://img.shields.io/crates/v/monstertruck-modeling.svg)](https://crates.io/crates/monstertruck-modeling) [![Docs.rs](https://docs.rs/monstertruck-modeling/badge.svg)](https://docs.rs/monstertruck-modeling)

integrated modeling algorithms by geometry and topology

### monstertruck-solid [![Crates.io](https://img.shields.io/crates/v/monstertruck-solid.svg)](https://crates.io/crates/monstertruck-solid) [![Docs.rs](https://docs.rs/monstertruck-solid/badge.svg)](https://docs.rs/monstertruck-solid)

Provides boolean operations to Solid

### monstertruck-platform [![Crates.io](https://img.shields.io/crates/v/monstertruck-platform.svg)](https://crates.io/crates/monstertruck-platform) [![Docs.rs](https://docs.rs/monstertruck-platform/badge.svg)](https://docs.rs/monstertruck-platform)

graphic utility library based on wgpu

### monstertruck-render [![Crates.io](https://img.shields.io/crates/v/monstertruck-render.svg)](https://crates.io/crates/monstertruck-render) [![Docs.rs](https://docs.rs/monstertruck-render/badge.svg)](https://docs.rs/monstertruck-render)

visualization of shape and polygon mesh based on platform

### monstertruck-wasm

Javascript wrapper of `truck`.

![dependencies](./dependencies.svg)

## Tutorials

There are some learning resources for using `truck` v0.6.x series.

- [truck-tutorial](https://ricos.gitlab.io/truck-tutorial/v0.6/)
- [truck-tutorial-ja](https://ricos.gitlab.io/truck-tutorial-ja/v0.6/) (Japanese version)
- [truck-tutorial-code](https://github.com/ricosjp/truck-tutorial-code/tree/v0.6) (pre-created sample code)
