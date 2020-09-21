use crate::*;
const N: usize = 50;

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
    /// let ctrl_pts0 = vec![Vector2::new(0.1, 0.1), Vector2::new(2.0, 1.0), Vector2::new(0.0, 1.0)];
    /// let curve0 = BSplineCurve::new(knot_vec0, ctrl_pts0);
    /// assert!(surface.include(&curve0));
    /// 
    /// let knot_vec1 = KnotVec::bezier_knot(2);
    /// let ctrl_pts1 = vec![Vector2::new(0.1, 0.1), Vector2::new(2.1, 1.0), Vector2::new(0.0, 1.0)];
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
        let degree = curve.degree() * 2;
        let (knots, _) = curve.knot_vec().to_single_multi();
        for i in 1..knots.len() {
            for j in 1..=degree {
                let p = j as f64 / degree as f64;
                let t = knots[i - 1] * (1.0 - p) + knots[i] * p;
                let pt = curve.subs(t);
                println!("{:?}", hint);
                hint = match self.search_nearest_parameter(pt, hint) {
                    Some(got) => got,
                    None => return false,
                };
                if !Point::near(&self.subs(hint.0, hint.1), &pt) {
                    return false;
                }
            }
        }
        true
    }
}
