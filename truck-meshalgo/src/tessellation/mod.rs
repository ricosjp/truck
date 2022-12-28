use crate::*;
use spade::{iterators::*, *};
use truck_topology::{compress::*, *};

#[cfg(not(target_arch = "wasm32"))]
mod meshables_traits {
    use super::*;
    /// Gathered the traits used in tessellation.
    pub trait PolylineableCurve:
        ParametricCurve3D + BoundedCurve + ParameterDivision1D<Point = Point3> + Send + Sync {
    }
    impl<
            C: ParametricCurve3D + BoundedCurve + ParameterDivision1D<Point = Point3> + Send + Sync,
        > PolylineableCurve for C
    {
    }
    /// Gathered the traits used in tessellation.
    pub trait MeshableSurface:
        ParametricSurface3D
        + ParameterDivision2D
        + SearchParameter<D2, Point = Point3>
        + Send
        + Sync {
    }
    impl<
            S: ParametricSurface3D
                + ParameterDivision2D
                + SearchParameter<D2, Point = Point3>
                + Send
                + Sync,
        > MeshableSurface for S
    {
    }
}

#[cfg(target_arch = "wasm32")]
mod meshables_traits {
    use super::*;
    /// Gathered the traits used in tessellation.
    pub trait PolylineableCurve:
        ParametricCurve3D + BoundedCurve + ParameterDivision1D<Point = Point3> {
    }
    impl<C: ParametricCurve3D + BoundedCurve + ParameterDivision1D<Point = Point3>>
        PolylineableCurve for C
    {
    }
    /// Gathered the traits used in tessellation.
    pub trait MeshableSurface:
        ParametricSurface3D + ParameterDivision2D + SearchParameter<D2, Point = Point3> {
    }
    impl<S: ParametricSurface3D + ParameterDivision2D + SearchParameter<D2, Point = Point3>>
        MeshableSurface for S
    {
    }
}

pub use meshables_traits::*;

type PolylineCurve = truck_polymesh::PolylineCurve<Point3>;

/// Trait for converting tessellated shape into polygon.
pub trait MeshedShape {
    /// Converts tessellated shape into polygon.
    fn to_polygon(&self) -> PolygonMesh;
}

impl MeshedShape for Shell<Point3, PolylineCurve, PolygonMesh> {
    fn to_polygon(&self) -> PolygonMesh {
        let mut polygon = PolygonMesh::default();
        self.face_iter().for_each(|face| {
            polygon.merge(face.oriented_surface());
        });
        polygon
    }
}

impl MeshedShape for Shell<Point3, PolylineCurve, Option<PolygonMesh>> {
    fn to_polygon(&self) -> PolygonMesh {
        let mut polygon = PolygonMesh::default();
        self.face_iter().for_each(|face| {
            if let Some(mut poly) = face.get_surface() {
                if !face.orientation() {
                    poly.invert();
                }
                polygon.merge(poly);
            }
        });
        polygon
    }
}

impl<P, C, S> MeshedShape for Solid<P, C, S>
where Shell<P, C, S>: MeshedShape
{
    fn to_polygon(&self) -> PolygonMesh {
        let mut polygon = PolygonMesh::default();
        self.boundaries().iter().for_each(|shell| {
            polygon.merge(shell.to_polygon());
        });
        polygon
    }
}

impl MeshedShape for CompressedShell<Point3, PolylineCurve, PolygonMesh> {
    fn to_polygon(&self) -> PolygonMesh {
        let mut polygon = PolygonMesh::default();
        self.faces.iter().for_each(|face| match face.orientation {
            true => polygon.merge(face.surface.clone()),
            false => polygon.merge(face.surface.inverse()),
        });
        polygon
    }
}

impl MeshedShape for CompressedShell<Point3, PolylineCurve, Option<PolygonMesh>> {
    fn to_polygon(&self) -> PolygonMesh {
        let mut polygon = PolygonMesh::default();
        self.faces.iter().for_each(|face| {
            if let Some(surface) = &face.surface {
                match face.orientation {
                    true => polygon.merge(surface.clone()),
                    false => polygon.merge(surface.inverse()),
                }
            }
        });
        polygon
    }
}

