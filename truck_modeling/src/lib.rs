extern crate truck_integral;
pub use truck_integral::*;

// geometrical elements
pub type BSplineCurve = truck_integral::BSplineCurve<Vector4>;
pub type BSplineSurface = truck_integral::BSplineSurface<Vector4>;
pub type CurveCollector = truck_integral::CurveCollector<Vector4>;

// topological elements
pub type Vertex = truck_integral::Vertex<Vector4>;
pub type Edge = truck_integral::Edge<Vector4, BSplineCurve>;
pub type Wire = truck_integral::Wire<Vector4, BSplineCurve>;
pub type Face = truck_integral::Face<Vector4, BSplineCurve, BSplineSurface>;
pub type Shell = truck_integral::Shell<Vector4, BSplineCurve, BSplineSurface>;
pub type Solid = truck_integral::Solid<Vector4, BSplineCurve, BSplineSurface>;

pub type VertexID = truck_integral::VertexID<Vector4>;
pub type EdgeID = truck_integral::EdgeID<BSplineCurve>;
pub type FaceID = truck_integral::FaceID<BSplineSurface>;

mod geom_impls;
pub mod builder;
//pub mod transform;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
