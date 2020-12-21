use crate::errors::Error;
use crate::*;
use std::slice::SliceIndex;
use std::vec::Vec;

impl KnotVec {
    /// empty constructor
    pub const fn new() -> KnotVec { KnotVec(Vec::new()) }

    /// Returns the length of range.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::from(vec![0.0, 6.0]);
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
    /// use truck_geometry::*;
    /// let knot_vec0 = KnotVec::new(); // empty knot vector
    /// let knot_vec1 = KnotVec::from(vec![0.0, 0.0, 1.0, 1.0]);
    /// let knot_vec2 = KnotVec::from(vec![0.0, 0.5, 1.0]);
    /// let knot_vec3 = KnotVec::from(vec![0.0, 0.0, 2.0, 2.0]);
    /// assert!(knot_vec0.same_range(&KnotVec::new())); // both empty knot vector
    /// assert!(!knot_vec0.same_range(&knot_vec1));
    /// assert!(knot_vec1.same_range(&knot_vec2)); // the range of both knot vector is [0, 1].
    /// assert!(!knot_vec1.same_range(&knot_vec3));
    /// ```
    #[inline(always)]
    pub fn same_range(&self, other: &KnotVec) -> bool {
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
    /// use truck_geometry::KnotVec;
    /// let mut knot_vec = KnotVec::from(vec![0.0, 0.0, 1.0, 2.0, 3.0, 3.0]);
    /// let idx = knot_vec.floor(1.5).unwrap();
    /// assert_eq!(idx, 2);
    /// ```
    #[inline(always)]
    pub fn floor(&self, x: f64) -> Option<usize> { self.iter().rposition(|t| *t <= x) }

    /// the multiplicity of the `i`th knot
    /// # Examples
    /// ```
    /// use truck_geometry::KnotVec;
    /// let knot_vec = KnotVec::from(vec![0.0, 0.0, 1.0, 2.0, 2.0, 2.0, 3.0, 3.0]);
    /// assert_eq!(knot_vec.multiplicity(5), 3);
    /// ```
    #[inline(always)]
    pub fn multiplicity(&self, i: usize) -> usize {
        self.iter().filter(|u| self[i].near(u)).count()
    }

    /// add a knot and return the index of the added knot.
    /// # Examples
    /// ```
    /// use truck_geometry::KnotVec;
    /// let mut knot_vec = KnotVec::from(vec![0.0, 0.0, 1.0, 2.0, 3.0, 3.0]);
    /// let idx0 = knot_vec.add_knot(1.5);
    /// assert_eq!(idx0, 3);
    /// let idx1 = knot_vec.add_knot(-1.0);
    /// assert_eq!(idx1, 0);
    /// let ansvec = KnotVec::from(vec![-1.0, 0.0, 0.0, 1.0, 1.5, 2.0, 3.0, 3.0]);
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

    /// Calculate B-spline basis functions at `t` with degree `degree`.
    /// # Panics
    /// If the length of `self` is not more than `degree`, panic occurs.
    /// # Remarks
    /// In this package, the B-spline basis function is based on the characteristic function of
    /// the right-open intervals [s, t). So, the value corresponding to the end point t = t_n is always 0.0.
    /// # Examples
    /// ```
    /// use truck_geometry::{Tolerance, KnotVec};
    /// const N : usize = 100; // sample size in tests
    ///
    /// // B-spline basis functions is a partition of unity in (t_k, t_{n - k}).
    /// let vec = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    /// let knot_vec = KnotVec::from(vec);
    /// let degree = 2;
    /// for i in 0..N {
    ///     let t = 2.0 + 4.0 / (N as f64) * (i as f64);
    ///     let res = knot_vec.bspline_basis_functions(degree, t);
    ///     let sum = res.iter().fold(0.0, |sum, a| sum + a);
    ///     f64::assert_near2(&sum, &1.0);
    /// }
    /// ```
    /// ```
    /// use truck_geometry::{Tolerance, KnotVec};
    /// const N : usize = 100; // sample size in tests
    ///
    /// // In some case, B-spline basis functions coincide with Bernstein polynomials.
    /// let vec = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
    /// let knot_vec = KnotVec::from(vec);
    /// let degree = 3;
    /// for i in 0..=N {
    ///     let t = 1.0 / (N as f64) * (i as f64);
    ///     let res = knot_vec.bspline_basis_functions(degree, t);
    ///     let ans = [
    ///         1.0 * (1.0 - t) * (1.0 - t) * (1.0 - t),
    ///         3.0 * t * (1.0 - t) * (1.0 - t),
    ///         3.0 * t * t * (1.0 - t),
    ///         1.0 * t * t * t,
    ///     ];
    ///     for i in 0..4 { f64::assert_near2(&res[i], &ans[i]); }
    /// }
    /// ```
    pub fn bspline_basis_functions(&self, degree: usize, t: f64) -> Vec<f64> {
        match self.try_bspline_basis_functions(degree, t) {
            Ok(got) => got,
            Err(error) => panic!("{}", error),
        }
    }

    /// Calculate B-spline basis functions at `t` with degree `degree`.
    /// # Failures
    /// - If the range of the knot vector is zero, returns [`Error::ZeroRange`].
    /// - If the length of `self` is not more than `degree`, returns [`Error::TooLargeDegree(length, degree)`].
    ///
    /// [`Error::ZeroRange`]: errors/enum.Error.html#variant.ZeroRange
    /// [`Error::TooLargeDegree(length, degree)`]: errors/enum.Error.html#variant.TooLargeDegree
    /// # Remarks
    /// In this package, the B-spline basis function is based on the characteristic function of
    /// the right-open intervals [s, t). So, the value corresponding to the end point t = t_n is always 0.0.
    /// # Examples
    /// ```
    /// use truck_geometry::{Tolerance, KnotVec};
    /// const N : usize = 100; // sample size in tests
    ///
    /// // B-spline basis functions is a partition of unity in (t_k, t_{n - k}).
    /// let vec = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    /// let knot_vec = KnotVec::from(vec);
    /// let degree = 2;
    /// for i in 0..N {
    ///     let t = 2.0 + 4.0 / (N as f64) * (i as f64);
    ///     let res = knot_vec.try_bspline_basis_functions(degree, t).unwrap();
    ///     let sum = res.iter().fold(0.0, |sum, a| sum + a);
    ///     f64::assert_near2(&sum, &1.0);
    /// }
    /// ```
    /// ```
    /// use truck_geometry::{Tolerance, KnotVec};
    /// const N : usize = 100; // sample size in tests
    ///
    /// // In some case, B-spline basis functions coincide with Bernstein polynomials.
    /// let vec = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
    /// let knot_vec = KnotVec::from(vec);
    /// let degree = 3;
    /// for i in 0..=N {
    ///     let t = i as f64 / N as f64;
    ///     let res = knot_vec.try_bspline_basis_functions(degree, t).unwrap();
    ///     let ans = [
    ///         1.0 * (1.0 - t) * (1.0 - t) * (1.0 - t),
    ///         3.0 * t * (1.0 - t) * (1.0 - t),
    ///         3.0 * t * t * (1.0 - t),
    ///         1.0 * t * t * t,
    ///     ];
    ///     println!("{:?}", res);
    ///     for i in 0..4 { f64::assert_near2(&res[i], &ans[i]); }
    /// }
    /// ```
    pub fn try_bspline_basis_functions(&self, degree: usize, t: f64) -> Result<Vec<f64>> {
        let n = self.len() - 1;
        if self[0].near(&self[n]) {
            return Err(Error::ZeroRange);
        } else if n < degree {
            return Err(Error::TooLargeDegree(n + 1, degree));
        }

        let idx = {
            let idx = self
                .floor(t)
                .unwrap_or_else(|| self.floor(self[0]).unwrap());
            if idx == n {
                n - self.multiplicity(n)
            } else {
                idx
            }
        };
        let mut res = vec![0.0; n];
        res[idx] = 1.0;

        for k in 1..=degree {
            let base = if idx < k { 0 } else { idx - k };
            let delta = self[base + k] - self[base];
            let max = if idx + k < n { idx } else { n - k - 1 };
            let mut a = inv_or_zero(delta) * (t - self[base]);
            for i in base..=max {
                let delta = self[i + k + 1] - self[i + 1];
                let b = inv_or_zero(delta) * (self[i + k + 1] - t);
                res[i] = a * res[i] + b * res[i + 1];
                a = 1.0 - b;
            }
        }

        res.truncate(n - degree);
        Ok(res)
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
            let vals = self.try_bspline_basis_functions(degree, t).unwrap();
            for j in 0..m {
                if max[j] < vals[j] {
                    max[j] = vals[j];
                    res[j] = t;
                }
            }
        }

        res
    }

