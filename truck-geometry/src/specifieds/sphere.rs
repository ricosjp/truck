use super::*;
use std::f64::consts::PI;

impl Sphere {
    /// Creates a sphere
    #[inline(always)]
    pub fn new(center: Point3, radius: f64) -> Sphere { Sphere { center, radius } }
    /// Returns the center
    #[inline(always)]
    pub fn center(&self) -> Point3 { self.center }
    /// Returns the radius
    #[inline(always)]
    pub fn radius(&self) -> f64 { self.radius }
    /// Returns whether the point `pt` is on sphere
    #[inline(always)]
    pub fn include(&self, pt: Point3) -> bool { self.center.distance(pt).near(&self.radius) }
}

impl ParametricSurface for Sphere {
    type Point = Point3;
    type Vector = Vector3;
    #[inline(always)]
    fn normal(&self, u: f64, v: f64) -> Vector3 {
        Vector3::new(
            f64::sin(u) * f64::cos(v),
            f64::sin(u) * f64::sin(v),
            f64::cos(u),
        )
    }
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
    fn uuder(&self, u: f64, v: f64) -> Vector3 {
        -self.radius * self.normal(u, v)
    }
    #[inline(always)]
    fn uvder(&self, u: f64, v: f64) -> Vector3 {
        self.radius * f64::cos(u) * Vector3::new(-f64::sin(v), f64::cos(v), 0.0)
    }
    #[inline(always)]
    fn vvder(&self, u: f64, v: f64) -> Vector3 {
        -self.radius * f64::sin(u) * Vector3::new(f64::cos(v), f64::sin(v), 0.0)
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

impl IncludeCurve<BSplineCurve<Vector3>> for Sphere {
    #[inline(always)]
    fn include(&self, curve: &BSplineCurve<Vector3>) -> bool {
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
    fn parameter_division(&self, tol: f64) -> (Vec<f64>, Vec<f64>) {
        let acos = f64::acos(1.0 - tol / self.radius);
        let u_div: usize = 1 + (PI / acos).floor() as usize;
        let v_div: usize = 1 + (2.0 * PI / acos).floor() as usize;
        (
            (0..=u_div).map(|i| PI * i as f64 / u_div as f64).collect(),
            (0..=v_div).map(|j| PI * j as f64 / v_div as f64).collect(),
        )
    }
}
