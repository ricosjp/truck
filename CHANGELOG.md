# Change Log

The version is of the bottom crate `truck-rendimpl`.

## Unreleased

- Using `absolute_clone`.
- Add `const` for many functions.
- Deserialize by `try_new`.
- Updates wgpu to v0.14
- Step output for several models.
- Add parallel iterators for topological structures and implement parallelized meshing algorithm.
- Fix the step output of solids with several boundaries.
- Add an example with several boundaries.
- Add `Face::edge_iter` and `Face::vertex_iter`.
- Renew `Face::cut_by_edge()`.
- Fix new clippy error.
- Fix `Shell::singular_vertices()`.
- Some refactoring by `EntryMap`.
- Adds new function `builder::try_wire_homotopy`.
- Make the return value `builder::bezier` `Curve::BSplineCurve`.
- Fix binary STL output of `PolygonMesh`.
- Implement TryInto for `Curve` or `Surface` in `truck-geometry`.
- Real time standard outputs for `example-pages-generator`.
- Changed the return value of the Euler operation to the newly inserted phase element and improved the method comments.
- Divide `wasm-test` to `wasm-test` and `page-build`.
- Change the profile of `truck-js` and remove dependencies to `wee_alloc`.
- Remove recursive loop method.
- Parse STEP cylindrical surface.
- Output STEP open shell.
- Implement `Serialize` and `Deserialize` for `stepio::r#in::alias::*`.
- Add derive macros for tuple structs.
- Fix typos.
- Fix typos in truck-topology.
- Adds `SURFACE_OF_REVOLUTION` and `SPHERICAL_SURFACE` to step input.
- Panic occurs if `example-pages-generator` is failed.
- Updates container and docker file.
- Get the ray of camera from uv-coordinate.
- Make `TextureFormat` of surfaces `BrgaU8norm`.
- Fix some bugs in `step-to-obj`.
- Make the output of tessellation face-wise `Option`.
- Parse STEP topology.
- Fix example pages.
- Updates wgpu to v0.13.
- The parsing of the STEP geometry was implemented up to BSplineSurfaceWithKnots.
- Direct `CompressedShell` tessellation.
- Direct serialize API for topological data structures.
- Update resources.
- Step output of `IntersectionCurve`.
- Intersection curve with B-spline leader.
- Derive macros for geometric traits.
- Implement cubic B-spline approximation.
- Fix clippy new lint.
- Adds `cut_face_by_edge` to `Solid`.
- Refactoring Search(Nearest)Parameter.
- Converting express `trimmed_curve` to truck `TrimmedCurve`.
- Use Line in modeling and simplify output shape of tsweep.
- Set `from_other_crate` feature for `truck-stepio` to stop rls building ap203.
- `IntersectionCurve` between `Plane`s can now be converted to `Line`.

## v0.4

