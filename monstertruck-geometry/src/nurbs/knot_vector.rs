use super::*;
use crate::errors::Error;
use smallvec::SmallVec;
use std::slice::SliceIndex;
use std::vec::Vec;

#[inline(always)]
fn inv_or_zero_strict(delta: f64) -> f64 { if delta == 0.0 { 0.0 } else { 1.0 / delta } }

impl KnotVector {
    /// empty constructor
    pub const fn new() -> KnotVector { KnotVector(Vec::new()) }

    /// Returns the length of range.
    /// # Examples
    /// ```
    /// use monstertruck_geometry::prelude::*;
    /// let knot_vec = KnotVector::from(vec![0.0, 6.0]);
    /// assert_eq!(knot_vec.range_length(), 6.0);
    /// ```
    #[inline(always)]
    pub fn range_length(&self) -> f64 {
        match self.is_empty() {
            true => 0.0,
            false => self[self.len() - 1] - self[0],
        }
    }

    /// Returns whether two knot vectors have the same range.
    /// # Examples
    /// ```
    /// use monstertruck_geometry::prelude::*;
    /// let knot_vec0 = KnotVector::new(); // empty knot vector
    /// let knot_vec1 = KnotVector::from(vec![0.0, 0.0, 1.0, 1.0]);
    /// let knot_vec2 = KnotVector::from(vec![0.0, 0.5, 1.0]);
    /// let knot_vec3 = KnotVector::from(vec![0.0, 0.0, 2.0, 2.0]);
    /// assert!(knot_vec0.same_range(&KnotVector::new())); // both empty knot vector
    /// assert!(!knot_vec0.same_range(&knot_vec1));
    /// assert!(knot_vec1.same_range(&knot_vec2)); // the range of both knot vector is [0, 1].
    /// assert!(!knot_vec1.same_range(&knot_vec3));
    /// ```
    #[inline(always)]
    pub fn same_range(&self, other: &KnotVector) -> bool {
        match (self.is_empty(), other.is_empty()) {
            (false, false) => {
                self[0].near(&other[0]) && self.range_length().near(&other.range_length())
            }
            (true, true) => true,
            _ => false,
        }
    }

    /// Removes one item.
    #[inline(always)]
    pub fn remove(&mut self, idx: usize) -> f64 { self.0.remove(idx) }

    /// Returns the maximum index `i` of `self[i] <= x`
    /// Return `None` if `x < self[0] or self.len() == 0`.
    /// # Examples
    /// ```
    /// use monstertruck_geometry::prelude::KnotVector;
    /// let mut knot_vec = KnotVector::from(vec![0.0, 0.0, 1.0, 2.0, 3.0, 3.0]);
    /// let idx = knot_vec.floor(1.5).unwrap();
    /// assert_eq!(idx, 2);
    /// ```
    #[inline(always)]
    pub fn floor(&self, x: f64) -> Option<usize> { self.iter().rposition(|t| *t <= x) }

    /// Returns the multiplicity of the `i`th knot
    /// # Examples
    /// ```
    /// use monstertruck_geometry::prelude::KnotVector;
    /// let knot_vec = KnotVector::from(vec![0.0, 0.0, 1.0, 2.0, 2.0, 2.0, 3.0, 3.0]);
    /// assert_eq!(knot_vec.multiplicity(5), 3);
    /// ```
    #[inline(always)]
    pub fn multiplicity(&self, i: usize) -> usize {
        self.iter().filter(|u| self[i].near(u)).count()
    }

    #[inline(always)]
    fn strict_multiplicity(&self, i: usize) -> usize {
        let knot = self[i];
        self.iter().filter(|value| **value == knot).count()
    }

