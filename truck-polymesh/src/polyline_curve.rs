use crate::*;
use std::ops::{Deref, DerefMut, Bound};
use truck_base::cgmath64::control_point::ControlPoint;

impl<P> Deref for PolylineCurve<P> {
    type Target = Vec<P>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<P> DerefMut for PolylineCurve<P> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl<P> From<Vec<P>> for PolylineCurve<P> {
    fn from(v: Vec<P>) -> Self { Self(v) }
}

impl<P> From<PolylineCurve<P>> for Vec<P> {
    fn from(v: PolylineCurve<P>) -> Self { v.0 }
}

impl<P> FromIterator<P> for PolylineCurve<P> {
    fn from_iter<I: IntoIterator<Item = P>>(iter: I) -> Self { Self(Vec::from_iter(iter)) }
}

impl<P: ControlPoint<f64>> ParametricCurve for PolylineCurve<P> {
    type Point = P;
    type Vector = P::Diff;
    #[inline(always)]
    fn subs(&self, t: f64) -> P {
        if self.is_empty() {
            P::origin()
        } else if t <= 0.0 {
            self[0]
        } else if t + 1.0 >= self.len() as f64 {
            self[self.len() - 1]
        } else {
            let n = t as usize;
            let t = t - n as f64;
            self[n] + (self[n + 1] - self[n]) * t
        }
    }
    #[inline(always)]
    fn der(&self, t: f64) -> P::Diff {
        if t < 0.0 || (self.len() as f64) < t + 1.0 {
            P::Diff::zero()
        } else {
            let n = t as usize;
            if n + 1 == self.len() {
                self[n] - self[n - 1]
            } else {
                self[n + 1] - self[n]
            }
        }
    }
    #[inline(always)]
    fn der2(&self, _: f64) -> P::Diff { P::Diff::zero() }
    #[inline(always)]
    fn parameter_range(&self) -> (Bound<f64>, Bound<f64>) { (Bound::Included(0.0), Bound::Included(self.len() as f64 - 1.0)) }
}

impl<P: ControlPoint<f64>> BoundedCurve for PolylineCurve<P> {
}

impl<P: Clone> Invertible for PolylineCurve<P> {
    #[inline(always)]
    fn invert(&mut self) { self.reverse(); }
    #[inline(always)]
    fn inverse(&self) -> Self { Self(self.iter().rev().cloned().collect()) }
}

impl<P: ControlPoint<f64>> Cut for PolylineCurve<P> {
    fn cut(&mut self, t: f64) -> Self {
        if t < 0.0 {
            PolylineCurve(Vec::new())
        } else if t + 1.0 > self.len() as f64 {
            let mut v = Vec::new();
            v.append(&mut self.0);
            PolylineCurve(v)
        } else {
            let n = t as usize;
            if t.near(&(n as f64)) {
                let mut v = Vec::new();
                v.extend(&self[n..]);
                self.truncate(n + 1);
                PolylineCurve(v)
            } else {
                let p = self.subs(t);
                let mut v = vec![p];
                v.extend(&self[(n + 1)..]);
                self.truncate(n + 1);
                self.push(p);
                PolylineCurve(v)
            }
        }
    }
}

impl<P> SearchParameter<D1> for PolylineCurve<P>
where
    P: ControlPoint<f64>,
    P::Diff: InnerSpace<Scalar = f64> + Tolerance,
{
    type Point = P;
    fn search_parameter<H: Into<SPHint1D>>(&self, point: P, _: H, _: usize) -> Option<f64> {
        for (i, p) in self.0.windows(2).enumerate() {
            let a = point - p[0];
            let b = p[1] - p[0];
            let t = f64::clamp(a.dot(b) / b.dot(b), 0.0, 1.0);
            let h = a - b * t;
            if h.so_small() {
                return Some(t + i as f64);
            }
        }
        None
    }
}

impl<P> SearchNearestParameter<D1> for PolylineCurve<P>
where
    P: ControlPoint<f64>,
    P::Diff: InnerSpace<Scalar = f64>,
{
    type Point = P;
    fn search_nearest_parameter<H: Into<SPHint1D>>(&self, point: P, _: H, _: usize) -> Option<f64> {
        let (mut t0, mut dist2) = (0.0, f64::INFINITY);
        for (i, p) in self.0.windows(2).enumerate() {
            let a = point - p[0];
            let b = p[1] - p[0];
            let t = f64::clamp(a.dot(b) / b.dot(b), 0.0, 1.0);
            let h = a - b * t;
            if h.dot(h) < dist2 {
                t0 = t + i as f64;
                dist2 = h.dot(h);
            }
        }
        Some(t0)
    }
}

impl<P: ControlPoint<f64>> ParameterDivision1D for PolylineCurve<P> {
    type Point = P;
    #[inline(always)]
    fn parameter_division(&self, range: (f64, f64), _: f64) -> (Vec<f64>, Vec<P>) {
        let r0 = range.0 as isize + 1;
        let r1 = range.1 as isize;
        let mut res = (vec![range.0], vec![self.subs(range.0)]);
        res.0.extend((r0..=r1).map(|i| i as f64));
        res.1.extend((r0..=r1).map(|i| self[i as usize]));
        res.0.push(range.1);
        res.1.push(self.subs(range.1));
        res
    }
}

impl<P, T> Transformed<T> for PolylineCurve<P>
where
    P: EuclideanSpace,
    T: Transform<P>,
{
    fn transform_by(&mut self, trans: T) {
        self.0
            .iter_mut()
            .for_each(|p| *p = trans.transform_point(*p))
    }
}

#[test]
fn polyline_test() {
    let vec = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(1.0, 1.0, 1.0),
    ];
    let polyline = PolylineCurve(vec);
    truck_base::assert_near!(polyline.subs(2.5), Point3::new(0.0, 0.5, 0.5));

