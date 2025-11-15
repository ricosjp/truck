# truck-geotrait

[![Crates.io](https://img.shields.io/crates/v/truck-geotrait.svg)](https://crates.io/crates/truck-geotrait) [![Docs.rs](https://docs.rs/truck-geotrait/badge.svg)](https://docs.rs/truck-geotrait)

Defines geometric traits: `ParametricCurve`, `ParametricSurface`, and so on.

## What is a parametric curve?

`ParametricCurve` abstracts any mapping from a single parameter `t` into Euclidean space. Think Bézier, B-spline, or analytic curves. The parameter usually lives on a finite interval (e.g., `[0, 1]`), and evaluating the curve returns a point in 2D/3D space along with optional derivatives. Truck treats the parameter as a continuous scalar so algorithms can differentiate, subdivide, and project onto the curve without reasoning about control points directly.

## Normalized parameter domain

Many curves and surfaces supply a *normalized* parameter domain, meaning the implementation maps its native domain into `[0, 1]` (or `[0, 1]^2` for surfaces). This normalization simplifies tessellation and intersection routines because every object shares the same parameter bounds regardless of its actual geometric extents. When you query `ParametricCurve::parameter_range` or `ParametricSurface::parameter_domain` you should expect normalized values; use helper methods to convert between normalized and “native” parameters if you need exact knot positions.

## Surfaces describe faces

`ParametricSurface` behaves like a 2D patch living in 3D space. A topological face references a surface plus trimming data (loops) that carve out the usable region. The surface itself only cares about the geometric mapping `(u, v) -> Point3`, along with partial derivatives, normals, and so on. Truck’s higher-level crates (e.g., `truck-topology`) consume these surfaces to build boundary representations: faces hold onto a surface instance and define their domain via wires/loops within that parameter space.

## Geometry vs topology

Geometry describes shape through continuous mappings (curves, surfaces, points, transforms). Topology describes how those shapes connect: vertices, edges, loops, faces, shells. `truck-geotrait` focuses entirely on the geometric side – evaluating points, derivatives, curvature – and leaves connectivity to crates like `truck-topology`. Separating concerns lets the same surface power multiple faces, and lets topological operations (Boolean, subdivision) operate without re-implementing geometry kernels.
