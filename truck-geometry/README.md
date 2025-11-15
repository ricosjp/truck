# truck-geometry

[![Crates.io](https://img.shields.io/crates/v/truck-geometry.svg)](https://crates.io/crates/truck-geometry) [![Docs.rs](https://docs.rs/truck-geometry/badge.svg)](https://docs.rs/truck-geometry)

Geometrical structs: knot vector, B-spline and NURBS

## What is a NURBS?

NURBS (Non-Uniform Rational B-Splines) are the workhorse representation for free-form geometry. They extend polynomial B-splines with weights, letting you represent exact conics (circles, ellipses) and smooth free-form shapes using a compact set of control points. In this crate `NurbsCurve`/`NurbsSurface` wrap the standard basis evaluation plus weight handling, so you can loft, sweep, or interpolate geometry that matches CAD-grade accuracy.

## How knot vectors work

A knot vector is a non-decreasing sequence of parameter values that partitions the parametric axis. The knots determine basis support, continuity, and multiplicity: repeated knots lower continuity, while knot spacing influences how control points pull the curve. Truck stores knots in normalized form and exposes helpers for clamping, inserting, and evaluating basis functions. Understanding knot layout is essential when you want to refine (insert knots) or reparameterize a spline without changing its shape.

## Evaluating a curve

Evaluating B-splines or NURBS curves amounts to computing basis functions at a parameter `t` and blending the corresponding control points and weights. The crate provides De Boor-style algorithms plus derivative evaluation (`evaluate`, `der`, etc.), so you can query positions, tangents, normals, and curvature. Make sure the parameter lies inside the valid domain (usually `[knots[p], knots[n-p])`), optionally normalized to `[0, 1]`, before calling these routines.

## Control points

Control points define the geometric “frame” the spline follows. Each control point carries a position (and optional weight for NURBS). Moving a point affects only the spans of the curve influenced by its supporting basis functions, allowing local edits. In `truck-geometry`, control points are stored as `Point3`/`Vector3` collections, and the API exposes safe accessors for inserting, removing, or iterating over them when performing modeling operations.