    /// Adds a knot and return the index of the added knot.
    /// # Examples
    /// ```
    /// use monstertruck_geometry::prelude::KnotVector;
    /// let mut knot_vec = KnotVector::from(vec![0.0, 0.0, 1.0, 2.0, 3.0, 3.0]);
    /// let idx0 = knot_vec.add_knot(1.5);
    /// assert_eq!(idx0, 3);
    /// let idx1 = knot_vec.add_knot(-1.0);
    /// assert_eq!(idx1, 0);
    /// let ansvec = KnotVector::from(vec![-1.0, 0.0, 0.0, 1.0, 1.5, 2.0, 3.0, 3.0]);
    /// assert_eq!(knot_vec, ansvec);
    /// ```
    #[inline(always)]
    pub fn add_knot(&mut self, knot: f64) -> usize {
        match self.floor(knot) {
            Some(idx) => {
                self.0.insert(idx + 1, knot);
                idx + 1
            }
            None => {
                self.0.insert(0, knot);
                0
            }
        }
    }

    /// Calculates B-spline basis functions at `t` with degree `degree` and `der_rank`th-order derivation.
    /// # Panics
    /// If the length of `self` is not more than `degree`, panic occurs.
    /// # Remarks
    /// In this package, the B-spline basis function is based on the characteristic function of
    /// the right-open intervals [s, t). So, the value corresponding to the end point t = t_n is always 0.0.
    /// # Examples
    /// ```
    /// use monstertruck_geometry::prelude::*;
    /// const N : usize = 100; // sample size in tests
    ///
    /// // B-spline basis functions is a partition of unity in (t_k, t_{n - k}).
    /// let vec = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    /// let knot_vec = KnotVector::from(vec);
    /// let degree = 2;
    /// for i in 0..N {
    ///     let t = 2.0 + 4.0 / (N as f64) * (i as f64);
    ///     let res = knot_vec.bspline_basis_functions(degree, 0, t);
    ///     let sum = res.iter().fold(0.0, |sum, a| sum + a);
    ///     assert_near2!(sum, 1.0);
    /// }
    /// ```
    /// ```
    /// use monstertruck_geometry::prelude::*;
    /// const N : usize = 100; // sample size in tests
    ///
    /// // In some case, B-spline basis functions coincide with Bernstein polynomials.
    /// let vec = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
    /// let knot_vec = KnotVector::from(vec);
    /// let degree = 3;
    /// for i in 0..=N {
    ///     let t = 1.0 / (N as f64) * (i as f64);
    ///     // substitution
    ///     let res = knot_vec.bspline_basis_functions(degree, 0, t);
    ///     let ans = [
    ///         1.0 * (1.0 - t) * (1.0 - t) * (1.0 - t),
    ///         3.0 * t * (1.0 - t) * (1.0 - t),
    ///         3.0 * t * t * (1.0 - t),
    ///         1.0 * t * t * t,
    ///     ];
    ///     for i in 0..4 { assert_near2!(res[i], ans[i]); }
    ///
    ///     // 2nd-order derivation
    ///     let res = knot_vec.bspline_basis_functions(degree, 2, t);
    ///     let ans = [
    ///         6.0 * (1.0 - t),
    ///         6.0 * (3.0 * t - 2.0),
    ///         6.0 * (1.0 - 3.0 * t),
    ///         6.0 * t,
    ///     ];
    ///     for i in 0..4 { assert_near2!(res[i], ans[i]); }
    /// }
    /// ```
    pub fn bspline_basis_functions(
        &self,
        degree: usize,
        der_rank: usize,
        t: f64,
    ) -> SmallVec<[f64; 32]> {
        match self.try_bspline_basis_functions(degree, der_rank, t) {
            Ok(got) => got,
            Err(error) => panic!("{}", error),
        }
    }

