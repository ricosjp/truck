# truck-polymesh

[![Crates.io](https://img.shields.io/crates/v/truck-polymesh.svg)](https://crates.io/crates/truck-polymesh) [![Docs.rs](https://docs.rs/truck-polymesh/badge.svg)](https://docs.rs/truck-polymesh)

Defines polyline-polygon data structure and some algorithms handling mesh.

## Half-edge mesh core

`truck-polymesh` implements a half-edge mesh: every edge is split into two directed half-edges that each know their origin vertex, adjacent face, opposite half-edge, and next/prev neighbors around the face loop. This representation makes it easy to traverse one-ring neighborhoods, compute adjacency, and modify topology (split, collapse, flip) without duplicating data. Vertices store positions, faces store references to one of their bounding half-edges, and the mesh as a whole maintains connectivity invariants.

## Tessellation pipeline

Tessellation converts analytic geometry (NURBS, parametric surfaces) into triangle meshes. The crate consumes curve/surface evaluators from `truck-geometry`, samples them according to curvature/flatness heuristics, and emits vertices plus faces while preserving shared edges so downstream operations (normals, shading, Booleans) remain watertight. Refinement runs adaptively: areas with high curvature get more samples, planar patches get fewer, and trimming data (UV loops) is respected so the resulting mesh matches the original topology.

## Sample Codes

### obj_stl

Converts OBJ and STL to each other.

usage:

```bash
cargo run --example obj_stl <input-file>
```