    /// Normalizes the knot vector i.e. makes the first value 0 and the last value 1.
    /// # Failures
    /// Returns [`Error::ZeroRange`] if the range of the knot vector is so small.
    ///
    /// [`Error::ZeroRange`]: errors/enum.Error.html#variant.ZeroRange
    /// # Examples
    /// ```
    /// use truck_geometry::KnotVec;
    /// let mut knot_vec = KnotVec::from(vec![1.0, 1.0, 2.0, 3.0, 4.0, 5.0, 5.0]);
    /// knot_vec.try_normalize().unwrap();
    /// let res : Vec<f64> = knot_vec.into();
    /// assert_eq!(res, vec![0.0, 0.0, 0.25, 0.5, 0.75, 1.0, 1.0]);
    /// ```
    pub fn try_normalize(&mut self) -> Result<&mut Self> {
        let range = self.range_length();
        if range.so_small() {
            return Err(Error::ZeroRange);
        }

        let start = self[0];
        for vec in self.0.as_mut_slice() {
            *vec -= start;
            *vec /= range;
        }

        Ok(self)
    }

    /// Normalizes the knot vector i.e. makes the first value 0 and the last value 1.
    /// # Panics
    /// Panic occurs if the range of the knot vector is so small.
    /// # Examples
    /// ```
    /// use truck_geometry::KnotVec;
    /// let mut knot_vec = KnotVec::from(vec![1.0, 1.0, 2.0, 3.0, 4.0, 5.0, 5.0]);
    /// knot_vec.normalize();
    /// let res : Vec<f64> = knot_vec.into();
    /// assert_eq!(res, vec![0.0, 0.0, 0.25, 0.5, 0.75, 1.0, 1.0]);
    /// ```
    #[inline(always)]
    pub fn normalize(&mut self) -> &mut Self {
        self.try_normalize()
            .unwrap_or_else(|error| panic!("{}", error))
    }