    /// Calculates B-spline basis functions at `t` with degree `degree` and `der_rank`th-order derivation.
    /// # Failures
    /// - If the range of the knot vector is zero, returns [`Error::ZeroRange`].
    /// - If the length of `self` is not more than `degree`, returns [`Error::TooLargeDegree`].
    /// # Remarks
    /// In this package, the B-spline basis function is based on the characteristic function of
    /// the right-open intervals [s, t). So, the value corresponding to the end point t = t_n is always 0.0.
    /// # Examples
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// use monstertruck_geometry::prelude::*;
    /// const N : usize = 100; // sample size in tests
    ///
    /// // B-spline basis functions is a partition of unity in (t_k, t_{n - k}).
    /// let vec = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    /// let knot_vec = KnotVector::from(vec);
    /// let degree = 2;
    /// for i in 0..N {
    ///     let t = 2.0 + 4.0 / (N as f64) * (i as f64);
    ///     let res = knot_vec.try_bspline_basis_functions(degree, 0, t)?;
    ///     let sum = res.iter().fold(0.0, |sum, a| sum + a);
    ///     assert_near2!(sum, 1.0);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// use monstertruck_geometry::prelude::*;
    /// const N : usize = 100; // sample size in tests
    ///
    /// // In some case, B-spline basis functions coincide with Bernstein polynomials.
    /// let vec = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
    /// let knot_vec = KnotVector::from(vec);
    /// let degree = 3;
    /// for i in 0..=N {
    ///     let t = i as f64 / N as f64;
    ///     // substitution
    ///     let res = knot_vec.try_bspline_basis_functions(degree, 0, t)?;
    ///     let ans = [
    ///         1.0 * (1.0 - t) * (1.0 - t) * (1.0 - t),
    ///         3.0 * t * (1.0 - t) * (1.0 - t),
    ///         3.0 * t * t * (1.0 - t),
    ///         1.0 * t * t * t,
    ///     ];
    ///     for i in 0..4 { assert_near2!(res[i], ans[i]); }
    ///
    ///     // 2nd-order derivation
    ///     let res = knot_vec.try_bspline_basis_functions(degree, 2, t)?;
    ///     let ans = [
    ///         6.0 * (1.0 - t),
    ///         6.0 * (3.0 * t - 2.0),
    ///         6.0 * (1.0 - 3.0 * t),
    ///         6.0 * t,
    ///     ];
    ///     for i in 0..4 { assert_near2!(res[i], ans[i]); }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_bspline_basis_functions(
        &self,
        degree: usize,
        der_rank: usize,
        t: f64,
    ) -> Result<SmallVec<[f64; 32]>> {
        let n = self.len() - 1;
        if self[0] == self[n] {
            return Err(Error::ZeroRange);
        } else if n < degree {
            return Err(Error::TooLargeDegree(n + 1, degree));
        }
        if degree < der_rank {
            return Ok(smallvec::smallvec![0.0; n - degree]);
        }

        let idx = {
            let idx = self
                .floor(t)
                // SAFETY: `self[0]` is always in the knot vector, so `floor` succeeds.
                .unwrap_or_else(|| self.floor(self[0]).unwrap());
            if idx == n {
                n.saturating_sub(self.strict_multiplicity(n))
            } else {
                idx
            }
        };

        if n < 32 {
            let mut eval = [0.0; 32];
            self.sub_bspline_basis_functions(degree, der_rank, t, idx, &mut eval);
            Ok(SmallVec::from_slice(&eval[..n - degree]))
        } else {
            let mut eval = vec![0.0; n];
            self.sub_bspline_basis_functions(degree, der_rank, t, idx, &mut eval);
            eval.truncate(n - degree);
            Ok(SmallVec::from_vec(eval))
        }
    }

    fn sub_bspline_basis_functions(
        &self,
        degree: usize,
        der_rank: usize,
        t: f64,
        idx: usize,
        eval: &mut [f64],
    ) {
        let n = self.len() - 1;
        eval[idx] = 1.0;

        for k in 1..=(degree - der_rank) {
            let base = idx.saturating_sub(k);
            let delta = self[base + k] - self[base];
            let mut a = inv_or_zero_strict(delta) * (t - self[base]);
            for i in base..=usize::min(idx, n - k - 1) {
                let delta = self[i + k + 1] - self[i + 1];
                let b = inv_or_zero_strict(delta) * (self[i + k + 1] - t);
                eval[i] = a * eval[i] + b * eval[i + 1];
                a = 1.0 - b;
            }
        }

        for k in (degree - der_rank + 1)..=degree {
            let base = idx.saturating_sub(k);
            let delta = self[base + k] - self[base];
            let mut a = inv_or_zero_strict(delta);
            for i in base..=usize::min(idx, n - k - 1) {
                let delta = self[i + k + 1] - self[i + 1];
                let b = inv_or_zero_strict(delta);
                eval[i] = (a * eval[i] - b * eval[i + 1]) * k as f64;
                a = b;
            }
        }
    }

    #[doc(hidden)]
    pub fn maximum_points(&self, degree: usize) -> Vec<f64> {
        let n = self.len();
        let m = n - degree - 1;
        let range = self.range_length();
        const N: i32 = 100;

        let mut res = vec![0.0; m];
        let mut max = vec![0.0; m];
        for i in 1..N {
            let t = self[0] + range * (i as f64) / (N as f64);
            // SAFETY: `t` is within the knot vector range, `degree` is valid for
            // this knot vector, and the range is non-zero.
            let vals = self.try_bspline_basis_functions(degree, 0, t).unwrap();
            for j in 0..m {
                if max[j] < vals[j] {
                    max[j] = vals[j];
                    res[j] = t;
                }
            }
        }

        res
    }

    /// Transforms the knot vector
    /// # Examples
    /// ```
    /// use std::vec::Vec;
    /// use monstertruck_geometry::prelude::KnotVector;
    /// let mut knot_vec = KnotVector::from(vec![1.0, 1.0, 2.0, 3.0, 4.0, 5.0, 5.0]);
    /// knot_vec.transform(2.0, 3.0);
    /// let res : Vec<f64> = knot_vec.into();
    /// assert_eq!(res, vec![5.0, 5.0, 7.0, 9.0, 11.0, 13.0, 13.0]);
    /// ```
    /// # Panics
    /// Panic occurs if `scalar` is not positive.
    pub fn transform(&mut self, scalar: f64, r#move: f64) -> &mut Self {
        assert!(scalar > 0.0, "The scalar {scalar} is not positive.");
        self.0
            .iter_mut()
            .for_each(move |vec| *vec = *vec * scalar + r#move);
        self
    }

    /// Normalizes the knot vector i.e. makes the first value 0 and the last value 1.
    /// # Failures
    /// Returns [`Error::ZeroRange`] if the range of the knot vector is so small.
    /// # Examples
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// use monstertruck_geometry::prelude::KnotVector;
    /// let mut knot_vec = KnotVector::from(vec![1.0, 1.0, 2.0, 3.0, 4.0, 5.0, 5.0]);
    /// knot_vec.try_normalize()?;
    /// let res : Vec<f64> = knot_vec.into();
    /// assert_eq!(res, vec![0.0, 0.0, 0.25, 0.5, 0.75, 1.0, 1.0]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_normalize(&mut self) -> Result<&mut Self> {
        let range = self.range_length();
        if range.so_small() {
            return Err(Error::ZeroRange);
        }
        Ok(self.transform(1.0 / range, -self[0] / range))
    }

    /// Normalizes the knot vector i.e. makes the first value 0 and the last value 1.
    /// # Panics
    /// Panic occurs if the range of the knot vector is so small.
    /// # Examples
    /// ```
    /// use monstertruck_geometry::prelude::KnotVector;
    /// let mut knot_vec = KnotVector::from(vec![1.0, 1.0, 2.0, 3.0, 4.0, 5.0, 5.0]);
    /// knot_vec.normalize();
    /// let res : Vec<f64> = knot_vec.into();
    /// assert_eq!(res, vec![0.0, 0.0, 0.25, 0.5, 0.75, 1.0, 1.0]);
    /// ```
    #[inline(always)]
    pub fn normalize(&mut self) -> &mut Self {
        self.try_normalize()
            .unwrap_or_else(|error| panic!("{}", error))
    }

    /// Translates the knot vector
    /// # Example
    /// ```
    /// use std::vec::Vec;
    /// use monstertruck_geometry::prelude::KnotVector;
    /// let mut knot_vec = KnotVector::from(vec![1.0, 1.0, 2.0, 3.0, 4.0, 5.0, 5.0]);
    /// knot_vec.translate(3.0);
    /// let res : Vec<f64> = knot_vec.into();
    /// assert_eq!(res, vec![4.0, 4.0, 5.0, 6.0, 7.0, 8.0, 8.0]);
    /// ```
    pub fn translate(&mut self, x: f64) -> &mut Self { self.transform(1.0, x) }

    /// Inverts the knot vector
    /// # Example
    /// ```
    /// use std::vec::Vec;
    /// use monstertruck_geometry::prelude::KnotVector;
    /// let mut knot_vec = KnotVector::from(vec![1.0, 1.0, 1.0, 3.0, 5.0, 6.0]);
    /// knot_vec.invert();
    /// let res : Vec<f64> = knot_vec.into();
    /// assert_eq!(res, vec![1.0, 2.0, 4.0, 6.0, 6.0, 6.0]);
    ///
    /// ```
    pub fn invert(&mut self) -> &mut Self {
        let n = self.len();
        if n == 0 {
            return self;
        }
        let range = self[0] + self[n - 1];
        let clone = self.0.clone();
        for (knot1, knot0) in clone.iter().rev().zip(&mut self.0) {
            *knot0 = range - knot1;
        }
        self
    }

    /// Determines the knot vector is clamped for the given degree.
    /// # Examples
    /// ```
    /// use monstertruck_geometry::prelude::KnotVector;
    /// let knot_vec = KnotVector::from(vec![0.0, 0.0, 0.0, 0.25, 0.5, 0.75, 1.0, 1.0, 1.0]);
    /// assert!(knot_vec.is_clamped(2));
    /// assert!(!knot_vec.is_clamped(3));
    /// ```
    #[inline(always)]
    pub fn is_clamped(&self, degree: usize) -> bool {
        self.multiplicity(0) > degree && self.multiplicity(self.len() - 1) > degree
    }

    /// Concats two knot vectors.
    /// # Examples
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// use monstertruck_geometry::prelude::KnotVector;
    /// let mut knot_vec0 = KnotVector::from(vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    /// let knot_vec1 = KnotVector::from(vec![1.0, 1.0, 1.0, 2.0, 2.0, 2.0]);
    /// knot_vec0.try_concat(&knot_vec1, 2)?;
    /// assert_eq!(knot_vec0.as_slice(), &[0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0]);
    /// # Ok(())
    /// # }
    /// ```
    /// # Failures
    /// - If at least one of `self` or `other` is not clamped, returns [`Error::NotClampedKnotVector`]
    /// - If the last knot of `self` and the first knot of `other` are different, returns [`Error::DifferentBackFront`].
    pub fn try_concat(&mut self, other: &KnotVector, degree: usize) -> Result<&mut Self> {
        if !self.is_clamped(degree) || !other.is_clamped(degree) {
            return Err(Error::NotClampedKnotVector);
        }
        // SAFETY: knot vectors are always non-empty by construction.
        let back = self.0.last().unwrap();
        let front = other.0.first().unwrap();
        if front < back || !front.near(back) {
            return Err(Error::DifferentBackFront(*back, *front));
        }

        self.0.truncate(self.len() - degree - 1);
        self.0.extend(other.0.iter().copied());

        Ok(self)
    }

    /// Concats two knot vectors.
    /// # Examples
    /// ```
    /// use monstertruck_geometry::prelude::KnotVector;
    /// let mut knot_vec0 = KnotVector::from(vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    /// let knot_vec1 = KnotVector::from(vec![1.0, 1.0, 1.0, 2.0, 2.0, 2.0]);
    /// knot_vec0.concat(&knot_vec1, 2);
    /// assert_eq!(knot_vec0.as_slice(), &[0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0]);
    /// ```
    /// # Panics
    /// Panic occurs if:
    /// - at least one of `self` or `other` is not clamped.
    /// - the last knot of `self` and the first knot of `other` are different.
    #[inline(always)]
    pub fn concat(&mut self, other: &KnotVector, degree: usize) -> &mut Self {
        self.try_concat(other, degree)
            .unwrap_or_else(|error| panic!("{}", error))
    }

    /// Returns trimmed vector by the specified range.
    /// # Examples
    /// ```
    /// use monstertruck_geometry::prelude::*;
    /// let knot_vec = KnotVector::from(vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0]);
    /// let sub_vec = knot_vec.sub_vec(1..3);
    /// assert_eq!(sub_vec, KnotVector::from(vec![1.0, 2.0]));
    /// ```
    #[inline(always)]
    pub fn sub_vec<I: SliceIndex<[f64], Output = [f64]>>(&self, range: I) -> KnotVector {
        KnotVector(Vec::from(&self.0[range]))
    }

    /// To single-multi description. i.e. decompose the unique vector of knots and the vector of
    /// multiplicity of knots.
    /// # Examples
    /// ```
    /// use monstertruck_geometry::prelude::KnotVector;
    /// let knot_vec = KnotVector::from(vec![0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 2.0, 2.0, 3.0, 3.0]);
    /// let (knots, mults) = knot_vec.to_single_multi();
    /// assert_eq!(knots, vec![0.0, 1.0, 2.0, 3.0]);
    /// assert_eq!(mults, vec![3, 1, 4, 2]);
    /// ```
    pub fn to_single_multi(&self) -> (Vec<f64>, Vec<usize>) {
        let mut knots = Vec::new();
        let mut mults = Vec::new();

        let mut iter = self.as_slice().iter().peekable();
        let mut mult = 1;
        while let Some(knot) = iter.next() {
            if let Some(next) = iter.peek() {
                if knot.near(next) {
                    mult += 1;
                } else {
                    knots.push(*knot);
                    mults.push(mult);
                    mult = 1;
                }
            } else {
                knots.push(*knot);
                mults.push(mult);
            }
        }
        (knots, mults)
    }

    /// Constructs from single-multi description.
    /// # Examples
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// use monstertruck_geometry::prelude::KnotVector;
    /// let knots = vec![0.0, 1.0, 2.0, 3.0];
    /// let mults = vec![3, 1, 4, 2];
    /// let knot_vec = KnotVector::from_single_multi(knots, mults)?;
    /// assert_eq!(knot_vec, KnotVector::from(vec![0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 2.0, 2.0, 3.0, 3.0]));
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_single_multi(knots: Vec<f64>, mults: Vec<usize>) -> Result<KnotVector> {
        for i in 1..knots.len() {
            if knots[i - 1] > knots[i] {
                return Err(Error::NotSortedVector);
            }
        }

        let vec: Vec<f64> = knots
            .iter()
            .zip(mults.iter())
            .flat_map(|(&k, &m)| std::iter::repeat_n(k, m))
            .collect();
        Ok(KnotVector(vec))
    }
    /// Constructs from `Vec<f64>`. do not sort, only check sorted.
    pub fn try_from(vec: Vec<f64>) -> Result<KnotVector> {
        for i in 1..vec.len() {
            if vec[i - 1] > vec[i] {
                return Err(Error::NotSortedVector);
            }
        }
        Ok(KnotVector(vec))
    }

    /// Constructs the knot vector for the bezier spline.
    /// # Examples
    /// ```
    /// use monstertruck_geometry::prelude::*;
    /// assert_eq!(
    ///     *KnotVector::bezier_knot(3),
    ///     vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
    /// );
    /// ```
    pub fn bezier_knot(degree: usize) -> KnotVector {
        let mut vec = vec![0.0; degree + 1];
        vec.extend(std::iter::repeat_n(1.0, degree + 1));
        KnotVector(vec)
    }

    /// Constructs the uniform knot vector
    /// # Examples
    /// ```
    /// use monstertruck_geometry::prelude::*;
    /// assert_eq!(
    ///     *KnotVector::uniform_knot(2, 5),
    ///     vec![0.0, 0.0, 0.0, 0.2, 0.4, 0.6, 0.8, 1.0, 1.0, 1.0],
    /// );
    /// ```
    pub fn uniform_knot(degree: usize, division: usize) -> KnotVector {
        let mut vec = vec![0.0; degree + 1];
        vec.extend((1..division).map(|i| i as f64 / division as f64));
        vec.extend(std::iter::repeat_n(1.0, degree + 1));
        KnotVector(vec)
    }
}

