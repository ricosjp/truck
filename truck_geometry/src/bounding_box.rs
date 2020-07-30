use crate::*;

impl<T: EntityArray<f64>> BoundingBox<T> {
    /// Creats an empty bounding box
    #[inline(always)]
    pub fn new() -> BoundingBox<T> {
        BoundingBox(Vector::from(T::INFINITY), Vector::from(T::NEG_INFINITY))
    }
    /// Adds a point to the bouding box.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut bdd_box = BoundingBox::new();
    /// bdd_box.push(&vector!(-1, 1));
    /// bdd_box.push(&vector!(1, -1));
    /// assert_eq!(bdd_box.min(), &vector!(-1, -1));
    /// assert_eq!(bdd_box.max(), &vector!(1, 1));
    /// ```
    /// # Remarks
    /// If the added point has NAN component, then the point is not added.
    /// ```
    /// use truck_geometry::*;
    /// let mut bdd_box = BoundingBox::new();
    /// bdd_box.push(&vector!(-1, 1));
    /// bdd_box.push(&vector!(1, -1));
    /// bdd_box.push(&vector!(std::f64::NAN, 100));
    /// bdd_box.push(&vector!(-100, std::f64::NAN));
    /// assert_eq!(bdd_box.min(), &vector!(-1, -1));
    /// assert_eq!(bdd_box.max(), &vector!(1, 1));
    /// ```
    #[inline(always)]
    pub fn push(&mut self, point: &Vector<T>) {
        if point.iter().all(|x| !x.is_nan()) {
            point.iter().zip(&mut self.0).for_each(move |(a, b)| {
                if *a < *b {
                    *b = *a;
                }
            });
            point.iter().zip(&mut self.1).for_each(move |(a, b)| {
                if *a > *b {
                    *b = *a;
                }
            });
        }
    }

    /// Returns the bounding box is empty or not.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut bdd_box = BoundingBox::new();
    /// assert!(bdd_box.is_empty());
    /// bdd_box.push(&vector!(-1, 1));
    /// assert!(!bdd_box.is_empty());
    /// ```
    #[inline(always)]
    pub fn is_empty(&self) -> bool { self.0[0] > self.1[0] }
    /// Returns the reference to the maximum point.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut bdd_box = BoundingBox::new();
    /// bdd_box.push(&vector!(-1, 1));
    /// bdd_box.push(&vector!(1, -1));
    /// assert_eq!(bdd_box.max(), &vector!(1, 1));
    /// ```
    /// # Remarks
    /// If the bounding box is empty, returned vector consists `f64::NEG_INFINITY` components.
    /// ```
    /// use truck_geometry::*;
    /// let bdd_box = BoundingBox::new();
    /// assert_eq!(bdd_box.max(), &vector![f64::NEG_INFINITY; 2]);
    /// ```
    #[inline(always)]
    pub fn max(&self) -> &Vector<T> { &self.1 }
    /// Returns the reference to the minimal point.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut bdd_box = BoundingBox::new();
    /// bdd_box.push(&vector!(-1, 1));
    /// bdd_box.push(&vector!(1, -1));
    /// assert_eq!(bdd_box.min(), &vector!(-1, -1));
    /// ```
    /// # Remarks
    /// If the bounding box is empty, returned vector consists `f64::INFINITY` components.
    /// ```
    /// use truck_geometry::*;
    /// let bdd_box = BoundingBox::new();
    /// assert_eq!(bdd_box.min(), &vector![f64::INFINITY; 2]);
    /// ```
    #[inline(always)]
    pub fn min(&self) -> &Vector<T> { &self.0 }
    /// Returns the diagonal vector.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut bdd_box = BoundingBox::new();
    /// bdd_box.push(&vector!(-2, -3));
    /// bdd_box.push(&vector!(6, 4));
    /// assert_eq!(bdd_box.diagonal(), vector!(8, 7));
    /// ```
    /// # Remarks
    /// If the bounding box is empty, returned vector consists `f64::NEG_INFINITY` components.
    /// ```
    /// use truck_geometry::*;
    /// let bdd_box = BoundingBox::new();
    /// assert_eq!(bdd_box.diagonal(), vector![f64::NEG_INFINITY; 2]);
    /// ```
    #[inline(always)]
    pub fn diagonal(&self) -> Vector<T> { &self.1 - &self.0 }

