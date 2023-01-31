use super::*;
use std::f64::consts::PI;

impl Sphere {
    /// Creates a sphere
    #[inline(always)]
    pub const fn new(center: Point3, radius: f64) -> Sphere { Sphere { center, radius } }
    /// Returns the center
    #[inline(always)]
    pub const fn center(&self) -> Point3 { self.center }
    /// Returns the radius
    #[inline(always)]
    pub const fn radius(&self) -> f64 { self.radius }
    /// Returns whether the point `pt` is on sphere
    #[inline(always)]
    pub fn include(&self, pt: Point3) -> bool { self.center.distance(pt).near(&self.radius) }
}

impl ParametricSurface for Sphere {
    type Point = Point3;
    type Vector = Vector3;
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Point3 { self.center() + self.radius * self.normal(u, v) }
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> Vector3 {
        self.radius
            * Vector3::new(
                f64::cos(u) * f64::cos(v),
                f64::cos(u) * f64::sin(v),
                -f64::sin(u),
            )
    }
    #[inline(always)]
    fn vder(&self, u: f64, v: f64) -> Vector3 {
        self.radius * f64::sin(u) * Vector3::new(-f64::sin(v), f64::cos(v), 0.0)
    }
    #[inline(always)]
    fn uuder(&self, u: f64, v: f64) -> Vector3 { -self.radius * self.normal(u, v) }
    #[inline(always)]
    fn uvder(&self, u: f64, v: f64) -> Vector3 {
        self.radius * f64::cos(u) * Vector3::new(-f64::sin(v), f64::cos(v), 0.0)
    }
    #[inline(always)]
    fn vvder(&self, u: f64, v: f64) -> Vector3 {
        -self.radius * f64::sin(u) * Vector3::new(f64::cos(v), f64::sin(v), 0.0)
    }
    #[inline(always)]
    fn v_period(&self) -> Option<f64> { Some(2.0 * PI) }
}

impl ParametricSurface3D for Sphere {
    #[inline(always)]
    fn normal(&self, u: f64, v: f64) -> Vector3 {
        Vector3::new(
            f64::sin(u) * f64::cos(v),
            f64::sin(u) * f64::sin(v),
            f64::cos(u),
        )
    }
}

#[test]
fn sphere_derivation_test() {
    let center = Point3::new(1.0, 2.0, 3.0);
    let radius = 4.56;
    let sphere = Sphere::new(center, radius);
    const N: usize = 100;
    for i in 0..N {
        for j in 0..N {
            let u = PI * i as f64 / N as f64;
            let v = 2.0 * PI * j as f64 / N as f64;
            let normal = sphere.normal(u, v);
            assert!(normal.dot(sphere.uder(u, v)).so_small());
            assert!(normal.dot(sphere.vder(u, v)).so_small());
        }
    }
}

impl BoundedSurface for Sphere {
    #[inline(always)]
    fn parameter_range(&self) -> ((f64, f64), (f64, f64)) { ((0.0, PI), (0.0, 2.0 * PI)) }
}

impl IncludeCurve<BSplineCurve<Point3>> for Sphere {
    #[inline(always)]
    fn include(&self, curve: &BSplineCurve<Point3>) -> bool {
        curve.is_const() && self.include(curve.front())
    }
}

impl IncludeCurve<NURBSCurve<Vector4>> for Sphere {
    fn include(&self, curve: &NURBSCurve<Vector4>) -> bool {
        let (knots, _) = curve.knot_vec().to_single_multi();
        let degree = curve.degree() * 2;
        knots
            .windows(2)
            .flat_map(move |window| (1..degree).map(move |i| (window, i)))
            .all(move |(window, i)| {
                let t = i as f64 / degree as f64;
                let t = window[0] * (1.0 - t) + window[1] * t;
                self.include(curve.subs(t))
            })
    }
}

