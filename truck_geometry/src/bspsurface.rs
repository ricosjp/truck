use crate::errors::Error;
use crate::tolerance::inv_or_zero;
use crate::*;

impl BSplineSurface {
    /// constructor.
    /// # Arguments
    /// * `knot_vecs` - the knot vectors
    /// * `control_points` - the vector of the control points
    /// # Panics
    /// There are 3 rules for construct B-spline curve.
    /// * The number of knots is more than the one of control points.
    /// * There exist at least two different knots.
    /// * There are at least one control point.
    #[inline(always)]
    pub fn new(knot_vecs: (KnotVec, KnotVec), control_points: Vec<Vec<Vector>>) -> BSplineSurface {
        if control_points.is_empty() {
            panic!("{}", Error::EmptyControlPoints)
        } else if control_points[0].is_empty() {
            panic!("{}", Error::EmptyControlPoints)
        } else if knot_vecs.0.len() <= control_points.len() {
            panic!(
                "{}",
                Error::TooShortKnotVector(knot_vecs.0.len(), control_points.len(),)
            )
        } else if knot_vecs.1.len() <= control_points[0].len() {
            panic!(
                "{}",
                Error::TooShortKnotVector(knot_vecs.1.len(), control_points[0].len(),)
            )
        } else if knot_vecs.0.range_length().so_small() || knot_vecs.1.range_length().so_small() {
            panic!("{}", Error::ZeroRange)
        } else {
            let len = control_points[0].len();
            if control_points
                .iter()
                .fold(false, |flag, vec| flag || vec.len() != len)
            {
                panic!("{}", Error::IrregularControlPoints)
            } else {
                BSplineSurface {
                    knot_vecs: knot_vecs,
                    control_points: control_points,
                    first_derivation: None,
                    second_derivation: None,
                }
            }
        }
    }
    /// constructor.
    /// # Arguments
    /// * `knot_vecs` - the knot vectors
    /// * `control_points` - the vector of the control points
    /// # Failures
    /// There are 3 rules for construct B-spline curve.
    /// * The number of knots is more than the one of control points.
    /// * There exist at least two different knots.
    /// * There are at least one control point.
    #[inline(always)]
    pub fn try_new(
        knot_vecs: (KnotVec, KnotVec),
        control_points: Vec<Vec<Vector>>,
    ) -> Result<BSplineSurface> {
        if control_points.is_empty() {
            Err(Error::EmptyControlPoints)
        } else if control_points[0].is_empty() {
            Err(Error::EmptyControlPoints)
        } else if knot_vecs.0.len() <= control_points.len() {
            Err(Error::TooShortKnotVector(
                knot_vecs.0.len(),
                control_points.len(),
            ))
        } else if knot_vecs.1.len() <= control_points[0].len() {
            Err(Error::TooShortKnotVector(
                knot_vecs.1.len(),
                control_points[0].len(),
            ))
        } else if knot_vecs.0.range_length().so_small() || knot_vecs.1.range_length().so_small() {
            Err(Error::ZeroRange)
        } else {
            let len = control_points[0].len();
            if control_points
                .iter()
                .fold(false, |flag, vec| flag || vec.len() != len)
            {
                Err(Error::IrregularControlPoints)
            } else {
                Ok(BSplineSurface {
                    knot_vecs: knot_vecs,
                    control_points: control_points,
                    first_derivation: None,
                    second_derivation: None,
                })
            }
        }
    }
    /// constructor.
    /// # Arguments
    /// * `knot_vecs` - the knot vectors
    /// * `control_points` - the vector of the control points
    /// # Failures
    /// This method is prepared only for performance-critical development and is not recommended.
    /// This method does NOT check the 3 rules for constructing B-spline surface.
    /// The programmer must guarantee these conditions before using this method.
    #[inline(always)]
    pub fn new_unchecked(
        knot_vecs: (KnotVec, KnotVec),
        control_points: Vec<Vec<Vector>>,
    ) -> BSplineSurface {
        BSplineSurface {
            knot_vecs: knot_vecs,
            control_points: control_points,
            first_derivation: None,
            second_derivation: None,
        }
    }