    /// Returns the diameter of the bounding box.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut bdd_box = BoundingBox::new();
    /// bdd_box.push(&vector!(-1, -3));
    /// bdd_box.push(&vector!(2, 1));
    /// assert_eq!(bdd_box.diameter(), 5.0);
    /// ```
    /// # Remarks
    /// If the bounding box is empty, returnes `f64::NEG_INFINITY`.
    /// ```
    /// use truck_geometry::*;
    /// let bdd_box = BoundingBox::<[f64; 3]>::new();
    /// assert_eq!(bdd_box.diameter(), f64::NEG_INFINITY);
    /// ```
    #[inline(always)]
    pub fn diameter(&self) -> f64 {
        if self.is_empty() {
            f64::NEG_INFINITY
        } else {
            self.diagonal().norm()
        }
    }

    /// Returns the maximum length of the edges of the bounding box.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut bdd_box = BoundingBox::new();
    /// bdd_box.push(&vector!(-1, -3, 2));
    /// bdd_box.push(&vector!(2, 1, 10));
    /// assert_eq!(bdd_box.size(), 8.0);
    /// ```
    /// # Remarks
    /// If the bounding box is empty, returnes `f64::NEG_INFINITY`.
    /// ```
    /// use truck_geometry::*;
    /// let bdd_box = BoundingBox::<[f64; 3]>::new();
    /// assert_eq!(bdd_box.size(), f64::NEG_INFINITY);
    /// ```
    #[inline(always)]
    pub fn size(&self) -> f64 {
        self.0.iter().zip(&self.1).fold(
            f64::NEG_INFINITY,
            move |max, (a, b)| {
                if max > b - a {
                    max
                } else {
                    b - a
                }
            },
        )
    }

    /// Returns the center of the bounding box.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut bdd_box = BoundingBox::new();
    /// bdd_box.push(&vector!(-1, -3));
    /// bdd_box.push(&vector!(5, 1));
    /// assert_eq!(bdd_box.center(), vector!(2, -1));
    /// ```
    /// # Remarks
    /// If the bounding box is empty, returned vector consists `std::f64::NAN` components.
    /// ```
    /// use truck_geometry::*;
    /// let bdd_box = BoundingBox::<[f64; 6]>::new();
    /// assert!(bdd_box.center().iter().all(|x| x.is_nan()));
    /// ```
    #[inline(always)]
    pub fn center(&self) -> Vector<T> { (&self.0 + &self.1) / 2.0 }
}

impl<'a, T: EntityArray<f64> + 'a> std::iter::FromIterator<&'a Vector<T>> for BoundingBox<T> {
    fn from_iter<I: IntoIterator<Item = &'a Vector<T>>>(iter: I) -> BoundingBox<T> {
        let mut bdd_box = BoundingBox::new();
        let bdd_box_mut = &mut bdd_box;
        iter.into_iter().for_each(move |pt| bdd_box_mut.push(pt));
        bdd_box
    }
}

impl<T: EntityArray<f64>> std::iter::FromIterator<Vector<T>> for BoundingBox<T> {
    fn from_iter<I: IntoIterator<Item = Vector<T>>>(iter: I) -> BoundingBox<T> {
        let mut bdd_box = BoundingBox::new();
        let bdd_box_mut = &mut bdd_box;
        iter.into_iter().for_each(move |pt| bdd_box_mut.push(&pt));
        bdd_box
    }
}

