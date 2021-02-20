# Change Log

The version is of the bottom crate `truck-rendimpl`.

## v0.1

### v0.1.0

- first version

### v0.1.1

- fixed a bug: `truck_modeling::builder::rsweep`, the boundary was incorrect.

### v0.1.2

- fixed a bug: `truck_modeling::builder::attach_plane`, the orientation of plane was incorrect.

### v0.1.3

- fixed two bugs
  - `truck_modeling::builder::homotopy`, the vertices were in the wrong order.
  - `truck_modeling::Mapped for Shell`, the orientation of surface was wrong.

### v0.1.4

- add a method: `truck_rendimpl::*Instance::clone_instance`
- `Clone::clone for *Instance` is deprecated, and will be abolished in v0.2.

### v0.1.5

- changed a behavior of `truck_topology::add_boundary`
  - flip the boundary over when adding a boundary to a face with a flipped orientation
  - renew the id of the face which was added boundary

## v0.2

### v0.2.0 (next release, developed items)
- removed WIP of `truck-polymesh`
  - The member variables of `PolygonMesh` becomes private.  
    - Destructive changes to the mesh are made `PolygonMeshEditor`, which checks the regularity of the mesh at dropped time.
  - Mesh handling algorithms are now a public API.
    - `MeshHandler` was abolished and algorithms are managed as traits.
- added inherit methods of `truck_geometry::NURBSSurface` from `BSplineSurface`.