    let mut part0 = polyline.clone();
    let part1 = part0.cut(2.5);
    assert_eq!(part0.len(), 4);
    assert_eq!(part1.len(), 6);
    truck_base::assert_near!(part0.subs(2.5), Point3::new(0.0, 0.75, 0.25));
    truck_base::assert_near!(part1.subs(0.5), Point3::new(0.0, 0.25, 0.75));
    let mut part0 = polyline.clone();
    let part1 = part0.cut(4.0);
    assert_eq!(part0.len(), 5);
    assert_eq!(part1.len(), 4);
    truck_base::assert_near!(part0.subs(2.5), Point3::new(0.0, 0.5, 0.5));
    truck_base::assert_near!(part1.subs(0.5), Point3::new(0.5, 0.5, 1.0));

    let pt = polyline.subs(2.13);
    let t = polyline.search_parameter(pt, None, 1).unwrap();
    truck_base::assert_near!(t, 2.13);

    let pt = Point3::new(2.0, 0.0, 0.0);
    assert!(polyline.search_parameter(pt, None, 1).is_none());
    truck_base::assert_near!(polyline.search_nearest_parameter(pt, None, 1).unwrap(), 1.0);
    let pt = Point3::new(0.5, 0.5, 0.51);
    assert!(polyline.search_parameter(pt, None, 1).is_none());
    let t = polyline.search_nearest_parameter(pt, None, 1).unwrap();
    assert!(polyline.der(t).dot(pt - polyline.subs(t)).so_small());

    let div = polyline.parameter_division((1.5, 6.2), 0.0);
    assert_eq!(div.0, vec![1.5, 2.0, 3.0, 4.0, 5.0, 6.0, 6.2]);
    assert_eq!(div.0.len(), div.1.len());
    truck_base::assert_near!(div.1[0], Point3::new(0.5, 0.5, 0.0));
    truck_base::assert_near!(div.1[1], Point3::new(0.0, 1.0, 0.0));
    truck_base::assert_near!(div.1[2], Point3::new(0.0, 0.0, 1.0));
    truck_base::assert_near!(div.1[3], Point3::new(0.0, 1.0, 1.0));
    truck_base::assert_near!(div.1[4], Point3::new(1.0, 0.0, 1.0));
    truck_base::assert_near!(div.1[5], Point3::new(1.0, 1.0, 0.0));
    truck_base::assert_near!(div.1[6], Point3::new(1.0, 1.0, 0.2));
}
