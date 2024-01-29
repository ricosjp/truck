#![allow(dead_code, unused_imports)]

use rustc_hash::FxHashMap as HashMap;
use truck_geometry::prelude::*;
use truck_meshalgo::rexport_polymesh::*;
use truck_topology::compress::*;

type Edge<C> = CompressedEdge<C>;
type EdgeIndex = CompressedEdgeIndex;
type Wire = Vec<EdgeIndex>;
type Face<S> = CompressedFace<S>;
type Shell<P, C, S> = CompressedShell<P, C, S>;

trait SP<S>: Fn(&S, Point3, Option<(f64, f64)>) -> Option<(f64, f64)> {}
impl<S, F> SP<S> for F where F: Fn(&S, Point3, Option<(f64, f64)>) -> Option<(f64, f64)> {}

mod split_closed_edges;
use split_closed_edges::split_closed_edges;

mod split_closed_faces;
use split_closed_faces::split_closed_faces;

#[cfg(test)]
mod tests;