    /// the reference of the knot vectors
    #[inline(always)]
    pub fn knot_vecs(&self) -> &(KnotVec, KnotVec) {
        &self.knot_vecs
    }

    /// the control points
    #[inline(always)]
    pub fn control_points(&self) -> &Vec<Vec<Vector>> {
        &self.control_points
    }

    /// the control point corresponding to the index `(idx0, idx1)`.
    #[inline(always)]
    pub fn control_point(&self, idx0: usize, idx1: usize) -> &Vector {
        &self.control_points[idx0][idx1]
    }

    /// the mutable reference of the control point corresponding to index `(idx0, idx1)`.
    #[inline(always)]
    pub fn control_point_mut(&mut self, idx0: usize, idx1: usize) -> &mut Vector {
        self.first_derivation = None;
        self.second_derivation = None;
        &mut self.control_points[idx0][idx1]
    }

    /// the degrees of B-spline surface
    #[inline(always)]
    pub fn degrees(&self) -> (usize, usize) {
        (
            self.knot_vecs.0.len() - self.control_points.len() - 1,
            self.knot_vecs.1.len() - self.control_points[0].len() - 1,
        )
    }

    /// substitution to B-spline surface. private method
    #[inline(always)]
    fn substitution(&self, u: f64, v: f64) -> Vector {
        let (degree0, degree1) = self.degrees();
        let basis0 = self
            .knot_vecs
            .0
            .bspline_basis_functions(degree0, u)
            .unwrap();
        let basis1 = self
            .knot_vecs
            .1
            .bspline_basis_functions(degree1, v)
            .unwrap();
        let mut res = Vector::zero();
        for i in 0..self.control_points.len() {
            for j in 0..self.control_points[i].len() {
                res += self.control_point(i, j) * (basis0[i] * basis1[j]);
            }
        }
        res
    }

    #[inline(always)]
    fn delta0_control_points(&self, i: usize, j: usize) -> Vector {
        if i == 0 {
            self.control_point(i, j).clone()
        } else if i == self.control_points.len() {
            -self.control_point(i - 1, j)
        } else {
            self.control_point(i, j) - self.control_point(i - 1, j)
        }
    }

    #[inline(always)]
    fn delta1_control_points(&self, i: usize, j: usize) -> Vector {
        if j == 0 {
            self.control_point(i, j).clone()
        } else if j == self.control_points[0].len() {
            -self.control_point(i, j - 1)
        } else {
            self.control_point(i, j) - self.control_point(i, j - 1)
        }
    }

    /// Calculate derived B-spline surface by the first parameter.
    pub fn first_derivation(&mut self) -> &BSplineSurface {
        if let Some(ref derivation) = self.first_derivation {
            return derivation;
        }

        let n0 = self.control_points.len();
        let n1 = self.control_points[0].len();
        let (k, _) = self.degrees();
        let (knot_vec0, knot_vec1) = self.knot_vecs.clone();

        let new_points = if k > 0 {
            let mut new_points = vec![vec![Vector::zero(); n1]; n0 + 1];
            for i in 0..=n0 {
                let delta = knot_vec0[i + k] - knot_vec0[i];
                let coef = (k as f64) * inv_or_zero(delta);
                for j in 0..n1 {
                    new_points[i][j] = coef * self.delta0_control_points(i, j);
                }
            }
            new_points
        } else {
            vec![vec![Vector::zero(); n1]; n0]
        };

        let bspsurface = BSplineSurface {
            knot_vecs: (knot_vec0, knot_vec1),
            control_points: new_points,
            first_derivation: None,
            second_derivation: None,
        };

        self.first_derivation = Some(Box::new(bspsurface));
        self.first_derivation.as_ref().unwrap()
    }

