use crate::cgmath_extend_traits::*;
use cgmath::*;

/// Maximum order that guarantees differential calculations
pub const MAX_DER_ORDER: usize = 31;

/// Calculation results of curve differentiation
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CurveDers<V> {
    array: [V; MAX_DER_ORDER + 1],
    max_order: usize,
}

impl<V> CurveDers<V> {
    /// Construct zeroed `CurveDers` with maximum order = `max_order`.
    #[inline]
    pub fn new(max_order: usize) -> Self
    where V: Zero + Copy {
        Self {
            array: [V::zero(); MAX_DER_ORDER + 1],
            max_order,
        }
    }
    /// Returns the maximum order
    #[inline]
    pub const fn max_order(&self) -> usize { self.max_order }
    /// Appends an element to the back of a collection.
    #[inline]
    pub fn push(&mut self, value: V) {
        self.max_order += 1;
        self.array[self.max_order] = value;
    }

    /// Returns the multi-orders derivations of the rational curve.
    /// # Examples
    /// ```
    /// use truck_base::cgmath64::*;
    /// // calculate the derivation at t = 1.5
    /// let t = 1.5;
    ///
    /// // the curve: c(t) = (t^2, t^3, t^4, t)
    /// let raw_ders = [
    ///     Vector4::new(t * t, t * t * t, t * t * t * t, t), // 0th-order
    ///     Vector4::new(2.0 * t, 3.0 * t * t, 4.0 * t * t * t, 1.0), // 1st-order
    ///     Vector4::new(2.0, 6.0 * t, 12.0 * t * t, 0.0), // 2nd-order
    ///     Vector4::new(0.0, 6.0, 24.0 * t, 0.0), // 3rd-order
    /// ];
    /// let ders = CurveDers::try_from(raw_ders).unwrap();
    /// let rat_ders = ders.rat_ders();
    ///
    /// // the projected curve: \bar{c}(t) = (t, t^2, t^3)
    /// let raw_ans = [
    ///     Vector3::new(t, t * t, t * t * t), // 0th-order
    ///     Vector3::new(1.0, 2.0 * t, 3.0 * t * t), // 1st-order
    ///     Vector3::new(0.0, 2.0, 6.0 * t), // 2nd-order
    ///     Vector3::new(0.0, 0.0, 6.0), // 3rd-order
    /// ];
    /// let ans = CurveDers::try_from(raw_ans).unwrap();
    /// assert_eq!(rat_ders, ans);
    /// ```
    pub fn rat_ders<S>(&self) -> CurveDers<<V::Point as EuclideanSpace>::Diff>
    where
        S: BaseFloat,
        V: Homogeneous<S>,
        V::Point: EuclideanSpace<Scalar = V::Scalar>, {
        let mut evals = [<V::Point as EuclideanSpace>::Diff::zero(); MAX_DER_ORDER + 1];
        for i in 0..=self.max_order {
            let mut c = 1;
            let sum = (1..i).fold(evals[0] * self[i].weight(), |sum, j| {
                c = c * (i - j + 1) / j;
                sum + evals[j] * (self[i - j].weight() * S::from(c).unwrap())
            });
            evals[i] = (self[i].truncate() - sum) / self[0].weight();
        }
        CurveDers {
            array: evals,
            max_order: self.max_order,
        }
    }

    /// Returns the multi-orders derivations of the magnitude of the curve of vector.
    /// # Examples
    /// ```
    /// use truck_base::{assert_near, cgmath64::*};
    ///
    /// let t = 0.3;
    ///
    /// // c(t) = (2t, 1 - t^2) -> |c(t)| = 1 + t^2
    /// let raw_ders = [
    ///     Vector2::new(2.0 * t, 1.0 - t * t),
    ///     Vector2::new(2.0, -2.0 * t),
    ///     Vector2::new(0.0, -2.0),
    ///     Vector2::new(0.0, 0.0),
    /// ];
    /// let ders = CurveDers::try_from(raw_ders).unwrap();
    /// let abs_ders = ders.abs_ders();
    ///
    /// assert_near!(abs_ders[0], 1.0 + t * t);
    /// assert_near!(abs_ders[1], 2.0 * t);
    /// assert_near!(abs_ders[2], 2.0);
    /// assert_near!(abs_ders[3], 0.0);
    /// ```
    pub fn abs_ders(&self) -> CurveDers<V::Scalar>
    where
        V: InnerSpace,
        V::Scalar: BaseFloat, {
        use cgmath::num_traits::NumCast;
        let mut evals = [V::Scalar::zero(); MAX_DER_ORDER + 1];
        evals[0] = self[0].magnitude();
        (1..=self.max_order).for_each(|m| {
            let mut c = 1;
            let sum = (0..m).fold(V::Scalar::zero(), |mut sum, i| {
                let x = self[i + 1].dot(self[m - 1 - i]);
                let y = evals[i + 1] * evals[m - 1 - i];
                let c_float = <V::Scalar as NumCast>::from(c).unwrap();
                sum += (x - y) * c_float;
                c = c * (m - 1 - i) / (i + 1);
                sum
            });
            evals[m] = sum / evals[0];
        });
        CurveDers {
            array: evals,
            max_order: self.max_order,
        }
    }
}

