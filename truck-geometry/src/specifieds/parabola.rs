use super::*;

impl<P> UnitParabola<P> {
    /// constructor
    #[inline]
    pub const fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl ParametricCurve for UnitParabola<Point2> {
    type Point = Point2;
    type Vector = Vector2;
    #[inline]
    fn subs(&self, t: f64) -> Self::Point {
        Point2::new(t * t, 2.0 * t)
    }
    #[inline]
    fn der(&self, t: f64) -> Self::Vector {
        Vector2::new(2.0 * t, 2.0)
    }
    #[inline]
    fn der2(&self, _: f64) -> Self::Vector {
        Vector2::new(2.0, 0.0)
    }
}

impl ParametricCurve for UnitParabola<Point3> {
    type Point = Point3;
    type Vector = Vector3;
    #[inline]
    fn subs(&self, t: f64) -> Self::Point {
        Point3::new(t * t, 2.0 * t, 0.0)
    }
    #[inline]
    fn der(&self, t: f64) -> Self::Vector {
        Vector3::new(2.0 * t, 2.0, 0.0)
    }
    #[inline]
    fn der2(&self, _: f64) -> Self::Vector {
        Vector3::new(2.0, 0.0, 0.0)
    }
}

impl<P> ParameterDivision1D for UnitParabola<P>
where
    UnitParabola<P>: ParametricCurve<Point = P>,
    P: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + HashGen<f64>,
{
    type Point = P;
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<P>) {
        algo::curve::parameter_division(self, range, tol)
    }
}

impl SearchNearestParameter for UnitParabola<Point2> {
    type Point = Point2;
    type Parameter = f64;
    #[inline]
    fn search_nearest_parameter(&self, pt: Point2, _: Option<f64>, _: usize) -> Option<f64> {
        let p = 2.0 - pt.x;
        let q = -pt.y;
        solve_cubic(p, q).into_iter().min_by(|s, t| {
            pt.distance2(self.subs(*s))
                .partial_cmp(&pt.distance2(self.subs(*t)))
                .unwrap()
        })
    }
}

impl SearchNearestParameter for UnitParabola<Point3> {
    type Point = Point3;
    type Parameter = f64;
    #[inline]
    fn search_nearest_parameter(
        &self,
        pt: Point3,
        _hint: Option<f64>,
        _trials: usize,
    ) -> Option<f64> {
        UnitParabola::<Point2>::new().search_nearest_parameter(
            Point2::new(pt.x, pt.y),
            _hint,
            _trials,
        )
    }
}

impl SearchParameter for UnitParabola<Point2> {
    type Point = Point2;
    type Parameter = f64;
    #[inline]
    fn search_parameter(&self, pt: Point2, _: Option<f64>, _: usize) -> Option<f64> {
        let t = pt.y / 2.0;
        let pt0 = self.subs(t);
        match pt.near(&pt0) {
            true => Some(t),
            false => None,
        }
    }
}

impl SearchParameter for UnitParabola<Point3> {
    type Point = Point3;
    type Parameter = f64;
    #[inline]
    fn search_parameter(&self, pt: Point3, _hint: Option<f64>, _trials: usize) -> Option<f64> {
        match pt.z.so_small() {
            true => UnitParabola::<Point2>::new().search_parameter(
                Point2::new(pt.x, pt.y),
                _hint,
                _trials,
            ),
            false => None,
        }
    }
}

/// solve equation: t^3 + p t + q = 0.
fn solve_cubic(p: f64, q: f64) -> Vec<f64> {
    use num::complex::Complex;
    const OMEGA: Complex<f64> = Complex::new(-0.5, 0.86602540378);
    const OMEGA2: Complex<f64> = Complex::new(-0.5, -0.86602540378);
    let p_3 = p / 3.0;
    let q_2 = q / 2.0;
    let alpha2 = q_2 * q_2 + p_3 * p_3 * p_3;
    let (x, y) = match alpha2 > -TOLERANCE {
        true => {
            let alpha = f64::sqrt(f64::max(alpha2, 0.0));
            let tmpx = -q_2 - alpha;
            let tmpy = -q_2 + alpha;
            (
                Complex::new(f64::signum(tmpx) * f64::powf(f64::abs(tmpx), 1.0 / 3.0), 0.0),
                Complex::new(f64::signum(tmpy) * f64::powf(f64::abs(tmpy), 1.0 / 3.0), 0.0),
            )
        },
        false => {
            let alphai = f64::sqrt(-alpha2);
            (
                Complex::powf(Complex::new(-q_2, alphai), 1.0 / 3.0),
                Complex::powf(Complex::new(-q_2, -alphai), 1.0 / 3.0),
            )
        },
    };
    [x + y, OMEGA * x + OMEGA2 * y, OMEGA2 * x + OMEGA * y]
        .iter()
        .filter_map(|z| {println!("x={x:?} y={y:?}\nz={z:?} {:?}\n", z * z * z + p * z + q); match z.im.so_small() {
            true => Some(z.re),
            false => None,
        }})
        .collect()
}

#[test]
fn solve_cubic_test() {
    // example
    let res = solve_cubic(-7.0, -6.0);
    assert_eq!(res.len(), 3);
    let mut ans = vec![-1.0, -2.0, 3.0];
    res.into_iter().for_each(|x| {
        let idx = ans
            .iter()
            .enumerate()
            .find_map(|(i, y)| match x.near(y) {
                true => Some(i),
                false => None,
            })
            .unwrap();
        ans.swap_remove(idx);
    });

    // random
    (0..50).for_each(|_| {
        let p = 100.0 * rand::random::<f64>() - 50.0;
        let q = 100.0 * rand::random::<f64>() - 50.0;
        let vec = solve_cubic(p, q);
        assert!(!vec.is_empty(), "{p} {q}");
        vec.into_iter().for_each(|t| assert!((t * t * t + p * t + q).so_small()));
    })
}

#[test]
fn snp_test() {
    let curve = UnitParabola::<Point2>::new();

    let p = Point2::new(-3.0, 0.0);
    assert_near!(curve.search_nearest_parameter(p, None, 0).unwrap(), 0.0);
    let p = Point2::new(-3.0, 6.0);
    assert_near!(curve.search_nearest_parameter(p, None, 0).unwrap(), 1.0);
    let p = Point2::new(1.5, 1.5);
    assert_near!(curve.search_nearest_parameter(p, None, 0).unwrap(), 1.0);
}

#[test]
fn sp_test() {
    let curve = UnitParabola::<Point2>::new();

    let p = Point2::new(4.0, -4.0);
    assert_near!(curve.search_parameter(p, None, 0).unwrap(), -2.0);
    let p = Point2::new(-3.0, 6.0);
    assert!(curve.search_parameter(p, None, 0).is_none());
}