impl<P, C, S> MeshedShape for CompressedSolid<P, C, S>
where CompressedShell<P, C, S>: MeshedShape
{
    fn to_polygon(&self) -> PolygonMesh {
        let mut polygon = PolygonMesh::default();
        self.boundaries.iter().for_each(|shell| {
            polygon.merge(shell.to_polygon());
        });
        polygon
    }
}

/// Trait for tessellating `Shell` and `Solid` in `truck-modeling`.
pub trait MeshableShape {
    /// Shape whose edges are made polylines and faces polygon surface.
    type MeshedShape: MeshedShape;
    /// Tessellates shapes. The division of curves and surfaces are by `ParameterDivision1D` and `ParameterDivision2D`,
    /// and the constrained Delauney triangulation is based on the crate [`spade`](https://crates.io/crates/spade).
    ///
    /// # Panics
    ///
    /// `tol` must be more than `TOLERANCE`.
    ///
    /// # Remarks
    ///
    /// The tessellated mesh is not necessarily closed even if `self` is `Solid`.
    /// If you want to get closed mesh, use `OptimizationFilter::put_together_same_attrs`.
    /// ```
    /// use truck_meshalgo::prelude::*;
    /// use truck_modeling::builder;
    /// use truck_topology::shell::ShellCondition;
    ///
    /// // modeling a unit cube
    /// let v = builder::vertex(Point3::origin());
    /// let e = builder::tsweep(&v, Vector3::unit_x());
    /// let f = builder::tsweep(&e, Vector3::unit_y());
    /// let cube = builder::tsweep(&f, Vector3::unit_z());
    ///
    /// // cube is Solid, however, the tessellated mesh is not closed.
    /// let mut mesh = cube.triangulation(0.01).to_polygon();
    /// assert!(mesh.shell_condition() != ShellCondition::Closed);
    ///
    /// // use optimization filters!
    /// mesh.put_together_same_attrs();
    /// assert!(mesh.shell_condition() == ShellCondition::Closed);
    /// ```
    fn triangulation(&self, tol: f64) -> Self::MeshedShape;
}

impl<C: PolylineableCurve, S: MeshableSurface> MeshableShape for Shell<Point3, C, S> {
    type MeshedShape = Shell<Point3, PolylineCurve, Option<PolygonMesh>>;
    fn triangulation(&self, tol: f64) -> Self::MeshedShape {
        nonpositive_tolerance!(tol);
        #[cfg(not(target_arch = "wasm32"))]
        let res = triangulation::shell_tessellation(self, tol);
        #[cfg(target_arch = "wasm32")]
        let res = triangulation::shell_tessellation_single_thread(self, tol);
        res
    }
}

impl<C: PolylineableCurve, S: MeshableSurface> MeshableShape for Solid<Point3, C, S> {
    type MeshedShape = Solid<Point3, PolylineCurve, Option<PolygonMesh>>;
    fn triangulation(&self, tol: f64) -> Self::MeshedShape {
        let boundaries = self
            .boundaries()
            .iter()
            .map(|shell| shell.triangulation(tol))
            .collect::<Vec<_>>();
        Solid::new(boundaries)
    }
}

impl<C: PolylineableCurve, S: MeshableSurface> MeshableShape for CompressedShell<Point3, C, S> {
    type MeshedShape = CompressedShell<Point3, PolylineCurve, Option<PolygonMesh>>;
    fn triangulation(&self, tol: f64) -> Self::MeshedShape {
        nonpositive_tolerance!(tol);
        triangulation::cshell_tessellation(self, tol)
    }
}

impl<C: PolylineableCurve, S: MeshableSurface> MeshableShape for CompressedSolid<Point3, C, S> {
    type MeshedShape = CompressedSolid<Point3, PolylineCurve, Option<PolygonMesh>>;
    fn triangulation(&self, tol: f64) -> Self::MeshedShape {
        let boundaries = self
            .boundaries
            .iter()
            .map(|shell| shell.triangulation(tol))
            .collect::<Vec<_>>();
        CompressedSolid { boundaries }
    }
}

mod triangulation;