    /// Calculate derived B-spline surface by the second parameter.
    pub fn second_derivation(&mut self) -> &BSplineSurface {
        if let Some(ref derivation) = self.second_derivation {
            return derivation;
        }

        let n0 = self.control_points.len();
        let n1 = self.control_points[0].len();
        let (_, k) = self.degrees();

        let (knot_vec0, knot_vec1) = self.knot_vecs.clone();

        let new_points = if k > 0 {
            let mut new_points = vec![vec![Vector::zero(); n1 + 1]; n0];
            for j in 0..=n1 {
                let delta = knot_vec1[j + k] - knot_vec1[j];
                let coef = (k as f64) * inv_or_zero(delta);
                for i in 0..n0 {
                    new_points[i][j] = coef * self.delta1_control_points(i, j);
                }
            }
            new_points
        } else {
            vec![vec![Vector::zero(); n1]; n0]
        };

        let bspsurface = BSplineSurface {
            knot_vecs: (knot_vec0, knot_vec1),
            control_points: new_points,
            first_derivation: None,
            second_derivation: None,
        };
        self.second_derivation = Some(Box::new(bspsurface));
        self.second_derivation.as_ref().unwrap()
    }

    /// get the normal unit vector at the parameter `(u, v)`.
    pub fn normal_vector(&mut self, u: f64, v: f64) -> Vector {
        let pt = self(u, v);
        let der0 = self.first_derivation()(u, v);
        let der1 = self.second_derivation()(u, v);
        let vec0 = pt.derivation_projection(&der0);
        let vec1 = pt.derivation_projection(&der1);
        vec0 ^ vec1
    }

    /// swap two parameters.
    pub fn swap_axes(&mut self) {
        let knot_vec = self.knot_vecs.0.clone();
        self.knot_vecs.0 = self.knot_vecs.1.clone();
        self.knot_vecs.1 = knot_vec;

        let n0 = self.control_points.len();
        let n1 = self.control_points[0].len();
        let mut new_points = Vec::with_capacity(n1);
        for i in 0..n1 {
            new_points.push(Vec::with_capacity(n0));
            for j in 0..n0 {
                new_points[i].push(self.control_point(j, i).clone());
            }
        }
        self.control_points = new_points;
    }

    /// add a knot `x` of the first parameter, and do not change `self` as a surface.  
    /// Return `false` if cannot add the knot, i.e.
    /// * the index of `x` will be lower than the degree, or
    /// * the index of `x` will be higher than the number of control points.
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// # let file = std::fs::File::open("tests/data/examples.tgb").unwrap();
    /// # let geomdata = truck_io::tgb::read(file).unwrap();
    /// # let bspline0 = geomdata.surfaces[0].clone();
    /// const N : usize = 100; // sample size for test
    ///
    /// // let mut bspline0 = BSplineSurface::new(...);
    /// let (knot, _) = bspline0.knot_vecs().0.to_single_multi();
    /// assert_eq!(&knot, &[0.0, 1.0]);
    ///
    /// // B-spline surface
    /// let mut bspline1 = bspline0.clone();
    /// bspline1.add_knot0(0.0);
    /// bspline1.add_knot0(0.3);
    /// bspline1.add_knot0(0.5);
    /// bspline1.add_knot0(1.0);
    /// assert!(bspline0.near2_as_surface(&bspline1));
    /// ```
    pub fn add_knot0(&mut self, x: f64) -> &mut Self {
        let (k, _) = self.degrees();
        let n0 = self.control_points.len();
        let n1 = self.control_points[0].len();
        if x < self.knot_vecs.0[0] {
            self.knot_vecs.0.add_knot(x);
            self.control_points.insert(0, vec![Vector::zero(); n1]);
            return self;
        }

        let idx = self.knot_vecs.0.add_knot(x);
        let start = if idx > k { idx - k } else { 0 };
        let end = if idx > n0 {
            self.control_points.push(vec![Vector::zero(); n1]);
            n0 + 1
        } else {
            self.control_points
                .insert(idx - 1, self.control_points[idx - 1].clone());
            idx
        };
        for i in start..end {
            let i0 = end + start - i - 1;
            let delta = self.knot_vecs.0[i0 + k + 1] - self.knot_vecs.0[i0];
            let a = inv_or_zero(delta) * (self.knot_vecs.0[idx] - self.knot_vecs.0[i0]);
            for j in 0..n1 {
                let p = (1.0 - a) * self.delta0_control_points(i0, j);
                self.control_points[i0][j] -= p;
            }
        }
        self
    }

