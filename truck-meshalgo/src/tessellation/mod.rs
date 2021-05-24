#![allow(dead_code)]

use spade::delaunay::*;
use spade::kernels::*;
use truck_geotrait::*;
use truck_polymesh::{Vertex, *};
use truck_topology::*;

#[cfg_attr(rustfmt, rustfmt_skip)]
pub trait PolylineableCurve: ParametricCurve<Point = Point3, Vector = Vector3> + Invertible + ParameterDivision1D {}
#[cfg_attr(rustfmt, rustfmt_skip)]
impl<C: ParametricCurve<Point = Point3, Vector = Vector3> + Invertible + ParameterDivision1D> PolylineableCurve for C {}
#[cfg_attr(rustfmt, rustfmt_skip)]
pub trait MeshableSurface: ParametricSurface3D + Invertible + ParameterDivision2D + SearchParameter<Point = Point3, Parameter = (f64, f64)> {}
#[cfg_attr(rustfmt, rustfmt_skip)]
impl<S: ParametricSurface3D + Invertible + ParameterDivision2D + SearchParameter<Point = Point3, Parameter = (f64, f64)>> MeshableSurface for S {}

pub trait MeshableShape {
    fn triangulation(&self, tol: f64) -> Option<PolygonMesh>;
}

impl<C: PolylineableCurve, S: MeshableSurface> MeshableShape for Shell<Point3, C, S> {
    fn triangulation(&self, tol: f64) -> Option<PolygonMesh> {
        triangulation::tessellation(self.face_iter(), tol)
    }
}

impl<C: PolylineableCurve, S: MeshableSurface> MeshableShape for Solid<Point3, C, S> {
    fn triangulation(&self, tol: f64) -> Option<PolygonMesh> {
        triangulation::tessellation(self.boundaries().iter().flat_map(Shell::face_iter), tol)
    }
}

mod triangulation;