impl<V> std::ops::Deref for CurveDers<V> {
    type Target = [V];
    #[inline]
    fn deref(&self) -> &[V] { &self.array[..=self.max_order] }
}

impl<V> std::ops::DerefMut for CurveDers<V> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [V] { &mut self.array[..=self.max_order] }
}

impl<V: Zero + Copy, const N: usize> TryFrom<[V; N]> for CurveDers<V> {
    type Error = &'static str;
    fn try_from(value: [V; N]) -> Result<Self, Self::Error> {
        if N == 0 {
            Err("empty array cannot convert to CurveDers.")
        } else if N > MAX_DER_ORDER + 1 {
            Err("the length of CurveDers must be less than MAX_DER_ORDER + 1.")
        } else {
            let mut array = [V::zero(); MAX_DER_ORDER + 1];
            array[..N].copy_from_slice(&value);
            Ok(Self {
                array,
                max_order: N - 1,
            })
        }
    }
}

impl<V: Zero + Copy> TryFrom<&[V]> for CurveDers<V> {
    type Error = &'static str;
    fn try_from(value: &[V]) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err("empty slice cannot convert CurveDers.")
        } else if value.len() > MAX_DER_ORDER + 1 {
            Err("the length of CurveDers must be less than MAX_DER_ORDER + 1.")
        } else {
            let mut array = [V::zero(); MAX_DER_ORDER + 1];
            array[..value.len()].copy_from_slice(&value);
            Ok(Self {
                array,
                max_order: value.len() - 1,
            })
        }
    }
}

/// Calculation results of surface differentiation
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceDers<V> {
    array: [[V; MAX_DER_ORDER + 1]; MAX_DER_ORDER + 1],
    max_order: usize,
}

