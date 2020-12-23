use cgmath::*;

/// Tangent spaces of euclidean spaces
/// The inverse of [`EuclideanSpace::Diff`](../cgmath/trait.EuclideanSpace.html)
pub trait TangentSpace<S: BaseFloat>: VectorSpace<Scalar = S> {
    /// The Euclidean space whose tangent space is `Self`.
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

/// Homogeneous coordinate of an Euclidean space and a vector space.
/// # Examples
/// ```
/// use truck_base::cgmath64::*;
/// use truck_base::cgmath_extend_traits::*;
/// assert_eq!(Vector4::new(8.0, 6.0, 4.0, 2.0).truncate(), Vector3::new(8.0, 6.0, 4.0));
/// assert_eq!(Vector4::new(8.0, 6.0, 4.0, 2.0).weight(), 2.0);
/// assert_eq!(Vector4::new(8.0, 6.0, 4.0, 2.0).to_point(), Point3::new(4.0, 3.0, 2.0));
/// assert_eq!(Vector4::from_point(Point3::new(4.0, 3.0, 2.0)), Vector4::new(4.0, 3.0, 2.0, 1.0));
/// ```
pub trait Homogeneous<S: BaseFloat>: VectorSpace<Scalar = S> {
    /// The tangent vector of `Self::Point`
    type Vector: VectorSpace<Scalar = S>;
    /// The point expressed by homogeneous coordinate
    type Point: EuclideanSpace<Scalar = S, Diff = Self::Vector>;
    /// Returns the first dim - 1 components.
    fn truncate(self) -> Self::Vector;
    /// Returns the last component.
    fn weight(self) -> S;
    /// Returns homogeneous coordinate.
    fn from_point(point: Self::Point) -> Self;
    /// Returns the projection to the plane whose the last component is `1.0`.
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
    
    /// Returns the cross derivation of the rational surface.
    ///
    /// For a surface s(u, v) = (s_0(u, v), s_1(u, v), s_2(u, v), s_3(u, v)), returns the derivation
    /// of the projected surface (s_0 / s_3, s_1 / s_3, s_2 / s_3) by both u and v.
    /// # Arguments
    /// * `self` - the point of the surface s(u, v)
    /// * `uder` - the u-derivation s_u(u, v) of the surface
    /// * `vder` - the v-derivation s_v(u, v) of the surface
    /// * `uvder` - the 2nd ordered derivation s_{uv}(u, v) of the surface
    /// # Examples
    /// ```
    /// use truck_base::cgmath64::*;
    /// // calculate the derivation at (u, v) = (1.0, 2.0).
    /// let (u, v) = (1.0, 2.0);
    /// // the curve: s(u, v) = (u^3 v^2, u^2 v^3, u v, u)
    /// let pt = Vector4::new(
    ///     u * u * u * v * v,
    ///     u * u * v * v * v,
    ///     u * v,
    ///     u,
    /// );
    /// // the u-derivation: s_u(u, v) = (3u^2 v^2, 2u * v^3, v, 1)
    /// let uder = Vector4::new(
    ///     3.0 * u * u * v * v,
    ///     2.0 * u * v * v * v,
    ///     v,
    ///     1.0,
    /// );
    /// // the v-derivation: s_v(u, v) = (2u^3 v, 3u^2 v^2, u, 0)
    /// let vder = Vector4::new(
    ///     2.0 * u * u * u * v,
    ///     3.0 * u * u * v * v,
    ///     u,
    ///     0.0,
    /// );
    /// // s_{uv}(u, v) = (6u^2 v, 6u v^2, 1, 0)
    /// let uvder = Vector4::new(6.0 * u * u * v, 6.0 * u * v * v, 1.0, 0.0);
    /// // the projected surface: \bar{s}(u, v) = (u^2 v^2, u v^3, v)
    /// // \bar{s}_u(u, v) = (2u v^2, v^3, 0)
    /// // \bar{s}_v(u, v) = (2u^2 v, 3u v^2, 1)
    /// // \bar{s}_{uv}(u, v) = (4uv, 3v^2, 0)
    /// let ans = Vector3::new(4.0 * u * v, 3.0 * v * v, 0.0);
    /// assert_eq!(pt.rat_cross_der(uder, vder, uvder), ans);
    /// ```
    #[inline(always)]
    fn rat_cross_der(&self, uder: Self, vder: Self, uvder: Self) -> Self::Vector {
        let self_weight2 = self.weight() * self.weight();
        let coef1 = vder.weight() / self_weight2;
        let coef2 = uder.weight() / self_weight2;
        let der_weight2 = uder.weight() * vder.weight();
        let coef3 = (der_weight2 + der_weight2 - uvder.weight() * self.weight())
            / (self_weight2 * self.weight());
        let res = uvder / self.weight() - uder * coef1 - vder * coef2 + *self * coef3;
        res.truncate()
    }
}

impl<S: BaseFloat> Homogeneous<S> for Vector2<S> {
    type Vector = Vector1<S>;
    type Point = Point1<S>;
    #[inline(always)]
    fn truncate(self) -> Vector1<S> { Vector1::new(self[0]) }
    #[inline(always)]
    fn weight(self) -> S { self[1] }
    #[inline(always)]
    fn from_point(point: Self::Point) -> Self { Vector2::new(point[0], S::one()) }
}

impl<S: BaseFloat> Homogeneous<S> for Vector3<S> {
    type Vector = Vector2<S>;
    type Point = Point2<S>;
    #[inline(always)]
    fn truncate(self) -> Vector2<S> { self.truncate() }
    #[inline(always)]
    fn weight(self) -> S { self[2] }
    #[inline(always)]
    fn from_point(point: Self::Point) -> Self { Vector3::new(point[0], point[1], S::one()) }
}

impl<S: BaseFloat> Homogeneous<S> for Vector4<S> {
    type Vector = Vector3<S>;
    type Point = Point3<S>;
    #[inline(always)]
    fn truncate(self) -> Vector3<S> { self.truncate() }
    #[inline(always)]
    fn weight(self) -> S { self[3] }
    #[inline(always)]
    fn from_point(point: Self::Point) -> Self { point.to_homogeneous() }
}
