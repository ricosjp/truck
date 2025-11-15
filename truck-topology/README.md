# truck-topology

[![Crates.io](https://img.shields.io/crates/v/truck-topology.svg)](https://crates.io/crates/truck-topology) [![Docs.rs](https://docs.rs/truck-topology/badge.svg)](https://docs.rs/truck-topology)

Topological structs: vertex, edge, wire, face, shell, and solid

## B-rep building blocks

Truck’s topology layer follows the classic boundary-representation (B-rep) stack:

- **Vertex** – a 3D point plus parameter references into adjacent geometry.
- **Edge** – an oriented reference to a parametric curve.
- **Wire** – a closed sequence of edges.
- **Face** – a surface plus one outer wire and zero-or-more inner wires (holes).
- **Shell** – a closed set of faces stitched together.
- **Solid** – one or more shells, often with a single outer shell and optional inner shells (voids).

This layered structure mirrors B-rep diagrams you’d see in CAD texts and makes it straightforward to traverse topology while deferring geometric evaluation to `truck-geometry` or `truck-geotrait`.

## Edge → Curve

An `Edge` doesn’t store geometry itself; it carries a reference to a parametric curve, a pair of vertices, and an orientation. The curve supplies the 3D embedding, while the edge’s orientation tells you whether the parameterization flows from `start → end` or the opposite (important for loop closures and trimming operations).

## Face → Surface + trim loops

Faces point to a `ParametricSurface` and use one wire as the *outer* trimming loop plus any number of *inner* loops to cut holes. Evaluating a face at `(u, v)` delegates to the surface, but point membership is determined by whether `(u, v)` lies inside the trimmed domain defined by those wires. This is how we represent pockets, slots, and other trimmed NURBS patches.

## Shells and solids

A shell is a closed, consistently oriented set of faces. Solids collect shells: the outer shell defines the boundary, while inner shells represent voids/cavities. Algorithms that compute volume, mass properties, or Boolean results operate at the shell/solid level, traversing down to faces/edges only when they need geometric detail.

## Orientation rules

Edges and faces carry orientation so traversal remains coherent. Edge orientation determines how its parameter domain maps onto a wire; reversing an edge flips parameter direction as well. Face orientation ensures outward-pointing normals and consistent shell closure. Keeping orientations consistent is critical when sewing faces together or exporting to other CAD kernels.

## UV-space trimming

Trimming loops live in the UV parameter space of the surface they bound. Each loop consists of edges whose underlying curves are parameter-space curves (or lifted from 3D and projected). When evaluating point inclusion or tessellating a face, we work entirely in UV-space first, then map to 3D via the surface. This separation keeps trimming robust and lets you reuse the same surface with different trim configurations.