impl<T: EntityArray<f64>> std::ops::AddAssign<&BoundingBox<T>> for BoundingBox<T> {
    /// Puts the points in `other` into `self`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use std::iter::FromIterator;
    /// let mut bdd_box = BoundingBox::from_iter(&[
    ///     vector!(3, 2), vector!(5, 6),
    /// ]);
    /// bdd_box += &BoundingBox::from_iter(&[
    ///     vector!(4, 1), vector!(7, 4),
    /// ]);
    /// assert_eq!(bdd_box.min(), &vector!(3, 1));
    /// assert_eq!(bdd_box.max(), &vector!(7, 6));
    /// 
    /// bdd_box += &BoundingBox::new();
    /// assert_eq!(bdd_box.min(), &vector!(3, 1));
    /// assert_eq!(bdd_box.max(), &vector!(7, 6));
    /// ```
    #[inline(always)]
    fn add_assign(&mut self, other: &BoundingBox<T>) {
        self.0.iter_mut().zip(&other.0).for_each(|(a, b)| {
            if *a > *b {
                *a = *b;
            }
        });
        self.1.iter_mut().zip(&other.1).for_each(|(a, b)| {
            if *a < *b {
                *a = *b;
            }
        });
    }
}

impl<T: EntityArray<f64>> std::ops::AddAssign<BoundingBox<T>> for BoundingBox<T> {
    /// Puts the points in `other` into `self`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use std::iter::FromIterator;
    /// let mut bdd_box = BoundingBox::from_iter(&[
    ///     vector!(3, 2), vector!(5, 6),
    /// ]);
    /// bdd_box += BoundingBox::from_iter(&[
    ///     vector!(4, 1), vector!(7, 4),
    /// ]);
    /// assert_eq!(bdd_box.min(), &vector!(3, 1));
    /// assert_eq!(bdd_box.max(), &vector!(7, 6));
    /// 
    /// bdd_box += BoundingBox::new();
    /// assert_eq!(bdd_box.min(), &vector!(3, 1));
    /// assert_eq!(bdd_box.max(), &vector!(7, 6));
    /// ```
    #[inline(always)]
    fn add_assign(&mut self, other: BoundingBox<T>) { *self += &other; }
}

impl<T: EntityArray<f64>> std::ops::Add<&BoundingBox<T>> for &BoundingBox<T> {
    type Output = BoundingBox<T>;
    /// Returns the direct sum of `self` and other.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use std::iter::FromIterator;
    /// let bdd_box0 = BoundingBox::from_iter(&[
    ///     vector!(3, 2), vector!(5, 6),
    /// ]);
    /// let bdd_box1 = BoundingBox::from_iter(&[
    ///     vector!(4, 1), vector!(7, 4),
    /// ]);
    /// let bdd_box = &bdd_box0 + &bdd_box1;
    /// assert_eq!(bdd_box.min(), &vector!(3, 1));
    /// assert_eq!(bdd_box.max(), &vector!(7, 6));
    /// 
    /// let cloned_bdd_box = &bdd_box + &BoundingBox::new();
    /// assert_eq!(cloned_bdd_box.min(), &vector!(3, 1));
    /// assert_eq!(cloned_bdd_box.max(), &vector!(7, 6));
    /// ```
    #[inline(always)]
    fn add(self, other: &BoundingBox<T>) -> BoundingBox<T> { self.clone() + other }
}

impl<T: EntityArray<f64>> std::ops::Add<&BoundingBox<T>> for BoundingBox<T> {
    type Output = BoundingBox<T>;
    /// Returns the direct sum of `self` and other.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use std::iter::FromIterator;
    /// let bdd_box0 = BoundingBox::from_iter(&[
    ///     vector!(3, 2), vector!(5, 6),
    /// ]);
    /// let bdd_box1 = BoundingBox::from_iter(&[
    ///     vector!(4, 1), vector!(7, 4),
    /// ]);
    /// let bdd_box = bdd_box0 + &bdd_box1;
    /// assert_eq!(bdd_box.min(), &vector!(3, 1));
    /// assert_eq!(bdd_box.max(), &vector!(7, 6));
    /// 
    /// let cloned_bdd_box = bdd_box + &BoundingBox::new();
    /// assert_eq!(cloned_bdd_box.min(), &vector!(3, 1));
    /// assert_eq!(cloned_bdd_box.max(), &vector!(7, 6));
    /// ```
    #[inline(always)]
    fn add(mut self, other: &BoundingBox<T>) -> BoundingBox<T> {
        self += other;
        self
    }
}

