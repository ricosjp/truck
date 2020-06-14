use crate::{Origin, Tolerance, Result, KnotVec};
use crate::errors::Error;
use crate::tolerance::inv_or_zero;
use std::slice::SliceIndex;
use std::vec::Vec;

impl KnotVec {
    /// empty constructor
    pub fn new() -> KnotVec { KnotVec(Vec::new()) }

    #[inline(always)]
    pub fn range_length(&self) -> f64 {
        if self.is_empty() {
            0.0
        } else {
            self[self.len() - 1] - self[0]
        }
    }

    /// remove one item
    #[inline(always)]
    pub fn remove(&mut self, idx: usize) -> f64 { self.0.remove(idx) }

    /// get the maximum index `i` of `self[i] <= x`
    /// Return `None` if `x < self[0] or self.len() == 0`.
    /// # Examples
    /// ```
    /// use truck_geometry::KnotVec;
    /// let mut knot_vec = KnotVec::from(vec![0.0, 0.0, 1.0, 2.0, 3.0, 3.0]);
    /// let idx = knot_vec.floor(1.5).unwrap();
    /// assert_eq!(idx, 2);
    /// ```
    #[inline(always)]
    pub fn floor(&self, x: f64) -> Result<usize> {
        if self.len() == 0 {
            return Err(Error::EmptyKnotVector);
        }
        if x < self[0] {
            return Err(Error::SmallerThanSmallestKnot(x, self[0]));
        }
        for i in 1..self.len() {
            if x < self[i] {
                return Ok(i - 1);
            }
        }
        Ok(self.len() - 1)
    }

    /// the multiplicity of the ith knot
    /// # Examples
    /// ```
    /// use truck_geometry::KnotVec;
    /// let knot_vec = KnotVec::from(vec![0.0, 0.0, 1.0, 2.0, 2.0, 2.0, 3.0, 3.0]);
    /// assert_eq!(knot_vec.multiplicity(5), 3);
    /// ```
    #[inline(always)]
    pub fn multiplicity(&self, i: usize) -> usize {
        let u_i = &self[i];
        let mut res = 0;
        for u in self.as_slice() {
            if u == u_i {
                res += 1;
            }
            if u_i < u {
                break;
            }
        }
        res
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
        self.0.push(knot);
        let n = self.len();
        let knot_vec = &mut self.0;
        for i in 1..n {
            if knot_vec[n - i - 1] <= knot_vec[n - i] {
                return n - i;
            } else {
                let tmp = knot_vec[n - i - 1];
                knot_vec[n - i - 1] = knot_vec[n - i];
                knot_vec[n - i] = tmp;
            }
        }
        0
    }

