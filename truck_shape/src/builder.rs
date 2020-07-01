use crate::elements::{GeometricalElement, TopologicalElement};
use crate::errors::Error;
use crate::geom_impls::*;
use crate::rsweep::RSweep;
use crate::topological_curve::TopologicalCurve;
use crate::transformed::Transformed;
use crate::tsweep::TSweep;

use crate::{Builder, Result, Transform, BSplineCurve};
use geometry::*;
use topology::*;

impl<'a> Builder<'a> {
    #[inline(always)]
    pub fn create_topology<T: GeometricalElement>(&mut self, geom: T) -> Result<T::Topology> {
        Ok(geom.create_topology(&mut self.director))
    }

    #[inline(always)]
    pub fn vertex(&mut self, coord: Vector3) -> Result<Vertex> {
        self.create_topology(Vector::new3(coord[0], coord[1], coord[2]))
    }

    pub fn line(&mut self, vertex0: Vertex, vertex1: Vertex) -> Result<Edge> {
        let director = &mut self.director;
        let pt0 = director
            .get_geometry(&vertex0)
            .ok_or(vertex0.no_geometry())?
            .clone();
        let pt1 = director
            .get_geometry(&vertex1)
            .ok_or(vertex1.no_geometry())?
            .clone();
        let edge = Edge::new(vertex0, vertex1);
        director.attach(&edge, line(pt0, pt1));
        Ok(edge)
    }

    pub fn bezier(
        &mut self,
        vertex0: Vertex,
        vertex1: Vertex,
        mut inter_points: Vec<Vector3>,
    ) -> Result<Edge>
    {
        let mut control_points: Vec<Vector3> = Vec::new();
        let pt0 = self
            .director
            .get_geometry(&vertex0)
            .ok_or(vertex0.no_geometry())?
            .projection();
        let pt1 = self
            .director
            .get_geometry(&vertex1)
            .ok_or(vertex1.no_geometry())?
            .projection();
        control_points.push(pt0.into());
        control_points.append(&mut inter_points);
        control_points.push(pt1.into());
        let curve = bezier_curve(control_points);
        let edge = Edge::try_new(vertex0, vertex1)?;
        self.director.attach(&edge, curve);
        Ok(edge)
    }

    /// create circle arc
    pub fn circle_arc(
        &mut self,
        vertex0: Vertex,
        vertex1: Vertex,
        transit: &Vector3,
    ) -> Result<Edge>
    {
        let director = &mut self.director;
        let pt0 = director
            .get_geometry(&vertex0)
            .ok_or(vertex0.no_geometry())?
            .projection();
        let pt1 = director
            .get_geometry(&vertex1)
            .ok_or(vertex1.no_geometry())?
            .projection();
        let edge = Edge::new(vertex0, vertex1);
        director.attach(&edge, circle_arc_by_three_points(pt0, pt1, transit));
        Ok(edge)
    }

    pub fn plane(&mut self, wire: Wire) -> Result<Face> {
        let surface = if wire.len() < 2 {
            Face::try_new(wire)?;
            return Err(Error::None);
        } else if wire.len() == 2 {
            let curve0 = self.get_curve(&wire[0])?;
            let curve2 = self.get_curve(&wire[1])?;
            plane_by_two_curves(curve0, curve2)
        } else if wire.len() == 3 {
            let curve0 = self.get_curve(&wire[0])?;
            let curve1 = self.get_curve(&wire[1])?;
            let curve3 = self.get_curve(&wire[2])?;
            plane_by_three_curves(curve0, curve1, curve3)
        } else {
            let wires = split_wire(&wire);
            let curve0 = self.get_curve(&wires[0])?;
            let curve1 = self.get_curve(&wires[1])?;
            let curve2 = self.get_curve(&wires[2])?;
            let curve3 = self.get_curve(&wires[3])?;
            BSplineSurface::by_boundary(curve0, curve1, curve2, curve3)
        };
        let face = Face::try_new(wire)?;
        self.director.attach(&face, surface);
        Ok(face)
    }

