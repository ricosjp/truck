use truck_base::tolerance::Tolerance;
use truck_geometry::prelude::*;

use super::error::FilletError;
use super::types::{self, Curve, ParamCurveLinear};

type InternalShell = types::Shell;

/// Surface types that can be converted to/from `NurbsSurface<Vector4>` for filleting.
pub trait FilletableSurface: Clone {
    /// Convert to the internal NURBS representation. Returns `None` if unsupported.
    fn to_nurbs_surface(&self) -> Option<NurbsSurface<Vector4>>;
    /// Wrap a NURBS surface back into this type.
    fn from_nurbs_surface(surface: NurbsSurface<Vector4>) -> Self;
}

/// Curve types that can be converted to/from fillet-internal curve types.
pub trait FilletableCurve: Clone {
    /// Convert to a NURBS curve. Returns `None` if unsupported.
    fn to_nurbs_curve(&self) -> Option<NurbsCurve<Vector4>>;
    /// Wrap a NURBS curve back into this type.
    fn from_nurbs_curve(c: NurbsCurve<Vector4>) -> Self;
    /// Convert from a PCurve (line on NURBS surface).
    fn from_pcurve(c: ParamCurveLinear) -> Self;
    /// Convert from an intersection curve.
    fn from_intersection_curve(
        c: IntersectionCurve<
            ParamCurveLinear,
            Box<NurbsSurface<Vector4>>,
            Box<NurbsSurface<Vector4>>,
        >,
    ) -> Self;
}

// Identity impl: NurbsSurface<Vector4> passes through.
impl FilletableSurface for NurbsSurface<Vector4> {
    fn to_nurbs_surface(&self) -> Option<NurbsSurface<Vector4>> { Some(self.clone()) }
    fn from_nurbs_surface(surface: NurbsSurface<Vector4>) -> Self { surface }
}

// Identity impl: internal Curve maps each variant directly.
impl FilletableCurve for Curve {
    fn to_nurbs_curve(&self) -> Option<NurbsCurve<Vector4>> {
        match self {
            Curve::NurbsCurve(c) => Some(c.clone()),
            _ => None,
        }
    }
    fn from_nurbs_curve(c: NurbsCurve<Vector4>) -> Self { Curve::NurbsCurve(c) }
    fn from_pcurve(c: ParamCurveLinear) -> Self { Curve::PCurve(c) }
    fn from_intersection_curve(
        c: IntersectionCurve<
            ParamCurveLinear,
            Box<NurbsSurface<Vector4>>,
            Box<NurbsSurface<Vector4>>,
        >,
    ) -> Self {
        Curve::IntersectionCurve(c)
    }
}

/// Convert an external shell to internal fillet types.
///
/// Returns the internal shell and the internal `EdgeID`s corresponding to
/// the selected external edges (matched by endpoint positions).
pub(super) fn convert_shell_in<C: FilletableCurve, S: FilletableSurface>(
    shell: &truck_topology::Shell<Point3, C, S>,
    edges: &[truck_topology::Edge<Point3, C>],
) -> std::result::Result<(InternalShell, Vec<types::EdgeID>), FilletError> {
    // Collect endpoint pairs for requested edges (front, back).
    let edge_endpoints: Vec<(Point3, Point3)> = edges
        .iter()
        .map(|e| (e.absolute_front().point(), e.absolute_back().point()))
        .collect();

    let internal_shell: InternalShell = shell
        .try_mapped(
            |p| Some(*p),
            |c| c.to_nurbs_curve().map(Curve::NurbsCurve),
            |s| s.to_nurbs_surface(),
        )
        .ok_or(FilletError::UnsupportedGeometry {
            context: "failed to convert shell curves or surfaces to NURBS",
        })?;

    // Match external edges to internal edges by endpoint positions.
    let internal_edge_ids: Vec<types::EdgeID> = edge_endpoints
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
) -> std::result::Result<truck_topology::Shell<Point3, C, S>, FilletError> {
    shell
        .try_mapped(
            |p| Some(*p),
            |c| {
                Some(match c {
                    Curve::NurbsCurve(nc) => C::from_nurbs_curve(nc.clone()),
                    Curve::PCurve(pc) => C::from_pcurve(pc.clone()),
                    Curve::IntersectionCurve(ic) => C::from_intersection_curve(ic.clone()),
                })
            },
            |s| Some(S::from_nurbs_surface(s.clone())),
        )
        .ok_or(FilletError::UnsupportedGeometry {
            context: "failed to convert internal shell back to external types",
        })
}
