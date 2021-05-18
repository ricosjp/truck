# truck-polymesh

[![Crates.io](https://img.shields.io/crates/v/truck-polymesh.svg)](https://crates.io/crates/truck-polymesh) [![Docs.rs](https://docs.rs/truck-polymesh/badge.svg)](https://docs.rs/truck-polymesh)

Defines polyline-polygon data structure and some algorithms handling mesh.

## Sample Codes

### filleted-cube

An experiment to decompose a mesh into elements for future NURBS-shape approximation of the mesh.
Contains `doc(hidden)` methods.

- Input: filleted_cube.obj
- Output: planes.obj, lower.obj, upper.obj

### irregular-sphere

Add the normal to the sphere containing
the irregular normal generated from the NURBS containing the critical point.

- Input: irregular_sphere.obj
- Output: regular_sphere.obj

### obj_stl

Converts OBJ and STL to each other.

usage:

```bash
cargo run --example obj_stl <input-file>
```

### requadrangulate-buddha

A benchmark that reads in heavy mesh data, applies triangulation and quadrangulation, and writes it out.

- Input: happy-buddha.obj
- Output: requadrangulated-buddha.obj

### smoothing-bunny

Adds smooth normals to the stanford bunny.

- Input: bunny.obj
- Output: smooth_bunny.obj

### splitting-sample

An experiment to decompose a mesh into elements for future NURBS-shape approximation of the mesh.

- Input: sample.obj
- Output: planes_parts_#.obj, others_parts_#.obj

### teapot

Adds smooth normals to and quadrangulate the famous teapot.

- Input: teapot.obj
- Output: quaded_pot.obj