    /// add a knot `x` for the second parameter, and do not change `self` as a surface.  
    /// Return `false` if cannot add the knot, i.e.
    /// * the index of `x` will be lower than the degree, or
    /// * the index of `x` will be higher than the number of control points.
    /// # Examples
    /// ```
    /// # let file = std::fs::File::open("tests/data/examples.tgb").unwrap();
    /// # let geomdata = truck_io::tgb::read(file).unwrap();
    /// # let bspline0 = geomdata.surfaces[0].clone();
    /// const N : usize = 100; // sample size for test
    ///
    /// // let mut bspline0 = BSplineSurface::new(...);
    /// let (knot, _) = bspline0.knot_vecs().1.to_single_multi();
    /// assert_eq!(&knot, &[0.0, 0.5, 1.0]);
    ///
    /// // B-spline surface
    /// let mut bspline1 = bspline0.clone();
    /// bspline1.add_knot0(0.0);
    /// bspline1.add_knot0(0.3);
    /// bspline1.add_knot0(0.5);
    /// bspline1.add_knot0(1.0);
    /// assert!(bspline0.near2_as_surface(&bspline1));
    /// ```
    pub fn add_knot1(&mut self, x: f64) -> &mut Self {
        if x < self.knot_vecs.1[0] {
            self.knot_vecs.1.add_knot(x);
            self.control_points
                .iter_mut()
                .for_each(|vec| vec.insert(0, Vector::zero()));
            return self;
        }

        let (_, k) = self.degrees();
        let n0 = self.control_points.len();
        let n1 = self.control_points[0].len();

        let idx = self.knot_vecs.1.add_knot(x);
        let start = if idx > k { idx - k } else { 0 };
        let end = if idx > n1 {
            self.control_points
                .iter_mut()
                .for_each(|vec| vec.push(Vector::zero()));
            n1 + 1
        } else {
            self.control_points
                .iter_mut()
                .for_each(|vec| vec.insert(idx - 1, vec[idx - 1].clone()));
            idx
        };
        for j in start..end {
            let j0 = end + start - j - 1;
            let delta = self.knot_vecs.1[j0 + k + 1] - self.knot_vecs.1[j0];
            let a = inv_or_zero(delta) * (self.knot_vecs.1[idx] - self.knot_vecs.1[j0]);
            for i in 0..n0 {
                let p = (1.0 - a) * self.delta1_control_points(i, j0);
                self.control_points[i][j0] -= p;
            }
        }
        self
    }

