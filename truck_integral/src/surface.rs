use crate::*;
const N: usize = 50;
use geometry::TOLERANCE;

fn snp2(surface: &BSplineSurface<Vector2>, pt: Vector2, hint: (f64, f64)) -> Option<(f64, f64)> {
    let mut surface = surface.clone();
    surface.transform_control_points(|v| *v = *v - pt);
    let uder = surface.uderivation();
    let vder = surface.vderivation();
    sub_snp2(&surface, &uder, &vder, hint.into(), 0).map(|res| res.into())
}

fn sub_snp2(
    surface: &BSplineSurface<Vector2>,
    uder: &BSplineSurface<Vector2>,
    vder: &BSplineSurface<Vector2>,
    hint: Vector2,
    count: usize,
) -> Option<Vector2>
{
    if count == 100 {
        return None;
    }
    let (u0, v0) = (hint[0], hint[1]);
    let pt = surface.subs(u0, v0);
    let jacobi = Matrix2::from_cols(uder.subs(u0, v0), vder.subs(u0, v0));
    let res = jacobi.invert().map(|inv| hint - inv * pt);
    match res {
        Some(entity) => {
            if Tolerance::near(&entity, &hint) {
                res
            } else {
                sub_snp2(surface, uder, vder, entity, count + 1)
            }
        }
        None => res,
    }
}

fn presearch2(surface: &BSplineSurface<Vector2>, point: Vector2) -> (f64, f64) {
    let mut res = (0.0, 0.0);
    let mut min = std::f64::INFINITY;
    for i in 0..=N {
        for j in 0..=N {
            let p = i as f64 / N as f64;
            let q = j as f64 / N as f64;
            let u = surface.uknot_vec()[0] + p * surface.uknot_vec().range_length();
            let v = surface.vknot_vec()[0] + q * surface.vknot_vec().range_length();
            let dist = surface.subs(u, v).distance2(point);
            if dist < min {
                min = dist;
                res = (u, v);
            }
        }
    }
    res
}

impl Surface for BSplineSurface<Vector2> {
    type Curve = BSplineCurve<Vector2>;
    #[inline(always)]
    fn inverse(&self) -> Self {
        let mut surface = self.clone();
        surface.swap_axes();
        surface
    }
    /// # Examples
    /// ```
    /// use truck_integral::*;
    /// use geometry::*;
    /// let knot_vec = KnotVec::uniform_knot(2, 3);
    /// let ctrl_pts = vec![
    ///     vec![Vector2::new(0.0, 0.0), Vector2::new(0.1, 0.0), Vector2::new(0.5, 0.0), Vector2::new(0.7, 0.0), Vector2::new(1.0, 0.0)],
    ///     vec![Vector2::new(0.0, 0.1), Vector2::new(0.2, 0.2), Vector2::new(0.4, 0.3), Vector2::new(0.6, 0.2), Vector2::new(1.0, 0.3)],
    ///     vec![Vector2::new(0.0, 0.5), Vector2::new(0.3, 0.6), Vector2::new(0.6, 0.4), Vector2::new(0.9, 0.6), Vector2::new(1.0, 0.5)],
    ///     vec![Vector2::new(0.0, 0.7), Vector2::new(0.2, 0.8), Vector2::new(0.3, 0.6), Vector2::new(0.5, 0.9), Vector2::new(1.0, 0.7)],
    ///     vec![Vector2::new(0.0, 1.0), Vector2::new(0.1, 1.0), Vector2::new(0.5, 1.0), Vector2::new(0.7, 1.0), Vector2::new(1.0, 1.0)],
    /// ];
    /// let surface = BSplineSurface::new((knot_vec.clone(), knot_vec), ctrl_pts);
    ///
    /// let knot_vec0 = KnotVec::bezier_knot(2);
    /// let ctrl_pts0 = vec![Vector2::new(0.0, 0.0), Vector2::new(1.0, 1.0), Vector2::new(0.0, 1.0)];
    /// let curve0 = BSplineCurve::new(knot_vec0, ctrl_pts0);
    /// assert!(surface.include(&curve0));
    ///
    /// let knot_vec1 = KnotVec::bezier_knot(2);
    /// let ctrl_pts1 = vec![Vector2::new(0.0, 0.0), Vector2::new(2.5, 1.0), Vector2::new(0.0, 1.0)];
    /// let curve1 = BSplineCurve::new(knot_vec1, ctrl_pts1);
    /// assert!(!surface.include(&curve1));
    /// ```
    fn include(&self, curve: &Self::Curve) -> bool {
        let pt = curve.subs(curve.knot_vec()[0]);
        let mut hint = presearch2(self, pt);
        hint = match self.search_nearest_parameter(pt, hint) {
            Some(got) => got,
            None => return false,
        };
        let uknot_vec = self.uknot_vec();
        let vknot_vec = self.vknot_vec();
        let degree = curve.degree() * 6;
        let (knots, _) = curve.knot_vec().to_single_multi();
        for i in 1..knots.len() {
            for j in 1..=degree {
                let p = j as f64 / degree as f64;
                let t = knots[i - 1] * (1.0 - p) + knots[i] * p;
                let pt = curve.subs(t);
                hint = match snp2(self, pt, hint) {
                    Some(got) => got,
                    None => return false,
                };
                println!("{:?}", hint);
                if !Point::near(&self.subs(hint.0, hint.1), &pt) {
                    return false;
                } else if hint.0 < uknot_vec[0] - TOLERANCE
                    || hint.0 - uknot_vec[0] > uknot_vec.range_length() + TOLERANCE
                {
                    return false;
                } else if hint.1 < vknot_vec[0] - TOLERANCE
                    || hint.1 - vknot_vec[0] > vknot_vec.range_length() + TOLERANCE
                {
                    return false;
                }
            }
        }
        true
    }
}