impl<V> SurfaceDers<V> {
    /// Construct zeroed `SurfaceDers` with maximum order = `max_order`.
    #[inline]
    pub fn new(max_order: usize) -> Self
    where V: Zero + Copy {
        Self {
            array: [[V::zero(); MAX_DER_ORDER + 1]; MAX_DER_ORDER + 1],
            max_order,
        }
    }
    /// Returns maximum order
    #[inline]
    pub const fn max_order(&self) -> usize { self.max_order }
    /// Returns the vector of slices
    /// # Examples
    /// ```
    /// use truck_base::ders::*;
    /// let ders = SurfaceDers::<f64>::new(5);
    /// let slices = ders.slice_vec();
    /// assert_eq!(slices.len(), 6);
    /// assert_eq!(slices[0].len(), 6);
    /// assert_eq!(slices[1].len(), 5);
    /// assert_eq!(slices[2].len(), 4);
    /// assert_eq!(slices[3].len(), 3);
    /// assert_eq!(slices[4].len(), 2);
    /// assert_eq!(slices[5].len(), 1);
    /// ```
    #[inline]
    pub fn slice_vec(&self) -> Vec<&[V]> {
        self.array[..=self.max_order]
            .iter()
            .enumerate()
            .map(|(i, arr)| &arr[..=self.max_order - i])
            .collect()
    }
    /// Returns the vector of slices
    /// # Examples
    /// ```
    /// use truck_base::ders::*;
    /// let mut ders = SurfaceDers::<f64>::new(5);
    /// let slices = ders.slice_mut_vec();
    /// assert_eq!(slices.len(), 6);
    /// assert_eq!(slices[0].len(), 6);
    /// assert_eq!(slices[1].len(), 5);
    /// assert_eq!(slices[2].len(), 4);
    /// assert_eq!(slices[3].len(), 3);
    /// assert_eq!(slices[4].len(), 2);
    /// assert_eq!(slices[5].len(), 1);
    /// ```
    #[inline]
    pub fn slice_mut_vec(&mut self) -> Vec<&mut [V]> {
        self.array[..=self.max_order]
            .iter_mut()
            .enumerate()
            .map(|(i, arr)| &mut arr[..=self.max_order - i])
            .collect()
    }
    /// Returns the multi-orders derivations of the rational surface.
    /// # Examples
    /// ```
    /// use truck_base::cgmath64::*;
    /// // calculate the derivation at (u, v) = (1.0, 2.0).
    /// let (u, v) = (1.0, 2.0);
    /// // the curve: s(u, v) = (u^3 v^2, u^2 v^3, u v, u)
    /// let raw_ders: &[&[Vector4]] = &[
    ///     &[
    ///         // u-rank = 0, v-rank = 0
    ///         (u * u * u * v * v, u * u * v * v * v, u * v, u).into(),
    ///         // u-rank = 0, v-rank = 1
    ///         (2.0 * u * u * u * v, 3.0 * u * u * v * v, u, 0.0).into(),
    ///         // u-rank = 0, v-rank = 2
    ///         (2.0 * u * u * u, 6.0 * u * u * v, 0.0, 0.0).into(),
    ///     ],
    ///     &[
    ///         // u-rank = 1, v-rank = 0
    ///         (3.0 * u * u * v * v, 2.0 * u * v * v * v, v, 1.0).into(),
    ///         // u-rank = 1, v-rank = 1
    ///         (6.0 * u * u * v, 6.0 * u * v * v, 1.0, 0.0).into(),
    ///     ],
    ///     &[
    ///         // u-rank = 2, v-rank = 0
    ///         (6.0 * u * v * v, 2.0 * v * v * v, 0.0, 0.0).into(),
    ///     ],
    /// ];
    /// let mut ders = SurfaceDers::try_from(raw_ders).unwrap();
    /// let rat_ders = ders.rat_ders();
    ///
    /// // the projected surface: \bar{s}(u, v) = (u^2 v^2, u v^3, v)
    /// let raw_ans: &[&[Vector3]] = &[
    ///     &[
    ///         // u-rank = 0, v-rank = 0
    ///         (u * u * v * v, u * v * v * v, v).into(),
    ///         // u-rank = 0, v-rank = 1
    ///         (2.0 * u * u * v, 3.0 * u * v * v, 1.0).into(),
    ///         // u-rank = 0, v-rank = 2
    ///         (2.0 * u * u, 6.0 * u * v, 0.0).into(),
    ///     ],
    ///     &[
    ///         // u-rank = 1, v-rank = 0
    ///         (2.0 * u * v * v, v * v * v, 0.0).into(),
    ///         // u-rank = 1, v-rank = 1
    ///         (4.0 * u * v, 3.0 * v * v, 0.0).into(),
    ///     ],
    ///     &[
    ///         // u-rank = 2, v-rank = 0
    ///         (2.0 * v * v, 0.0, 0.0).into(),
    ///     ],
    /// ];
    /// let mut ans_ders = SurfaceDers::try_from(raw_ans).unwrap();
    ///
    /// assert_eq!(rat_ders, ans_ders);
    /// ```
    pub fn rat_ders<S>(&self) -> SurfaceDers<<V::Point as EuclideanSpace>::Diff>
    where
        S: BaseFloat,
        V: Homogeneous<S>,
        V::Point: EuclideanSpace<Scalar = V::Scalar>, {
        let zero = <V::Point as EuclideanSpace>::Diff::zero();
        let mut evals = [[zero; MAX_DER_ORDER + 1]; MAX_DER_ORDER + 1];
        for m in 0..=self.max_order {
            for n in 0..=self.max_order - m {
                let mut sum = zero;
                let mut c0 = 1;
                for i in 0..=m {
                    let mut c1 = 1;
                    let (evals, ders) = (evals[i].as_mut(), &self[m - i]);
                    for j in 0..=n {
                        let (c0_s, c1_s) = (S::from(c0).unwrap(), S::from(c1).unwrap());
                        sum = sum + evals[j] * (ders[n - j].weight() * c0_s * c1_s);
                        c1 = c1 * (n - j) / (j + 1);
                    }
                    c0 = c0 * (m - i) / (i + 1);
                }
                let (eval_mn, der_mn) = (&mut evals[m].as_mut()[n], self[m][n]);
                *eval_mn = (der_mn.truncate() - sum) / self[0][0].weight();
            }
        }
        SurfaceDers {
            array: evals,
            max_order: self.max_order,
        }
    }
    /// Returns the derivation of composite curve.
    /// # Example
    /// ```
    /// use truck_base::cgmath64::*;
    ///
    /// let t = 2.0;
    ///
    /// // s(u, v) = u^3 + v, c(t) = (t, t^2), s(c(t)) = t^3 + t^2
    ///
    /// let raw_cders: [Vector2; 3] = [
    ///     (t, t * t).into(),
    ///     (1.0, 2.0 * t).into(),
    ///     (0.0, 2.0).into(),
    /// ];
    /// let cders = CurveDers::try_from(raw_cders).unwrap();
    ///
    /// let Vector2 { x: u, y: v } = cders[0];
    /// let raw_ders: &[&[Vector1]] = &[
    ///     &[(u * u * u + v,).into(), (1.0,).into(), (0.0,).into()],
    ///     &[(3.0 * u * u,).into(), (0.0,).into()],
    ///     &[(6.0 * u,).into()],
    /// ];
    /// let sders = SurfaceDers::try_from(raw_ders).unwrap();
    ///
    /// let der = sders.composite_der(&cders, 1);
    /// let ans = Vector1::new(3.0 * t * t + 2.0 * t);
    /// assert_eq!(der, ans);
    /// ```
    pub fn composite_der(&self, curve_ders: &CurveDers<Vector2<V::Scalar>>, order: usize) -> V
    where
        V: VectorSpace,
        V::Scalar: BaseFloat, {
        use cgmath::num_traits::NumCast;
        if order > self.max_order || order > curve_ders.max_order {
            panic!("calculating derivative with order={order}, but the orders of given derivatives are less than {order}.");
        }
        (1..=order).fold(V::zero(), |sum, len| {
            let iter = CompositionIter::<32>::try_new(order, len).unwrap();
            iter.fold(sum, |sum, idx| {
                let idx = &idx[..len];
                let mult = <V::Scalar as NumCast>::from(multiplicity(idx)).unwrap();
                sum + tensor(self, curve_ders, idx) * mult
            })
        })
    }
    /// Returns the derivation of composite curve.
    /// # Example
    /// ```
    /// use truck_base::cgmath64::*;
    ///
    /// let t = 2.0;
    ///
    /// // s(u, v) = u^3 + v, c(t) = (t, t^2), s(c(t)) = t^3 + t^2
    ///
    /// let raw_cders: [Vector2; 3] = [
    ///     (t, t * t).into(),
    ///     (1.0, 2.0 * t).into(),
    ///     (0.0, 2.0).into(),
    /// ];
    /// let cders = CurveDers::try_from(raw_cders).unwrap();
    ///
    /// let Vector2 { x: u, y: v } = cders[0];
    /// let raw_ders: &[&[Vector1]] = &[
    ///     &[(u * u * u + v,).into(), (1.0,).into(), (0.0,).into()],
    ///     &[(3.0 * u * u,).into(), (0.0,).into()],
    ///     &[(6.0 * u,).into()],
    /// ];
    /// let sders = SurfaceDers::try_from(raw_ders).unwrap();
    /// let ders = sders.composite_ders(&cders);
    ///
    /// let raw_ans: &[Vector1] = &[
    ///     (t * t * t + t * t,).into(),
    ///     (3.0 * t * t + 2.0 * t,).into(),
    ///     (6.0 * t + 2.0,).into(),
    /// ];
    /// let ans = CurveDers::try_from(raw_ans).unwrap();
    /// assert_eq!(ders, ans);
    pub fn composite_ders(&self, curve_ders: &CurveDers<Vector2<V::Scalar>>) -> CurveDers<V>
    where
        V: VectorSpace,
        V::Scalar: BaseFloat, {
        let mut res = CurveDers::new(curve_ders.max_order);
        res[0] = self[0][0];
        let iter = res[1..].iter_mut().enumerate();
        iter.for_each(|(i, o)| *o = self.composite_der(curve_ders, i + 1));
        res
    }
}