    /// translate the knot vector
    /// # Example
    /// ```
    /// use std::vec::Vec;
    /// use truck_geometry::KnotVec;
    /// let mut knot_vec = KnotVec::from(vec![1.0, 1.0, 2.0, 3.0, 4.0, 5.0, 5.0]);
    /// knot_vec.translate(3.0);
    /// let res : Vec<f64> = knot_vec.into();
    /// assert_eq!(res, vec![4.0, 4.0, 5.0, 6.0, 7.0, 8.0, 8.0]);
    ///
    /// ```
    pub fn translate(&mut self, x: f64) -> &mut Self {
        for vec in &mut self.0 {
            *vec += x;
        }
        self
    }

    /// Inverts the knot vector
    /// # Example
    /// ```
    /// use std::vec::Vec;
    /// use truck_geometry::KnotVec;
    /// let mut knot_vec = KnotVec::from(vec![1.0, 1.0, 1.0, 3.0, 5.0, 6.0]);
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

    /// determine the knot vector is clamped for the given degree.
    /// # Examples
    /// ```
    /// use truck_geometry::KnotVec;
    /// let knot_vec = KnotVec::from(vec![0.0, 0.0, 0.0, 0.25, 0.5, 0.75, 1.0, 1.0, 1.0]);
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
    /// use truck_geometry::KnotVec;
    /// let mut knot_vec0 = KnotVec::from(vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    /// let knot_vec1 = KnotVec::from(vec![1.0, 1.0, 1.0, 2.0, 2.0, 2.0]);
    /// knot_vec0.try_concat(&knot_vec1, 2).unwrap();
    /// assert_eq!(knot_vec0.as_slice(), &[0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0]);
    /// ```
    /// # Failures
    /// * If at least one of `self` or `other` is not clamped, returns [`Error::NotClampedKnotVector`]
    /// * If the last knot of `self` and the first knot of `other` are different, returns
    /// [`Error::DifferentBackFront(self.last, other.first)`].
    ///
    /// [`Error::NotClampedKnotVector`]: errors/enum.Error.html#variant.NotClampedKnotVector
    /// [`Error::DifferentBackFront(self.last, other.first)`]: errors/enum.Error.html#variant.DifferentBackFront
    pub fn try_concat(&mut self, other: &KnotVec, degree: usize) -> Result<&mut Self> {
        if !self.is_clamped(degree) || !other.is_clamped(degree) {
            return Err(Error::NotClampedKnotVector);
        }
        let back = self.0.last().unwrap();
        let front = other.0.first().unwrap();
        if front < back || !front.near(back) {
            return Err(Error::DifferentBackFront(*back, *front));
        }

        self.0.truncate(self.len() - degree - 1);
        for knot in &other.0 {
            self.0.push(*knot);
        }

        Ok(self)
    }

