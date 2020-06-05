use crate::{Director, Result};
use geometry::*;
use topology::*;

impl Director {
    pub fn line(&mut self, vertex0: Vertex, vertex1: Vertex) -> Result<Edge> {
        let knot_vec = KnotVec::bezier_knot(1);
        let pt0 = self.get_point(&vertex0)?.clone();
        let pt1 = self.get_point(&vertex1)?.clone();
        let line = BSplineCurve::try_new(knot_vec, vec![pt0, pt1])?;
        self.create_edge(vertex0, vertex1, line)
    }

    pub fn plane(&mut self, wire: Wire) -> Result<Face> {
        if wire.len() < 2 {
            Ok(Face::try_new(wire)?)
        } else if wire.len() == 2 {
            let curve0 = self.get_oriented_curve(&wire[0])?;
            let mut curve1 = self.get_curve(&wire[1])?.clone();
            let surface = BSplineSurface::homotopy(&curve0, curve1.inverse());
            self.create_face(wire, surface)
        } else if wire.len() == 3 {
            let mut curve0 = self.get_oriented_curve(&wire[0])?;
            let mut curve1 = self.get_oriented_curve(&wire[1])?;
            let mut curve2 = self.get_oriented_curve(&wire[2])?;
            curve0.knot_normalize().concat(curve1.knot_normalize().knot_translate(1.0))?;
            let surface = BSplineSurface::homotopy(&curve0, curve2.inverse());
            self.create_face(wire, surface)
        } else {
            let wires = split_wire(&wire);
            let mut curve0 = self.bspline_by_wire(&wires[0])?;
            let mut curve1 = self.bspline_by_wire(&wires[1])?;
            let mut curve2 = self.bspline_by_wire(&wires[2])?;
            let mut curve3 = self.bspline_by_wire(&wires[3])?;
            curve2.inverse();
            curve3.inverse();
            curve0.syncro_degree(&mut curve2);
            curve0.optimize();
            curve2.optimize();
            curve0.syncro_knot(&mut curve2);
            curve3.syncro_degree(&mut curve1);
            curve3.optimize();
            curve1.optimize();
            curve3.syncro_knot(&mut curve1);
            let knot_vecs = (
                curve0.knot_vec().clone(),
                curve3.knot_vec().clone(),
            );
            let mut control_points = Vec::new();
            control_points.push(curve3.control_points().clone());
            let n = curve0.control_points().len();
            let m = curve3.control_points().len();
            for i in 1..(n - 1) {
                let u = (i as f64) / (n as f64);
                let pt0 = curve0.control_point(i) * u + curve2.control_point(i) * (1.0 - u);
                let mut new_row = Vec::new();
                for j in 1..(m - 1) {
                    let v = (j as f64) / (m as f64);
                    let pt1 = curve3.control_point(j) * v + curve1.control_point(j) * (1.0 - v);
                    new_row.push((&pt0 + pt1) / 2.0);
                }
                control_points.push(new_row);
            }
            control_points.push(curve1.control_points().clone());
            let surface = BSplineSurface::try_new(knot_vecs, control_points)?;
            self.create_face(wire, surface)
        }
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