    /// Calculate B-spline basis functions at `t` with degree `degree`.
    /// # Panics
    /// If the length of `self` is lower than `degree + 1`, panic is caused.
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
    ///     let res = knot_vec.bspline_basis_functions(degree, t).unwrap();
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
    ///     let res = knot_vec.bspline_basis_functions(degree, t).unwrap();
    ///     let ans = [
    ///         1.0 * (1.0 - t) * (1.0 - t) * (1.0 - t),
    ///         3.0 * t * (1.0 - t) * (1.0 - t),
    ///         3.0 * t * t * (1.0 - t),
    ///         1.0 * t * t * t,
    ///     ];
    ///     for i in 0..4 { f64::assert_near2(&res[i], &ans[i]); }
    /// }
    /// ```
    pub fn bspline_basis_functions(&self, degree: usize, t: f64) -> Result<Vec<f64>> {
        let n = self.len() - 1;
        if self[0].near(&self[n]) {
            return Err(Error::ZeroRange);
        } else if n < degree {
            return Err(Error::TooLargeDegree(n + 1, degree));
        } else if self[n] <= t {
            let result = self.clone().inverse().bspline_basis_functions(degree, self[0] + self[n] - t);
            return result.map(|mut x| {
                x.reverse();
                x
            });
        }

        let idx = self.floor(t);
        if idx.is_err() {
            return Ok(vec![0.0; n - degree]);
        }
        let idx = idx.unwrap();

        let mut res = vec![0.0; n + 1];
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

    pub fn maximum_points(&self, degree: usize) -> Vec<f64> {
        let n = self.len();
        let m = n - degree - 1;
        let range = self.range_length();
        const N: i32 = 100;

        let mut res = vec![0.0; m];
        let mut max = vec![0.0; m];
        for i in 1..N {
            let t = self[0] + range * (i as f64) / (N as f64);
            let vals = self.bspline_basis_functions(degree, t).unwrap();
            for j in 0..m {
                if max[j] < vals[j] {
                    max[j] = vals[j];
                    res[j] = t;
                }
            }
        }

        res
    }

    /// normalize the knot vector i.e. make the first value 0 and the last value 1.
    /// Return error if the range of the knot vector is so small.
    /// # Examples
    /// ```
    /// use std::vec::Vec;
    /// use truck_geometry::KnotVec;
    /// let mut knot_vec = KnotVec::from(vec![1.0, 1.0, 2.0, 3.0, 4.0, 5.0, 5.0]);
    /// knot_vec.normalize();
    /// let res : Vec<f64> = knot_vec.into();
    /// assert_eq!(res, vec![0.0, 0.0, 0.25, 0.5, 0.75, 1.0, 1.0]);
    /// ```
    pub fn normalize(&mut self) -> Result<()> {
        let range = self.range_length();
        if range.so_small() {
            return Err(Error::ZeroRange);
        }

        let start = self[0];
        for vec in self.0.as_mut_slice() {
            *vec -= start;
            *vec /= range;
        }

        Ok(())
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
    pub fn translate(&mut self, x: f64) {
        for vec in self.0.as_mut_slice() {
            *vec += x;
        }
    }

    /// inverse the knot vector
    /// # Example
    /// ```
    /// # use std::vec::Vec;
    /// # use truck_geometry::KnotVec;
    /// let mut knot_vec = KnotVec::from(vec![1.0, 1.0, 1.0, 3.0, 5.0, 6.0]);
    /// knot_vec.inverse();
    /// let res : Vec<f64> = knot_vec.into();
    /// assert_eq!(res, vec![1.0, 2.0, 4.0, 6.0, 6.0, 6.0]);
    ///
    /// ```
    pub fn inverse(&mut self) -> &mut KnotVec {
        let n = self.len();
        if n == 0 {
            return self;
        }
        let range = self[0] + self[n - 1];
        let clone = self.0.clone();
        for (knot0, knot1) in self.0.iter_mut().zip(clone.iter().rev()) {
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

    /// concat two knot vectors. Return error if:
    /// * At least one of `self` or `other` is not clamped.
    /// * The last knot of `self` and the first knot of `other` are different.
    /// # Examples
    /// ```
    /// use truck_geometry::KnotVec;
    /// let mut knot_vec0 = KnotVec::from(vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    /// let knot_vec1 = KnotVec::from(vec![1.0, 1.0, 1.0, 2.0, 2.0, 2.0]);
    /// knot_vec0.concat(&knot_vec1, 2);
    /// assert_eq!(knot_vec0.as_slice(), &[0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0]);
    /// ```
    pub fn concat(&mut self, other: &KnotVec, degree: usize) -> Result<()> {
        if !self.is_clamped(degree) || !other.is_clamped(degree) {
            return Err(Error::NotClampedKnotVector(self.clone().into(), other.clone().into()));
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

        Ok(())
    }
    
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
    /// # use truck_geometry::KnotVec;
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
    /// # use truck_geometry::KnotVec;
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

    /// get the knot vector for the bezier spline i.e. [0,...,0,1,...,1].
    pub fn bezier_knot(degree: usize) -> KnotVec {
        let mut vec = vec![0.0; degree + 1];
        vec.extend(std::iter::repeat(1.0).take(degree + 1));
        KnotVec(vec)
    }
}

impl std::convert::From<Vec<f64>> for KnotVec {
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

impl std::convert::From<&Vec<f64>> for KnotVec {
    /// construct by the reference of vector. The clone of vector is sorted by the order.
    /// ```
    /// use truck_geometry::KnotVec;
    /// let knot_vec = KnotVec::by_vec_ref(&vec![1.0, 0.0, 3.0, 2.0]);
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

impl std::convert::From<KnotVec> for Vec<f64> {
    /// KnotVec into Vec<f64>
    /// ```
    /// use truck_geometry::KnotVec;
    /// let vec = vec![0.0, 1.0, 2.0, 3.0];
    /// let knot_vec = KnotVec::by_vec_ref(&vec);
    /// let vec0 : Vec<f64> = knot_vec.into();
    /// assert_eq!(vec, vec0);
    /// ```
    #[inline(always)]
    fn from(knotvec: KnotVec) -> Vec<f64> { knotvec.0 }
}

impl std::ops::Deref for KnotVec {
    type Target = Vec<f64>;
    #[inline(always)]
    fn deref(&self) -> &Vec<f64> { &self.0 }
}