impl From<Vec<f64>> for KnotVector {
    /// constructs from `Vec<f64>`. The vector will sorted by the order.
    /// ```
    /// use monstertruck_geometry::prelude::KnotVector;
    /// let knot_vec = KnotVector::from(vec![1.0, 0.0, 3.0, 2.0]);
    /// let arr : Vec<f64> = knot_vec.into();
    /// assert_eq!(arr, vec![0.0, 1.0, 2.0, 3.0]);
    /// ```
    fn from(mut vec: Vec<f64>) -> KnotVector {
        // SAFETY: knot values are finite `f64` (not NaN), so `partial_cmp` always returns `Some`.
        vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
        KnotVector(vec)
    }
}

impl From<&[f64]> for KnotVector {
    /// Constructs by the reference of vector. The clone of vector is sorted by the order.
    /// ```
    /// use monstertruck_geometry::prelude::KnotVector;
    /// let knot_vec = KnotVector::from([1.0, 0.0, 3.0, 2.0].as_slice());
    /// let arr : Vec<f64> = knot_vec.into();
    /// assert_eq!(arr, vec![0.0, 1.0, 2.0, 3.0]);
    /// ```
    #[inline(always)]
    fn from(vec: &[f64]) -> KnotVector {
        let mut copy_vec = vec.to_vec();
        // SAFETY: knot values are finite `f64` (not NaN), so `partial_cmp` always returns `Some`.
        copy_vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
        KnotVector(copy_vec)
    }
}

