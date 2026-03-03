use monstertruck_core::tolerance::Tolerance;
use monstertruck_geometry::prelude::*;

use super::error::FilletError;
use super::types::{self, Curve, ParameterCurveLinear};

type InternalShell = types::Shell;

/// Intersection curve type used internally by fillet operations.
pub type FilletIntersectionCurve =
    IntersectionCurve<ParameterCurveLinear, Box<NurbsSurface<Vector4>>, Box<NurbsSurface<Vector4>>>;

/// Surface types that can participate in fillet operations.
///
/// Automatically implemented for any type satisfying the bounds.
pub trait FilletableSurface:
    Clone + TryInto<NurbsSurface<Vector4>> + From<NurbsSurface<Vector4>> {
}

impl<T> FilletableSurface for T where T: Clone + TryInto<NurbsSurface<Vector4>> + From<NurbsSurface<Vector4>> {}

/// Curve types that can participate in fillet operations.
///
/// Automatically implemented for any type satisfying the bounds.
pub trait FilletableCurve:
    Clone
    + TryInto<NurbsCurve<Vector4>>
    + From<NurbsCurve<Vector4>>
    + From<ParameterCurveLinear>
    + From<FilletIntersectionCurve> {
}

impl<T> FilletableCurve for T where T: Clone
        + TryInto<NurbsCurve<Vector4>>
        + From<NurbsCurve<Vector4>>
        + From<ParameterCurveLinear>
        + From<FilletIntersectionCurve>
{
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
            |c| c.clone().try_into().ok().map(Curve::NurbsCurve),
            |s| s.clone().try_into().ok(),
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
