use cgmath::*;

/// declare control point
pub mod control_point {
    use super::*;
    use std::fmt::Debug;
    use std::ops::*;
    /// trait for abstract control points of polylines and B-splines
    pub trait ControlPoint<S>:
        Add<Self::Diff, Output = Self>
        + Sub<Self::Diff, Output = Self>
        + Sub<Self, Output = Self::Diff>
        + Mul<S, Output = Self>
        + Div<S, Output = Self>
        + AddAssign<Self::Diff>
        + SubAssign<Self::Diff>
        + MulAssign<S>
        + DivAssign<S>
        + Copy
        + Clone
        + Debug {
        /// differential vector
        type Diff: Add<Self::Diff, Output = Self::Diff>
            + Sub<Self::Diff, Output = Self::Diff>
            + Mul<S, Output = Self::Diff>
            + Div<S, Output = Self::Diff>
            + AddAssign<Self::Diff>
            + SubAssign<Self::Diff>
            + MulAssign<S>
            + DivAssign<S>
            + Zero
            + Copy
            + Clone
            + Debug;
        /// origin
        fn origin() -> Self;
        /// into the vector
        fn to_vec(self) -> Self::Diff;
    }

    impl<S: BaseFloat> ControlPoint<S> for Point1<S> {
        type Diff = Vector1<S>;
        fn origin() -> Self { EuclideanSpace::origin() }
        fn to_vec(self) -> Self::Diff { EuclideanSpace::to_vec(self) }
    }
    impl<S: BaseFloat> ControlPoint<S> for Point2<S> {
        type Diff = Vector2<S>;
        fn origin() -> Self { EuclideanSpace::origin() }
        fn to_vec(self) -> Self::Diff { EuclideanSpace::to_vec(self) }
    }
    impl<S: BaseFloat> ControlPoint<S> for Point3<S> {
        type Diff = Vector3<S>;
        fn origin() -> Self { EuclideanSpace::origin() }
        fn to_vec(self) -> Self::Diff { EuclideanSpace::to_vec(self) }
    }
    impl<S: BaseFloat> ControlPoint<S> for Vector1<S> {
        type Diff = Vector1<S>;
        fn origin() -> Self { Zero::zero() }
        fn to_vec(self) -> Self { self }
    }
    impl<S: BaseFloat> ControlPoint<S> for Vector2<S> {
        type Diff = Vector2<S>;
        fn origin() -> Self { Zero::zero() }
        fn to_vec(self) -> Self { self }
    }
    impl<S: BaseFloat> ControlPoint<S> for Vector3<S> {
        type Diff = Vector3<S>;
        fn origin() -> Self { Zero::zero() }
        fn to_vec(self) -> Self { self }
    }
    impl<S: BaseFloat> ControlPoint<S> for Vector4<S> {
        type Diff = Vector4<S>;
        fn origin() -> Self { Zero::zero() }
        fn to_vec(self) -> Self { self }
    }
}

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
    /// Returns homogeneous coordinate from point and weight.
    fn from_point_weight(point: Self::Point, weight: S) -> Self;
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
    #[inline(always)]
    fn from_point_weight(point: Self::Point, weight: S) -> Self { Vector2::new(point[0], weight) }
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
    #[inline(always)]
    fn from_point_weight(point: Self::Point, weight: S) -> Self {
        Vector3::new(point[0], point[1], weight)
    }
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
    #[inline(always)]
    fn from_point_weight(point: Self::Point, weight: S) -> Self {
        Vector4::new(point[0], point[1], point[2], weight)
    }
}

/// Trait to calculate Iwasawa decomposition on matrices.
pub trait IwasawaDecomposition: Sized {
    /// Iwasawa decomposition `M = NAK` where
    /// - `N`: upper half unipotent matrix
    /// - `A`: diagonal matrix
    /// - `K`: orthogonal matrix
    ///
    /// # Failure
    /// Returns `None` if `self` is not invertible.
    fn iwasawa_decomposition(self) -> Option<(Self, Self, Self)>;
}

impl<S: BaseFloat> IwasawaDecomposition for Matrix2<S> {
    fn iwasawa_decomposition(self) -> Option<(Self, Self, Self)> {
        let v0 = self.row(0);
        let v1 = self.row(1);
        let mut n = Matrix2::identity();
        let u1 = v1;
        let a1 = u1.magnitude();
        if a1 == S::zero() {
            return None;
        }
        n[0][1] = v0.dot(u1) / (a1 * a1);
        let u0 = v0 - u1 * n[0][1];
        let a0 = u0.magnitude();
        if a0 == S::zero() {
            return None;
        }

        let k = Matrix2::from_cols(u0 / a0, u1 / a1);
        let a = Matrix2::from_diagonal(Vector2::new(a0, a1));
        Some((n.transpose(), a, k.transpose()))
    }
}

