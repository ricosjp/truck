use crate::errors::Error;
use crate::elements::TopologicalElement;
use crate::{Director, Result, BSplineCurve, BSplineSurface};
use geometry::Tolerance;
use std::iter::FromIterator;
use topology::*;

pub trait TopologicalCurve: Sized {
    fn front_vertex(&self) -> Vertex;
    fn back_vertex(&self) -> Vertex;
    fn get_geometry(&self, director: &Director) -> Result<BSplineCurve>;
    fn clone_wire(&self) -> Wire;
    fn for_each<F: FnMut(&Edge)>(&self, closure: F);
    fn is_closed(&self) -> bool;
    fn split_wire(&self) -> Option<[Wire; 2]>;
    fn homotopy<T>(&self, other: &T, director: &mut Director) -> Result<Shell>
    where T: TopologicalCurve {
        let closed0 = self.is_closed();
        let closed1 = other.is_closed();
        if closed0 && closed1 {
            closed_homotopy(self, other, director)
        } else if !closed0 && !closed1 {
            open_homotopy(self, other, director)
        } else {
            Err(Error::DifferentHomotopyType)
        }
    }
}

fn open_homotopy<C0, C1>(elem0: &C0, elem1: &C1, director: &mut Director) -> Result<Shell>
where
    C0: TopologicalCurve,
    C1: TopologicalCurve, {
    let curve0 = elem0.get_geometry(director)?;
    let curve1 = elem1.get_geometry(director)?;
    let surface = BSplineSurface::homotopy(curve0, curve1);
    let edge0 = director
        .get_builder()
        .line(elem0.back_vertex(), elem1.back_vertex())?;
    let edge1 = director
        .get_builder()
        .line(elem1.front_vertex(), elem0.front_vertex())?;
    let mut wire = elem0.clone_wire();
    wire.push_back(edge0);
    elem1.for_each(|edge| wire.push_back(edge.inverse()));
    wire.push_back(edge1);
    let face = Face::try_new(wire)?;
    director.attach(&face, surface);
    Ok(vec![face].into())
}

fn closed_homotopy<C0, C1>(elem0: &C0, elem1: &C1, director: &mut Director) -> Result<Shell>
where
    C0: TopologicalCurve,
    C1: TopologicalCurve, {
    let [mut wire0, mut wire1] = elem0.split_wire().unwrap();
    let [mut wire2, mut wire3] = elem1.split_wire().unwrap();
    let curve0 = wire0.get_geometry(director)?.clone();
    let curve2 = wire2.get_geometry(director)?.clone();
    let surface0 = BSplineSurface::homotopy(curve0, curve2);
    let curve1 = wire1.get_geometry(director)?.clone();
    let curve3 = wire3.get_geometry(director)?.clone();
    let surface1 = BSplineSurface::homotopy(curve1, curve3);
    let edge0 = director
        .get_builder()
        .line(wire0.front_vertex().unwrap(), wire2.front_vertex().unwrap())?;
    let edge1 = director
        .get_builder()
        .line(wire0.back_vertex().unwrap(), wire2.back_vertex().unwrap())?;
    wire0.push_back(edge1);
    wire0.append(wire2.invert());
    wire0.push_back(edge0.inverse());
    wire1.push_back(edge0);
    wire1.append(wire3.invert());
    wire1.push_back(edge1.inverse());
    let face0 = Face::try_new(wire0)?;
    let face1 = Face::try_new(wire1)?;
    director.attach(&face0, surface0);
    director.attach(&face1, surface1);
    Ok(vec![face0, face1].into())
}

impl TopologicalCurve for Edge {
    fn front_vertex(&self) -> Vertex { self.front() }
    fn back_vertex(&self) -> Vertex { self.back() }
    fn get_geometry(&self, director: &Director) -> Result<BSplineCurve> {
        let mut curve = director
            .get_geometry(self)
            .ok_or(self.no_geometry())?
            .clone();
        if self.front() != self.absolute_front() {
            curve.invert();
        }
        Ok(curve)
    }
    fn clone_wire(&self) -> Wire { Wire::from_iter(&[*self]) }
    fn for_each<F: FnMut(&Edge)>(&self, mut closure: F) { closure(self) }
    fn is_closed(&self) -> bool { false }
    fn split_wire(&self) -> Option<[Wire; 2]> { None }
}

impl TopologicalCurve for Wire {
    fn front_vertex(&self) -> Vertex { self.front_vertex().unwrap() }
    fn back_vertex(&self) -> Vertex { self.back_vertex().unwrap() }
    fn get_geometry(&self, director: &Director) -> Result<BSplineCurve> {
        let mut iter = self.edge_iter();
        let mut curve = iter.next().unwrap().get_geometry(director)?.clone();
        curve.knot_normalize();
        for (i, edge) in iter.enumerate() {
            let mut tmp_curve = edge.get_geometry(director)?.clone();
            let pt0 = curve.control_points().last().unwrap();
            let pt1 = tmp_curve.control_point(0);
            if !pt0[3].near(&pt1[3]) {
                let scalar = pt0[3] / pt1[3];
                tmp_curve *= scalar;
            }
            tmp_curve.knot_normalize().knot_translate((i + 1) as f64);
            curve.concat(&mut tmp_curve);
        }
        Ok(curve)
    }
    fn clone_wire(&self) -> Wire { self.clone() }
    fn for_each<F: FnMut(&Edge)>(&self, closure: F) { self.edge_iter().for_each(closure) }
    fn is_closed(&self) -> bool { self.is_closed() }
    fn split_wire(&self) -> Option<[Wire; 2]> {
        if self.len() < 2 {
            None
        } else {
            let mut part0 = self.clone();
            let part1 = part0.split_off(self.len() / 2);
            Some([part0, part1])
        }
    }
}
