# truck-base

[![Crates.io](https://img.shields.io/crates/v/truck-base.svg)](https://crates.io/crates/truck-base) [![Docs.rs](https://docs.rs/truck-base/badge.svg)](https://docs.rs/truck-base)

Basic structs and traits: importing cgmath, curve and surface traits, tolerance

## Coordinate systems

`truck-base` follows the same right-handed, Z-up Cartesian convention that cgmath uses. World coordinates typically measure lengths in meters and angles in radians, but only the type (`f64`) is fixed – you can opt into alternate units as long as you are consistent. Local coordinate frames (for example, a curve’s parameter space or a surface patch) are always expressed relative to their parent frame and should be converted into world coordinates through the transform utilities described below. We treat homogeneous coordinates as implementation details; API consumers mostly interact with affine points (`Point2`, `Point3`) and vectors (`Vector2`, `Vector3`).

## Tolerance philosophy

Geometry kernels need a globally consistent epsilon to compare floating-point numbers. The `tolerance` module exposes `TOLERANCE = 1e-6` and the derived `TOLERANCE2`, plus helper traits and macros (`Tolerance`, `assert_near!`, `assert_near2!`, etc.). All equality or incidence tests in higher-level crates must flow through these helpers so round-off error remains predictable. When authoring algorithms, clamp comparisons to `Tolerance::near`/`near2` instead of hand-rolled thresholds, and only tighten the tolerance when downstream invariants truly require it. Looser tolerances should be handled by normalizing inputs before entering the kernel.

## Point and vector conventions

The `cgmath64` module re-exports cgmath’s point, vector, and matrix types specialized to `f64`. Use `Point3` for positions and `Vector3` for directions/offsets – avoid mixing the two because we rely on CGMath’s type system to express affine combinations correctly. Normals are represented as unit `Vector3` values; use the extension traits in `cgmath_extend_traits` for conversions (e.g., homogeneous coordinates) and for derivative helpers from `ders`. Indexing into these structures should be treated as `[x, y, z]`, and we reserve `w` components for intermediate homogeneous math only.

## Transforms

Rigid and affine transforms are represented by `Matrix4`/`Matrix3` from `cgmath64`, plus the helper constructors (`Matrix4::from_translation`, `Matrix4::from_nonuniform_scale`, `Quaternion` → `Matrix3`, etc.). Always apply transforms via the provided cgmath traits (`Transform`, `EuclideanSpace`) so points and vectors adopt the correct translation/rotation semantics. When chaining transforms, multiply in parent-to-child order (`world_from_local * local_from_shape`). Serialization or interchange formats should keep transforms in column-major order to match cgmath’s default.
