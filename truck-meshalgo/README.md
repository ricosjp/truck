# truck-meshalgo

[![Crates.io](https://img.shields.io/crates/v/truck-meshalgo.svg)](https://crates.io/crates/truck-meshalgo) [![Docs.rs](https://docs.rs/truck-meshalgo/badge.svg)](https://docs.rs/truck-meshalgo)

Mesh algorithms, including tessellation of shapes.

## Sample Codes

### filleted-cube

An experiment to decompose a mesh into elements for future NURBS surface approximation.
Contains `doc(hidden)` methods.

- Input: filleted_cube.obj
- Output: planes.obj, lower.obj, upper.obj

### irregular-sphere

Add normals to a sphere that contains irregular normals generated from a NURBS surface with critical points.

- Input: irregular_sphere.obj
- Output: regular_sphere.obj

### octahedron-subdivision

Apply loop subdivision to a regular octahedron.

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

An experiment to decompose a mesh into elements for future NURBS surface approximation.

- Input: sample.obj
- Output: planes*parts*#.obj, others*parts*#.obj

### teapot

Add smooth normals to and quadrangulate the famous teapot.

- Input: teapot.obj
- Output: quaded_pot.obj

### tessellate-shape

Tessellate a shape and output an obj file.

```bash
usage: tessellate-shape <input json file> <output json file>
```

The default `<output file>` is output.obj.
