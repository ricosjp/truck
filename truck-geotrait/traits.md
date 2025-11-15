# truck-geotrait Trait Guide

This crate defines the core geometric interfaces used throughout the Truck kernel. Use this document as a quick-reference when implementing your own geometry.

## Shared helpers

- **`Invertible`** – for oriented geometry that can be reversed (edges, wires, parameter ranges). Provides `invert` (mutating) and `inverse` (by value).
- **`Transformed<T>`** – apply rigid/affine transforms. Implement `transform_by` to mutate in-place; `transformed` clones and applies the transform. Blanket impls cover `Point2`/`Point3` with `Matrix3`/`Matrix4`.
- **`ToSameGeometry<T>`** – returns an equivalent geometry object (useful for boxing/erasing concrete types).

## Curves

### `ParametricCurve`

Represents a mapping `t -> Point`. Key requirements:

- `subs(t)` – evaluate point.
- `der`, `der2`, `der_n`, `ders` – derivatives of any order.
- `parameter_range()` – returns `(Bound<f64>, Bound<f64>)`.
- `period()` – optional periodicity.

Specializations:

- `ParametricCurve2D`, `ParametricCurve3D` – constrain point/vector types.
- `BoundedCurve` – guarantees finite parameter range. Adds `range_tuple`, `front`, `back`.
- `ParameterDivision1D` – produce adaptive parameter samples with tolerance.
- `ParameterTransform` – affine remap of the parameter domain, including normalization to `[0, 1]`.
- `Cut` (in curve module) – split the curve at a parameter and return the trailing segment.

### Search helpers

Under `traits/search_parameter.rs` you’ll find utilities for projecting points onto curves (Newton iteration, etc.). They operate on the traits above and respect global tolerance.

## Surfaces

### `ParametricSurface`

Maps `(u, v) -> Point`. Required methods mirror the curve trait but in two dimensions:

- `subs(u, v)`
- Partial derivatives (`uder`, `vder`, `uuder`, `uvder`, `vvder`, `der_mn`, `ders`)
- `parameter_range()` returning two `ParameterRange` values for `u` and `v`
- Optional periods via `u_period`/`v_period`

Extensions:

- `ParametricSurface2D`, `ParametricSurface3D`
  - 3D surfaces gain `normal`, `normal_uder`, `normal_vder`.
- `BoundedSurface` – guarantees finite ranges and exposes `range_tuple`.
- `IncludeCurve<C>` – tells whether a curve lies on the surface (used for trims).
- `ParameterDivision2D` – adaptive subdivision in UV space.

## Parameter utilities

The shared type alias `ParameterRange = (Bound<f64>, Bound<f64>)` plus `bound2opt` helper convert between ranges and optional endpoints. All bounded traits must fail fast if `Bound::Unbounded` is returned, making debugging easier.

## Implementation tips

- Always honor the global tolerance (`truck-base::tolerance::TOLERANCE`) when subdividing or comparing parameters.
- Normalize parameter domains for interoperability with modeling/topology layers.
- Use the blanket implementations (`&T`, `Box<T>`, `Vec<T>`) where possible to avoid recreating glue code.