impl<T: EntityArray<f64>> std::ops::Add<BoundingBox<T>> for &BoundingBox<T> {
    type Output = BoundingBox<T>;
    /// Returns the direct sum of `self` and other.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use std::iter::FromIterator;
    /// let bdd_box0 = BoundingBox::from_iter(&[
    ///     vector!(3, 2), vector!(5, 6),
    /// ]);
    /// let bdd_box1 = BoundingBox::from_iter(&[
    ///     vector!(4, 1), vector!(7, 4),
    /// ]);
    /// let bdd_box = &bdd_box0 + bdd_box1;
    /// assert_eq!(bdd_box.min(), &vector!(3, 1));
    /// assert_eq!(bdd_box.max(), &vector!(7, 6));
    /// 
    /// let cloned_bdd_box = &bdd_box + BoundingBox::new();
    /// assert_eq!(cloned_bdd_box.min(), &vector!(3, 1));
    /// assert_eq!(cloned_bdd_box.max(), &vector!(7, 6));
    /// ```
    #[inline(always)]
    fn add(self, other: BoundingBox<T>) -> BoundingBox<T> { other + self }
}

impl<T: EntityArray<f64>> std::ops::Add<BoundingBox<T>> for BoundingBox<T> {
    type Output = BoundingBox<T>;
    /// Returns the direct sum of `self` and other.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use std::iter::FromIterator;
    /// let bdd_box0 = BoundingBox::from_iter(&[
    ///     vector!(3, 2), vector!(5, 6),
    /// ]);
    /// let bdd_box1 = BoundingBox::from_iter(&[
    ///     vector!(4, 1), vector!(7, 4),
    /// ]);
    /// let bdd_box = bdd_box0 + bdd_box1;
    /// assert_eq!(bdd_box.min(), &vector!(3, 1));
    /// assert_eq!(bdd_box.max(), &vector!(7, 6));
    /// 
    /// let cloned_bdd_box = bdd_box + BoundingBox::new();
    /// assert_eq!(cloned_bdd_box.min(), &vector!(3, 1));
    /// assert_eq!(cloned_bdd_box.max(), &vector!(7, 6));
    /// ```
    #[inline(always)]
    fn add(self, other: BoundingBox<T>) -> BoundingBox<T> { self + &other }
}

impl<T: EntityArray<f64>> std::ops::BitXorAssign<&BoundingBox<T>> for BoundingBox<T> {
    /// Assigns the intersection of `self` and `other` to `self`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use std::iter::FromIterator;
    /// let mut bdd_box = BoundingBox::from_iter(&[
    ///     vector!(3, 2), vector!(5, 6),
    /// ]);
    /// bdd_box ^= &BoundingBox::from_iter(&[
    ///     vector!(4, 1), vector!(7, 4),
    /// ]);
    /// assert_eq!(bdd_box.min(), &vector!(4, 2));
    /// assert_eq!(bdd_box.max(), &vector!(5, 4));
    /// 
    /// bdd_box ^= &BoundingBox::new();
    /// assert!(bdd_box.is_empty());
    /// ```
    #[inline(always)]
    fn bitxor_assign(&mut self, other: &BoundingBox<T>) {
        self.0.iter_mut().zip(&other.0).for_each(|(a, b)| {
            if *a < *b {
                *a = *b;
            }
        });
        self.1.iter_mut().zip(&other.1).for_each(|(a, b)| {
            if *a > *b {
                *a = *b;
            }
        });
    }
}

impl<T: EntityArray<f64>> std::ops::BitXorAssign<BoundingBox<T>> for BoundingBox<T> {
    /// Assigns the intersection of `self` and `other` to `self`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use std::iter::FromIterator;
    /// let mut bdd_box = BoundingBox::from_iter(&[
    ///     vector!(3, 2), vector!(5, 6),
    /// ]);
    /// bdd_box ^= BoundingBox::from_iter(&[
    ///     vector!(4, 1), vector!(7, 4),
    /// ]);
    /// assert_eq!(bdd_box.min(), &vector!(4, 2));
    /// assert_eq!(bdd_box.max(), &vector!(5, 4));
    /// 
    /// bdd_box ^= BoundingBox::new();
    /// assert!(bdd_box.is_empty());
    /// ```
    #[inline(always)]
    fn bitxor_assign(&mut self, other: BoundingBox<T>) { *self ^= &other; }
}