impl<S: BaseFloat> IwasawaDecomposition for Matrix3<S> {
    fn iwasawa_decomposition(self) -> Option<(Self, Self, Self)> {
        let v0 = self.row(0);
        let v1 = self.row(1);
        let v2 = self.row(2);
        let mut n = Matrix3::identity();
        let u2 = v2;
        let a2 = u2.magnitude();
        if a2 == S::zero() {
            return None;
        }
        n[1][2] = v1.dot(u2) / (a2 * a2);
        let u1 = v1 - u2 * n[1][2];
        let a1 = u1.magnitude();
        if a1 == S::zero() {
            return None;
        }
        n[0][1] = v0.dot(u1) / (a1 * a1);
        n[0][2] = v0.dot(u2) / (a2 * a2);
        let u0 = v0 - u1 * n[0][1] - u2 * n[0][2];
        let a0 = u0.magnitude();
        if a0 == S::zero() {
            return None;
        }

        let k = Matrix3::from_cols(u0 / a0, u1 / a1, u2 / a2);
        let a = Matrix3::from_diagonal(Vector3::new(a0, a1, a2));
        Some((n.transpose(), a, k.transpose()))
    }
}

impl<S: BaseFloat> IwasawaDecomposition for Matrix4<S> {
    fn iwasawa_decomposition(self) -> Option<(Self, Self, Self)> {
        let v0 = self.row(0);
        let v1 = self.row(1);
        let v2 = self.row(2);
        let v3 = self.row(3);
        let mut n = Matrix4::identity();

        let u3 = v3;
        let a3 = u3.magnitude();
        if a3 == S::zero() {
            return None;
        }
        n[2][3] = v2.dot(u3) / (a3 * a3);
        let u2 = v2 - u3 * n[2][3];
        let a2 = u2.magnitude();
        if a2 == S::zero() {
            return None;
        }
        n[1][2] = v1.dot(u2) / (a2 * a2);
        n[1][3] = v1.dot(u3) / (a3 * a3);
        let u1 = v1 - u2 * n[1][2] - u3 * n[1][3];
        let a1 = u1.magnitude();
        if a1 == S::zero() {
            return None;
        }
        n[0][1] = v0.dot(u1) / (a1 * a1);
        n[0][2] = v0.dot(u2) / (a2 * a2);
        n[0][3] = v0.dot(u3) / (a3 * a3);
        let u0 = v0 - u1 * n[0][1] - u2 * n[0][2] - u3 * n[0][3];
        let a0 = u0.magnitude();
        if a0 == S::zero() {
            return None;
        }

        let k = Matrix4::from_cols(u0 / a0, u1 / a1, u2 / a2, u3 / a3);
        let a = Matrix4::from_diagonal(Vector4::new(a0, a1, a2, a3));
        Some((n.transpose(), a, k.transpose()))
    }
}

#[test]
fn iwasawa_matrix2() {
    use crate::{assert_near, tolerance::Tolerance};
    let m = Matrix2::<f64>::new(2.0, 3.0, -1.0, 4.0);
    let (n, a, k) = m.iwasawa_decomposition().unwrap();
    assert_near!(m, (n * a * k));
    assert_near!(n[0][1], 0.0);
    assert_eq!(n[0][0], 1.0);
    assert_eq!(n[1][1], 1.0);
    assert!(a.is_diagonal());
    assert_near!(k * k.invert().unwrap(), Matrix2::<f64>::identity());
}

#[test]
fn iwasawa_matrix3() {
    use crate::{assert_near, tolerance::Tolerance};
    let m = Matrix3::<f64>::new(2.0, 3.0, -1.0, -4.0, 6.0, 8.0, 9.0, -1.0, 2.0);
    let (n, a, k) = m.iwasawa_decomposition().unwrap();
    assert_near!(m, (n * a * k));
    assert_near!(n[0][1], 0.0);
    assert_near!(n[0][2], 0.0);
    assert_near!(n[1][2], 0.0);
    assert_eq!(n[0][0], 1.0);
    assert_eq!(n[1][1], 1.0);
    assert_eq!(n[2][2], 1.0);
    assert!(a.is_diagonal());
    assert_near!(k * k.invert().unwrap(), Matrix3::<f64>::identity());
}

#[test]
fn iwasawa_matrix4() {
    use crate::{assert_near, tolerance::Tolerance};
    let m = Matrix4::<f64>::new(
        2.0, 3.0, -1.0, -4.0, 6.0, 8.0, 9.0, -1.0, -2.0, 6.0, 8.0, -3.0, 1.0, -10.0, 3.0, 4.0,
    );
    let (n, a, k) = m.iwasawa_decomposition().unwrap();
    assert_near!(m, (n * a * k));
    assert_near!(n[0][1], 0.0);
    assert_near!(n[0][2], 0.0);
    assert_near!(n[0][3], 0.0);
    assert_near!(n[1][2], 0.0);
    assert_near!(n[1][3], 0.0);
    assert_near!(n[2][3], 0.0);
    assert_eq!(n[0][0], 1.0);
    assert_eq!(n[1][1], 1.0);
    assert_eq!(n[2][2], 1.0);
    assert_eq!(n[3][3], 1.0);
    assert!(a.is_diagonal());
    assert_near!(k * k.invert().unwrap(), Matrix4::<f64>::identity());
}
