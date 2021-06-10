use truck_base::{assert_near, cgmath64::*, tolerance::*};
use truck_geotrait::*;
use truck_topology::*;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Segment {
    ends: (Point3, Point3),
    range: (f64, f64),
}

impl Segment {
    fn new(p: Point3, q: Point3) -> Segment {
        Segment {
            ends: (p, q),
            range: (0.0, 1.0),
        }
    }
}

impl ParametricCurve for Segment {
    type Point = Point3;
    type Vector = Vector3;
    #[inline(always)]
    fn subs(&self, t: f64) -> Point3 {
        self.ends.0
            + (self.ends.1 - self.ends.0) * (t - self.range.0) / (self.range.1 - self.range.0)
    }
    #[inline(always)]
    fn der(&self, _: f64) -> Vector3 { (self.ends.1 - self.ends.0) / (self.range.1 - self.range.0) }
    #[inline(always)]
    fn der2(&self, _: f64) -> Vector3 { Vector3::zero() }
    #[inline(always)]
    fn parameter_range(&self) -> (f64, f64) { self.range }
}

impl ParameterTransform for Segment {
    #[inline(always)]
    fn parameter_transform(&mut self, scalar: f64, r#move: f64) -> &mut Self {
        self.range.0 = self.range.0 * scalar + r#move;
        self.range.1 = self.range.1 * scalar + r#move;
        self
    }
}

impl Cut for Segment {
    #[inline(always)]
    fn cut(&mut self, t: f64) -> Self {
        let p = self.subs(t);
        let res = Segment {
            ends: (p, self.ends.1),
            range: (t, self.range.1),
        };
        *self = Segment {
            ends: (self.ends.0, p),
            range: (self.range.0, t),
        };
        res
    }
}

impl Concat<Segment> for Segment {
    type Output = Segment;
    #[inline(always)]
    fn try_concat(&self, rhs: &Self) -> std::result::Result<Self, ConcatError<Point3>> {
        if !self.range.1.near(&rhs.range.0) {
            Err(ConcatError::DisconnectedParameters(
                self.range.1,
                rhs.range.0,
            ))
        } else if !self.ends.1.near(&rhs.ends.0) {
            Err(ConcatError::DisconnectedPoints(self.ends.1, rhs.ends.0))
        // by this branch, this is not correctly implementation of concat
        } else if !(self.ends.1 - self.ends.0)
            .cross(rhs.ends.1 - rhs.ends.0)
            .so_small()
        {
            Err(ConcatError::DisconnectedPoints(self.ends.1, rhs.ends.0))
        } else {
            Ok(Segment {
                ends: (self.ends.0, rhs.ends.1),
                range: (self.range.0, rhs.range.1),
            })
        }
    }
}

impl SearchParameter for Segment {
    type Point = Point3;
    type Parameter = f64;
    fn search_parameter(&self, point: Point3, _: Option<f64>, _: usize) -> Option<f64> {
        let p = point - self.ends.0;
        let r = self.ends.1 - self.ends.0;
        let t = p.dot(r) / r.dot(r);
        if (p - t * r).so_small() && -TOLERANCE < t && t < 1.0 + TOLERANCE {
            Some(self.range.0 + (self.range.1 - self.range.0) * t)
        } else {
            None
        }
    }
}

impl Invertible for Segment {
    #[inline(always)]
    fn invert(&mut self) {
        *self = Segment {
            ends: (self.ends.1, self.ends.0),
            range: self.range,
        };
    }
}

#[test]
fn segment_test() {
    let seg = Segment::new(Point3::new(1.2, 2.3, 3.4), Point3::new(2.4, 3.5, 4.6));
    assert_near!(seg.subs(0.5), Point3::new(1.8, 2.9, 4.0));
    assert_near!(seg.der(0.5), Vector3::new(1.2, 1.2, 1.2));
    assert_eq!(seg.der2(0.5), Vector3::zero());
    parameter_transform_random_test(&seg, 100);
    cut_random_test(&seg, 100);

    let mut seg0 = Segment::new(Point3::new(2.4, 3.5, 4.6), Point3::new(3.6, 4.7, 5.8));
    assert_eq!(
        seg.try_concat(&seg0).unwrap_err(),
        ConcatError::DisconnectedParameters(1.0, 0.0)
    );
    seg0.parameter_transform(1.0, 1.0);
    concat_random_test(&seg, &seg0, 100);
    seg0.ends.0 += Vector3::new(1.0, 0.0, 0.0);
    seg0.ends.1 += Vector3::new(1.0, 0.0, 0.0);
    assert_eq!(
        seg.try_concat(&seg0).unwrap_err(),
        ConcatError::DisconnectedPoints(Point3::new(2.4, 3.5, 4.6), Point3::new(3.4, 3.5, 4.6))
    );

    let pt = seg.subs(0.324);
    let a = seg.search_parameter(pt, None, 0).unwrap();
    assert_near!(a, 0.324);
    assert!(seg
        .search_parameter(pt + Vector3::new(0.1, 0.0, -0.4), None, 0)
        .is_none());
}

#[test]
fn solid_cut_edge() {
    let p = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(0.0, 0.5, 0.0),
        Point3::new(0.5, 0.5, 0.5),
    ];
    let v = Vertex::news(&p);
    let edge = vec![
        Edge::new(&v[0], &v[1], Segment::new(p[0], p[1])),
        Edge::new(&v[0], &v[2], Segment::new(p[0], p[2])),
        Edge::new(&v[0], &v[3], Segment::new(p[0], p[3])),
        Edge::new(&v[1], &v[2], Segment::new(p[1], p[2])),
        Edge::new(&v[1], &v[3], Segment::new(p[1], p[3])),
        Edge::new(&v[2], &v[3], Segment::new(p[2], p[3])),
    ];
    let shell: Shell<_, _, _> = vec![
        Face::new(
            vec![vec![edge[0].clone(), edge[3].clone(), edge[1].inverse()].into()],
            (),
        ),
        Face::new(
            vec![vec![edge[1].clone(), edge[5].clone(), edge[2].inverse()].into()],
            (),
        ),
        Face::new(
            vec![vec![edge[2].clone(), edge[4].inverse(), edge[0].inverse()].into()],
            (),
        ),
        Face::new(
            vec![vec![edge[3].clone(), edge[5].clone(), edge[4].inverse()].into()],
            (),
        )
        .inverse(),
    ]
    .into();
    let mut tri = Solid::new(vec![shell]);
    assert!(tri.cut_edge(edge[1].id(), &v[4]));
    let count = tri.edge_iter().count();
    assert_eq!(count, 14);

    let new_shells: Vec<_> = tri
        .boundaries()
        .iter()
        .map(|shell| {
            shell
                .into_iter()
                .map(|face| {
                    let wires = face.boundaries().into_iter().map(|wire| wire).collect();
                    Face::new(wires, ())
                })
                .collect()
        })
        .collect();
    Solid::new(new_shells);

    assert!(tri.remove_vertex_by_concat_edges(v[4].id()));
    let count = tri.edge_iter().count();
    assert_eq!(count, 12);
}
