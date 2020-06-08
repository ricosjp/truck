use crate::{Builder, Result, Transform};
use crate::curve_element::CurveElement;
use crate::elements::{TopologicalElement, GeometricalElement};
use crate::transformed::Transformed;
use crate::tsweep::TSweep;
use geometry::*;
use topology::*;

impl<'a> Builder<'a> {
    #[inline(always)]
    pub fn create_by_geometry<T: GeometricalElement>(&mut self, geom: T) -> Result<T::Topology> {
        Ok(T::Topology::create_by_geometry(geom, &mut self.director))
    }

    pub fn vertex(&mut self, coord: Vector3) -> Result<Vertex> {
        self.create_by_geometry(Vector::new3(coord[0], coord[1], coord[2]))
    }

    pub fn line(&mut self, vertex0: Vertex, vertex1: Vertex) -> Result<Edge> {
        let knot_vec = KnotVec::bezier_knot(1);
        let pt0 = self.director.get_geometry(&vertex0)?.clone();
        let pt1 = self.director.get_geometry(&vertex1)?.clone();
        let line = BSplineCurve::try_new(knot_vec, vec![pt0, pt1])?;
        let edge = Edge::new(vertex0, vertex1);
        self.director.insert(&edge, line);
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
        let tmp = self.director.get_geometry(&vertex0)?.projection();
        let pt0 = Vector3::new(tmp[0], tmp[1], tmp[2]);
        let tmp = self.director.get_geometry(&vertex1)?.projection();
        let pt1 = Vector3::new(tmp[0], tmp[1], tmp[2]);

        // circum_center
        let org = circum_center(&pt0, &pt1, transit);
        let trsf0 = Transform::translate(&org);

        // scalar component
        let scalar = (&pt0 - &org).norm();
        let trsf1 = Transform::scale(&Vector3::new(scalar, scalar, scalar));

        // the cosine of half of the angle of arc
        let cos = -(&pt0 - transit).cos_angle(&(&pt1 - transit));
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

        let edge = Edge::new(vertex0, vertex1);
        self.director.insert(&edge, curve);
        Ok(edge)
    }

    pub fn plane(&mut self, wire: Wire) -> Result<Face> {
        if wire.len() < 2 {
            Ok(Face::try_new(wire)?)
        } else if wire.len() == 2 {
            let mut curve0 = self.director.get_oriented_curve(&wire[0])?;
            let t = curve0.knot_vec()[0] + curve0.knot_vec().range_length() / 2.0;
            let curve1 = curve0.cut(t);
            let mut curve2 = self.director.get_oriented_curve(&wire[1])?;
            let t = curve2.knot_vec()[0] + curve2.knot_vec().range_length() / 2.0;
            let curve3 = curve2.cut(t);
            let surface = BSplineSurface::by_boundary(curve0, curve1, curve2, curve3);
            let face = Face::try_new(wire)?;
            self.director.insert(&face, surface);
            Ok(face)
        } else if wire.len() == 3 {
            let curve0 = self.director.get_oriented_curve(&wire[0])?;
            let curve1 = self.director.get_oriented_curve(&wire[1])?;
            let curve3 = self.director.get_oriented_curve(&wire[2])?;
            let curve2 = BSplineCurve::new(
                KnotVec::bezier_knot(1),
                vec![curve1.end_points().1, curve3.end_points().0],
            );
            let surface = BSplineSurface::by_boundary(curve0, curve1, curve2, curve3);
            let face = Face::try_new(wire)?;
            self.director.insert(&face, surface);
            Ok(face)
        } else {
            let wires = split_wire(&wire);
            let curve0 = self.director.bspline_by_wire(&wires[0])?;
            let curve1 = self.director.bspline_by_wire(&wires[1])?;
            let curve2 = self.director.bspline_by_wire(&wires[2])?;
            let curve3 = self.director.bspline_by_wire(&wires[3])?;
            let surface = BSplineSurface::by_boundary(curve0, curve1, curve2, curve3);
            let face = Face::try_new(wire)?;
            self.director.insert(&face, surface);
            Ok(face)
        }
    }

    pub fn homotopy<T: CurveElement>(&mut self, elem0: &T, elem1: &T) -> Result<Shell> {
        elem0.homotopy(elem1, &mut self.director)
    }
    
    pub fn create_copy<T: Transformed>(&mut self, elem: &T) -> Result<T> { elem.copy(self) }

    pub fn create_transformed<T: Transformed>(&mut self, elem: &T, trsf: &Transform) -> Result<T> {
        elem.transformed(trsf, self)
    }
    
    pub fn create_translated<T: Transformed>(&mut self, elem: &T, vector: &Vector3) -> Result<T> {
        elem.translated(vector, self)
    }

    pub fn create_rotated<T: Transformed>(
        &mut self,
        elem: &T,
        origin: &Vector3,
        axis: &Vector3,
        angle: f64,
    ) -> Result<T>
    {
        elem.rotated(origin, axis, angle, self)
    }

    pub fn create_scaled<T: Transformed>(
        &mut self,
        elem: &T,
        origin: &Vector3,
        scalar: &Vector3,
    ) -> Result<T>
    {
        elem.scaled(origin, scalar, self)
    }

    pub fn tsweep<T: TSweep>(&mut self, elem: &T, vector: &Vector3) -> Result<T::Output> {
        elem.tsweep(vector, self)
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
    let mut pt0 = Vector::new3(0.17_f64.cos(), 0.17_f64.sin(), 0.0);
    let mut pt1 = Vector::new3(1.64_f64.cos(), 1.64_f64.sin(), 0.0);
    let vector = Vector3::new(-2, 5, 10);
    let mut axis = Vector3::new(2, 5, 4);
    axis /= axis.norm();
    let trsf = Transform::rotate(&axis, 0.56) * Transform::translate(&vector);
    let mut transit = Vector::new3(1.12_f64.cos(), 1.12_f64.sin(), 0.0);
    pt0 *= &trsf.0;
    pt1 *= &trsf.0;
    transit *= &trsf.0;

    let mut director = crate::Director::new();
    let edge = director.building(|builder| {
        let vertex0 = builder.create_by_geometry(pt0.clone()).unwrap();
        let vertex1 = builder.create_by_geometry(pt1.clone()).unwrap();
        let transit = Vector3::new(transit[0], transit[1], transit[2]);

        builder.circle_arc(vertex0, vertex1, &transit).unwrap()
    });

    let curve = director.get_geometry(&edge).unwrap();
    println!("{:?}", pt0.projection() - curve.subs(0.0).projection());
    println!("{:?}", pt1.projection() - curve.subs(1.0).projection());
    for i in 0..N {
        let t = (i as f64) / (N as f64);
        let pt = curve.subs(t).projection();
        let pt = Vector3::new(pt[0], pt[1], pt[2]);
        println!("{}", (&pt - &vector).norm());
    }
}