impl<V> std::ops::Index<usize> for SurfaceDers<V> {
    type Output = [V];
    fn index(&self, index: usize) -> &[V] {
        if index > self.max_order {
            panic!("the index must be no more than {}.", self.max_order);
        }
        &self.array[index][..=self.max_order - index]
    }
}

impl<V> std::ops::IndexMut<usize> for SurfaceDers<V> {
    fn index_mut(&mut self, index: usize) -> &mut [V] {
        if index > self.max_order {
            panic!("the index must be no more than {}.", self.max_order);
        }
        &mut self.array[index][..=self.max_order - index]
    }
}

impl<V: Zero + Copy> TryFrom<&[&[V]]> for SurfaceDers<V> {
    type Error = &'static str;
    fn try_from(value: &[&[V]]) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err("Empty array cannot convert to `SurfaceDers`.");
        }
        let mut array = [[V::zero(); MAX_DER_ORDER + 1]; MAX_DER_ORDER + 1];
        let max_order = value.len() - 1;

        let mut iter = value.iter().zip(&mut array).enumerate();
        iter.try_for_each(|(i, (&slice, subarray))| {
            if i + slice.len() != max_order + 1 {
                Err("Inconsistent slice length and order.")
            } else {
                subarray[..=max_order - i].copy_from_slice(slice);
                Ok(())
            }
        })?;

        Ok(Self { array, max_order })
    }
}