    /// remove a knot corresponding to the indice `idx` for the first parameter, and do not change `self` as a curve.  
    /// Return `false` if cannot remove the knot.
    /// # Examples
    /// ```
    /// # let file = std::fs::File::open("tests/data/examples.tgb").unwrap();
    /// # let geomdata = truck_io::tgb::read(file).unwrap();
    /// # let bspline0 = geomdata.surfaces[0].clone();
    /// const N : usize = 100; // sample size for test
    ///
    /// // let mut bspline0 = BSplineSurface::new(...);
    /// let knot: Vec<f64> = bspline0.knot_vecs().0.clone().into();
    /// assert_eq!(&knot, &[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]);
    ///
    /// // B-spline surface
    /// let mut bspline1 = bspline0.clone();
    /// bspline1.add_knot0(0.3);
    /// bspline1.add_knot0(0.5);
    /// bspline1.remove_knot0(4);
    /// bspline1.remove_knot0(4);
    /// assert!(bspline0.near2_as_surface(&bspline1));
    /// ```
    pub fn remove_knot0(&mut self, idx0: usize) -> Result<&mut Self> {
        let (k, _) = self.degrees();
        let n0 = self.control_points.len();
        let n1 = self.control_points[0].len();

        // multiplicity and floor
        let knot_vec0 = &self.knot_vecs.0;
        let s = knot_vec0.multiplicity(idx0);
        let idx = knot_vec0.floor(knot_vec0[idx0]).unwrap();

        // index condition
        if idx < k + 1 || idx > n0 {
            return Err(Error::CannotRemoveKnot(idx0));
        }

        // In ths case of k < s
        if k < s {
            for i in (idx - s)..(idx - k) {
                for j in 0..n1 {
                    if self.control_point(i, j) != self.control_point(i + 1, j) {
                        return Err(Error::CannotRemoveKnot(idx0));
                    }
                }
            }
            self.control_points.remove(idx - s);
            self.knot_vecs.0.remove(idx);
            return Ok(self);
        }

        // able to remove condition
        for j in 0..n1 {
            let a = (knot_vec0[idx] - knot_vec0[idx - s])
                / (knot_vec0[idx - s + k + 1] - knot_vec0[idx - s]);
            let p = (1.0 / a) * self.control_point(idx - s, j)
                - ((1.0 - a) / a) * self.control_point(idx - s - 1, j);
            if !p.near(self.control_point(idx - s + 1, j)) {
                return Err(Error::CannotRemoveKnot(idx0));
            }
        }
        for i in (idx - k)..(idx - s) {
            let i0 = 2 * idx - k - s - i - 1;
            let a = (knot_vec0[idx] - knot_vec0[i0]) / (knot_vec0[i0 + k + 1] - knot_vec0[i0]);
            for j in 0..n1 {
                self.control_points[i0][j] = (1.0 / a) * self.control_point(i0, j)
                    - ((1.0 - a) / a) * self.control_point(i0 - 1, j);
            }
        }
        self.control_points.remove(idx - s);
        self.knot_vecs.0.remove(idx);
        Ok(self)
    }

    /// remove a knot corresponding to the indice `idx` for the first parameter, and do not change `self` as a curve.  
    /// Return `false` if cannot remove the knot.
    /// # Examples
    /// ```
    /// # let file = std::fs::File::open("tests/data/examples.tgb").unwrap();
    /// # let geomdata = truck_io::tgb::read(file).unwrap();
    /// # let bspline0 = geomdata.surfaces[2].clone();
    /// const N : usize = 100; // sample size for test
    ///
    /// // let mut bspline0 = BSplineSurface::new(...);
    /// let knot: Vec<f64> = bspline0.knot_vecs().1.clone().into();
    /// assert_eq!(&knot, &[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    ///
    /// // B-spline surface
    /// let mut bspline1 = bspline0.clone();
    /// bspline1.add_knot1(0.3);
    /// bspline1.add_knot1(0.5);
    /// bspline1.remove_knot1(3);
    /// bspline1.remove_knot1(3);
    /// assert!(bspline0.near2_as_surface(&bspline1));
    /// ```
    pub fn remove_knot1(&mut self, idx1: usize) -> Result<&mut Self> {
        let (_, k) = self.degrees();
        let n0 = self.control_points.len();
        let n1 = self.control_points[0].len();

        // multiplicity and floor
        let knot_vec1 = &self.knot_vecs.1;
        let s = knot_vec1.multiplicity(idx1);
        let idx = knot_vec1.floor(knot_vec1[idx1]).unwrap();

        // index condition
        if idx < k + 1 || idx > n1 {
            return Err(Error::CannotRemoveKnot(idx1));
        }

        // In ths case of k < s
        if k < s {
            for j in (idx - s)..(idx - k) {
                for i in 0..n0 {
                    if !self.control_point(i, j).near(self.control_point(i, j + 1)) {
                        return Err(Error::CannotRemoveKnot(idx1));
                    }
                }
            }
            for i in 0..n0 {
                self.control_points[i].remove(idx - s);
            }
            self.knot_vecs.1.remove(idx);
            return Ok(self);
        }

        let a = (knot_vec1[idx] - knot_vec1[idx - s])
            / (knot_vec1[idx - s + k + 1] - knot_vec1[idx - s]);
        for vec in &self.control_points {
            let p = (1.0 / a) * &vec[idx - s] - ((1.0 - a) / a) * &vec[idx - s - 1];
            if !p.near(&vec[idx - s + 1]) {
                return Err(Error::CannotRemoveKnot(idx1));
            }
        }
        for j in (idx - k)..(idx - s) {
            let j0 = 2 * idx - k - s - j - 1;
            let a = (knot_vec1[idx] - knot_vec1[j0]) / (knot_vec1[j0 + k + 1] - knot_vec1[j0]);
            self.control_points
                .iter_mut()
                .for_each(|vec| vec[j0] = 1.0 / a * &vec[j0] - ((1.0 - a) / a) * &vec[j0 - 1]);
        }
        self.control_points.iter_mut().for_each(|vec| {
            vec.remove(idx - s);
        });
        self.knot_vecs.1.remove(idx);
        Ok(self)
    }