    /// Concats two knot vectors.
    /// # Examples
    /// ```
    /// use truck_geometry::KnotVec;
    /// let mut knot_vec0 = KnotVec::from(vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    /// let knot_vec1 = KnotVec::from(vec![1.0, 1.0, 1.0, 2.0, 2.0, 2.0]);
    /// knot_vec0.concat(&knot_vec1, 2);
    /// assert_eq!(knot_vec0.as_slice(), &[0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0]);
    /// ```
    /// # Panics
    /// Panic occurs if:
    /// * At least one of `self` or `other` is not clamped.
    /// * The last knot of `self` and the first knot of `other` are different.
    #[inline(always)]
    pub fn concat(&mut self, other: &KnotVec, degree: usize) -> &mut Self {
        self.try_concat(other, degree)
            .unwrap_or_else(|error| panic!("{}", error))
    }

    /// Returns trimmed vector by the specified range.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::from(vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0]);
    /// let sub_vec = knot_vec.sub_vec(1..3);
    /// assert_eq!(sub_vec, KnotVec::from(vec![1.0, 2.0]));
    /// ```
    #[inline(always)]
    pub fn sub_vec<I: SliceIndex<[f64], Output = [f64]>>(&self, range: I) -> KnotVec {
        KnotVec {
            0: Vec::from(&self.0[range]),
        }
    }

    /// To single-multi discription. i.e. decompose the unique vector of knots and the vector of
    /// multiplicity of knots.
    /// # Examples
    /// ```
    /// use truck_geometry::KnotVec;
    /// let knot_vec = KnotVec::from(vec![0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 2.0, 2.0, 3.0, 3.0]);
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

    /// construct from single-multi description.
    /// # Examples
    /// ```
    /// use truck_geometry::KnotVec;
    /// let knots = vec![0.0, 1.0, 2.0, 3.0];
    /// let mults = vec![3, 1, 4, 2];
    /// let knot_vec = KnotVec::from_single_multi(knots, mults).unwrap();
    /// assert_eq!(knot_vec, KnotVec::from(vec![0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 2.0, 2.0, 3.0, 3.0]));
    /// ```
    pub fn from_single_multi(knots: Vec<f64>, mults: Vec<usize>) -> Result<KnotVec> {
        for i in 1..knots.len() {
            if knots[i - 1] > knots[i] {
                return Err(Error::NotSortedVector);
            }
        }

        let mut vec = Vec::new();
        for i in 0..knots.len() {
            for _ in 0..mults[i] {
                vec.push(knots[i]);
            }
        }
        Ok(KnotVec { 0: vec })
    }
    /// construct from `Vec<f64>`. do not sort, only check sorted.
    pub fn try_from(vec: Vec<f64>) -> Result<KnotVec> {
        for i in 1..vec.len() {
            if vec[i - 1] > vec[i] {
                return Err(Error::NotSortedVector);
            }
        }
        Ok(KnotVec(vec))
    }