    #[inline(always)]
    fn get_curve<T: TopologicalCurve>(&self, curve_element: &T) -> Result<BSplineCurve> {
        curve_element.get_geometry(&self.director)
    }
    pub fn homotopy<T: TopologicalCurve>(&mut self, elem0: &T, elem1: &T) -> Result<Shell> {
        elem0.homotopy(elem1, &mut self.director)
    }
    pub fn copy<T: Transformed>(&mut self, elem: &T) -> Result<T> { elem.copy(self.director) }

    pub fn transformed<T: Transformed>(&mut self, elem: &T, trsf: &Transform) -> Result<T> {
        elem.transformed(trsf, self.director)
    }
    pub fn translated<T: Transformed>(&mut self, elem: &T, vector: &Vector3) -> Result<T> {
        elem.translated(vector, self.director)
    }

    pub fn rotated<T: Transformed>(
        &mut self,
        elem: &T,
        origin: &Vector3,
        axis: &Vector3,
        angle: f64,
    ) -> Result<T>
    {
        elem.rotated(origin, axis, angle, self.director)
    }

    pub fn scaled<T: Transformed>(
        &mut self,
        elem: &T,
        origin: &Vector3,
        scalar: &Vector3,
    ) -> Result<T>
    {
        elem.scaled(origin, scalar, self.director)
    }

    pub fn tsweep<T: TSweep>(&mut self, elem: T, vector: &Vector3) -> Result<T::Output> {
        if vector.so_small() {
            Err(Error::ZeroVectorTSweep)
        } else {
            elem.tsweep(vector, self.director)
        }
    }
    pub fn rsweep<T: RSweep>(
        &mut self,
        elem: T,
        origin: &Vector3,
        axis: &Vector3,
        angle: f64,
    ) -> Result<T::Output>
    {
        elem.rsweep(origin, axis, angle, self.director)
    }
}

fn split_wire(wire: &Wire) -> [Wire; 4] {
    let div = wire.len() / 4;
    let rem = wire.len() % 4;
    let lower_uppest = (div + 1) * rem;
    let mut new_wire = [Wire::new(), Wire::new(), Wire::new(), Wire::new()];
    let mut cursor = 0;
    for (i, edge) in wire.edge_iter().enumerate() {
        if cursor < rem {
            if i == (div + 1) * (cursor + 1) {
                cursor += 1;
            }
        } else {
            let tmp = lower_uppest + div * (cursor - rem + 1);
            if i == tmp {
                cursor += 1;
            }
        }
        new_wire[cursor].push_back(*edge);
    }
    new_wire
}

#[test]
fn test_circle_arc() {
    const N: usize = 100;
    let mut pt0 = Vector4::new3(0.17_f64.cos(), 0.17_f64.sin(), 0.0);
    let mut pt1 = Vector4::new3(1.64_f64.cos(), 1.64_f64.sin(), 0.0);
    let vector = vector!(-2, 5, 10);
    let mut axis = vector!(2, 5, 4);
    axis /= axis.norm();
    let trsf = Transform::rotate(&axis, 0.56) * Transform::translate(&vector);
    let mut transit = Vector4::new3(1.12_f64.cos(), 1.12_f64.sin(), 0.0);
    pt0 *= &trsf.0;
    pt1 *= &trsf.0;
    transit *= &trsf.0;

    let mut director = crate::Director::new();
    let edge = director.building(|builder| {
        let vertex0 = builder.create_topology(pt0.clone()).unwrap();
        let vertex1 = builder.create_topology(pt1.clone()).unwrap();
        let transit = vector!(transit[0], transit[1], transit[2]);

        builder.circle_arc(vertex0, vertex1, &transit).unwrap()
    });

    let curve = director.get_geometry(&edge).unwrap();
    Vector::assert_near(&pt0.projection(), &curve.subs(0.0).projection());
    Vector::assert_near(&pt1.projection(), &curve.subs(1.0).projection());
    for i in 0..N {
        let t = (i as f64) / (N as f64);
        let pt = curve.subs(t).projection();
        let pt = vector!(pt[0], pt[1], pt[2]);
        f64::assert_near(&(&pt - &vector).norm(), &1.0);
    }
}