impl ParameterDivision2D for Sphere {
    #[inline(always)]
    fn parameter_division(
        &self,
        (urange, vrange): ((f64, f64), (f64, f64)),
        tol: f64,
    ) -> (Vec<f64>, Vec<f64>) {
        nonpositive_tolerance!(tol);
        assert!(
            tol < self.radius,
            "Tolerance is larger than the radius of sphere."
        );
        let acos = f64::acos(1.0 - tol / self.radius);
        let u_div: usize = 1 + ((urange.1 - urange.0) / acos).floor() as usize;
        let v_div: usize = 1 + ((vrange.1 - vrange.0) / acos).floor() as usize;
        (
            (0..=u_div)
                .map(|i| urange.0 + (urange.1 - urange.0) * i as f64 / u_div as f64)
                .collect(),
            (0..=v_div)
                .map(|j| vrange.0 + (vrange.1 - vrange.0) * j as f64 / v_div as f64)
                .collect(),
        )
    }
}

impl SearchParameter<D2> for Sphere {
    type Point = Point3;
    #[inline(always)]
    fn search_parameter<H: Into<SPHint2D>>(
        &self,
        point: Point3,
        hint: H,
        _: usize,
    ) -> Option<(f64, f64)> {
        let radius = point - self.center;
        if (self.radius * self.radius).near(&radius.magnitude2()) {
            let radius = radius.normalize();
            let u = f64::acos(radius[2]);
            let sinu = f64::sqrt(1.0 - radius[2] * radius[2]);
            let cosv = f64::clamp(radius[0] / sinu, -1.0, 1.0);
            let v = if sinu.so_small() {
                match hint.into() {
                    SPHint2D::Parameter(_, hint) => hint,
                    _ => 0.0,
                }
            } else if radius[1] > 0.0 {
                f64::acos(cosv)
            } else {
                2.0 * PI - f64::acos(cosv)
            };
            Some((u, v))
        } else {
            None
        }
    }
}

impl SearchNearestParameter<D2> for Sphere {
    type Point = Point3;
    #[inline(always)]
    fn search_nearest_parameter<H: Into<SPHint2D>>(
        &self,
        point: Point3,
        _: H,
        _: usize,
    ) -> Option<(f64, f64)> {
        let radius = (point - self.center).normalize();
        let u = f64::acos(radius[2]);
        let sinu = f64::sqrt(1.0 - radius[2] * radius[2]);
        let cosv = radius[0] / sinu;
        let v = if radius[1] > 0.0 {
            f64::acos(cosv)
        } else {
            2.0 * PI - f64::acos(cosv)
        };
        Some((u, v))
    }
}

#[cfg(test)]
fn exec_search_parameter_test() {
    let center = Point3::new(
        100.0 * rand::random::<f64>() - 50.0,
        100.0 * rand::random::<f64>() - 50.0,
        100.0 * rand::random::<f64>() - 50.0,
    );
    let radius = 100.0 * rand::random::<f64>();
    let sphere = Sphere::new(center, radius);
    let u = PI * rand::random::<f64>();
    let v = 2.0 * PI * rand::random::<f64>();
    let pt = sphere.subs(u, v);
    let (u0, v0) = sphere.search_parameter(pt, None, 100).unwrap();
    assert_near!(Vector2::new(u, v), Vector2::new(u0, v0));
    let pt = pt
        + Vector3::new(
            (0.1 * rand::random::<f64>() + 0.01) * f64::signum(rand::random::<f64>() - 0.5),
            (0.1 * rand::random::<f64>() + 0.01) * f64::signum(rand::random::<f64>() - 0.5),
            (0.1 * rand::random::<f64>() + 0.01) * f64::signum(rand::random::<f64>() - 0.5),
        );
    assert!(sphere.search_parameter(pt, None, 100).is_none());
    let (u, v) = sphere.search_nearest_parameter(pt, None, 100).unwrap();
    assert_near!(
        sphere.subs(u, v),
        center + (pt - center).normalize() * radius
    );
}

#[test]
fn search_parameter_test() { (0..10).for_each(|_| exec_search_parameter_test()) }
