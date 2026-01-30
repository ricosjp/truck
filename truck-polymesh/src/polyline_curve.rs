use crate::*;
use itertools::Itertools;
use std::ops::{Bound, Deref, DerefMut};
use truck_base::cgmath64::control_point::ControlPoint;

impl PolylineCurve<Point2> {
    /// Signed area of area enclosed when endpoints are connected
    /// # Example
    /// ```
    /// use truck_polymesh::*;
    /// use std::f64::consts::PI;
    /// let mut hexagon = (0..6).map(|i| {
    ///         let t = PI / 3.0 * i as f64;
    ///         Point2::new(f64::cos(t), f64::sin(t))
    ///     })
    ///     .collect::<PolylineCurve<_>>();
    /// assert_near!(hexagon.area(), 1.5 * f64::sqrt(3.0));
    /// hexagon.invert();
    /// assert_near!(hexagon.area(), -1.5 * f64::sqrt(3.0));
    /// ```
    #[inline(always)]
    pub fn area(&self) -> f64 {
        let sum = |sum, (p, q): (&Point2, &Point2)| sum + (q.x + p.x) * (q.y - p.y);
        self.iter().circular_tuple_windows().fold(0.0, sum) / 2.0
    }

    /// whether `c` is included in enclosed domain when endpoints are connected
    /// # Example
    /// ```
    /// use truck_polymesh::*;
    /// use std::f64::consts::PI;
    /// let hexagon = (0..6).map(|i| {
    ///         let t = PI / 3.0 * i as f64;
    ///         Point2::new(f64::cos(t), f64::sin(t))
    ///     })
    ///     .collect::<PolylineCurve<_>>();
    /// let p0 = Point2::new(0.9, 0.0);
    /// assert!(hexagon.include(p0));
    /// let p1 = Point2::new(0.0, 1.0);
    /// assert!(!hexagon.include(p1));
    /// ```
    pub fn include(&self, c: Point2) -> bool {
        if self.iter().any(|p| (*p - c).so_small()) {
            return true;
        }
        let t = 2.0 * std::f64::consts::PI * HashGen::hash1(c);
        let r = Vector2::new(f64::cos(t), f64::sin(t));
        self.iter()
            .circular_tuple_windows()
            .try_fold(0_i32, move |counter, (p0, p1)| {
                let a = p0 - c;
                let b = p1 - c;
                let s0 = r.x * a.y - r.y * a.x; // v times a
                let s1 = r.x * b.y - r.y * b.x; // v times b
                let s2 = a.x * b.y - a.y * b.x; // a times b
                let x = s2 / (s1 - s0);
                if x.so_small() && s0 * s1 < 0.0 {
                    None
                } else if x > 0.0 && s0 <= 0.0 && s1 > 0.0 {
                    Some(counter + 1)
                } else if x > 0.0 && s0 >= 0.0 && s1 < 0.0 {
                    Some(counter - 1)
                } else {
                    Some(counter)
                }
            })
            .map(|counter| counter > 0)
            .unwrap_or(true)
    }
}

/// Calculate the area of a region bounded by multiple polylines
/// # Example
/// ```
/// use truck_polymesh::*;
/// use std::f64::consts::PI;
/// // the outer boundary is counter-clockwise
/// let large_hexagon = (0..6).map(|i| {
///         let t = PI / 3.0 * i as f64;
///         Point2::new(2.0 * f64::cos(t), 2.0 * f64::sin(t))
///     })
///     .collect::<PolylineCurve<_>>();
/// // the inner boundary is clockwise
/// let small_hexagon = (0..6).map(|i| {
///         let t = PI / 3.0 * i as f64;
///         Point2::new(f64::sin(t), f64::cos(t))
///     })
///     .collect::<PolylineCurve<_>>();
/// let boundaries = [large_hexagon, small_hexagon];
/// assert_near!(polyline_curve::area(&boundaries), 4.5 * f64::sqrt(3.0));
/// ```
pub fn area<'a>(boundaries: impl IntoIterator<Item = &'a PolylineCurve<Point2>>) -> f64 {
    boundaries.into_iter().map(|poly| poly.area()).sum::<f64>()
}