impl From<&Vec<f64>> for KnotVector {
    /// Constructs by the reference of vector. The clone of vector is sorted by the order.
    /// ```
    /// use monstertruck_geometry::prelude::KnotVector;
    /// let knot_vec = KnotVector::from(&vec![1.0, 0.0, 3.0, 2.0]);
    /// let arr : Vec<f64> = knot_vec.into();
    /// assert_eq!(arr, vec![0.0, 1.0, 2.0, 3.0]);
    /// ```
    #[inline(always)]
    fn from(vec: &Vec<f64>) -> KnotVector {
        let mut copy_vec = vec.clone();
        // SAFETY: knot values are finite `f64` (not NaN), so `partial_cmp` always returns `Some`.
        copy_vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
        KnotVector(copy_vec)
    }
}

impl From<KnotVector> for Vec<f64> {
    /// `KnotVector` into `Vec<f64>`
    /// ```
    /// use monstertruck_geometry::prelude::KnotVector;
    /// let vec = vec![0.0, 1.0, 2.0, 3.0];
    /// let knot_vec = KnotVector::from(&vec);
    /// let vec0 : Vec<f64> = knot_vec.into();
    /// assert_eq!(vec, vec0);
    /// ```
    #[inline(always)]
    fn from(knotvec: KnotVector) -> Vec<f64> { knotvec.0 }
}

impl FromIterator<f64> for KnotVector {
    #[inline(always)]
    fn from_iter<I: IntoIterator<Item = f64>>(iter: I) -> KnotVector {
        // SAFETY: panicking convenience impl; callers must provide sorted, non-NaN values.
        KnotVector::try_from(iter.into_iter().collect::<Vec<_>>()).unwrap()
    }
}

impl<'a> IntoIterator for &'a KnotVector {
    type Item = &'a f64;
    type IntoIter = std::slice::Iter<'a, f64>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}

impl std::ops::Deref for KnotVector {
    type Target = Vec<f64>;
    #[inline(always)]
    fn deref(&self) -> &Vec<f64> { &self.0 }
}

impl AsRef<[f64]> for KnotVector {
    #[inline(always)]
    fn as_ref(&self) -> &[f64] { &self.0 }
}

impl<'de> Deserialize<'de> for KnotVector {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let vec = Vec::<f64>::deserialize(deserializer)?;
        Self::try_from(vec).map_err(serde::de::Error::custom)
    }
}
