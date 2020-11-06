use cgmath::*;

pub trait TangentSpace<S>: VectorSpace<Scalar = S> {
    type Space: EuclideanSpace<Scalar = S, Diff = Self>;
}
impl<S: BaseFloat> TangentSpace<S> for Vector1<S> {
    type Space = Point1<S>;
}
impl<S: BaseFloat> TangentSpace<S> for Vector2<S> {
    type Space = Point2<S>;
}
impl<S: BaseFloat> TangentSpace<S> for Vector3<S> {
    type Space = Point3<S>;
}

pub trait Homogeneous<S: BaseFloat>: VectorSpace<Scalar = S> {
    type Point: EuclideanSpace<Scalar = S>;
    fn truncate(self) -> <Self::Point as EuclideanSpace>::Diff;
    fn weight(self) -> S;
    /// Returns the projection to the plane whose the last component is `1.0`.
    /// # Examples
    /// ```
    /// use truck_base::cgmath64::*;
    /// use truck_base::cgmath_extend_traits::*;
    /// let pt = Vector4::new(8.0, 4.0, 6.0, 2.0).to_point();
    /// assert_eq!(pt, Point3::new(4.0, 2.0, 3.0));
    /// ```
    fn from_point(point: Self::Point) -> Self;
    #[inline(always)]
    fn to_point(self) -> Self::Point { Self::Point::from_vec(self.truncate() / self.weight()) }
    /// Returns the derivation of the rational curve.
    ///
    /// For a curve c(t) = (c_0(t), c_1(t), c_2(t), c_3(t)), returns the derivation
    /// of the projected curve (c_0 / c_3, c_1 / c_3, c_2 / c_3, 1.0).
    /// # Arguments
    /// * `self` - the point of the curve c(t)
    /// * `der` - the derivation c'(t) of the curve
    /// # Examples
    /// ```
    /// use truck_base::cgmath64::*;
    /// use truck_base::cgmath_extend_traits::*;
    /// // calculate the derivation at t = 1.5
    /// let t = 1.5;
    /// // the curve: c(t) = (t^2, t^3, t^4, t)
    /// let pt = Vector4::new(t * t, t * t * t, t * t * t * t, t);
    /// // the derivation: c'(t) = (2t, 3t^2, 4t^3, 1)
    /// let der = Vector4::new(2.0 * t, 3.0 * t * t, 4.0 * t * t * t, 1.0);
    /// // the projected curve: \bar{c}(t) = (t, t^2, t^3, 1)
    /// // the derivation of the proj'ed curve: \bar{c}'(t) = (1, 2t, 3t^2, 0)
    /// let ans = Vector3::new(1.0, 2.0 * t, 3.0 * t * t);
    /// assert_eq!(pt.rat_der(der), ans);
    /// ```
    #[inline(always)]
    fn rat_der(self, der: Self) -> <Self::Point as EuclideanSpace>::Diff {
        let res = (der * self.weight() - self * der.weight()) / (self.weight() * self.weight());
        res.truncate()
    }
    /// Returns the 2nd-ord derivation of the rational curve.
    ///
    /// For a curve c(t) = (c_0(t), c_1(t), c_2(t), c_3(t)), returns the 2nd ordered derivation
    /// of the projected curve (c_0 / c_3, c_1 / c_3, c_2 / c_3).
    /// # Arguments
    /// * `self` - the point of the curve c(t)
    /// * `der` - the derivation c'(t) of the curve
    /// * `der2` - the 2nd ordered derivation c''(t) of the curve
    /// # Examples
    /// ```
    /// use truck_base::cgmath64::*;
    /// use truck_base::cgmath_extend_traits::*;
    /// // calculate the derivation at t = 1.5
    /// let t = 1.5;
    /// // the curve: c(t) = (t^2, t^3, t^4, t)
    /// let pt = Vector4::new(t * t, t * t * t, t * t * t * t, t);
    /// // the derivation: c'(t) = (2t, 3t^2, 4t^3, 1)
    /// let der = Vector4::new(2.0 * t, 3.0 * t * t, 4.0 * t * t * t, 1.0);
    /// // the 2nd ord. deri.: c''(t) = (2, 6t, 12t^2, 0)
    /// let der2 = Vector4::new(2.0, 6.0 * t, 12.0 * t * t, 0.0);
    /// // the projected curve: \bar{c}(t) = (t, t^2, t^3, 1)
    /// // the derivation of the proj'ed curve: \bar{c}'(t) = (1, 2t, 3t^2, 0)
    /// // the 2nd ord. deri. of the proj'ed curve: \bar{c}''(t) = (0, 2, 6t, 0)
    /// let ans = Vector3::new(0.0, 2.0, 6.0 * t);
    /// assert_eq!(pt.rat_der2(der, der2), ans);
    /// ```
    #[inline(always)]
    fn rat_der2(self, der: Self, der2: Self) -> <Self::Point as EuclideanSpace>::Diff {
        let pre_coef1 = der.weight() / (self.weight() * self.weight());
        let coef1 = pre_coef1 + pre_coef1;
        let der_last2 = der.weight() * der.weight();
        let coef2 = (der_last2 + der_last2 - der2.weight() * self.weight())
            / (self.weight() * self.weight() * self.weight());
        let res = der2 / self.weight() - der * coef1 + self * coef2;
        res.truncate()
    }
}

impl<S: BaseFloat> Homogeneous<S> for Vector2<S> {
    type Point = Point1<S>;
    #[inline(always)]
    fn truncate(self) -> Vector1<S> { Vector1::new(self[0]) }
    #[inline(always)]
    fn weight(self) -> S { self[1] }
    #[inline(always)]
    fn from_point(point: Self::Point) -> Self { Vector2::new(point[0], S::one()) }
}

impl<S: BaseFloat> Homogeneous<S> for Vector3<S> {
    type Point = Point2<S>;
    #[inline(always)]
    fn truncate(self) -> Vector2<S> { self.truncate() }
    #[inline(always)]
    fn weight(self) -> S { self[2] }
    #[inline(always)]
    fn from_point(point: Self::Point) -> Self { Vector3::new(point[0], point[1], S::one()) }
}

impl<S: BaseFloat> Homogeneous<S> for Vector4<S> {
    type Point = Point3<S>;
    #[inline(always)]
    fn truncate(self) -> Vector3<S> { self.truncate() }
    #[inline(always)]
    fn weight(self) -> S { self[3] }
    #[inline(always)]
    fn from_point(point: Self::Point) -> Self { point.to_homogeneous() }
}