fn can_init(len: usize, n: usize, max: usize) -> bool { !(len > n || max * len < n) }

fn init(array: &mut [usize], n: usize, max: usize) {
    if array.is_empty() {
        return;
    }
    array[0] = (n - array.len() + 1).min(max);
    let (n, max) = (n - array[0], array[0]);
    init(&mut array[1..], n, max)
}

fn next(array: &mut [usize]) -> bool {
    let n = array[1..].iter().sum::<usize>() + 1;
    let max = array[0] - 1;
    if array.len() == 1 {
        false
    } else if next(&mut array[1..]) {
        true
    } else if can_init(array.len() - 1, n, max) {
        array[0] -= 1;
        init(&mut array[1..], n, max);
        true
    } else {
        false
    }
}

#[derive(Clone, Debug)]
struct CompositionIter<const MAX: usize> {
    current: [usize; MAX],
    end: bool,
    len: usize,
}

impl<const MAX: usize> CompositionIter<MAX> {
    fn try_new(n: usize, len: usize) -> Option<Self> {
        if !(len < MAX && can_init(len, n, n)) {
            return None;
        }
        let mut current = [0; MAX];
        init(&mut current[..len], n, n);
        Some(Self {
            current,
            len,
            end: false,
        })
    }
}

impl<const MAX: usize> Iterator for CompositionIter<MAX> {
    type Item = [usize; MAX];
    fn next(&mut self) -> Option<Self::Item> {
        if self.end {
            return None;
        }
        let current = self.current;
        self.end = !next(&mut self.current[..self.len]);
        Some(current)
    }
}

fn factorial(n: usize) -> u128 { (2..=n).fold(1, |f, i| f * i as u128) }

fn multiplicity(array: &[usize]) -> u128 {
    let n = array.iter().sum::<usize>();
    let mut res = factorial(n);
    array.iter().for_each(|&a| res /= factorial(a));
    let mut mult = 1;
    array.windows(2).for_each(|x| {
        if x[0] == x[1] {
            mult += 1;
        } else {
            res /= factorial(mult);
            mult = 1;
        }
    });
    res / factorial(mult)
}

fn tensor<S, V, A>(sder: &A, cder: &[Vector2<S>], idx: &[usize]) -> V
where
    S: BaseFloat,
    V: VectorSpace<Scalar = S>,
    A: std::ops::Index<usize, Output = [V]>, {
    let n: u128 = 2u128.pow(idx.len() as u32);
    (0..n).fold(V::zero(), |sum, mut i| {
        let (t, mult) = idx.iter().fold((0, S::one()), |(t, mult), &j| {
            let k = (i % 2) as usize;
            i /= 2;
            (t + k, mult * cder[j][k])
        });
        sum + sder[idx.len() - t][t] * mult
    })
}

#[test]
fn test_composition_iter() {
    let iter = CompositionIter::<8>::try_new(10, 4).unwrap();
    let vec: Vec<_> = iter.collect();
    let iter = vec.iter().map(|idx| {
        idx[..4].iter().for_each(|&i| assert_ne!(i, 0));
        idx[4..].iter().for_each(|&i| assert_eq!(i, 0));
        &idx[..4]
    });
    let vec: Vec<_> = iter.collect();

    assert_eq!(vec.len(), 9);
    assert_eq!(vec[0], &[7, 1, 1, 1]);
    assert_eq!(vec[1], &[6, 2, 1, 1]);
    assert_eq!(vec[2], &[5, 3, 1, 1]);
    assert_eq!(vec[3], &[5, 2, 2, 1]);
    assert_eq!(vec[4], &[4, 4, 1, 1]);
    assert_eq!(vec[5], &[4, 3, 2, 1]);
    assert_eq!(vec[6], &[4, 2, 2, 2]);
    assert_eq!(vec[7], &[3, 3, 3, 1]);
    assert_eq!(vec[8], &[3, 3, 2, 2]);
}