impl<T: EntityArray<f64>> std::ops::BitXor<&BoundingBox<T>> for &BoundingBox<T> {
    type Output = BoundingBox<T>;
    /// Returns the intersection of `self` and `other`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use std::iter::FromIterator;
    /// let bdd_box0 = BoundingBox::from_iter(&[
    ///     vector!(3, 2), vector!(5, 6),
    /// ]);
    /// let bdd_box1 = BoundingBox::from_iter(&[
    ///     vector!(4, 1), vector!(7, 4),
    /// ]);
    /// let bdd_box = &bdd_box0 ^ &bdd_box1;
    /// assert_eq!(bdd_box.min(), &vector!(4, 2));
    /// assert_eq!(bdd_box.max(), &vector!(5, 4));
    /// 
    /// let new_empty = &bdd_box ^ &BoundingBox::new();
    /// assert!(new_empty.is_empty());
    /// ```
    #[inline(always)]
    fn bitxor(self, other: &BoundingBox<T>) -> BoundingBox<T> { self.clone() ^ other }
}

impl<T: EntityArray<f64>> std::ops::BitXor<&BoundingBox<T>> for BoundingBox<T> {
    type Output = BoundingBox<T>;
    /// Returns the intersection of `self` and `other`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use std::iter::FromIterator;
    /// let bdd_box0 = BoundingBox::from_iter(&[
    ///     vector!(3, 2), vector!(5, 6),
    /// ]);
    /// let bdd_box1 = BoundingBox::from_iter(&[
    ///     vector!(4, 1), vector!(7, 4),
    /// ]);
    /// let bdd_box = bdd_box0 ^ &bdd_box1;
    /// assert_eq!(bdd_box.min(), &vector!(4, 2));
    /// assert_eq!(bdd_box.max(), &vector!(5, 4));
    /// 
    /// let new_empty = bdd_box ^ &BoundingBox::new();
    /// assert!(new_empty.is_empty());
    /// ```
    #[inline(always)]
    fn bitxor(mut self, other: &BoundingBox<T>) -> BoundingBox<T> {
        self ^= other;
        self
    }
}

impl<T: EntityArray<f64>> std::ops::BitXor<BoundingBox<T>> for &BoundingBox<T> {
    type Output = BoundingBox<T>;
    /// Returns the intersection of `self` and `other`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use std::iter::FromIterator;
    /// let bdd_box0 = BoundingBox::from_iter(&[
    ///     vector!(3, 2), vector!(5, 6),
    /// ]);
    /// let bdd_box1 = BoundingBox::from_iter(&[
    ///     vector!(4, 1), vector!(7, 4),
    /// ]);
    /// let bdd_box = &bdd_box0 ^ bdd_box1;
    /// assert_eq!(bdd_box.min(), &vector!(4, 2));
    /// assert_eq!(bdd_box.max(), &vector!(5, 4));
    /// 
    /// let new_empty = &bdd_box ^ BoundingBox::new();
    /// assert!(new_empty.is_empty());
    /// ```
    #[inline(always)]
    fn bitxor(self, other: BoundingBox<T>) -> BoundingBox<T> { other ^ self }
}

impl<T: EntityArray<f64>> std::ops::BitXor<BoundingBox<T>> for BoundingBox<T> {
    type Output = BoundingBox<T>;
    /// Returns the intersection of `self` and `other`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use std::iter::FromIterator;
    /// let bdd_box0 = BoundingBox::from_iter(&[
    ///     vector!(3, 2), vector!(5, 6),
    /// ]);
    /// let bdd_box1 = BoundingBox::from_iter(&[
    ///     vector!(4, 1), vector!(7, 4),
    /// ]);
    /// let bdd_box = bdd_box0 ^ bdd_box1;
    /// assert_eq!(bdd_box.min(), &vector!(4, 2));
    /// assert_eq!(bdd_box.max(), &vector!(5, 4));
    /// 
    /// let new_empty = bdd_box ^ BoundingBox::new();
    /// assert!(new_empty.is_empty());
    /// ```
    #[inline(always)]
    fn bitxor(self, other: BoundingBox<T>) -> BoundingBox<T> { self ^ &other }
}
