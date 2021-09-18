mod divide_face;
mod faces_classification;
mod intersection_curve;
mod loops_store;
mod polyline_construction;
pub use intersection_curve::{intersection_curves, IntersectionCurve};

#[cfg(test)]
mod test_util;
