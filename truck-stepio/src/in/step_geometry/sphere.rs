use super::*;
use std::f64::consts::PI;
use std::ops::Bound;
impl ParametricSurface for Sphere {
    type Point = Point3;
    type Vector = Vector3;
    #[inline]
    fn subs(&self, u: f64, v: f64) -> Point3 { self.0.subs(PI / 2.0 - v, u) }
    #[inline]
    fn uder(&self, u: f64, v: f64) -> Vector3 { self.0.vder(PI / 2.0 - v, u) }
    #[inline]
    fn vder(&self, u: f64, v: f64) -> Vector3 { -self.0.uder(PI / 2.0 - v, u) }
    #[inline]
    fn uuder(&self, u: f64, v: f64) -> Vector3 { self.0.vvder(PI / 2.0 - v, u) }
    #[inline]
    fn uvder(&self, u: f64, v: f64) -> Vector3 { -self.0.uvder(PI / 2.0 - v, u) }
    #[inline]
    fn vvder(&self, u: f64, v: f64) -> Vector3 { self.0.uuder(PI / 2.0 - v, u) }
    #[inline]
    fn parameter_range(&self) -> (ParameterRange, ParameterRange) {
        (
            (Bound::Included(0.0), Bound::Excluded(2.0 * PI)),
            (Bound::Included(-PI / 2.0), Bound::Excluded(PI / 2.0)),
        )
    }
    #[inline]
    fn u_period(&self) -> Option<f64> { Some(2.0 * PI) }
}
impl ParametricSurface3D for Sphere {
    #[inline]
    fn normal(&self, u: f64, v: f64) -> Vector3 { self.0.normal(PI / 2.0 - v, u) }
}
impl SearchNearestParameter<D2> for Sphere {
    type Point = Point3;
    #[inline]
    fn search_nearest_parameter<H: Into<SPHint2D>>(
        &self,
        point: Self::Point,
        hint: H,
        trials: usize,
    ) -> Option<(f64, f64)> {
        self.0
            .search_nearest_parameter(point, hint, trials)
            .map(|(u, v)| (v, PI / 2.0 - u))
    }
}
impl SearchParameter<D2> for Sphere {
    type Point = Point3;
    #[inline]
    fn search_parameter<H: Into<SPHint2D>>(
        &self,
        point: Self::Point,
        hint: H,
        trials: usize,
    ) -> Option<(f64, f64)> {
        self.0
            .search_parameter(point, hint, trials)
            .map(|(u, v)| (v, PI / 2.0 - u))
    }
}
impl ParameterDivision2D for Sphere {
    #[inline]
    fn parameter_division(
        &self,
        ((u0, u1), (v0, v1)): ((f64, f64), (f64, f64)),
        tol: f64,
    ) -> (Vec<f64>, Vec<f64>) {
        let range = ((PI / 2.0 - v1, PI / 2.0 - v0), (u0, u1));
        let (udiv0, vdiv0) = self.0.parameter_division(range, tol);
        let vdiv = udiv0.into_iter().map(|u| PI / 2.0 - u).collect();
        (vdiv0, vdiv)
    }
}

#[cfg(test)]
proptest::proptest! {
    #[test]
    fn surface(
        center in proptest::array::uniform3(-100.0f64..100.0f64),
        radius in 0.1f64..100.0f64,
        (u, v) in (0.0..=2.0 * PI, -PI / 2.0..=PI / 2.0),
    ) {
        const EPS: f64 = 1.0e-3;
        let sphere = Sphere(truck_geometry::prelude::Sphere::new(center.into(), radius));

        let uder0 = sphere.uder(u, v);
        let uder1 = (sphere.subs(u + EPS, v) - sphere.subs(u - EPS, v)) / (2.0 * EPS);
        assert!(
            (uder0 - uder1).magnitude2() < EPS,
            "uder failed: {uder0:?}, {uder1:?}"
        );

        let vder0 = sphere.vder(u, v);
        let vder1 = (sphere.subs(u, v + EPS) - sphere.subs(u, v - EPS)) / (2.0 * EPS);
        assert!(
            (vder0 - vder1).magnitude2() < EPS,
            "vder failed: {vder0:?}, {vder1:?}"
        );

        let uuder0 = sphere.uuder(u, v);
        let uuder1 = (sphere.uder(u + EPS, v) - sphere.uder(u - EPS, v)) / (2.0 * EPS);
        assert!(
            (uuder0 - uuder1).magnitude2() < EPS,
            "uuder failed: {uuder0:?}, {uuder1:?}"
        );

        let uvder0 = sphere.uvder(u, v);
        let uvder1 = (sphere.uder(u, v + EPS) - sphere.uder(u, v - EPS)) / (2.0 * EPS);
        assert!(
            (uvder0 - uvder1).magnitude2() < EPS,
            "uvder failed: {uvder0:?}, {uvder1:?}"
        );

        let vvder0 = sphere.vvder(u, v);
        let vvder1 = (sphere.vder(u, v + EPS) - sphere.vder(u, v - EPS)) / (2.0 * EPS);
        assert!(
            (vvder0 - vvder1).magnitude2() < EPS,
            "vvder failed: {vvder0:?}, {vvder1:?}"
        );

        let n0 = sphere.normal(u, v);
        let n1 = sphere.uder(u, v).cross(sphere.vder(u, v)).normalize();
        assert!(
            (n0 - n1).magnitude2() < EPS,
            "normal failed: {n0:?}, {n1:?}"
        );
    }
}
