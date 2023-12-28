# truck-meshalgo

[![Crates.io](https://img.shields.io/crates/v/truck-meshalgo.svg)](https://crates.io/crates/truck-meshalgo) [![Docs.rs](https://docs.rs/truck-meshalgo/badge.svg)](https://docs.rs/truck-meshalgo)

Mesh algorighms, include tessellations of the shape.

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

### octahedron-subdivision

Apply loop subdivision to regular octahedron.

- Input: hardcoded octahedron
- Output: octahedron.obj, subdivision-octahedron.obj

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

### tessellate-shape

Tessellate a shape and output an obj file.

```bash
usage: tessellate-shape <input json file> <output json file>
```

The default `<output file>` is output.obj.
