use monstertruck_core::tolerance::Tolerance;
use monstertruck_geometry::prelude::*;
use monstertruck_traits::{BoundedCurve, ParametricCurve, ParametricSurface};

use super::error::FilletError;
use super::types::{self, Curve, ParameterCurveLinear};

type InternalShell = types::Shell;
const CURVE_SAMPLE_COUNT: usize = 24;
const SURFACE_SAMPLE_COUNT: usize = 64;

/// Intersection curve type used internally by fillet operations.
pub type FilletIntersectionCurve =
    IntersectionCurve<ParameterCurveLinear, Box<NurbsSurface<Vector4>>, Box<NurbsSurface<Vector4>>>;

/// Surface types that can participate in fillet operations.
///
/// Automatically implemented for any type satisfying the bounds.
pub trait FilletableSurface:
    Clone
    + ParametricSurface<Point = Point3>
    + TryInto<NurbsSurface<Vector4>>
    + From<NurbsSurface<Vector4>> {
    /// Converts this surface to a NURBS surface used by internal fillet logic.
    fn to_nurbs_surface(&self) -> Option<NurbsSurface<Vector4>> {
        self.clone()
            .try_into()
            .ok()
            .or_else(|| sample_surface_to_nurbs(self, SURFACE_SAMPLE_COUNT))
    }
}

impl<T> FilletableSurface for T where T: Clone
        + ParametricSurface<Point = Point3>
        + TryInto<NurbsSurface<Vector4>>
        + From<NurbsSurface<Vector4>>
{
}

/// Curve types that can participate in fillet operations.
///
/// Automatically implemented for any type satisfying the bounds.
pub trait FilletableCurve:
    Clone
    + ParametricCurve<Point = Point3>
    + BoundedCurve
    + TryInto<NurbsCurve<Vector4>>
    + From<NurbsCurve<Vector4>>
    + From<ParameterCurveLinear>
    + From<FilletIntersectionCurve> {
    /// Converts this curve to a NURBS curve used by internal fillet logic.
    fn to_nurbs_curve(&self) -> NurbsCurve<Vector4> {
        self.clone().try_into().ok().unwrap_or_else(|| {
            sample_curve_to_nurbs(self.range_tuple(), |t| self.evaluate(t), CURVE_SAMPLE_COUNT)
        })
    }
}

impl<T> FilletableCurve for T where T: Clone
        + ParametricCurve<Point = Point3>
        + BoundedCurve
        + TryInto<NurbsCurve<Vector4>>
        + From<NurbsCurve<Vector4>>
        + From<ParameterCurveLinear>
        + From<FilletIntersectionCurve>
{
}

fn sample_curve_to_nurbs(
    range: (f64, f64),
    evaluate: impl Fn(f64) -> Point3,
    sample_count: usize,
) -> NurbsCurve<Vector4> {
    let (t0, t1) = range;
    let points: Vec<Point3> = (0..=sample_count)
        .map(|i| t0 + (t1 - t0) * (i as f64) / (sample_count as f64))
        .map(evaluate)
        .collect();
    let knot_vector = KnotVector::uniform_knot(1, sample_count);
    NurbsCurve::from(BsplineCurve::new(knot_vector, points))
}

fn sample_surface_to_nurbs<S: ParametricSurface<Point = Point3>>(
    surface: &S,
    sample_count: usize,
) -> Option<NurbsSurface<Vector4>> {
    let (u_range, v_range) = surface.try_range_tuple();
    let ((u0, u1), (v0, v1)) = u_range.zip(v_range)?;
    let control_points: Vec<Vec<Point3>> = (0..=sample_count)
        .map(|iu| {
            let u = u0 + (u1 - u0) * (iu as f64) / (sample_count as f64);
            (0..=sample_count)
                .map(|iv| {
                    let v = v0 + (v1 - v0) * (iv as f64) / (sample_count as f64);
                    surface.evaluate(u, v)
                })
                .collect()
        })
        .collect();
    let u_knot = KnotVector::uniform_knot(1, sample_count);
    let v_knot = KnotVector::uniform_knot(1, sample_count);
    Some(NurbsSurface::from(BsplineSurface::new(
        (u_knot, v_knot),
        control_points,
    )))
}

// TryFrom for extracting NurbsCurve from internal Curve type.
impl TryFrom<Curve> for NurbsCurve<Vector4> {
    type Error = ();
    fn try_from(curve: Curve) -> std::result::Result<Self, ()> {
        match curve {
            Curve::NurbsCurve(c) => Ok(c),
            _ => Err(()),
        }
    }
}

/// Convert an external shell to internal fillet types.
///
/// Returns the internal shell and the internal `EdgeId`s corresponding to
/// the selected external edges (matched by endpoint positions).
pub(super) fn convert_shell_in<C: FilletableCurve, S: FilletableSurface>(
    shell: &monstertruck_topology::Shell<Point3, C, S>,
    edges: &[monstertruck_topology::Edge<Point3, C>],
) -> std::result::Result<(InternalShell, Vec<types::EdgeId>), FilletError> {
    // Collect endpoint pairs for requested edges (front, back).
    let edge_endpoints: Vec<(Point3, Point3)> = edges
        .iter()
        .map(|e| (e.absolute_front().point(), e.absolute_back().point()))
        .collect();

    let internal_shell: InternalShell = shell
        .try_mapped(
            |p| Some(*p),
            |c| Some(Curve::NurbsCurve(c.to_nurbs_curve())),
            |s| s.to_nurbs_surface(),
        )
        .ok_or(FilletError::UnsupportedGeometry {
            context: "failed to convert shell curves or surfaces to NURBS",
        })?;

    // Match external edges to internal edges by endpoint positions.
    let internal_edge_ids: Vec<types::EdgeId> = edge_endpoints
        .iter()
        .map(|(ext_front, ext_back)| {
            internal_shell
                .edge_iter()
                .find(|ie| {
                    let f = ie.absolute_front().point();
                    let b = ie.absolute_back().point();
                    (f.near(ext_front) && b.near(ext_back))
                        || (f.near(ext_back) && b.near(ext_front))
                })
                .map(|ie| ie.id())
                .ok_or(FilletError::EdgeNotFound)
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok((internal_shell, internal_edge_ids))
}

/// Convert an internal fillet shell back to external types.
pub(super) fn convert_shell_out<C: FilletableCurve, S: FilletableSurface>(
    shell: &InternalShell,
) -> std::result::Result<monstertruck_topology::Shell<Point3, C, S>, FilletError> {
    shell
        .try_mapped(
            |p| Some(*p),
            |c| {
                Some(match c {
                    Curve::NurbsCurve(nc) => C::from(nc.clone()),
                    Curve::ParameterCurve(pc) => C::from(pc.clone()),
                    Curve::IntersectionCurve(ic) => C::from(ic.clone()),
                })
            },
            |s| Some(S::from(s.clone())),
        )
        .ok_or(FilletError::UnsupportedGeometry {
            context: "failed to convert internal shell back to external types",
        })
}
