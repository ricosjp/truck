use super::*;
use std::{f64::consts::PI, ops::Add};

impl Cylinder {
    /// Creates a sphere
    #[inline(always)]
    pub fn new(position: Point3, direction: Vector3, radius: f64) -> Cylinder { Cylinder { position, direction, radius } }
    /// Returns the center
    #[inline(always)]
    pub fn center(&self) -> Point3 { self.position }
    /// Returns the radius
    #[inline(always)]
    pub fn radius(&self) -> f64 { self.radius }
    /// Returns whether the point `pt` is on sphere
    #[inline(always)]
    pub fn include(&self, pt: Point3) -> bool { self.position.distance(pt).near(&self.radius) }
}

impl Invertible for Cylinder {
    fn invert(&mut self) {
        self.direction = -self.direction;
    }
}

impl ParametricSurface for Cylinder {
    type Point = Point3;
    type Vector = Vector3;
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Point3 { self.center().add(self.normal(u, v) * self.radius) }
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> Vector3 {
        self.radius
            * Vector3::new(
                f64::sin(u),
                -f64::cos(u),
                v
            )
    }
    #[inline(always)]
    fn vder(&self, u: f64, _v: f64) -> Vector3 {
        self.radius * Vector3::new(
            f64::cos(u),
            f64::sin(u),
            1.0)
    }
    #[inline(always)]
    fn uuder(&self, u: f64, v: f64) -> Vector3 { -self.radius * self.normal(u, v) }
    #[inline(always)]
    fn uvder(&self, u: f64, _v: f64) -> Vector3 {
        self.radius * self.radius
            * Vector3::new(
                f64::sin(u),
                -f64::cos(u),
                1.0
            )
    }
    #[inline(always)]
    fn vvder(&self, u: f64, _v: f64) -> Vector3 {
        self.radius * Vector3::new(
            f64::cos(u),
            f64::sin(u),
            0.0)
    }
}

impl ParametricSurface3D for Cylinder {
    #[inline(always)]
    fn normal(&self, u: f64, v: f64) -> Vector3 {
        Vector3::new(
            f64::cos(u),
            f64::sin(u),
            v
        )
    }
}

impl BoundedSurface for Cylinder {
    #[inline(always)]
    fn parameter_range(&self) -> ((f64, f64), (f64, f64)) { ((0.0, PI), (0.0, 2.0 * PI)) }
}

impl IncludeCurve<BSplineCurve<Point3>> for Cylinder {
    #[inline(always)]
    fn include(&self, curve: &BSplineCurve<Point3>) -> bool {
        curve.is_const() && self.include(curve.front())
    }
}

impl IncludeCurve<NURBSCurve<Vector4>> for Cylinder {
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

impl ParameterDivision2D for Cylinder {
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

impl SearchParameter<D2> for Cylinder {
    type Point = Point3;
    #[inline(always)]
    fn search_parameter<H: Into<SPHint2D>>(
        &self,
        point: Point3,
        hint: H,
        _: usize,
    ) -> Option<(f64, f64)> {
        let radius = point - self.position;
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

impl SearchNearestParameter<D2> for Cylinder {
    type Point = Point3;
    #[inline(always)]
    fn search_nearest_parameter<H: Into<SPHint2D>>(
        &self,
        point: Point3,
        _: H,
        _: usize,
    ) -> Option<(f64, f64)> {
        let radius = (point - self.position).normalize();
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
