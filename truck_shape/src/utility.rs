use crate::{Director, Result, Transform};
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

    /// create circle arc
    pub fn circle_arc(
        &mut self,
        vertex0: Vertex,
        vertex1: Vertex,
        transit: &Vector3,
    ) -> Result<Edge>
    {
        let tmp = self.get_point(&vertex0)?.projection();
        let pt0 = Vector3::new(tmp[0], tmp[1], tmp[2]);
        let tmp = self.get_point(&vertex1)?.projection();
        let pt1 = Vector3::new(tmp[0], tmp[1], tmp[2]);

        // circum_center
        let org = circum_center(&pt0, &pt1, transit);
        let trsf0 = Transform::translate(&org);

        // scalar component
        let scalar = (&pt0 - &org).norm();
        let trsf1 = Transform::scale(&Vector3::new(scalar, scalar, scalar));

        // the cosine of half of the angle of arc
        let cos = - (&pt0 - transit).cos_angle(&(&pt1 - transit));
        let mut curve = unit_circle_arc(cos);

        // orthogonal part
        let vec0 = &pt1 - &pt0;
        let mid = (&pt0 + &pt1) / 2.0;
        let vec1 = transit - mid;
        let mut radius = &vec1 - (&vec0 * &vec1) / (&vec0 * &vec0) * vec0;
        radius /= radius.norm();
        let mut n = (&pt1 - transit) ^ (&pt0 - transit);
        n /= n.norm();
        let trsf2 = Transform::by_axes(&radius, &(&n ^ &radius), &n);

        let trsf = trsf1 * trsf2 * trsf0;
        curve *= trsf.0;

        self.create_edge(vertex0, vertex1, curve)
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
            curve0
                .knot_normalize()
                .concat(curve1.knot_normalize().knot_translate(1.0))?;
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
            let knot_vecs = (curve0.knot_vec().clone(), curve3.knot_vec().clone());
            let mut control_points = Vec::new();
            control_points.push(curve3.control_points().clone());
            let n = curve0.control_points().len();
            let m = curve3.control_points().len();
            for i in 1..(n - 1) {
                let u = (i as f64) / (n as f64);
                let pt0 = curve0.control_point(i) * u + curve2.control_point(i) * (1.0 - u);
                let mut new_row = Vec::new();
                new_row.push(curve0.control_point(i).clone());
                for j in 1..(m - 1) {
                    let v = (j as f64) / (m as f64);
                    let pt1 = curve3.control_point(j) * v + curve1.control_point(j) * (1.0 - v);
                    new_row.push((&pt0 + pt1) / 2.0);
                }
                new_row.push(curve2.control_point(i).clone());
                control_points.push(new_row);
            }
            control_points.push(curve1.control_points().clone());
            let surface = BSplineSurface::try_new(knot_vecs, control_points)?;
            self.create_face(wire, surface)
        }
    }

    pub fn homotopy(&mut self, wire0: &Wire, wire1: &Wire) -> Result<Face> {
        let curve0 = self.bspline_by_wire(wire0)?;
        let curve1 = self.bspline_by_wire(wire1)?;
        let surface = BSplineSurface::homotopy(&curve0, &curve1);
        let edge0 = self.line(wire0.back_vertex().unwrap(), wire1.back_vertex().unwrap())?;
        let edge1 = self.line(wire1.front_vertex().unwrap(), wire0.front_vertex().unwrap())?;
        let mut wire = wire0.clone();
        wire.push_back(edge0);
        for edge in wire1.edge_iter() {
            wire.push_back(edge.inverse());
        }
        wire.push_back(edge1);
        self.create_face(wire, surface)
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

fn circum_center(pt0: &Vector3, pt1: &Vector3, pt2: &Vector3) -> Vector3 {
    let vec0 = pt1 - pt0;
    let vec1 = pt2 - pt0;
    let a2 = &vec0 * &vec0;
    let ab = &vec0 * &vec1;
    let b2 = &vec1 * &vec1;
    let det = a2 * b2 - ab * ab;
    let u = (b2 * a2 - ab * b2) / (2.0 * det);
    let v = (-ab * a2 + b2 * a2) / (2.0 * det);
    pt0 + u * vec0 + v * vec1
}

fn unit_circle_arc(cos: f64) -> BSplineCurve {
    let knot_vec = KnotVec::bezier_knot(2);
    let sin = (1.0 - cos * cos).sqrt();
    let control_points = vec![
        Vector::new(cos, -sin, 0.0, 1.0),
        Vector::new(1.0, 0.0, 0.0, cos),
        Vector::new(cos, sin, 0.0, 1.0),
    ];
    BSplineCurve::new_unchecked(knot_vec, control_points)
}

#[test]
fn test_circle_arc() {
    const N: usize = 100;
    let mut director = Director::new();

    let mut axis = Vector3::new(2, 5, 4);
    axis /= axis.norm();
    let vector = Vector3::new(-2, 5, 10);
    let trsf = Transform::rotate(&axis, 0.56) * Transform::translate(&vector);
    let mut pt0 = Vector::new3(0.17_f64.cos(), 0.17_f64.sin(), 0.0);
    let mut pt1 = Vector::new3(1.64_f64.cos(), 1.64_f64.sin(), 0.0);
    let mut transit = Vector::new3(1.12_f64.cos(), 1.12_f64.sin(), 0.0);
    pt0 *= &trsf.0;
    pt1 *= &trsf.0;
    transit *= &trsf.0;

    let vertex0 = director.create_vertex(pt0.clone());
    let vertex1 = director.create_vertex(pt1.clone());
    let transit = Vector3::new(transit[0], transit[1], transit[2]);

    let edge = director.circle_arc(vertex0, vertex1, &transit).unwrap();
    let curve = director.get_curve(&edge).unwrap();
    println!("{:?}", pt0.projection() - curve.subs(0.0).projection());
    println!("{:?}", pt1.projection() - curve.subs(1.0).projection());
    for i in 0..N {
        let t = (i as f64) / (N as f64);
        let pt = curve.subs(t).projection();
        let pt = Vector3::new(pt[0], pt[1], pt[2]);
        println!("{}", (&pt - &vector).norm());
    }
}
