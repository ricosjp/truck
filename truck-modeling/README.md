# truck-modeling

[![Crates.io](https://img.shields.io/crates/v/truck-modeling.svg)](https://crates.io/crates/truck-modeling) [![Docs.rs](https://docs.rs/truck-modeling/badge.svg)](https://docs.rs/truck-modeling)

Integrated modeling algorithms by geometry and topology

## Sweeps and face generation

Truck’s modeling operators convert wireframe inputs (from `truck-topology`) and geometric primitives (from `truck-geometry`) into shells/solids. The main sweeps are extrude, revolve, and sweep-along-path; each creates new faces by pairing input edges with motion.

### Extrude: new faces from motion

Extruding a planar wire along a vector creates:

- **Side faces** – every edge in the wire produces a ruled surface between the original curve and its translated copy.
- **Cap faces** – optional top/bottom faces if the wire is closed.

The algorithm keeps the source edge’s curve, translates it, and builds a face whose surface is a loft between the two. Edge orientation determines the outward normal, so consistent wires are important.

### Edge propagation

When you sweep a wire, each edge propagates along the sweep path. Modeling tracks this propagation, creating new edges at each step:

- Original edge → *generator* face edge.
- Translated copy → *offset* face edge.
- Connecting edges → *rails* between generator and offset.

Maintaining this mapping lets downstream operations (Boolean, chamfer) know how faces relate back to their source curves.

### Revolve: splitting faces

Revolving a profile around an axis sweeps each edge through an angular range. Closed profiles produce surfaces of revolution; open profiles may create partial surfaces that need trimming. To avoid singular faces (e.g., touching the axis), the revolve operator splits faces where the profile intersects the axis or where the sweep angle crosses `2π`. The result is multiple faces sharing the same revolution surface but trimmed to non-overlapping angular spans.

### Wires → faces

The modeling layer turns wires into faces by attaching them to a surface:

1. Choose or build a support surface (plane for planar wires, lofted surface for sweeps, etc.).
2. Use the wire as the outer trimming loop.
3. Optionally add inner wires for holes.

This mirrors the topology rules from `truck-topology`: wires define the boundary, while surfaces provide the geometry. Sweeps automate step (1) by generating surfaces from motion, then reuse the input wires as trim loops.

## Sample Codes

### bottle

Modeling a bottle.

This is a technical indicator for comparing with Open CASCADE Technology, a great senior.
We want to reproduce the bottle made in the [OCCT tutorial].
Now, one cannot make a fillet or run boolean operations by truck.
So, the bottle made by this script is not completed.

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