    /// Constructs the knot vector for the bezier spline.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// assert_eq!(
    ///     *KnotVec::bezier_knot(3),
    ///     vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
    /// );
    /// ```
    pub fn bezier_knot(degree: usize) -> KnotVec {
        let mut vec = vec![0.0; degree + 1];
        vec.extend(std::iter::repeat(1.0).take(degree + 1));
        KnotVec(vec)
    }

    /// Constructs the uniform knot vector
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// assert_eq!(
    ///     *KnotVec::uniform_knot(2, 5),
    ///     vec![0.0, 0.0, 0.0, 0.2, 0.4, 0.6, 0.8, 1.0, 1.0, 1.0],
    /// );
    /// ```
    pub fn uniform_knot(degree: usize, division: usize) -> KnotVec {
        let mut vec = vec![0.0; degree + 1];
        vec.extend((1..division).map(|i| (i as f64) / (division as f64)));
        vec.extend(std::iter::repeat(1.0).take(degree + 1));
        KnotVec(vec)
    }
}

impl From<Vec<f64>> for KnotVec {
    /// construct from `Vec<f64>`. The vector will sorted by the order.
    /// ```
    /// use truck_geometry::KnotVec;
    /// let knot_vec = KnotVec::from(vec![1.0, 0.0, 3.0, 2.0]);
    /// let arr : Vec<f64> = knot_vec.into();
    /// assert_eq!(arr, vec![0.0, 1.0, 2.0, 3.0]);
    /// ```
    fn from(mut vec: Vec<f64>) -> KnotVec {
        vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
        KnotVec(vec)
    }
}

impl From<&Vec<f64>> for KnotVec {
    /// construct by the reference of vector. The clone of vector is sorted by the order.
    /// ```
    /// use truck_geometry::KnotVec;
    /// let knot_vec = KnotVec::from(&vec![1.0, 0.0, 3.0, 2.0]);
    /// let arr : Vec<f64> = knot_vec.into();
    /// assert_eq!(arr, vec![0.0, 1.0, 2.0, 3.0]);
    /// ```
    #[inline(always)]
    fn from(vec: &Vec<f64>) -> KnotVec {
        let mut copy_vec = vec.clone();
        copy_vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
        KnotVec(copy_vec)
    }
}

impl From<KnotVec> for Vec<f64> {
    /// KnotVec into Vec<f64>
    /// ```
    /// use truck_geometry::KnotVec;
    /// let vec = vec![0.0, 1.0, 2.0, 3.0];
    /// let knot_vec = KnotVec::from(&vec);
    /// let vec0 : Vec<f64> = knot_vec.into();
    /// assert_eq!(vec, vec0);
    /// ```
    #[inline(always)]
    fn from(knotvec: KnotVec) -> Vec<f64> { knotvec.0 }
}

impl std::iter::FromIterator<f64> for KnotVec {
    #[inline(always)]
    fn from_iter<I: IntoIterator<Item = f64>>(iter: I) -> KnotVec {
        KnotVec::try_from(iter.into_iter().collect::<Vec<_>>()).unwrap()
    }
}

impl<'a> IntoIterator for &'a KnotVec {
    type Item = &'a f64;
    type IntoIter = std::slice::Iter<'a, f64>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}

impl std::ops::Deref for KnotVec {
    type Target = Vec<f64>;
    #[inline(always)]
    fn deref(&self) -> &Vec<f64> { &self.0 }
}

impl AsRef<[f64]> for KnotVec {
    #[inline(always)]
    fn as_ref(&self) -> &[f64] { &self.0 }
}
