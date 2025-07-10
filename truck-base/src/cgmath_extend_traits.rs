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
        + Debug
        + Index<usize, Output = S>
        + IndexMut<usize, Output = S> {
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
            + Debug
            + Index<usize, Output = S>
            + IndexMut<usize, Output = S>;
        /// dimension
        const DIM: usize;
        /// origin
        fn origin() -> Self;
        /// into the vector
        fn to_vec(self) -> Self::Diff;
        /// from the vector
        fn from_vec(vec: Self::Diff) -> Self;
    }

    impl<S: BaseFloat> ControlPoint<S> for Point1<S> {
        type Diff = Vector1<S>;
        const DIM: usize = 1;
        fn origin() -> Self { EuclideanSpace::origin() }
        fn to_vec(self) -> Self::Diff { EuclideanSpace::to_vec(self) }
        fn from_vec(vec: Self::Diff) -> Self { EuclideanSpace::from_vec(vec) }
    }
    impl<S: BaseFloat> ControlPoint<S> for Point2<S> {
        type Diff = Vector2<S>;
        const DIM: usize = 2;
        fn origin() -> Self { EuclideanSpace::origin() }
        fn to_vec(self) -> Self::Diff { EuclideanSpace::to_vec(self) }
        fn from_vec(vec: Self::Diff) -> Self { EuclideanSpace::from_vec(vec) }
    }
    impl<S: BaseFloat> ControlPoint<S> for Point3<S> {
        type Diff = Vector3<S>;
        const DIM: usize = 3;
        fn origin() -> Self { EuclideanSpace::origin() }
        fn to_vec(self) -> Self::Diff { EuclideanSpace::to_vec(self) }
        fn from_vec(vec: Self::Diff) -> Self { EuclideanSpace::from_vec(vec) }
    }
    impl<S: BaseFloat> ControlPoint<S> for Vector1<S> {
        type Diff = Vector1<S>;
        const DIM: usize = 1;
        fn origin() -> Self { Zero::zero() }
        fn to_vec(self) -> Self { self }
        fn from_vec(vec: Self::Diff) -> Self { vec }
    }
    impl<S: BaseFloat> ControlPoint<S> for Vector2<S> {
        type Diff = Vector2<S>;
        const DIM: usize = 2;
        fn origin() -> Self { Zero::zero() }
        fn to_vec(self) -> Self { self }
        fn from_vec(vec: Self::Diff) -> Self { vec }
    }
    impl<S: BaseFloat> ControlPoint<S> for Vector3<S> {
        type Diff = Vector3<S>;
        const DIM: usize = 3;
        fn origin() -> Self { Zero::zero() }
        fn to_vec(self) -> Self { self }
        fn from_vec(vec: Self::Diff) -> Self { vec }
    }
    impl<S: BaseFloat> ControlPoint<S> for Vector4<S> {
        type Diff = Vector4<S>;
        const DIM: usize = 4;
        fn origin() -> Self { Zero::zero() }
        fn to_vec(self) -> Self { self }
        fn from_vec(vec: Self::Diff) -> Self { vec }
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
/// assert_eq!(Vector4::new(8.0, 6.0, 4.0, 2.0).truncate(), Vector3::new(8.0, 6.0, 4.0));
/// assert_eq!(Vector4::new(8.0, 6.0, 4.0, 2.0).weight(), 2.0);
/// assert_eq!(Vector4::new(8.0, 6.0, 4.0, 2.0).to_point(), Point3::new(4.0, 3.0, 2.0));
/// assert_eq!(Vector4::from_point(Point3::new(4.0, 3.0, 2.0)), Vector4::new(4.0, 3.0, 2.0, 1.0));
/// ```
pub trait Homogeneous<S: BaseFloat>: VectorSpace<Scalar = S> {
    /// The point expressed by homogeneous coordinate
    type Point: EuclideanSpace<Scalar = S>;
    /// Returns the first dim - 1 components.
    fn truncate(self) -> <Self::Point as EuclideanSpace>::Diff;
    /// Returns the last component.
    fn weight(self) -> S;
    /// Returns homogeneous coordinate.
    fn from_point(point: Self::Point) -> Self;
    /// Returns homogeneous coordinate from point and weight.
    #[inline(always)]
    fn from_point_weight(point: Self::Point, weight: S) -> Self { Self::from_point(point) * weight }
    /// Returns the projection to the plane whose the last component is `1.0`.
    #[inline(always)]
    fn to_point(self) -> Self::Point { Self::Point::from_vec(self.truncate() / self.weight()) }
}

/// Returns the higher order derivation of the rational curve.
/// # Examples
/// ```
/// use truck_base::cgmath64::*;
/// // calculate the derivation at t = 1.5
/// let t = 1.5;
///
/// // the curve: c(t) = (t^2, t^3, t^4, t)
/// let pt = Vector4::new(t * t, t * t * t, t * t * t * t, t);
/// // the derivation: c'(t) = (2t, 3t^2, 4t^3, 1)
/// let der = Vector4::new(2.0 * t, 3.0 * t * t, 4.0 * t * t * t, 1.0);
/// // the 2nd ord. deri.: c''(t) = (2, 6t, 12t^2, 0)
/// let der2 = Vector4::new(2.0, 6.0 * t, 12.0 * t * t, 0.0);
/// // the 3rd ord. deri.: c''(t) = (0, 6, 24t, 0)
/// let der3 = Vector4::new(0.0, 6.0, 24.0 * t, 0.0);
///
/// // the projected curve: \bar{c}(t) = (t, t^2, t^3)
/// // the 1st ord. deri. of the proj'ed curve: \bar{c}''(t) = (1, 2t, 3t^2)
/// let ans_der1 = Vector3::new(1.0, 2.0 * t, 3.0 * t * t);
/// assert_eq!(rat_der(&[pt, der]), ans_der1);
/// // the 2nd ord. deri. of the proj'ed curve: \bar{c}''(t) = (0, 2, 6t)
/// let ans_der2 = Vector3::new(0.0, 2.0, 6.0 * t);
/// assert_eq!(rat_der(&[pt, der, der2]), ans_der2);
/// // the 3rd ord. deri. of the proj'ed curve: \bar{c}''(t) = (0, 0, 6)
/// let ans_der3 = Vector3::new(0.0, 0.0, 6.0);
/// assert_eq!(rat_der(&[pt, der, der2, der3]), ans_der3);
/// ```
pub fn rat_der<S, V, Diff>(ders: &[V]) -> Diff
where
    S: BaseFloat,
    V: Homogeneous<S>,
    V::Point: EuclideanSpace<Diff = Diff>,
    Diff: VectorSpace<Scalar = S>, {
    let len = ders.len();
    if len == 0 {
        Diff::zero()
    } else if len == 1 {
        ders[0].to_point().to_vec()
    } else if len == 2 {
        let (s, sw) = (ders[0].truncate(), ders[0].weight());
        let (d, dw) = (ders[1].truncate(), ders[1].weight());
        (d * sw - s * dw) / (sw * sw)
    } else if len == 3 {
        let (s, sw) = (ders[0].truncate(), ders[0].weight());
        let (d, dw) = (ders[1].truncate(), ders[1].weight());
        let (d2, d2w) = (ders[2].truncate(), ders[2].weight());
        let two = S::from(2).unwrap();
        let sw2 = sw * sw;
        d2 / sw - d * (dw / sw2 * two) + s * (dw * dw * two / (sw2 * sw) - d2w / sw2)
    } else if len < 32 {
        let mut evals = [Diff::zero(); 32];
        rat_ders(ders, &mut evals);
        evals[ders.len() - 1]
    } else {
        let mut evals = vec![Diff::zero(); ders.len()];
        rat_ders(ders, &mut evals);
        evals[ders.len() - 1]
    }
}

/// Store the multi-orders derivations of the rational curve in `evals`.
/// # Examples
/// ```
/// use truck_base::cgmath64::*;
/// // calculate the derivation at t = 1.5
/// let t = 1.5;
///
/// // the curve: c(t) = (t^2, t^3, t^4, t)
/// let ders = [
///     Vector4::new(t * t, t * t * t, t * t * t * t, t), // 0th-order
///     Vector4::new(2.0 * t, 3.0 * t * t, 4.0 * t * t * t, 1.0), // 1st-order
///     Vector4::new(2.0, 6.0 * t, 12.0 * t * t, 0.0), // 2nd-order
///     Vector4::new(0.0, 6.0, 24.0 * t, 0.0), // 3rd-order
/// ];
/// let mut evals = [Vector3::zero(); 4];
/// rat_ders(&ders, &mut evals);
///
/// // the projected curve: \bar{c}(t) = (t, t^2, t^3)
/// let ans = [
///     Vector3::new(t, t * t, t * t * t), // 0th-order
///     Vector3::new(1.0, 2.0 * t, 3.0 * t * t), // 1st-order
///     Vector3::new(0.0, 2.0, 6.0 * t), // 2nd-order
///     Vector3::new(0.0, 0.0, 6.0), // 3rd-order
/// ];
/// assert_eq!(evals, ans);
/// ```
pub fn rat_ders<S, V, Diff>(ders: &[V], evals: &mut [Diff])
where
    S: BaseFloat,
    V: Homogeneous<S>,
    V::Point: EuclideanSpace<Diff = Diff>,
    Diff: VectorSpace<Scalar = S>, {
    for i in 0..ders.len() {
        let mut c = 1;
        let sum = (1..i).fold(evals[0] * ders[i].weight(), |sum, j| {
            c = c * (i - j + 1) / j;
            sum + evals[j] * (ders[i - j].weight() * S::from(c).unwrap())
        });
        evals[i] = (ders[i].truncate() - sum) / ders[0].weight();
    }
}

/// Returns the multi-orders derivations of the rational surface.
/// # Examples
/// ```
/// use truck_base::cgmath64::*;
/// // calculate the derivation at (u, v) = (1.0, 2.0).
/// let (u, v) = (1.0, 2.0);
/// // the curve: s(u, v) = (u^3 v^2, u^2 v^3, u v, u)
/// let ders: [[Vector4; 3]; 3] = [
///     [
///         // u-rank = 0, v-rank = 0
///         (u * u * u * v * v, u * u * v * v * v, u * v, u).into(),
///         // u-rank = 0, v-rank = 1
///         (2.0 * u * u * u * v, 3.0 * u * u * v * v, u, 0.0).into(),
///         // u-rank = 0, v-rank = 2
///         (2.0 * u * u * u, 6.0 * u * u * v, 0.0, 0.0).into(),
///     ],
///     [
///         // u-rank = 1, v-rank = 0
///         (3.0 * u * u * v * v, 2.0 * u * v * v * v, v, 1.0).into(),
///         // u-rank = 1, v-rank = 1
///         (6.0 * u * u * v, 6.0 * u * v * v, 1.0, 0.0).into(),
///         // u-rank = 1, v-rank = 2
///         (6.0 * u * u, 12.0 * u * v, 0.0, 0.0).into(),
///     ],
///     [
///         // u-rank = 2, v-rank = 0
///         (6.0 * u * v * v, 2.0 * v * v * v, 0.0, 0.0).into(),
///         // u-rank = 2, v-rank = 1
///         (12.0 * u * v, 6.0 * v * v, 0.0, 0.0).into(),
///         // u-rank = 2, v-rank = 2
///         (12.0 * u, 12.0 * v, 0.0, 0.0).into(),
///     ],
/// ];
///
/// // the projected surface: \bar{s}(u, v) = (u^2 v^2, u v^3, v)
/// assert_eq!(multi_rat_der(&ders), Vector3::new(4.0, 0.0, 0.0));
/// ```
pub fn multi_rat_der<S, V, Diff, A>(ders: &[A]) -> Diff
where
    S: BaseFloat,
    V: Homogeneous<S>,
    V::Point: EuclideanSpace<Diff = Diff>,
    Diff: VectorSpace<Scalar = S>,
    A: AsRef<[V]>, {
    if ders.is_empty() {
        return Diff::zero();
    }
    let (m, n) = (ders.len(), ders[0].as_ref().len());
    if n == 0 {
        Diff::zero()
    } else if (m, n) == (1, 1) {
        ders[0].as_ref()[0].to_point().to_vec()
    } else if m == 1 {
        rat_der(ders[0].as_ref())
    } else if (m, n) == (2, 1) {
        rat_der(&[ders[0].as_ref()[0], ders[1].as_ref()[0]])
    } else if n == 1 && m < 32 {
        let mut vders = [V::zero(); 32];
        for (vder, array) in vders.iter_mut().zip(ders) {
            *vder = array.as_ref()[0];
        }
        rat_der(&vders[..m])
    } else if n == 1 {
        let mut vders = vec![V::zero(); m];
        for (vder, array) in vders.iter_mut().zip(ders) {
            *vder = array.as_ref()[0];
        }
        rat_der(&vders)
    } else if (m, n) == (2, 2) {
        let two = S::from(2).unwrap();
        let (der0, der1) = (ders[0].as_ref(), ders[1].as_ref());
        let (s, u, v, uv) = (der0[0], der1[0], der0[1], der1[1]);
        let (s, sw) = (s.truncate(), s.weight());
        let (u, uw) = (u.truncate(), u.weight());
        let (v, vw) = (v.truncate(), v.weight());
        let (uv, uvw) = (uv.truncate(), uv.weight());
        let sw2 = sw * sw;
        uv / sw - u * (vw / sw2) - v * (uw / sw2) + s * (uw * vw * two / (sw2 * sw) - uvw / sw2)
    } else if m < 8 && n < 8 {
        let mut evals = [[Diff::zero(); 8]; 8];
        multi_rat_ders(ders, &mut evals);
        evals[m - 1][n - 1]
    } else {
        let mut evals = vec![vec![Diff::zero(); m]; n];
        multi_rat_ders(ders, &mut evals);
        evals[m - 1][n - 1]
    }
}
/// Store the multi-orders derivations of the rational surface in `evals`.
/// # Remarks
/// - `evals` must be initialized by zero.
/// - `ders` and `evals` are must be rectangulars.
/// - The lengths of `evals` must not be shorter than the ones of `ders`.
/// # Examples
/// ```
/// use truck_base::cgmath64::*;
/// // calculate the derivation at (u, v) = (1.0, 2.0).
/// let (u, v) = (1.0, 2.0);
/// // the curve: s(u, v) = (u^3 v^2, u^2 v^3, u v, u)
/// let ders: [[Vector4; 3]; 3] = [
///     [
///         // u-rank = 0, v-rank = 0
///         (u * u * u * v * v, u * u * v * v * v, u * v, u).into(),
///         // u-rank = 0, v-rank = 1
///         (2.0 * u * u * u * v, 3.0 * u * u * v * v, u, 0.0).into(),
///         // u-rank = 0, v-rank = 2
///         (2.0 * u * u * u, 6.0 * u * u * v, 0.0, 0.0).into(),
///     ],
///     [
///         // u-rank = 1, v-rank = 0
///         (3.0 * u * u * v * v, 2.0 * u * v * v * v, v, 1.0).into(),
///         // u-rank = 1, v-rank = 1
///         (6.0 * u * u * v, 6.0 * u * v * v, 1.0, 0.0).into(),
///         // u-rank = 1, v-rank = 2
///         (6.0 * u * u, 12.0 * u * v, 0.0, 0.0).into(),
///     ],
///     [
///         // u-rank = 2, v-rank = 0
///         (6.0 * u * v * v, 2.0 * v * v * v, 0.0, 0.0).into(),
///         // u-rank = 2, v-rank = 1
///         (12.0 * u * v, 6.0 * v * v, 0.0, 0.0).into(),
///         // u-rank = 2, v-rank = 2
///         (12.0 * u, 12.0 * v, 0.0, 0.0).into(),
///     ],
/// ];
/// // evals must be initialized by zero.
/// let mut evals = [[Vector3::zero(); 3]; 3];
/// multi_rat_ders(&ders, &mut evals);
///
/// // the projected surface: \bar{s}(u, v) = (u^2 v^2, u v^3, v)
/// let ans: [[Vector3; 3]; 3] = [
///     [
///         // u-rank = 0, v-rank = 0
///         (u * u * v * v, u * v * v * v, v).into(),
///         // u-rank = 0, v-rank = 1
///         (2.0 * u * u * v, 3.0 * u * v * v, 1.0).into(),
///         // u-rank = 0, v-rank = 2
///         (2.0 * u * u, 6.0 * u * v, 0.0).into(),
///     ],
///     [
///         // u-rank = 1, v-rank = 0
///         (2.0 * u * v * v, v * v * v, 0.0).into(),
///         // u-rank = 1, v-rank = 1
///         (4.0 * u * v, 3.0 * v * v, 0.0).into(),
///         // u-rank = 1, v-rank = 2
///         (4.0 * u, 6.0 * v, 0.0).into(),
///     ],
///     [
///         // u-rank = 2, v-rank = 0
///         (2.0 * v * v, 0.0, 0.0).into(),
///         // u-rank = 2, v-rank = 1
///         (4.0 * v, 0.0, 0.0).into(),
///         // u-rank = 2, v-rank = 2
///         (4.0, 0.0, 0.0).into(),
///     ],
/// ];
/// assert_eq!(evals, ans);
/// ```
pub fn multi_rat_ders<S, V, Diff, A0, A1>(ders: &[A0], evals: &mut [A1])
where
    S: BaseFloat,
    V: Homogeneous<S>,
    V::Point: EuclideanSpace<Diff = Diff>,
    Diff: VectorSpace<Scalar = S>,
    A0: AsRef<[V]>,
    A1: AsMut<[Diff]>, {
    let (m_max, n_max) = (ders.len(), ders[0].as_ref().len());
    for m in 0..m_max {
        for n in 0..n_max {
            let mut sum = Diff::zero();
            let mut c0 = 1;
            for i in 0..=m {
                let mut c1 = 1;
                let (evals, ders) = (evals[i].as_mut(), ders[m - i].as_ref());
                for j in 0..=n {
                    let (c0_s, c1_s) = (S::from(c0).unwrap(), S::from(c1).unwrap());
                    sum = sum + evals[j] * (ders[n - j].weight() * c0_s * c1_s);
                    c1 = c1 * (n - j) / (j + 1);
                }
                c0 = c0 * (m - i) / (i + 1);
            }
            let (eval_mn, der_mn) = (&mut evals[m].as_mut()[n], ders[m].as_ref()[n]);
            *eval_mn = (der_mn.truncate() - sum) / ders[0].as_ref()[0].weight();
        }
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