/// whether `c` is included in enclosed multiple domains when endpoints of each polyline are connected
/// # Example
/// ```
/// use truck_polymesh::*;
/// use std::f64::consts::PI;
/// // the outer boundary is counter-clockwise
/// let large_hexagon = (0..6).map(|i| {
///         let t = PI / 3.0 * i as f64;
///         Point2::new(2.0 * f64::cos(t), 2.0 * f64::sin(t))
///     })
///     .collect::<PolylineCurve<_>>();
/// // the inner boundary is clockwise
/// let small_hexagon = (0..6).map(|i| {
///         let t = PI / 3.0 * i as f64;
///         Point2::new(f64::sin(t), f64::cos(t))
///     })
///     .collect::<PolylineCurve<_>>();
/// let boundaries = [large_hexagon, small_hexagon];
/// let p0 = Point2::new(1.5, 0.0);
/// assert!(polyline_curve::include(&boundaries, p0));
/// let p1 = Point2::new(0.0, 0.0);
/// assert!(!polyline_curve::include(&boundaries, p1));
/// ```
pub fn include<'a>(
    boundaries: impl IntoIterator<Item = &'a PolylineCurve<Point2>>,
    c: Point2,
) -> bool {
    let t = 2.0 * std::f64::consts::PI * HashGen::hash1(c);
    let r = Vector2::new(f64::cos(t), f64::sin(t));
    boundaries
        .into_iter()
        .flat_map(|boundary| boundary.iter().circular_tuple_windows())
        .try_fold(0_i32, move |counter, (p0, p1)| {
            if (*p0 - c).so_small() {
                return None;
            } // Vertex check
            let a = p0 - c;
            let b = p1 - c;
            let s0 = r.x * a.y - r.y * a.x; // v times a
            let s1 = r.x * b.y - r.y * b.x; // v times b
            let s2 = a.x * b.y - a.y * b.x; // a times b
            let x = s2 / (s1 - s0);
            if x.so_small() && s0 * s1 < 0.0 {
                None
            } else if x > 0.0 && s0 <= 0.0 && s1 > 0.0 {
                Some(counter + 1)
            } else if x > 0.0 && s0 >= 0.0 && s1 < 0.0 {
                Some(counter - 1)
            } else {
                Some(counter)
            }
        })
        .map(|counter| counter > 0)
        // If the point is on the boundary, the winding number logic might return false (or be undefined).
        // unique for truck: We treat the boundary as part of the domain (closed set).
        .unwrap_or(true)
}

impl<P> AsRef<Vec<P>> for PolylineCurve<P> {
    #[inline(always)]
    fn as_ref(&self) -> &Vec<P> { &self.0 }
}

impl<P> AsMut<Vec<P>> for PolylineCurve<P> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut Vec<P> { &mut self.0 }
}

impl<P> AsRef<[P]> for PolylineCurve<P> {
    #[inline(always)]
    fn as_ref(&self) -> &[P] { &self.0 }
}

impl<P> AsMut<[P]> for PolylineCurve<P> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut [P] { &mut self.0 }
}

impl<P> Deref for PolylineCurve<P> {
    type Target = Vec<P>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<P> DerefMut for PolylineCurve<P> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl<P> From<Vec<P>> for PolylineCurve<P> {
    #[inline(always)]
    fn from(v: Vec<P>) -> Self { Self(v) }
}

impl<P> From<PolylineCurve<P>> for Vec<P> {
    #[inline(always)]
    fn from(v: PolylineCurve<P>) -> Self { v.0 }
}

impl<P> FromIterator<P> for PolylineCurve<P> {
    #[inline(always)]
    fn from_iter<I: IntoIterator<Item = P>>(iter: I) -> Self { Self(Vec::from_iter(iter)) }
}

impl<P> IntoIterator for PolylineCurve<P> {
    type Item = P;
    type IntoIter = std::vec::IntoIter<P>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl<'a, P> IntoIterator for &'a PolylineCurve<P> {
    type Item = &'a P;
    type IntoIter = std::slice::Iter<'a, P>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}

impl<P: ControlPoint<f64>> ParametricCurve for PolylineCurve<P> {
    type Point = P;
    type Vector = P::Diff;
    #[inline(always)]
    fn der_n(&self, n: usize, t: f64) -> Self::Vector {
        match n {
            0 => self.subs(t).to_vec(),
            1 => self.der(t),
            _ => Self::Vector::zero(),
        }
    }
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
    fn parameter_range(&self) -> ParameterRange {
        (
            Bound::Included(0.0),
            Bound::Included(self.len() as f64 - 1.0),
        )
    }
}

impl<P: ControlPoint<f64>> BoundedCurve for PolylineCurve<P> {}

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
