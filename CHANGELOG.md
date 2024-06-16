# Change Log

The version is of the bottom crate `truck-rendimpl`.

## Unreleased

- Fix new clippy(2024-06-16)
- Make `put_toghether_each_attrs` faster.
- Improve `put_together_each_attrs`.
- Fix a bug on partial `rsweep` with a negative angle.
- Fix typo in `truck-meshalgo`.
- Add `Face::cut_by_wire`.
- Implelment `AsRef`, `Borrow`, and `Extend` for `Wire` and `Shell`.
- Stop GPU test for a bug caused by some drop methods.
- Output shapes from step files to step files.
- Update `wgpu` to `0.19.x`.
- Derive macros for `StepLength` and `DisplayByStep`.
- Calculate volume and center of the gravity of PolygonMesh.
- Split closed edges and faces.
- Add `sphere.json` to tessellate test.
- Tolerance setting of `put_together_same_attrs`.
- Increase the capacity of step file in adhoc-viewer.
- Fix clippy.
- Add tessellate test with ABC Dataset.
- Tessellate sphere modeling by `builder::cone`.
- Refactoring by `itertools` and `array_macro`.
- Enabled meshing when the boundary is not closed in the parameter space.
- Robust tessellation in adhoc viewer.
- New `SearchNearestParameter` for `RevolutedCurve`.
- type def `ParameterRange`.
- Change the STEP output precision.
- Add some options to `step-to-mesh`.
- `SearchNearestParameter` for `Processor`.
- Derive macros are supported for cases with generics.
- Remove `#[inline(always)]` in derive macro for release build of `truck-stepio`.
- Parse `ELLIPSE` in STEP.
- Fix the parameter lookup of the circle.
- Output `CompressedShell` to `vtu` files.
- Re-export `ShellCondition` in `truck-meshalgo`.
- fix clippy
- Re-export `truck-polymesh::*` in `truck-rendimpl`.
- Add test for tessellate elementary step files.
- Implementation for closed surface tessellation.
- Refactor bounded geometries.
- Parse `RATIONAL_B_SPLINE_SURFACE` in STEP.
- Parse `CONICAL_SURFACE` in STEP.
- Fix clippy.
- Separate scheduled CI jobs to another repository.
- STEP input test for surfaces.
- Updates CI container `wasm-test`.
- Fix build error of `bsp-animation` in wasm.
- Fix the compile error with `webgl` feature.
- cargo install by `--locked` in `wasm-test`.
- STEP input test for primitives and curves.
- `cargo fmt`
- Fix building webgpu examples.
- Set `--no-cache` option to build CI images.
- Fix wasm-test container build.
- Fix dependency check.
- Put `truck_geometry::prelude` for resolve multiple re-export.
- Fix dependency check.
- Add dependency check.
- Remove test build from `wasm-test`.
- Updates wgpu to v0.16.
- Updates CI containers.
- Output `Vertex`, `Edge`, `Wire`, and `Face` to `vtu` files.
- Add tests for traits in `truck_modeling::topo_traits`.
- Refactor and renew test for `truck_modeling::geom_impl` by `proptest`.
- Output `Shell<Point3, PolylineCurve, PolygonMesh>` to `vtu` files.
- Output `PolygonMesh` to `vtp` files.
- Separate features for meshalgo.
- cargo upgrade
- Remove `get_` prefix and replace `Mutex` and `Arc` more faster and compact mem.
- JS wrapper of STEP API.
- Parse `SURFACE_CURVE` (not translate to `IntersectionCurve`).
- Refactoring for `ruststep` updating.
- Parse of `PCURVE` in STEP.
- Dummy struct for STEP parse failure.
- Parse of `SURFACE_OF_LINEAR_EXTRUSION` in STEP.
- Parse of `TOROIDAL_SURFACE` in STEP.
- Parse of `B_SPLINE_SURFACE`s in STEP.
- Implement robust meshing by `SearchNearestParameter`.
- Remove invertible from tessellating traits.
- Add "periodic" identifer to `ParametricCurve` and `ParametricSurface`.
- Make struct naming canonical, resolve bump deps, remove unused deps

Step input API is W.I.P. and hidden.

- Remove recursive loop method.
- Parse STEP cylindrical surface.
- Implement `Serialize` and `Deserialize` for `stepio::r#in::alias::*`.
- Adds `SURFACE_OF_REVOLUTION` and `SPHERICAL_SURFACE` to step input.
- Fix some bugs in `step-to-obj`.
- Parse STEP topology.
- The parsing of the STEP geometry was implemented up to `BSplineSurfaceWithKnots`.

### Latest `cargo upgrade`

2024-06-10

## v0.5

### Additional APIs

- derive macros for geometric traits [`truck-geoderive`](truck-geoderive)
- step output of open shell, worlds including several models, and `IntersectionCurve`
- parallel iterators for topological structures
- direct tessellation of `CompressedShell` and `CompressedSolid`
- direct serialization for topological data structures.
- cubic B-spline approximation
- `builder::try_wire_homotopy`
- `Solid::cut_face_by_edge`
- `Face::edge_iter` and `Face::vertex_iter`
- `IntersectionCurve` between `Plane`s can now be converted to `Line`.
- `Camera::ray`
- `EntryMap`

### Updated APIs

- `MeshableShape::triangulation`
- the Euler operations
- `Face::cut_by_edge`
- Refactoring `Search(Nearest)Parameter`.

### Bug fix

- The orientation of the normal of `builder::try_attach_plane`.
- `Shell::singular_vertices`
- binary STL output of `PolygonMesh`

### Internal Improvements

- Data integrity check during deserialization of `KnotVec`, `BSplineCurve`, and all structs constructed by `try_new`.
- Improve meshing algorithm by parallelization.
- Intersection curve with B-spline leader.
- Implement some geometric traits for `TrimmedCurve`, `UnitHyperbola` and `UnitParabola`.
- Use Line in modeling and simplify output shape of tsweep.

### Misc

- Make `TextureFormat` of surfaces `BrgaU8norm`.
- Add an example with several boundaries.
- Updates `wgpu` to `v0.14`
- Updates `spade` to `v2`.
- Change the profile of `truck-js` and remove dependencies to `wee_alloc`.

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
