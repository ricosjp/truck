use derive_more::From;
use truck_geometry::prelude::*;

/// A parametric curve defined by a line in parameter space on a NURBS surface.
pub type ParamCurveLinear = PCurve<Line<Point2>, NurbsSurface<Vector4>>;

#[allow(clippy::enum_variant_names)]
#[derive(
    Clone,
    Debug,
    ParametricCurve,
    BoundedCurve,
    ParameterDivision1D,
    Cut,
    From,
    Invertible,
    SearchParameterD1,
    SearchNearestParameterD1,
)]
pub(crate) enum Curve {
    NurbsCurve(NurbsCurve<Vector4>),
    PCurve(ParamCurveLinear),
    IntersectionCurve(
        IntersectionCurve<ParamCurveLinear, Box<NurbsSurface<Vector4>>, Box<NurbsSurface<Vector4>>>,
    ),
}

truck_topology::prelude!(Point3, Curve, NurbsSurface<Vector4>, pub(super));

/// Trait alias for curves usable in fillet operations.
pub trait FilletCurve: ParametricCurve3D + BoundedCurve + ParameterDivision1D {}
impl<C: ParametricCurve3D + BoundedCurve + ParameterDivision1D> FilletCurve for C {}

pub(super) trait NotStrictlyCut: Sized {
    fn pre_cut(&self, vertex: &Vertex, curve: Curve, t: f64) -> (Self, Self);
    fn not_strictly_cut(&self, vertex: &Vertex) -> Option<(Self, Self)>;
    fn not_strictly_cut_with_parameter(&self, vertex: &Vertex, t: f64) -> Option<(Self, Self)>;
}

impl NotStrictlyCut for Edge {
    fn pre_cut(&self, vertex: &Vertex, mut curve0: Curve, t: f64) -> (Self, Self) {
        let curve1 = curve0.cut(t);
        let mut edge0 = Edge::new(self.absolute_front(), vertex, curve0);
        let mut edge1 = Edge::new(vertex, self.absolute_back(), curve1);
        match self.orientation() {
            true => (edge0, edge1),
            false => {
                edge0.invert();
                edge1.invert();
                (edge1, edge0)
            }
        }
    }
    fn not_strictly_cut(&self, vertex: &Vertex) -> Option<(Self, Self)> {
        let curve0 = self.curve();
        let t = curve0.search_nearest_parameter(vertex.point(), None, 100)?;
        let (t0, t1) = curve0.range_tuple();
        if t < t0 + TOLERANCE || t1 - TOLERANCE < t {
            return None;
        }
        Some(self.pre_cut(vertex, curve0, t))
    }

    fn not_strictly_cut_with_parameter(&self, vertex: &Vertex, t: f64) -> Option<(Self, Self)> {
        let curve0 = self.curve();
        let (t0, t1) = curve0.range_tuple();
        if t < t0 + TOLERANCE || t1 - TOLERANCE < t {
            return None;
        }
        Some(self.pre_cut(vertex, curve0, t))
    }
}