- The first version of `truck-stepio` has been released! One can output shapes modeled by `truck-modeling`.
- WGSL utility `math.wgsl` has been released! One can calculate invert matrices and rotation matrices.
- The processing related to linear algebra has been isolated from `truck-base` to [`matext4cgmath`](https://crates.io/crates/matext4cgmath).
- New mesh filter `Subdivision::loop_subdivision` was implemented in `truck-meshalgo`!
- In `truck-geotrait`, the trait `ParametricCurve` is decomposed into `ParametricCurve` and `BoundedCurve`.
- The method `swap_vertex` has been added to `WireFrameInstance`.
- Geometric traits has been derived to `Box`.
- Some specified geometries has been added for STEP I/O
- Comparing `BoundingBox` by inclusion relationship.
- In order to make meshing reproducible, we decided to implement random perturbations by means of a deterministic hash function.
- Some lints has been added.

## v0.3

- Specified surface for STEP I/O and modeling revolved sphere and cone.
  - In `truck-base`, the trait `Surface` is decomposed into `ParametricSurface`, `BoundedSurface`, `IncludeCurve` and `Invertible`.
  - In `truck-geometry`, specified surface, `Plane` and `Sphere`, and some decorators are prepared.
- STL handling module `stl` in `truck-polymesh`.
- In `truck-rendimpl`, wireframe for polygon.
  - Abort traits `Shape` and `Polygon`, and add new traits `IntoInstance` and `TryIntoInstance`.
- Applied wgpu v0.11 and made all shaders WGSL, including shaders for test. Now, all dependence on cmake has been removed!
  - The sample code `glsl-sandbox` becomes `wgsl-sandbox`. You can easily experience WGSL shading.
- Split `truck-base::geom_trait` into `truck-geotrait` and added some algorithms `algo`. Some methods in curves and surfaces were standardized.
- Added a new crate `truck-meshalgo`. Moved the polygon processing algorithm from polymesh to meshalgo.
- Added a new CAD meshing algorithm. Meshing trimmed surfaces. The same edge is made into the same polyline. A solid is made into a closed polygon.
- Added some meshing algorithms, including mesh collision.
- `ShapeInstance` has been removed. Tessellation should be done in advance by `truck-meshalgo` when drawing the modeled shape.
- `BSplineCurve<Point3>` was made to be `ParametricCurve3D`. Conflicts related to methods `subs` have been resolved.
- Added a new crate `truck-shapeops`, which provides solid boolean operator functions: `and` and `or`.
- Added a new crate `truck-js`, which provides wasm bindings of CAD APIs. (not released to crates.io)

## v0.2

### v0.2.1

- a small behavior change: [`NormalFilters::add_smooth_normals`](https://docs.rs/truck-polymesh/0.2.1/truck_polymesh/prelude/trait.NormalFilters.html#tymethod.add_smooth_normals).
- fix a bug: [`Splitting::into_components`](https://docs.rs/truck-polymesh/0.2.1/truck_polymesh/prelude/trait.Splitting.html#tymethod.into_components).
- an internal change: [`RenderID::gen`](https://docs.rs/truck-platform/0.2.1/truck_platform/struct.RenderID.html#method.gen).

### v0.2.0

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
    - The descriptor for wire frames is [`WireFrameInstanceDescriptor`](https://docs.rs/truck-rendimpl/0.2.0/truck_rendimpl/struct.WireFrameInstanceDescriptor.html).
  - added [`InstanceCreator`](https://docs.rs/truck-rendimpl/0.2.0/truck_rendimpl/struct.InstanceCreator.html) for generating instances.
    - `InstanceCreator` has pre-compiled shader modules as member variables.
    - [`CreateInstance`](https://docs.rs/truck-rendimpl/0.1.5/truck_rendimpl/trait.CreateInstance.html) for `Scene` is abolished.
    - `InstanceCreator` is created by [`Scene::instance_creator`](https://docs.rs/truck-rendimpl/0.2.0/truck_rendimpl/trait.CreatorCreator.html#tymethod.instance_creator).
  - Face-wise rendering of shape is abolished.
    - Now, `ShapeInstance` is one `Rendered` struct.
    - [`RenderFace`](https://docs.rs/truck-rendimpl/0.1.5/truck_rendimpl/struct.RenderFace.html) was abolished.
  - abolished implementations `Clone` for `*Instance`. Use `*Instance::clone_instance`.
  - The texture of `InstanceState` was changed `wgpu::Texture` from `image::DynamicImage`.  
  One can generate `Texture` from `DynamicImage` by [`InstanceCreator::create_texture`](https://docs.rs/truck-rendimpl/0.2.0/truck_rendimpl/struct.InstanceCreator.html#method.create_texture).
- added inherit methods of `truck_geometry::NURBSSurface` from `BSplineSurface`.
- added a feature `serde` to `cgmath` at `truck-base`.
  - remove the explicit dependency to `cgmath` from `truck-polymesh`.
  - plans to add `nalgebra` as an alternative backend (unreleased in this version).
- abolished [`truck_platform::RenderID::default`](https://docs.rs/truck-platform/0.1.0/truck_platform/struct.RenderID.html#impl-Default) and added [`RenderID::gen`](https://docs.rs/truck-platform/0.2.0/truck_platform/struct.RenderID.html#method.gen).
- added [`Error`](https://docs.rs/truck-modeling/0.2.1/truck_modeling/errors/enum.Error.html) to `truck_modeling`.
- made [`truck_topology::CompressedShell`](https://docs.rs/truck-topology/0.2.0/truck_topology/struct.CompressedShell.html) public API and added [`truck_topology::CompressedSolid`](https://docs.rs/truck-topology/0.2.0/truck_topology/struct.CompressedSolid.html).

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