    pub fn homotopy(bspcurve0: &BSplineCurve, bspcurve1: &BSplineCurve) -> BSplineSurface {
        let mut bspcurve0 = bspcurve0.clone();
        let mut bspcurve1 = bspcurve1.clone();

        bspcurve0.syncro_degree(&mut bspcurve1);

        bspcurve0.optimize();
        bspcurve1.optimize();

        bspcurve0.syncro_knot(&mut bspcurve1);

        let knot_vec0 = bspcurve0.knot_vec().clone();
        let knot_vec1 = KnotVec::from(vec![0.0, 0.0, 1.0, 1.0]);
        let mut control_points = Vec::new();
        for i in 0..bspcurve0.control_points().len() {
            control_points.push(Vec::new());
            control_points[i].push(bspcurve0.control_point(i).clone());
            control_points[i].push(bspcurve1.control_point(i).clone());
        }
        BSplineSurface::new_unchecked((knot_vec0, knot_vec1), control_points)
    }

    #[inline(always)]
    pub fn knot_normalize(&mut self) -> &mut Self {
        if let Some(ref mut derivation) = self.first_derivation {
            derivation.knot_normalize();
        }
        if let Some(ref mut derivation) = self.second_derivation {
            derivation.knot_normalize();
        }
        self.knot_vecs.0.normalize().unwrap();
        self.knot_vecs.1.normalize().unwrap();
        self
    }

    /// remove knots in order from the back
    pub fn optimize(&mut self) -> &mut Self {
        loop {
            let (n0, n1) = (self.knot_vecs.0.len(), self.knot_vecs.1.len());
            let mut flag = true;
            for i in 1..=n0 {
                flag = flag && self.remove_knot0(n0 - i).is_err();
            }
            for j in 1..=n1 {
                flag = flag && self.remove_knot1(n1 - j).is_err();
            }
            if flag {
                break;
            }
        }
        self
    }

