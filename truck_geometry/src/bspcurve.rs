use crate::errors::Error;
use crate::tolerance::inv_or_zero;
use crate::*;

impl BSplineCurve {
    /// constructor.
    /// # Arguments
    /// * `knot_vec` - the knot vector
    /// * `control_points` - the vector of the control points
    /// # Panics
    /// There are 3 rules for construct B-spline curve.
    /// * The number of knots is more than the one of control points.
    /// * There exist at least two different knots.
    /// * There are at least one control point.
    pub fn new(knot_vec: KnotVec, control_points: Vec<Vector>) -> BSplineCurve {
        if control_points.is_empty() {
            panic!("{}", Error::EmptyControlPoints)
        } else if knot_vec.len() <= control_points.len() {
            panic!(
                "{}",
                Error::TooShortKnotVector(knot_vec.len(), control_points.len())
            )
        } else if knot_vec.range_length().so_small() {
            panic!("{}", Error::ZeroRange)
        } else {
            BSplineCurve {
                knot_vec: knot_vec,
                control_points: control_points,
                derivation: None,
            }
        }
    }
    /// constructor.
    /// # Arguments
    /// * `knot_vec` - the knot vector
    /// * `control_points` - the vector of the control points
    /// # Failures
    /// There are 3 rules for construct B-spline curve.
    /// * The number of knots is more than the one of control points.
    /// * There exist at least two different knots.
    /// * There are at least one control point.
    pub fn try_new(knot_vec: KnotVec, control_points: Vec<Vector>) -> Result<BSplineCurve> {
        if control_points.is_empty() {
            Err(Error::EmptyControlPoints)
        } else if knot_vec.len() <= control_points.len() {
            Err(Error::TooShortKnotVector(
                knot_vec.len(),
                control_points.len(),
            ))
        } else if knot_vec.range_length().so_small() {
            Err(Error::ZeroRange)
        } else {
            Ok(BSplineCurve {
                knot_vec: knot_vec,
                control_points: control_points,
                derivation: None,
            })
        }
    }
    /// constructor.
    /// # Arguments
    /// * `knot_vec` - the knot vector
    /// * `control_points` - the vector of the control points
    /// # Remarks
    /// This method is prepared only for performance-critical development and is not recommended.
    /// This method does NOT check the 3 rules for constructing B-spline curve.
    /// The programmer must guarantee these conditions before using this method.
    pub fn new_unchecked(knot_vec: KnotVec, control_points: Vec<Vector>) -> BSplineCurve {
        BSplineCurve {
            knot_vec: knot_vec,
            control_points: control_points,
            derivation: None,
        }
    }

    /// the reference of the knot vector
    #[inline(always)]
    pub fn knot_vec(&self) -> &KnotVec {
        &self.knot_vec
    }

    /// the ith knot
    #[inline(always)]
    pub fn knot(&self, idx: usize) -> f64 {
        self.knot_vec[idx]
    }

    /// get the reference of the control points.
    #[inline(always)]
    pub fn control_points(&self) -> &Vec<Vector> {
        &self.control_points
    }

    /// get the reference of the control point corresponding to the index `idx`.
    #[inline(always)]
    pub fn control_point(&self, idx: usize) -> &Vector {
        &self.control_points[idx]
    }
    /// get the mutable reference of the control point corresponding to index `idx`.
    #[inline(always)]
    pub fn control_point_mut(&mut self, idx: usize) -> &mut Vector {
        self.derivation = None;
        &mut self.control_points[idx]
    }

    /// the degree of B-spline curve
    #[inline(always)]
    pub fn degree(&self) -> usize {
        self.knot_vec.len() - self.control_points.len() - 1
    }

    /// determine the knot vector is clamped
    #[inline(always)]
    pub fn is_clamped(&self) -> bool {
        self.knot_vec.is_clamped(self.degree())
    }

    /// determine whether constant curve or not, i.e. all control points are same.
    pub fn is_const(&self) -> bool {
        for vec in &self.control_points {
            if !vec.near(&self.control_points[0]) {
                return false;
            }
        }
        true
    }

    /// substitution to B-spline curve. private method
    #[inline(always)]
    pub fn subs(&self, t: f64) -> Vector {
        let basis = self
            .knot_vec
            .bspline_basis_functions(self.degree(), t)
            .unwrap();
        let iter = self.control_points.iter().zip(basis.iter());
        let mut sum = Vector::zero();
        iter.for_each(|(vec, basis)| sum += vec * *basis);
        sum
    }

