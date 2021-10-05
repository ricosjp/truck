use crate::*;
use spade::delaunay::*;
use spade::kernels::*;
use truck_topology::{*, Vertex};

/// Gathered the traits used in tessellation.
#[rustfmt::skip]
pub trait PolylineableCurve: ParametricCurve3D + Invertible + ParameterDivision1D<Point = Point3> {}
#[rustfmt::skip]
impl<C: ParametricCurve3D + Invertible + ParameterDivision1D<Point = Point3>> PolylineableCurve for C {}
#[rustfmt::skip]
/// Gathered the traits used in tessellation.
pub trait MeshableSurface: ParametricSurface3D + Invertible + ParameterDivision2D + SearchParameter<Point = Point3, Parameter = (f64, f64)> {}
#[rustfmt::skip]
impl<S: ParametricSurface3D + Invertible + ParameterDivision2D + SearchParameter<Point = Point3, Parameter = (f64, f64)>> MeshableSurface for S {}

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

impl MeshedShape for Solid<Point3, PolylineCurve, PolygonMesh> {
    fn to_polygon(&self) -> PolygonMesh {
        let mut polygon = PolygonMesh::default();
        self.boundaries().iter().for_each(|shell| {
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
    /// let mut mesh = cube.triangulation(0.01).unwrap().to_polygon();
    /// assert!(mesh.shell_condition() != ShellCondition::Closed);
    ///
    /// // use optimization filters!
    /// mesh.put_together_same_attrs();
    /// assert!(mesh.shell_condition() == ShellCondition::Closed);
    /// ```
    fn triangulation(&self, tol: f64) -> Option<Self::MeshedShape>;
}

impl<C: PolylineableCurve, S: MeshableSurface> MeshableShape for Shell<Point3, C, S> {
    type MeshedShape = Shell<Point3, PolylineCurve, PolygonMesh>;
    fn triangulation(&self, tol: f64) -> Option<Self::MeshedShape> {
        nonpositive_tolerance!(tol);
        triangulation::tessellation(self, tol)
    }
}

impl<C: PolylineableCurve, S: MeshableSurface> MeshableShape for Solid<Point3, C, S> {
    type MeshedShape = Solid<Point3, PolylineCurve, PolygonMesh>;
    fn triangulation(&self, tol: f64) -> Option<Self::MeshedShape> {
        let boundaries = self
            .boundaries()
            .iter()
            .map(|shell| shell.triangulation(tol))
            .collect::<Option<Vec<_>>>()?;
        Solid::try_new(boundaries).ok()
    }
}

mod triangulation;
