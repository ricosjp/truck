use monstertruck_geometry::prelude::*;
use monstertruck_meshing::rexport_polymesh::*;
use monstertruck_topology::compress::*;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

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

/// Splits closed edges and faces
///
/// # Details
/// The topology of the shapes handled by monstertruck has the following rules
/// - The endpoints of the edges must be different.
/// - The boundaries of the faces must be a simple wire.
///
/// Shapes created in other CAD systems do not necessarily follow these rules.
/// When such shapes are handled by monstertruck, this method is applied at the stage
/// of `CompressedShell` and `CompressedSolid`, which are intermediate forms.
///
/// # Remarks
/// Boundary simplification is still only implemented for cylinders.
/// It has not yet been implemented for cases involving singularities, such as spherical surfaces.
pub trait SplitClosedEdgesAndFaces {
    /// Splits closed edges and faces
    fn split_closed_edges_and_faces(&mut self, tol: f64);
}

impl<C, S> SplitClosedEdgesAndFaces for CompressedShell<Point3, C, S>
where
    C: ParametricCurve3D
        + BoundedCurve
        + Cut
        + ParameterDivision1D<Point = Point3>
        + SearchNearestParameter<D1, Point = Point3>
        + TryFrom<ParameterCurve<Line<Point2>, S>>,
    S: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
{
    fn split_closed_edges_and_faces(&mut self, tol: f64) {
        fn sp<S>(surface: &S, point: Point3, hint: Option<(f64, f64)>) -> Option<(f64, f64)>
        where S: SearchParameter<D2, Point = Point3> {
            surface.search_parameter(point, hint, 100)
        }
        split_closed_edges(self);
        split_closed_faces(self, tol, sp);
    }
}

impl<C, S> SplitClosedEdgesAndFaces for CompressedSolid<Point3, C, S>
where
    C: ParametricCurve3D
        + BoundedCurve
        + Cut
        + ParameterDivision1D<Point = Point3>
        + SearchNearestParameter<D1, Point = Point3>
        + TryFrom<ParameterCurve<Line<Point2>, S>>,
    S: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
{
    fn split_closed_edges_and_faces(&mut self, tol: f64) {
        self.boundaries
            .iter_mut()
            .for_each(|shell| shell.split_closed_edges_and_faces(tol))
    }
}

/// robust version of splitting closed edges and faces.
///
/// # Details
/// Robust version of [`SplitClosedEdgesAndFaces`] based on [`SearchNearestParameter`].
pub trait RobustSplitClosedEdgesAndFaces {
    /// Splits closed edges and faces
    fn robust_split_closed_edges_and_faces(&mut self, tol: f64);
}

impl<C, S> RobustSplitClosedEdgesAndFaces for CompressedShell<Point3, C, S>
where
    C: ParametricCurve3D
        + BoundedCurve
        + Cut
        + ParameterDivision1D<Point = Point3>
        + SearchNearestParameter<D1, Point = Point3>
        + TryFrom<ParameterCurve<Line<Point2>, S>>,
    S: ParametricSurface3D
        + SearchParameter<D2, Point = Point3>
        + SearchNearestParameter<D2, Point = Point3>,
{
    fn robust_split_closed_edges_and_faces(&mut self, tol: f64) {
        fn sp<S>(surface: &S, point: Point3, hint: Option<(f64, f64)>) -> Option<(f64, f64)>
        where S: SearchParameter<D2, Point = Point3> + SearchNearestParameter<D2, Point = Point3>
        {
            surface
                .search_parameter(point, hint, 100)
                .or_else(|| surface.search_parameter(point, None, 100))
                .or_else(|| surface.search_nearest_parameter(point, hint, 100))
                .or_else(|| surface.search_nearest_parameter(point, None, 100))
        }
        split_closed_edges(self);
        split_closed_faces(self, tol, sp);
    }
}

impl<C, S> RobustSplitClosedEdgesAndFaces for CompressedSolid<Point3, C, S>
where
    C: ParametricCurve3D
        + BoundedCurve
        + Cut
        + ParameterDivision1D<Point = Point3>
        + SearchNearestParameter<D1, Point = Point3>
        + TryFrom<ParameterCurve<Line<Point2>, S>>,
    S: ParametricSurface3D
        + SearchParameter<D2, Point = Point3>
        + SearchNearestParameter<D2, Point = Point3>,
{
    fn robust_split_closed_edges_and_faces(&mut self, tol: f64) {
        let fs = RobustSplitClosedEdgesAndFaces::robust_split_closed_edges_and_faces;
        self.boundaries.iter_mut().for_each(|shell| fs(shell, tol))
    }
}

/// Convenience function: heal a compressed shell and extract it.
///
/// Applies [`RobustSplitClosedEdgesAndFaces`] before calling
/// [`Shell::extract`](monstertruck_topology::Shell::extract), which avoids
/// `NotSimpleWire` errors from STEP files with repeated vertices.
pub fn extract_healed<C, S>(
    mut cshell: CompressedShell<Point3, C, S>,
    tol: f64,
) -> monstertruck_topology::Result<monstertruck_topology::Shell<Point3, C, S>>
where
    C: ParametricCurve3D
        + BoundedCurve
        + Cut
        + ParameterDivision1D<Point = Point3>
        + SearchNearestParameter<D1, Point = Point3>
        + TryFrom<ParameterCurve<Line<Point2>, S>>
        + Clone,
    S: ParametricSurface3D
        + SearchParameter<D2, Point = Point3>
        + SearchNearestParameter<D2, Point = Point3>
        + Clone,
{
    cshell.robust_split_closed_edges_and_faces(tol);
    monstertruck_topology::Shell::extract(cshell)
}

#[cfg(test)]
mod tests;