    #[inline(always)]
    pub fn get_closure(&self) -> impl Fn(f64) -> Vector + '_ {
        move |t| self.subs(t)
    }

    /// inverse as curve
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// # let file = std::fs::File::open("tests/data/examples.tgb").unwrap();
    /// # let geomdata = truck_io::tgb::read(file).unwrap();
    /// # let bspline0 = geomdata.curves[0].clone();
    /// const N: usize = 100; // sample size
    ///
    /// // bspline0 = BSplineCurve::new(...);
    /// let (knots, _) = bspline0.knot_vec().to_single_multi();
    /// assert_eq!(&knots, &[0.0, 1.0, 2.0, 3.0]);
    ///
    /// let mut bspline1 = bspline0.clone();
    /// bspline1.inverse();
    /// for i in 0..=N {
    ///     let t = 3.0 * (i as f64) / (N as f64);
    ///     Vector::assert_near(&bspline0.subs(t), &bspline1.subs(3.0 - t));
    /// }
    /// ```
    #[inline(always)]
    pub fn inverse(&mut self) -> &mut Self {
        self.knot_vec.inverse();
        self.control_points.reverse();
        self
    }
    /// normalize the knot vector  
    /// Return error if the knot vector is consisted by only one value.
    #[inline(always)]
    pub fn knot_normalize(&mut self) -> &mut Self {
        if let Some(ref mut derivation) = self.derivation {
            derivation.knot_vec.normalize().unwrap();
        }
        self.knot_vec.normalize().unwrap();
        self
    }

    /// translate the knot vector
    #[inline(always)]
    pub fn knot_translate(&mut self, x: f64) -> &mut Self {
        if let Some(ref mut derivation) = self.derivation {
            derivation.knot_vec.translate(x);
        }
        self.knot_vec.translate(x);
        self
    }

    #[inline(always)]
    fn delta_control_points(&self, i: usize) -> Vector {
        if i == 0 {
            self.control_point(i).clone()
        } else if i == self.control_points.len() {
            -self.control_point(i - 1)
        } else {
            self.control_point(i) - self.control_point(i - 1)
        }
    }

    fn calculate_derivation(&mut self) -> &mut Self {
        if let Some(ref mut derivation) = self.derivation {
            return derivation;
        }
        let n = self.control_points.len();
        let k = self.degree();
        let knot_vec = self.knot_vec.clone();
        let mut new_points = Vec::with_capacity(n + 1);
        if k > 0 {
            for i in 0..=n {
                let delta = knot_vec[i + k] - knot_vec[i];
                let coef = (k as f64) * inv_or_zero(delta);
                new_points.push(coef * self.delta_control_points(i));
            }
        } else {
            new_points = vec![Vector::zero(); n];
        }
        let bspcurve = BSplineCurve::new_unchecked(knot_vec, new_points);
        self.derivation = Some(Box::new(bspcurve));
        self.derivation.as_mut().unwrap()
    }

    /// Calculate derived B-spline curve.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// const N : usize = 100; // sample size in test
    ///
    /// // the knot vector
    /// let knot_vec = KnotVec::from(vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    ///
    /// // the control points
    /// let control_points = vec![
    ///     Vector::new3(1.0, 0.0, 1.0),
    ///     Vector::new3(1.0, 1.0, 1.0),
    ///     Vector::new3(0.0, 2.0, 2.0),
    /// ];
    ///
    /// // `bpsline = (1 - t^2, 2t, 1 + t^2, 1), derived = (-2t, 2, 2t, 0)`
    /// let mut bspline = BSplineCurve::new(knot_vec, control_points);
    /// let derived = bspline.derivation();
    /// for i in 0..N {
    ///     let t = 1.0 / (N as f64) * (i as f64);
    ///     Vector::assert_near2(&derived.subs(t), &Vector::new(-2.0 * t, 2.0, 2.0 * t, 0.0));
    /// }
    /// ```
    pub fn derivation(&mut self) -> &BSplineCurve {
        self.calculate_derivation();
        self.derivation.as_ref().unwrap()
    }

    fn derivation_with_degree_mut(&mut self, degree: usize) -> &mut Self {
        if degree == 0 {
            self
        } else {
            self.calculate_derivation()
                .derivation_with_degree_mut(degree - 1)
        }
    }

    pub fn derivation_with_degree(&mut self, degree: usize) -> &BSplineCurve {
        &*self.derivation_with_degree_mut(degree)
    }

    /// add a knot `x`, and do not change `self` as a curve.  
    /// Return `false` if cannot add the knot, i.e.
    /// * the index of `x` will be lower than the degree, or
    /// * the index of `x` will be higher than the number of control points.
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// # let file = std::fs::File::open("tests/data/examples.tgb").unwrap();
    /// # let geomdata = truck_io::tgb::read(file).unwrap();
    /// # let bspline0 = geomdata.curves[1].clone();
    /// // let bspline0 = BSplineCurve::new(...);
    /// assert_eq!(bspline0.knot_vec().as_slice(), &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0]);
    ///
    /// let mut bspline1 = bspline0.clone();
    /// bspline1.add_knot(0.0);
    /// bspline1.add_knot(1.3);
    /// bspline1.add_knot(2.5);
    /// bspline1.add_knot(3.8);
    /// bspline1.add_knot(4.0);
    /// bspline1.add_knot(5.0);
    /// assert_eq!(bspline1.knot_vec().as_slice(),
    ///     &[0.0, 0.0, 1.0, 1.3, 2.0, 2.5, 3.0, 3.8, 4.0, 4.0, 5.0, 5.0]);
    ///
    /// assert!(bspline0.near2_as_curve(&bspline1));
    /// ```
    pub fn add_knot(&mut self, x: f64) -> &mut Self {
        if x < self.knot_vec[0] {
            self.knot_vec.add_knot(x);
            self.control_points.insert(0, Vector::zero());
            return self;
        }

        let k = self.degree();
        let n = self.control_points.len();

        let idx = self.knot_vec.add_knot(x);
        let start = if idx > k { idx - k } else { 0 };
        let end = if idx > n {
            self.control_points.push(Vector::zero());
            n + 1
        } else {
            self.control_points
                .insert(idx - 1, self.control_point(idx - 1).clone());
            idx
        };
        for i in start..end {
            let i0 = end + start - i - 1;
            let delta = self.knot_vec[i0 + k + 1] - self.knot_vec[i0];
            let a = inv_or_zero(delta) * (self.knot_vec[idx] - self.knot_vec[i0]);
            let p = (1.0 - a) * self.delta_control_points(i0);
            self.control_points[i0] -= p;
        }
        self
    }

    /// remove a knot corresponding to the indice `idx`, and do not change `self` as a curve.  
    /// Return `false` if cannot remove the knot.
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// # let file = std::fs::File::open("tests/data/examples.tgb").unwrap();
    /// # let geomdata = truck_io::tgb::read(file).unwrap();
    /// # let bspline0 = geomdata.curves[0].clone();
    /// // bspline0 = BSplineCurve::new(knot_vec, control_points);
    /// assert_eq!(bspline0.knot_vec().as_slice(),
    ///     &[0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 3.0]);
    ///
    /// let mut bspline1 = bspline0.clone();
    /// bspline1.add_knot(2.5);
    /// bspline1.add_knot(1.3);
    ///
    /// bspline1.remove_knot(6).unwrap(); // remove 2.5
    /// bspline1.remove_knot(4).unwrap(); // remove 1.3
    /// assert!(bspline0.near2_as_curve(&bspline1));
    /// ```
    pub fn remove_knot(&mut self, idx: usize) -> Result<&mut BSplineCurve> {
        let k = self.degree();
        let n = self.control_points.len();
        let knot_vec = &self.knot_vec;

        if idx < k + 1 || idx >= n {
            return Err(Error::CannotRemoveKnot(idx));
        }

        let mut new_points = Vec::with_capacity(k + 1);
        new_points.push(self.control_point(idx - k - 1).clone());
        for i in (idx - k)..idx {
            let delta = knot_vec[i + k + 1] - knot_vec[i];
            let a = inv_or_zero(delta) * (knot_vec[idx] - knot_vec[i]);
            if a.so_small() {
                break;
            } else {
                let p =
                    (1.0 / a) * self.control_point(i) - (1.0 - a) / a * new_points.last().unwrap();
                new_points.push(p);
            }
        }

        if !new_points.last().unwrap().near(self.control_point(idx)) {
            return Err(Error::CannotRemoveKnot(idx));
        }

        for (i, vec) in new_points.into_iter().skip(1).enumerate() {
            self.control_points[idx - k + i] = vec;
        }

        self.control_points.remove(idx);
        self.knot_vec.remove(idx);
        Ok(self)
    }

    /// elevate 1 degree for bezier curve.
    fn elevate_degree_bezier(&mut self) -> &mut Self {
        let k = self.degree();
        self.knot_vec.add_knot(self.knot_vec[0]);
        self.knot_vec
            .add_knot(self.knot_vec[self.knot_vec.len() - 1]);
        self.control_points.push(Vector::zero());
        for i in 0..=(k + 1) {
            let i0 = k + 1 - i;
            let a = (i0 as f64) / ((k + 1) as f64);
            let p = a * self.delta_control_points(i0);
            self.control_points[i0] -= p;
        }
        self
    }

    /// elevate 1 degree.
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// # let file = std::fs::File::open("tests/data/examples.tgb").unwrap();
    /// # let geomdata = truck_io::tgb::read(file).unwrap();
    /// # let bspline0 = geomdata.curves[0].clone();
    /// // bspline0 = BSplineCurve::new(...);
    ///
    /// let mut bspline1 = bspline0.clone();
    /// bspline1.elevate_degree();
    /// assert_eq!(bspline1.degree(), bspline0.degree() + 1);
    /// assert!(bspline0.near_as_curve(&bspline1));
    /// ```
    pub fn elevate_degree(&mut self) -> &mut Self {
        let mut bezier_iter = self.bezier_decomposition().into_iter();
        let mut result = bezier_iter.next().unwrap();
        result.elevate_degree_bezier();
        for mut bezier in bezier_iter {
            result.concat(bezier.elevate_degree_bezier()).unwrap();
        }
        *self = result;
        self
    }

    /// make the B-spline curve clamped
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// # let file = std::fs::File::open("tests/data/examples.tgb").unwrap();
    /// # let geomdata = truck_io::tgb::read(file).unwrap();
    /// # let mut bspline = geomdata.curves[1].clone();
    /// // let bspline = BSplineCurve::new(...);
    /// assert_eq!(bspline.degree(), 2);
    ///
    /// let (_, mults) = bspline.knot_vec().to_single_multi();
    /// assert_eq!(&mults, &[1, 1, 1, 1, 1, 1]);
    ///
    /// bspline.clamp();
    ///
    /// let (_, mults) = bspline.knot_vec().to_single_multi();
    /// assert_eq!(&mults, &[3, 1, 1, 1, 1, 3]);
    /// ```
    #[inline(always)]
    pub fn clamp(&mut self) -> &mut Self {
        let degree = self.degree();

        let s = self.knot_vec.multiplicity(0);
        for _ in s..=degree {
            self.add_knot(self.knot_vec[0]);
        }

        let n = self.knot_vec.len();
        let s = self.knot_vec.multiplicity(n - 1);
        for _ in s..=degree {
            self.add_knot(self.knot_vec[n - 1]);
        }
        self
    }

    /// remove knots in order from the back
    /// # Remarks
    /// All other B-spline algorithms (add_knot, remove_knot, etc...) do not call `optimize` at the end.
    /// If you want to keep the curve in optimal condition, you can call "optimize" manually.
    pub fn optimize(&mut self) -> &mut Self {
        self.derivation = None;
        loop {
            let n = self.knot_vec.len();
            let mut flag = true;
            for i in 1..=n {
                flag = flag && self.remove_knot(n - i).is_err();
            }
            if flag {
                break;
            }
        }
        self
    }

    /// make two splines have the same degrees.
    pub fn syncro_degree(&mut self, other: &mut BSplineCurve) {
        let (degree0, degree1) = (self.degree(), other.degree());
        for _ in degree0..degree1 {
            self.elevate_degree();
        }
        for _ in degree1..degree0 {
            other.elevate_degree();
        }
    }
    /// make two splines have the same normalized knot vectors.
    pub fn syncro_knot(self: &mut BSplineCurve, other: &mut BSplineCurve) {
        self.knot_normalize();
        other.knot_normalize();

        let mut i = 0;
        let mut j = 0;
        while !self.knot(i).near2(&1.0) || !other.knot(j).near2(&1.0) {
            if self.knot(i) - other.knot(j) > f64::TOLERANCE {
                self.add_knot(other.knot(j));
            } else if other.knot(j) - self.knot(i) > f64::TOLERANCE {
                other.add_knot(self.knot(i));
            }
            i += 1;
            j += 1;
        }

        if self.knot_vec.len() < other.knot_vec.len() {
            for _ in 0..(other.knot_vec.len() - self.knot_vec.len()) {
                self.add_knot(1.0);
            }
        } else if other.knot_vec.len() < self.knot_vec.len() {
            for _ in 0..(self.knot_vec.len() - other.knot_vec.len()) {
                other.add_knot(1.0);
            }
        }
    }

    /// cut the curve to two curves at the parameter `t`
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// # let file = std::fs::File::open("tests/data/examples.tgb").unwrap();
    /// # let geomdata = truck_io::tgb::read(file).unwrap();
    /// # let mut bspline = geomdata.curves[0].clone();
    /// const N : usize = 100; // sample size for test
    ///
    /// // let bspline = BSplineCurve::new(...).unwrap();
    /// let (knots, _) = bspline.knot_vec().to_single_multi();
    /// assert_eq!(&knots, &[0.0, 1.0, 2.0, 3.0]);
    ///
    /// let mut part0 = bspline.clone();
    /// let part1 = part0.cut(1.8);
    /// for i in 0..=N {
    ///     let t = 1.8 * (i as f64) / (N as f64);
    ///     Vector::assert_near2(&bspline.subs(t), &part0.subs(t));
    /// }
    /// for i in 0..=N {
    ///     let t = 1.8 + 1.2 * (i as f64) / (N as f64);
    ///     Vector::assert_near2(&bspline.subs(t), &part1.subs(t));
    /// }
    /// ```
    pub fn cut(&mut self, mut t: f64) -> BSplineCurve {
        let degree = self.degree();

        let idx = match self.knot_vec.floor(t) {
            Ok(idx) => idx,
            Err(_) => {
                let bspline = self.clone();
                let knot_vec = KnotVec::from(vec![t, self.knot_vec[0]]);
                let ctrl_pts = vec![Vector::zero()];
                *self = BSplineCurve::new(knot_vec, ctrl_pts);
                return bspline;
            }
        };
        let s = if t.near(&self.knot_vec[idx]) {
            t = self.knot_vec[idx];
            self.knot_vec.multiplicity(idx)
        } else {
            0
        };

        for _ in s..=degree {
            self.add_knot(t);
        }

        let k = self.knot_vec.floor(t).unwrap();
        let m = self.knot_vec.len();
        let n = self.control_points.len();
        let knot_vec0 = self.knot_vec.sub_vec(0..=k);
        let knot_vec1 = self.knot_vec.sub_vec((k - degree)..m);
        let control_points0 = Vec::from(&self.control_points[0..(k - degree)]);
        let control_points1 = Vec::from(&self.control_points[(k - degree)..n]);
        *self = BSplineCurve::new_unchecked(knot_vec0, control_points0);
        BSplineCurve::new_unchecked(knot_vec1, control_points1)
    }

    /// separate `self` to some parts of Bezier curves.
    pub fn bezier_decomposition(&self) -> Vec<BSplineCurve> {
        let mut bspline = self.clone();
        bspline.clamp();
        let (knots, _) = self.knot_vec.to_single_multi();
        let n = knots.len();

        let mut result = Vec::new();
        for i in 2..n {
            result.push(bspline.cut(knots[n - i]));
        }
        result.push(bspline);
        result.reverse();
        result
    }

    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// # let file = std::fs::File::open("tests/data/examples.tgb").unwrap();
    /// # let geomdata = truck_io::tgb::read(file).unwrap();
    /// # let mut bspline = geomdata.curves[0].clone();
    ///
    /// // let bspline = BSplineCurve::new(...).unwrap();
    /// let (knots, _) = bspline.knot_vec().to_single_multi();
    /// assert_eq!(&knots, &[0.0, 1.0, 2.0, 3.0]);
    ///
    /// let mut part0 = bspline.clone();
    /// let mut part1 = part0.cut(1.8);
    /// part0.concat(&mut part1);
    /// assert!(bspline.near_as_curve(&part0));
    /// ```
    pub fn concat(&mut self, other: &mut BSplineCurve) -> Result<&mut Self> {
        self.syncro_degree(other);
        self.clamp();
        other.clamp();
        self.knot_vec.concat(&other.knot_vec, self.degree())?;
        for point in &other.control_points {
            self.control_points.push(point.clone());
        }
        Ok(self)
    }

    /// make the curve to locally injective.
    /// # Remarks
    /// If `self` is a constant curve, return the segment from the first knot to the second knot.
    /// # Example
    /// ```
    /// use truck_geometry::*;
    /// const N : usize = 100; // sample size for test
    ///
    /// let knot_vec = KnotVec::from(vec![0.0, 0.0, 0.0, 1.0, 3.0, 4.0, 4.0, 4.0]);
    /// let control_points = vec![
    ///     Vector::new(1.0, 0.0, 0.0, 0.0),
    ///     Vector::new(0.0, 1.0, 0.0, 0.0),
    ///     Vector::new(0.0, 1.0, 0.0, 0.0),
    ///     Vector::new(0.0, 1.0, 0.0, 0.0),
    ///     Vector::new(0.0, 0.0, 0.0, 1.0),
    /// ];
    ///
    /// let bspline = BSplineCurve::new(knot_vec, control_points);
    /// let mut flag = false;
    /// for i in 0..N {
    ///     let t = 4.0 * (i as f64) / (N as f64);
    ///     flag = flag || bspline.subs(t).near(&bspline.subs(t + 1.0 / (N as f64)));
    /// }
    /// assert!(flag);
    ///
    /// let mut bspline0 = bspline.clone();
    /// bspline0.make_locally_injective().knot_normalize();
    /// let mut flag = false;
    /// for i in 0..N {
    ///     let t = 1.0 * (i as f64) / (N as f64);
    ///     flag = flag || bspline.subs(t).near(&bspline.subs(t + 1.0 / (N as f64)));
    /// }
    /// assert!(!flag);
    /// ```
    pub fn make_locally_injective(&mut self) -> &mut Self {
        let beziers = self.bezier_decomposition();
        *self = beziers[0].clone();
        let mut x = 0.0;
        for mut bezier in beziers.into_iter().skip(1) {
            if bezier.is_const() {
                x += bezier.knot_vec.range_length();
            } else {
                self.concat(bezier.knot_translate(-x)).unwrap();
            }
        }
        self
    }

    /// serch the parameter `t` which minimize |self(t) - point| by Newton's method with initial guess `hint`.
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// # let file = std::fs::File::open("tests/data/examples.tgb").unwrap();
    /// # let geomdata = truck_io::tgb::read(file).unwrap();
    /// # let mut bspline = geomdata.curves[0].clone();
    ///
    /// // let bspline = BSplineCurve::new(...).unwrap();
    /// let (knots, _) = bspline.knot_vec().to_single_multi();
    /// assert_eq!(&knots, &[0.0, 1.0, 2.0, 3.0]);
    ///
    /// let pt = bspline.subs(1.2);
    /// let t = bspline.search_nearest_parameter(&pt, 1.0).unwrap();
    /// f64::assert_near(&t, &1.2);
    /// ```
    pub fn search_nearest_parameter(&mut self, point: &Vector, hint: f64) -> Result<f64> {
        self.sub_search_nearest_parameter(point, hint, 0)
    }

    fn sub_search_nearest_parameter(
        &mut self,
        point: &Vector,
        hint: f64,
        counter: usize,
    ) -> Result<f64> {
        let pt = self.subs(hint) - point;
        let der = self.derivation().subs(hint);
        let der2 = self.derivation_with_degree(2).subs(hint);
        let f = &der * &pt;
        let fprime = &der2 * &pt + der.norm2();
        let t = hint - f / fprime;
        if t.near(&hint) {
            Ok(t)
        } else if counter == 100 {
            Err(Error::NotConverge)
        } else {
            self.sub_search_nearest_parameter(point, t, counter + 1)
        }
    }
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// # let file = std::fs::File::open("tests/data/examples.tgb").unwrap();
    /// # let geomdata = truck_io::tgb::read(file).unwrap();
    /// # let mut bspline = geomdata.curves[0].clone();
    ///
    /// // let bspline = BSplineCurve::new(...).unwrap();
    /// let (knots, _) = bspline.knot_vec().to_single_multi();
    /// assert_eq!(&knots, &[0.0, 1.0, 2.0, 3.0]);
    ///
    /// let mut part = bspline.clone();
    /// let mut part = part.cut(0.6);
    /// part.cut(2.8);
    /// assert!(part.is_arc_of(&mut bspline, 0.6).is_some());
    /// *part.control_point_mut(2) += Vector::new(1.0, 2.0, 3.0, 4.0);
    /// assert!(part.is_arc_of(&mut bspline, 0.6).is_none());
    /// ```
    pub fn is_arc_of(&self, curve: &mut BSplineCurve, hint: f64) -> Option<f64> {
        let degree = std::cmp::max(self.degree(), curve.degree()) * 3 + 1;
        let (knots, _) = self.knot_vec.to_single_multi();
        if !self.subs(knots[0]).near(&curve.subs(hint)) {
            return None;
        }

        let mut hint = hint;
        for i in 1..knots.len() {
            let range = knots[i] - knots[i - 1];
            for j in 1..=degree {
                let t = knots[i - 1] + range * (j as f64) / (degree as f64);
                let pt = self.subs(t);
                let res = curve.search_nearest_parameter(&pt, hint);
                if let Ok(res) = res {
                    if hint > res {
                        return None;
                    } else {
                        hint = res;
                    }
                } else {
                    return None;
                }
                if !curve.subs(hint).near(&pt) {
                    return None;
                }
            }
        }
        Some(hint)
    }

    /// serch the parameter `t` which minimize |self(t) - point| in the projected space
    /// by Newton's method with initial guess `hint`.
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// # let file = std::fs::File::open("tests/data/examples.tgb").unwrap();
    /// # let geomdata = truck_io::tgb::read(file).unwrap();
    /// # let mut bspline = geomdata.curves[0].clone();
    ///
    /// // let bspline = BSplineCurve::new(...).unwrap();
    /// let (knots, _) = bspline.knot_vec().to_single_multi();
    /// assert_eq!(&knots, &[0.0, 1.0, 2.0, 3.0]);
    ///
    /// let pt = bspline.subs(1.2);
    /// let t = bspline.search_projected_nearest_parameter(&pt, 1.0).unwrap();
    /// f64::assert_near(&t, &1.2);
    /// ```
    pub fn search_projected_nearest_parameter(&mut self, point: &Vector, hint: f64) -> Result<f64> {
        self.sub_search_projected_nearest_parameter(point, hint, 0)
    }

    fn sub_search_projected_nearest_parameter(
        &mut self,
        point: &Vector,
        hint: f64,
        counter: usize,
    ) -> Result<f64> {
        let pt = self.subs(hint);
        let der = self.derivation().subs(hint);
        let der2 = self.derivation_with_degree(2).subs(hint);
        let der2 = pt.derivation2_projection(&der, &der2);
        let der = pt.derivation_projection(&der);
        let pt = pt.projection() - point.projection();
        let f = &der * &pt;
        let fprime = &der2 * &pt + der.norm2();
        let t = hint - f / fprime;
        if t.near2(&hint) {
            Ok(t)
        } else if counter == 100 {
            Err(Error::NotConverge)
        } else {
            self.sub_search_projected_nearest_parameter(point, t, counter + 1)
        }
    }
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// # let file = std::fs::File::open("tests/data/examples.tgb").unwrap();
    /// # let geomdata = truck_io::tgb::read(file).unwrap();
    /// # let mut bspline = geomdata.curves[2].clone();
    ///
    /// // let bspline = BSplineCurve::new(...).unwrap();
    /// let (knots, _) = bspline.knot_vec().to_single_multi();
    /// assert_eq!(&knots, &[0.0, 0.5, 1.0]);
    ///
    /// let mut part = bspline.clone();
    /// let mut part = part.cut(0.2);
    /// part.cut(0.8);
    /// assert!(part.is_projected_arc_of(&mut bspline, 0.2).is_some());
    /// *part.control_point_mut(1) += Vector::new(1.0, 2.0, 3.0, 4.0);
    /// assert!(part.is_projected_arc_of(&mut bspline, 0.2).is_none());
    /// ```
    pub fn is_projected_arc_of(&self, curve: &mut BSplineCurve, hint: f64) -> Option<f64> {
        let degree = std::cmp::max(self.degree(), curve.degree()) * 3 + 1;
        let (knots, _) = self.knot_vec.to_single_multi();
        if !self.subs(knots[0]).projection().near(&curve.subs(hint).projection()) {
            return None;
        }

        let mut hint = hint;
        for i in 1..knots.len() {
            let range = knots[i] - knots[i - 1];
            for j in 1..=degree {
                let t = knots[i - 1] + range * (j as f64) / (degree as f64);
                let pt = self.subs(t);
                let res = curve.search_projected_nearest_parameter(&pt, hint);
                if let Ok(res) = res {
                    if hint <= res {
                        hint = res;
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
                if !curve.subs(hint).projection().near(&pt.projection()) {
                    return None;
                }
            }
        }
        Some(hint)
    }

    fn sub_near_as_curve<F: Fn(&Vector, &Vector) -> bool>(
        &self,
        other: &BSplineCurve,
        div_coef: usize,
        ord: F,
    ) -> bool {
        if !self.knot_vec[0].near(&other.knot_vec[0])
            || !self
                .knot_vec
                .range_length()
                .near(&other.knot_vec.range_length())
        {
            return false;
        }

        let division = std::cmp::max(self.degree(), other.degree()) * div_coef;
        for i in 0..(self.knot_vec.len() - 1) {
            let delta = self.knot_vec[i + 1] - self.knot_vec[i];
            if delta.so_small() {
                continue;
            }

            for j in 0..division {
                let t = self.knot_vec[i] + delta * (j as f64) / (division as f64);
                if !ord(&self.subs(t), &other.subs(t)) {
                    return false;
                }
            }
        }
        true
    }

    /// determine `self` and `other` is near as the B-spline curves.  
    /// Divide each knot interval into the number of degree equal parts,
    /// and check `|self(t) - other(t)| < TOLERANCE`for each end points `t`.
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// # let file = std::fs::File::open("tests/data/examples.tgb").unwrap();
    /// # let geomdata = truck_io::tgb::read(file).unwrap();
    /// # let bspline0 = geomdata.curves[2].clone();
    /// // bspline0: BSplineCurve
    /// let mut bspline1 = bspline0.clone();
    /// assert!(bspline0.near_as_curve(&bspline1));
    /// *bspline1.control_point_mut(1) += Vector::new(1.0, 2.0, 3.0, 4.0);
    /// assert!(!bspline0.near_as_curve(&bspline1));
    #[inline(always)]
    pub fn near_as_curve(&self, other: &BSplineCurve) -> bool {
        self.sub_near_as_curve(other, 1, |x, y| x.near(y))
    }

    /// determine `self` and `other` is near in square order as the B-spline curves.  
    /// Divide each knot interval into the number of degree equal parts,
    /// and check `|self(t) - other(t)| < TOLERANCE`for each end points `t`.
    #[inline(always)]
    pub fn near2_as_curve(&self, other: &BSplineCurve) -> bool {
        self.sub_near_as_curve(other, 1, |x, y| x.near2(y))
    }

    /// determine `self` and `other` is near order as the NURBS curve in 3D space.  
    /// Divide each knot interval into the number of degree + 1 equal parts,
    /// and check `|self(t) - other(t)| < TOLERANCE`for each end points `t`.
    #[inline(always)]
    pub fn near_as_projected_curve(&self, other: &BSplineCurve) -> bool {
        self.sub_near_as_curve(other, 2, |x, y| x.projection().near(&y.projection()))
    }

    /// determine `self` and `other` is near in square order as the NURBS curves in 3D space.  
    /// Divide each knot interval into the number of degree + 1 equal parts,
    /// and check `|self(t) - other(t)| < TOLERANCE`for each end points `t`.
    #[inline(always)]
    pub fn near2_as_projected_curve(&self, other: &BSplineCurve) -> bool {
        self.sub_near_as_curve(other, 2, |x, y| x.projection().near2(&y.projection()))
    }
}

impl std::ops::MulAssign<&Matrix> for BSplineCurve {
    /// A matrix `mat` acts to each control points.
    #[inline(always)]
    fn mul_assign(&mut self, mat: &Matrix) {
        for vec in &mut self.control_points {
            *vec *= mat;
        }
    }
}

impl std::ops::MulAssign<Matrix> for BSplineCurve {
    /// A matrix `mat` acts to each control points.
    #[inline(always)]
    fn mul_assign(&mut self, mat: Matrix) {
        self.mul_assign(&mat);
    }
}

impl std::ops::Mul<&Matrix> for &BSplineCurve {
    type Output = BSplineCurve;

    /// A matrix `mat` acts to each control points.
    #[inline(always)]
    fn mul(self, mat: &Matrix) -> BSplineCurve {
        let mut new_spline = self.clone();
        new_spline *= mat;
        new_spline
    }
}

impl std::ops::Mul<Matrix> for &BSplineCurve {
    type Output = BSplineCurve;

    /// A matrix `mat` acts to each control points.
    #[inline(always)]
    fn mul(self, mat: Matrix) -> BSplineCurve {
        self * &mat
    }
}

impl std::ops::Mul<&Matrix> for BSplineCurve {
    type Output = BSplineCurve;

    /// A matrix `mat` acts to each control points.
    #[inline(always)]
    fn mul(mut self, mat: &Matrix) -> BSplineCurve {
        self *= mat;
        self
    }
}

impl std::ops::Mul<Matrix> for BSplineCurve {
    type Output = BSplineCurve;

    /// A matrix `mat` acts to each control points.
    #[inline(always)]
    fn mul(self, mat: Matrix) -> BSplineCurve {
        self * &mat
    }
}

impl std::ops::Mul<&BSplineCurve> for &Matrix {
    type Output = BSplineCurve;

    /// A matrix `mat` acts on each control points.
    #[inline(always)]
    fn mul(self, bspline: &BSplineCurve) -> BSplineCurve {
        let mut new_spline = bspline.clone();
        for vec in &mut new_spline.control_points {
            *vec = self * &*vec;
        }
        new_spline
    }
}

impl std::ops::Mul<&BSplineCurve> for Matrix {
    type Output = BSplineCurve;

    /// A matrix `mat` acts on each control points.
    #[inline(always)]
    fn mul(self, bspline: &BSplineCurve) -> BSplineCurve {
        &self * bspline
    }
}

impl std::ops::Mul<BSplineCurve> for &Matrix {
    type Output = BSplineCurve;

    /// A matrix `mat` acts on each control points.
    #[inline(always)]
    fn mul(self, mut bspline: BSplineCurve) -> BSplineCurve {
        for vec in &mut bspline.control_points {
            *vec = self * &*vec;
        }
        bspline
    }
}

impl std::ops::Mul<BSplineCurve> for Matrix {
    type Output = BSplineCurve;

    /// A matrix `mat` acts on each control points.
    #[inline(always)]
    fn mul(self, bspline: BSplineCurve) -> BSplineCurve {
        &self * bspline
    }
}

impl std::ops::Mul<&BSplineCurve> for &BSplineCurve {
    type Output = BSplineSurface;

    /// tensor surface
    #[inline(always)]
    fn mul(self, other: &BSplineCurve) -> BSplineSurface {
        let knot_vecs = (self.knot_vec.clone(), other.knot_vec.clone());
        let mut control_points = Vec::new();
        for i in 0..self.control_points.len() {
            control_points.push(Vec::new());
            for j in 0..other.control_points.len() {
                control_points[i].push(self.control_point(i) % other.control_point(j));
            }
        }
        BSplineSurface::new_unchecked(knot_vecs, control_points)
    }
}

#[test]
fn test_near_as_curve() {
    let knot_vec = KnotVec::from(vec![
        0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 5.0, 5.0, 5.0,
    ]);
    let control_points = vec![
        Vector::new(1.0, 0.0, 0.0, 0.0),
        Vector::new(0.0, 1.0, 0.0, 0.0),
        Vector::new(0.0, 0.0, 1.0, 0.0),
        Vector::new(0.0, 0.0, 0.0, 1.0),
        Vector::new(1.0, 1.0, 0.0, 0.0),
        Vector::new(1.0, 0.0, 1.0, 0.0),
        Vector::new(1.0, 0.0, 0.0, 1.0),
        Vector::new(1.0, 1.0, 1.0, 0.0),
    ];
    let bspline0 = BSplineCurve::new(knot_vec, control_points.clone());
    let knot_vec = KnotVec::from(vec![
        0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 2.5, 3.0, 4.0, 5.0, 5.0, 5.0, 5.0,
    ]);
    let control_points = vec![
        control_points[0].clone(),
        control_points[1].clone(),
        control_points[2].clone(),
        &control_points[3] * (5.0 / 6.0) + &control_points[2] * (1.0 / 6.0),
        &control_points[4] * 0.5 + &control_points[3] * 0.5,
        &control_points[5] * (1.0 / 6.0) + &control_points[4] * (5.0 / 6.0),
        control_points[5].clone(),
        control_points[6].clone(),
        control_points[7].clone(),
    ];
    let bspline1 = BSplineCurve::new(knot_vec, control_points.clone());
    let knot_vec = KnotVec::from(vec![
        0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 5.0, 5.0, 5.0,
    ]);
    let control_points = vec![
        Vector::new(1.0, 0.0, 0.0, 0.0),
        Vector::new(0.0, 1.0, 0.0, 0.0),
        Vector::new(0.0, 0.0, 1.0, 0.0),
        Vector::new(0.0, 0.0, 0.0, 1.0),
        Vector::new(1.0, 1.01, 0.0, 0.0),
        Vector::new(1.0, 0.0, 1.0, 0.0),
        Vector::new(1.0, 0.0, 0.0, 1.0),
        Vector::new(1.0, 1.0, 1.0, 0.0),
    ];
    let bspline2 = BSplineCurve::new(knot_vec, control_points.clone());
    assert!(bspline0.near_as_curve(&bspline1));
    assert!(!bspline0.near_as_curve(&bspline2));
}