    /// extract boundary of surface
    pub fn boundary(&self) -> BSplineCurve {
        let (knot_vec0, knot_vec1) = self.knot_vecs.clone();
        let (range0, range1) = (knot_vec0.range_length(), knot_vec1.range_length());
        let control_points0 = self.control_points.iter().map(|x| x[0].clone()).collect();
        let control_points1 = self.control_points.last().unwrap().clone();
        let control_points2 = self
            .control_points
            .iter()
            .map(|x| x.last().unwrap().clone())
            .collect();
        let control_points3 = self.control_points[0].clone();
        let mut bspline0 = BSplineCurve::new_unchecked(knot_vec0.clone(), control_points0);
        let mut bspline1 = BSplineCurve::new_unchecked(knot_vec1.clone(), control_points1);
        let mut bspline2 = BSplineCurve::new_unchecked(knot_vec0.clone(), control_points2);
        let mut bspline3 = BSplineCurve::new_unchecked(knot_vec1.clone(), control_points3);
        bspline0
            .concat(&mut bspline1.knot_translate(range0))
            .unwrap()
            .concat(&mut bspline2.inverse().knot_translate(range0 + range1))
            .unwrap()
            .concat(&mut bspline3.inverse().knot_translate(range0 * 2.0 + range1))
            .unwrap();
        bspline0
    }
    fn sub_near_as_surface<F: Fn(&Vector, &Vector) -> bool>(
        &self,
        other: &BSplineSurface,
        div_coef: usize,
        ord: F,
    ) -> bool {
        if !self.knot_vecs.0[0].near(&other.knot_vecs.0[0])
            || !self
                .knot_vecs
                .0
                .range_length()
                .near(&other.knot_vecs.0.range_length())
        {
            return false;
        }
        if !self.knot_vecs.1[0].near(&other.knot_vecs.1[0])
            || !self
                .knot_vecs
                .1
                .range_length()
                .near(&other.knot_vecs.1.range_length())
        {
            return false;
        }

        let (self_degree0, self_degree1) = self.degrees();
        let (other_degree0, other_degree1) = other.degrees();
        let division0 = std::cmp::max(self_degree0, other_degree0) * div_coef;
        let division1 = std::cmp::max(self_degree1, other_degree1) * div_coef;

        for i0 in 1..self.knot_vecs.0.len() {
            let delta0 = self.knot_vecs.0[i0] - self.knot_vecs.0[i0 - 1];
            if delta0.so_small() {
                continue;
            }
            for j0 in 0..division0 {
                let s = self.knot_vecs.0[i0 - 1] + delta0 * (j0 as f64) / (division0 as f64);
                for i1 in 1..self.knot_vecs.1.len() {
                    let delta1 = self.knot_vecs.1[i1] - self.knot_vecs.1[i1 - 1];
                    if delta1.so_small() {
                        continue;
                    }
                    for j1 in 0..division1 {
                        let t = self.knot_vecs.1[i1 - 1] + delta1 * (j1 as f64) / (division1 as f64);
                        if !ord(&self(s, t), &other(s, t)) {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }
    
    #[inline(always)]
    pub fn near_as_surface(&self, other: &BSplineSurface) -> bool {
        self.sub_near_as_surface(other, 1, |x, y| x.near(y))
    }
    
    #[inline(always)]
    pub fn near2_as_surface(&self, other: &BSplineSurface) -> bool {
        self.sub_near_as_surface(other, 1, |x, y| x.near2(y))
    }
    
    #[inline(always)]
    pub fn near_as_projected_surface(&self, other: &BSplineSurface) -> bool {
        self.sub_near_as_surface(other, 2, |x, y| x.projection().near(&y.projection()))
    }
    
    #[inline(always)]
    pub fn near2_as_projected_surface(&self, other: &BSplineSurface) -> bool {
        self.sub_near_as_surface(other, 2, |x, y| x.projection().near2(&y.projection()))
    }
}

impl FnOnce<(f64, f64)> for BSplineSurface {
    type Output = Vector;

    /// substitution to B-spline surface.
    #[inline(always)]
    extern "rust-call" fn call_once(self, (s, t): (f64, f64)) -> Vector {
        self.substitution(s, t)
    }
}

impl FnMut<(f64, f64)> for BSplineSurface {
    /// substitution to B-spline surface.
    #[inline(always)]
    extern "rust-call" fn call_mut(&mut self, (s, t): (f64, f64)) -> Vector {
        self.substitution(s, t)
    }
}

impl Fn<(f64, f64)> for BSplineSurface {
    /// substitution to B-spline surface.
    #[inline(always)]
    extern "rust-call" fn call(&self, (s, t): (f64, f64)) -> Vector {
        self.substitution(s, t)
    }
}
