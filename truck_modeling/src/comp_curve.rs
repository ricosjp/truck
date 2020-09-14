use crate::*;

pub trait CompoundCurve: Sized {
    fn front_vertex(&self) -> &Vertex;
    fn back_vertex(&self) -> &Vertex;
    fn get_curve(&self) -> CurveCollector;
    fn clone_wire(&self) -> Wire;
    fn for_each<F: FnMut(&Edge)>(&self, closure: F);
    fn is_closed(&self) -> bool;
    fn split_wire(&self) -> Option<[Wire; 2]>;
}

fn open_homotopy<C0, C1>(elem0: &C0, elem1: &C1) -> Shell
where
    C0: CompoundCurve,
    C1: CompoundCurve, {
    let curve0 = elem0.get_curve().unwrap();
    let curve1 = elem1.get_curve().unwrap();
    let surface = BSplineSurface::homotopy(curve0, curve1);
    let edge0 = builder::line(elem0.back_vertex(), elem1.back_vertex());
    let edge1 = builder::line(elem1.front_vertex(), elem0.front_vertex());
    let mut wire = elem0.clone_wire();
    wire.push_back(edge0);
    elem1.for_each(|edge| wire.push_back(edge.inverse()));
    wire.push_back(edge1);
    let face = Face::new(vec![wire], surface);
    vec![face].into()
}

impl CompoundCurve for Edge {
    fn front_vertex(&self) -> &Vertex { self.front() }
    fn back_vertex(&self) -> &Vertex { self.back() }
    fn get_curve(&self) -> CurveCollector {
        let mut curve = self.try_lock_curve().unwrap().clone();
        if self.front() != self.absolute_front() {
            curve.invert();
        }
        CurveCollector::Curve(curve)
    }
    fn clone_wire(&self) -> Wire { vec![self.clone()].into() }
    fn for_each<F: FnMut(&Edge)>(&self, mut closure: F) { closure(self) }
    fn is_closed(&self) -> bool { false }
    fn split_wire(&self) -> Option<[Wire; 2]> { None }
}

impl CompoundCurve for Wire {
    fn front_vertex(&self) -> &Vertex { self.front_vertex().unwrap() }
    fn back_vertex(&self) -> &Vertex { self.back_vertex().unwrap() }
    fn get_curve(&self) -> CurveCollector {
        let mut cc = CurveCollector::Singleton;
        for edge in self.edge_iter() {
            let mut curve = edge.get_curve().unwrap();
            if let CurveCollector::Curve(ref already) = cc {
                let pt0 = already.control_points().last().unwrap();
                let pt1 = curve.control_points().first().unwrap();
                if !Tolerance::near(&pt0[3], &pt1[3]) {
                    let scalar = pt0[3] / pt1[3];
                    curve = curve * scalar;
                }
                cc.concat(&mut curve);
            }
        }
        cc
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