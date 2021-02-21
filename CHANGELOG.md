# Change Log

The version is of the bottom crate `truck-rendimpl`.

## v0.2

### v0.2.0 (next release, developed items)

#### Released

- made `truck-polymesh` stable (well-tested and safety)
  - The member variables of [`PolygonMesh`](https://docs.rs/truck-polymesh/0.2.0/truck_polymesh/struct.PolygonMesh.html) becomes private.  
    - Destructive changes to the mesh are provided by [`PolygonMeshEditor`](https://docs.rs/truck-polymesh/0.2.0/truck_polymesh/polygon_mesh/struct.PolygonMeshEditor.html), which checks the regularity of the mesh at dropped time.
  - Mesh handling algorithms are now a public API.
    - The hidden structure `MeshHandler` was abolished and algorithms are managed as traits.
    - You can use them by importing [`truck_polymesh::prelude::*`](https://docs.rs/truck-polymesh/0.2.0/truck_polymesh/prelude/index.html).
- improved `truck-rendimpl` for higher performance and better usability
  - Wire frame rendering for shapes are now available.
    - One can create [`WireFrameInstance`](https://docs.rs/truck-rendimpl/0.2.0/truck_rendimpl/struct.WireFrameInstance.html) by [`InstanceCreator::create_wire_frame_instance`](https://docs.rs/truck-rendimpl/0.2.0/truck_rendimpl/struct.InstanceCreator.html#method.create_wire_frame_instance).
    - Try to run `cargo run --example wireframe`.
  - [`InstanceDescriptor`](https://docs.rs/truck-rendimpl/0.1.5/truck_rendimpl/struct.InstanceDescriptor.html) is separated into [`PolygonInstanceDescriptor`](https://docs.rs/truck-rendimpl/0.2.0/truck_rendimpl/struct.PolygonInstanceDescriptor.html) and [`ShapeInstanceDescriptor`](https://docs.rs/truck-rendimpl/0.2.0/truck_rendimpl/struct.ShapeInstanceDescriptor.html).
    - One can specify the precision of meshing faces by `ShapeInstanceDescriptor::mesh_precision`.
    - The old `InstanceDescriptor` is renamed to [`InstanceState`](https://docs.rs/truck-rendimpl/0.2.0/truck_rendimpl/struct.InstanceState.html).
    - The descriptor for wire frames is [`WIreFrameInstanceDescriptor`](https://docs.rs/truck-rendimpl/0.2.0/truck_rendimpl/struct.WireFrameInstanceDescriptor.html).
  - added [`InstanceCreator`](https://docs.rs/truck-rendimpl/0.2.0/truck_rendimpl/struct.WireFrameInstanceDescriptor.html) for generating instances.
    - `InstanceCreator` has pre-compiled shader modules as member variables.
    - [`CreateInstance`](https://docs.rs/truck-rendimpl/0.1.5/truck_rendimpl/trait.CreateInstance.html) for `Scene` is abolished.
    - `InstanceCreator` is created by [`Scene::instance_creator`](https://docs.rs/truck-rendimpl/0.2.0/truck_rendimpl/trait.CreatorCreator.html#tymethod.instance_creator).
  - Face-wise rendering of shape is abolished.
    - Now, `ShapeInstance` is one `Rendered` struct.
    - [`RenderFace`](https://docs.rs/truck-rendimpl/0.1.5/truck_rendimpl/struct.RenderFace.html) was abolished.
- added inherit methods of `truck_geometry::NURBSSurface` from `BSplineSurface`.
- added a feature `serde` to `cgmath` at `truck-base`.
  - remove the explicit dependency to `cgmath` from `truck-polymesh`.
  - plans to add `nalgebra` as an alternative backend (unreleased in this version).

#### Unreleased

- Do not support to `wgpu v0.7.0`. WGSL is very attractive, but `wgpu v0.7.0` is still unstable.

## v0.1

### v0.1.5

- changed a behavior of [`truck_topology::try_add_boundary`](https://docs.rs/truck-topology/0.1.1/truck_topology/struct.Face.html#method.try_add_boundary) and [`truck_topology::add_boundary`](https://docs.rs/truck-topology/0.1.1/truck_topology/struct.Face.html#method.add_boundary).
  - flip the boundary over when adding a boundary to a face with a flipped orientation
  - renew the id of the face which was added boundary

### v0.1.4

- add a method: `truck_rendimpl::*Instance::clone_instance`
- `Clone::clone for *Instance` is deprecated, and will be abolished in v0.2.

### v0.1.3

- fixed two bugs
  - [`truck_modeling::builder::homotopy`](https://docs.rs/truck-modeling/0.1.3/truck_modeling/builder/fn.homotopy.html), the vertices were in the wrong order.
  - [`truck_modeling::Mapped for Shell`](https://docs.rs/truck-modeling/0.1.3/truck_modeling/topo_traits/trait.Mapped.html#impl-Mapped%3CP%2C%20C%2C%20S%3E-for-Shell%3CP%2C%20C%2C%20S%3E), the orientation of surface was wrong.

### v0.1.2

- fixed a bug: [`truck_modeling::builder::try_attach_plane`](https://docs.rs/truck-modeling/0.1.2/truck_modeling/builder/fn.try_attach_plane.html), the orientation of plane was incorrect.

### v0.1.1

- fixed a bug: [`truck_modeling::builder::rsweep`](https://docs.rs/truck-modeling/0.1.1/truck_modeling/builder/fn.rsweep.html), the boundary was incorrect.

### v0.1.0

- first version
