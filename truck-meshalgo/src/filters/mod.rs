use crate::*;
use polygon_mesh::PolygonMeshEditor;

mod normal_filters;
mod optimizing;
mod structuring;
mod subdivision;

pub use normal_filters::NormalFilters;
pub use optimizing::{sub_put_together_same_attrs, OptimizingFilter, SameAttr};
pub use structuring::StructuringFilter;
pub use subdivision::Subdivision;
