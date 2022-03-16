use cgmath::*;
use serde::*;
use std::cmp::Ordering;
use std::ops::Index;

/// bounding box
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BoundingBox<V>(V, V);

/// The trait for defining the bounding box
pub trait Bounded<S> {
    /// the result of subtraction
    type Vector;
    #[doc(hidden)]
    fn infinity() -> Self;
    #[doc(hidden)]
    fn neg_infinity() -> Self;
    #[doc(hidden)]
    fn max(&self, other: &Self) -> Self;
    #[doc(hidden)]
    fn min(&self, other: &Self) -> Self;
    #[doc(hidden)]
    fn max_component(one: Self::Vector) -> S;
    #[doc(hidden)]
    fn diagonal(self, other: Self) -> Self::Vector;
    #[doc(hidden)]
    fn mid(self, other: Self) -> Self;
}

macro_rules! pr2 {
    ($a: expr, $b: expr) => {
        $b
    };
}
macro_rules! impl_bounded {
        ($typename: ident, $vectortype: ident, $($num: expr),*) => {
            impl<S: BaseFloat> Bounded<S> for $typename<S> {
                type Vector = $vectortype<S>;
                fn infinity() -> $typename<S> {
                    $typename::new($(pr2!($num, S::infinity())),*)
                }
                fn neg_infinity() -> $typename<S> {
                    $typename::new($(pr2!($num, S::neg_infinity())),*)
                }
                fn max(&self, other: &Self) -> Self {
                    $typename::new(
                        $(
                            if self[$num] < other[$num] {
                                other[$num]
                            } else {
                                self[$num]
                            }
                        ),*
                    )
                }
                fn min(&self, other: &Self) -> Self {
                    $typename::new(
                        $(
                            if self[$num] > other[$num] {
                                other[$num]
                            } else {
                                self[$num]
                            }
                        ),*
                    )
                }
                fn max_component(one: Self::Vector) -> S {
                    let mut max = S::neg_infinity();
                    $(if max < one[$num] { max = one[$num] })*
                    max
                }
                fn diagonal(self, other: Self) -> Self::Vector { self - other }
                fn mid(self, other: Self) -> Self {
                    self + (other - self) / (S::one() + S::one())
                }
            }
        };
    }
impl_bounded!(Vector1, Vector1, 0);
impl_bounded!(Point1, Vector1, 0);
impl_bounded!(Vector2, Vector2, 0, 1);
impl_bounded!(Point2, Vector2, 0, 1);
impl_bounded!(Vector3, Vector3, 0, 1, 2);
impl_bounded!(Point3, Vector3, 0, 1, 2);
impl_bounded!(Vector4, Vector4, 0, 1, 2, 3);

impl<F, V> Default for BoundingBox<V>
where
    F: BaseFloat,
    V: MetricSpace<Metric = F> + Index<usize, Output = F> + Bounded<F> + Copy,
{
    #[inline(always)]
    fn default() -> Self { BoundingBox(V::infinity(), V::neg_infinity()) }
}

impl<F, V> BoundingBox<V>
where
    F: BaseFloat,
    V: MetricSpace<Metric = F> + Index<usize, Output = F> + Bounded<F> + Copy,
{
    /// Creats an empty bounding box
    #[inline(always)]
    pub fn new() -> Self { Self::default() }
    /// Adds a point to the bouding box.
    /// # Examples
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let mut bdd_box = BoundingBox::new();
    /// bdd_box.push(&Vector2::new(-1.0,  1.0));
    /// bdd_box.push(&Vector2::new(1.0,  -1.0));
    /// assert_eq!(bdd_box.min(), &Vector2::new(-1.0,  -1.0));
    /// assert_eq!(bdd_box.max(), &Vector2::new(1.0,  1.0));
    /// ```
    /// # Remarks
    /// If the added point has NAN component, then the point is not added.
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let mut bdd_box = BoundingBox::new();
    /// bdd_box.push(&Vector2::new(-1.0,  1.0));
    /// bdd_box.push(&Vector2::new(1.0,  -1.0));
    /// bdd_box.push(&Vector2::new(std::f64::NAN, 1.0));
    /// bdd_box.push(&Vector2::new(-1.0, std::f64::NAN));
    /// assert_eq!(bdd_box.min(), &Vector2::new(-1.0,  -1.0));
    /// assert_eq!(bdd_box.max(), &Vector2::new(1.0,  1.0));
    /// ```
    #[inline(always)]
    pub fn push(&mut self, point: &V) {
        self.0 = self.0.min(point);
        self.1 = self.1.max(point);
    }

    /// Returns the bounding box is empty or not.
    /// # Examples
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let mut bdd_box = BoundingBox::new();
    /// assert!(bdd_box.is_empty());
    /// bdd_box.push(&Vector2::new(-1.0,  1.0));
    /// assert!(!bdd_box.is_empty());
    /// ```
    #[inline(always)]
    pub fn is_empty(&self) -> bool { self.0[0] > self.1[0] }
    /// Returns the reference to the maximum point.
    /// # Examples
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let mut bdd_box = BoundingBox::new();
    /// bdd_box.push(&Vector2::new(-1.0,  1.0));
    /// bdd_box.push(&Vector2::new(1.0,  -1.0));
    /// assert_eq!(bdd_box.max(), &Vector2::new(1.0,  1.0));
    /// ```
    /// # Remarks
    /// If the bounding box is empty, returned vector consists `NEG_INFINITY` components.
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let bdd_box = BoundingBox::<Vector2>::new();
    /// assert_eq!(bdd_box.max(), &Vector2::from([f64::NEG_INFINITY; 2]));
    /// ```
    #[inline(always)]
    pub fn max(&self) -> &V { &self.1 }
    /// Returns the reference to the minimal point.
    /// # Examples
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let mut bdd_box = BoundingBox::new();
    /// bdd_box.push(&Vector2::new(-1.0,  1.0));
    /// bdd_box.push(&Vector2::new(1.0,  -1.0));
    /// assert_eq!(bdd_box.min(), &Vector2::new(-1.0,  -1.0));
    /// ```
    /// # Remarks
    /// If the bounding box is empty, returned vector consists `INFINITY` components.
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let bdd_box = BoundingBox::<Vector2>::new();
    /// assert_eq!(bdd_box.min(), &Vector2::from([f64::INFINITY; 2]));
    /// ```
    #[inline(always)]
    pub fn min(&self) -> &V { &self.0 }
    /// Returns the diagonal vector.
    /// # Examples
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let mut bdd_box = BoundingBox::new();
    /// bdd_box.push(&Vector2::new(-2.0,  -3.0));
    /// bdd_box.push(&Vector2::new(6.0,  4.0));
    /// assert_eq!(bdd_box.diagonal(), Vector2::new(8.0,  7.0));
    /// ```
    /// # Remarks
    /// If the bounding box is empty, returned vector consists `f64::NEG_INFINITY` components.
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let bdd_box = BoundingBox::<Vector2>::new();
    /// assert_eq!(bdd_box.diagonal(), Vector2::new(f64::NEG_INFINITY, f64::NEG_INFINITY));
    /// ```
    #[inline(always)]
    pub fn diagonal(&self) -> V::Vector { self.1.diagonal(self.0) }

    /// Returns the diameter of the bounding box.
    /// # Examples
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let mut bdd_box = BoundingBox::new();
    /// bdd_box.push(&Vector2::new(-1.0,  -3.0));
    /// bdd_box.push(&Vector2::new(2.0,  1.0));
    /// assert_eq!(bdd_box.diameter(), 5.0);
    /// ```
    /// # Remarks
    /// If the bounding box is empty, returnes `f64::NEG_INFINITY`.
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let bdd_box = BoundingBox::<Vector3>::new();
    /// assert_eq!(bdd_box.diameter(), f64::NEG_INFINITY);
    /// ```
    #[inline(always)]
    pub fn diameter(&self) -> F {
        if self.is_empty() {
            F::neg_infinity()
        } else {
            self.0.distance(self.1)
        }
    }

    /// Returns the maximum length of the edges of the bounding box.
    /// # Examples
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let mut bdd_box = BoundingBox::new();
    /// bdd_box.push(&Vector3::new(-1.0, -3.0,  2.0));
    /// bdd_box.push(&Vector3::new(2.0, 1.0,  10.0));
    /// assert_eq!(bdd_box.size(), 8.0);
    /// ```
    /// # Remarks
    /// If the bounding box is empty, returnes `f64::NEG_INFINITY`.
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let bdd_box = BoundingBox::<Vector3>::new();
    /// assert_eq!(bdd_box.size(), f64::NEG_INFINITY);
    /// ```
    #[inline(always)]
    pub fn size(&self) -> F { V::max_component(self.diagonal()) }

    /// Returns the center of the bounding box.
    /// # Examples
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let mut bdd_box = BoundingBox::new();
    /// bdd_box.push(&Vector2::new(-1.0,  -3.0));
    /// bdd_box.push(&Vector2::new(5.0,  1.0));
    /// assert_eq!(bdd_box.center(), Vector2::new(2.0,  -1.0));
    /// ```
    /// # Remarks
    /// If the bounding box is empty, returned vector consists `std::f64::NAN` components.
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let bdd_box = BoundingBox::<Vector3>::new();
    /// let center = bdd_box.center();
    /// assert!(center[0].is_nan());
    /// assert!(center[1].is_nan());
    /// assert!(center[2].is_nan());
    /// ```
    #[inline(always)]
    pub fn center(&self) -> V { self.0.mid(self.1) }
}

impl<'a, F, V> FromIterator<&'a V> for BoundingBox<V>
where
    F: BaseFloat,
    V: MetricSpace<Metric = F> + Copy + Index<usize, Output = F> + Bounded<F>,
{
    fn from_iter<I: IntoIterator<Item = &'a V>>(iter: I) -> BoundingBox<V> {
        let mut bdd_box = BoundingBox::new();
        let bdd_box_mut = &mut bdd_box;
        iter.into_iter().for_each(move |pt| bdd_box_mut.push(pt));
        bdd_box
    }
}

impl<F, V> FromIterator<V> for BoundingBox<V>
where
    F: BaseFloat,
    V: MetricSpace<Metric = F> + Copy + Index<usize, Output = F> + Bounded<F>,
{
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> BoundingBox<V> {
        let mut bdd_box = BoundingBox::new();
        let bdd_box_mut = &mut bdd_box;
        iter.into_iter().for_each(move |pt| bdd_box_mut.push(&pt));
        bdd_box
    }
}

impl<F, V> std::ops::AddAssign<&BoundingBox<V>> for BoundingBox<V>
where
    F: BaseFloat,
    V: MetricSpace<Metric = F> + Copy + Index<usize, Output = F> + Bounded<F>,
{
    /// Puts the points in `other` into `self`.
    /// # Examples
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let mut bdd_box = BoundingBox::from_iter(&[
    ///     Vector2::new(3.0, 2.0), Vector2::new(5.0,  6.0),
    /// ]);
    /// bdd_box += &BoundingBox::from_iter(&[
    ///     Vector2::new(4.0, 1.0), Vector2::new(7.0,  4.0),
    /// ]);
    /// assert_eq!(bdd_box.min(), &Vector2::new(3.0,  1.0));
    /// assert_eq!(bdd_box.max(), &Vector2::new(7.0,  6.0));
    ///
    /// bdd_box += &BoundingBox::new();
    /// assert_eq!(bdd_box.min(), &Vector2::new(3.0,  1.0));
    /// assert_eq!(bdd_box.max(), &Vector2::new(7.0,  6.0));
    /// ```
    #[inline(always)]
    fn add_assign(&mut self, other: &BoundingBox<V>) {
        self.0 = self.0.min(&other.0);
        self.1 = self.1.max(&other.1);
    }
}

impl<F, V> std::ops::AddAssign<BoundingBox<V>> for BoundingBox<V>
where
    F: BaseFloat,
    V: MetricSpace<Metric = F> + Copy + Index<usize, Output = F> + Bounded<F>,
{
    /// Puts the points in `other` into `self`.
    /// # Examples
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let mut bdd_box = BoundingBox::from_iter(&[
    ///     Vector2::new(3.0, 2.0), Vector2::new(5.0,  6.0),
    /// ]);
    /// bdd_box += BoundingBox::from_iter(&[
    ///     Vector2::new(4.0, 1.0), Vector2::new(7.0,  4.0),
    /// ]);
    /// assert_eq!(bdd_box.min(), &Vector2::new(3.0,  1.0));
    /// assert_eq!(bdd_box.max(), &Vector2::new(7.0,  6.0));
    ///
    /// bdd_box += BoundingBox::new();
    /// assert_eq!(bdd_box.min(), &Vector2::new(3.0,  1.0));
    /// assert_eq!(bdd_box.max(), &Vector2::new(7.0,  6.0));
    /// ```
    #[inline(always)]
    fn add_assign(&mut self, other: BoundingBox<V>) { *self += &other; }
}

impl<F, V> std::ops::Add<&BoundingBox<V>> for &BoundingBox<V>
where
    F: BaseFloat,
    V: MetricSpace<Metric = F> + Copy + Index<usize, Output = F> + Bounded<F>,
{
    type Output = BoundingBox<V>;
    /// Returns the direct sum of `self` and other.
    /// # Examples
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let bdd_box0 = BoundingBox::from_iter(&[
    ///     Vector2::new(3.0, 2.0), Vector2::new(5.0, 6.0),
    /// ]);
    /// let bdd_box1 = BoundingBox::from_iter(&[
    ///     Vector2::new(4.0, 1.0), Vector2::new(7.0, 4.0),
    /// ]);
    /// let bdd_box = &bdd_box0 + &bdd_box1;
    /// assert_eq!(bdd_box.min(), &Vector2::new(3.0, 1.0));
    /// assert_eq!(bdd_box.max(), &Vector2::new(7.0, 6.0));
    ///
    /// let cloned_bdd_box = &bdd_box + &BoundingBox::new();
    /// assert_eq!(cloned_bdd_box.min(), &Vector2::new(3.0, 1.0));
    /// assert_eq!(cloned_bdd_box.max(), &Vector2::new(7.0, 6.0));
    /// ```
    #[inline(always)]
    fn add(self, other: &BoundingBox<V>) -> BoundingBox<V> { self.clone() + other }
}

impl<F, V> std::ops::Add<&BoundingBox<V>> for BoundingBox<V>
where
    F: BaseFloat,
    V: MetricSpace<Metric = F> + Copy + Index<usize, Output = F> + Bounded<F>,
{
    type Output = BoundingBox<V>;
    /// Returns the direct sum of `self` and other.
    /// # Examples
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let bdd_box0 = BoundingBox::from_iter(&[
    ///     Vector2::new(3.0, 2.0), Vector2::new(5.0,  6.0),
    /// ]);
    /// let bdd_box1 = BoundingBox::from_iter(&[
    ///     Vector2::new(4.0, 1.0), Vector2::new(7.0,  4.0),
    /// ]);
    /// let bdd_box = bdd_box0 + &bdd_box1;
    /// assert_eq!(bdd_box.min(), &Vector2::new(3.0,  1.0));
    /// assert_eq!(bdd_box.max(), &Vector2::new(7.0,  6.0));
    ///
    /// let cloned_bdd_box = bdd_box + &BoundingBox::new();
    /// assert_eq!(cloned_bdd_box.min(), &Vector2::new(3.0,  1.0));
    /// assert_eq!(cloned_bdd_box.max(), &Vector2::new(7.0,  6.0));
    /// ```
    #[inline(always)]
    fn add(mut self, other: &BoundingBox<V>) -> BoundingBox<V> {
        self += other;
        self
    }
}

impl<F, V> std::ops::Add<BoundingBox<V>> for &BoundingBox<V>
where
    F: BaseFloat,
    V: MetricSpace<Metric = F> + Copy + Index<usize, Output = F> + Bounded<F>,
{
    type Output = BoundingBox<V>;
    /// Returns the direct sum of `self` and other.
    /// # Examples
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let bdd_box0 = BoundingBox::from_iter(&[
    ///     Vector2::new(3.0, 2.0), Vector2::new(5.0,  6.0),
    /// ]);
    /// let bdd_box1 = BoundingBox::from_iter(&[
    ///     Vector2::new(4.0, 1.0), Vector2::new(7.0,  4.0),
    /// ]);
    /// let bdd_box = &bdd_box0 + bdd_box1;
    /// assert_eq!(bdd_box.min(), &Vector2::new(3.0,  1.0));
    /// assert_eq!(bdd_box.max(), &Vector2::new(7.0,  6.0));
    ///
    /// let cloned_bdd_box = &bdd_box + BoundingBox::new();
    /// assert_eq!(cloned_bdd_box.min(), &Vector2::new(3.0,  1.0));
    /// assert_eq!(cloned_bdd_box.max(), &Vector2::new(7.0,  6.0));
    /// ```
    #[inline(always)]
    fn add(self, other: BoundingBox<V>) -> BoundingBox<V> { other + self }
}

impl<F, V> std::ops::Add<BoundingBox<V>> for BoundingBox<V>
where
    F: BaseFloat,
    V: MetricSpace<Metric = F> + Copy + Index<usize, Output = F> + Bounded<F>,
{
    type Output = BoundingBox<V>;
    /// Returns the direct sum of `self` and other.
    /// # Examples
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let bdd_box0 = BoundingBox::from_iter(&[
    ///     Vector2::new(3.0, 2.0), Vector2::new(5.0,  6.0),
    /// ]);
    /// let bdd_box1 = BoundingBox::from_iter(&[
    ///     Vector2::new(4.0, 1.0), Vector2::new(7.0,  4.0),
    /// ]);
    /// let bdd_box = bdd_box0 + bdd_box1;
    /// assert_eq!(bdd_box.min(), &Vector2::new(3.0,  1.0));
    /// assert_eq!(bdd_box.max(), &Vector2::new(7.0,  6.0));
    ///
    /// let cloned_bdd_box = bdd_box + BoundingBox::new();
    /// assert_eq!(cloned_bdd_box.min(), &Vector2::new(3.0,  1.0));
    /// assert_eq!(cloned_bdd_box.max(), &Vector2::new(7.0,  6.0));
    /// ```
    #[inline(always)]
    fn add(self, other: BoundingBox<V>) -> BoundingBox<V> { self + &other }
}

impl<F, V> std::ops::BitXorAssign<&BoundingBox<V>> for BoundingBox<V>
where
    F: BaseFloat,
    V: MetricSpace<Metric = F> + Copy + Index<usize, Output = F> + Bounded<F>,
{
    /// Assigns the intersection of `self` and `other` to `self`.
    /// # Examples
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let mut bdd_box = BoundingBox::from_iter(&[
    ///     Vector2::new(3.0, 2.0), Vector2::new(5.0,  6.0),
    /// ]);
    /// bdd_box ^= &BoundingBox::from_iter(&[
    ///     Vector2::new(4.0, 1.0), Vector2::new(7.0,  4.0),
    /// ]);
    /// assert_eq!(bdd_box.min(), &Vector2::new(4.0,  2.0));
    /// assert_eq!(bdd_box.max(), &Vector2::new(5.0,  4.0));
    ///
    /// bdd_box ^= &BoundingBox::new();
    /// assert!(bdd_box.is_empty());
    /// ```
    #[inline(always)]
    fn bitxor_assign(&mut self, other: &BoundingBox<V>) {
        self.0 = self.0.max(&other.0);
        self.1 = self.1.min(&other.1);
    }
}

impl<F, V> std::ops::BitXorAssign<BoundingBox<V>> for BoundingBox<V>
where
    F: BaseFloat,
    V: MetricSpace<Metric = F> + Copy + Index<usize, Output = F> + Bounded<F>,
{
    /// Assigns the intersection of `self` and `other` to `self`.
    /// # Examples
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let mut bdd_box = BoundingBox::from_iter(&[
    ///     Vector2::new(3.0, 2.0), Vector2::new(5.0,  6.0),
    /// ]);
    /// bdd_box ^= BoundingBox::from_iter(&[
    ///     Vector2::new(4.0, 1.0), Vector2::new(7.0,  4.0),
    /// ]);
    /// assert_eq!(bdd_box.min(), &Vector2::new(4.0,  2.0));
    /// assert_eq!(bdd_box.max(), &Vector2::new(5.0,  4.0));
    ///
    /// bdd_box ^= BoundingBox::new();
    /// assert!(bdd_box.is_empty());
    /// ```
    #[inline(always)]
    fn bitxor_assign(&mut self, other: BoundingBox<V>) { *self ^= &other; }
}

impl<F, V> std::ops::BitXor<&BoundingBox<V>> for &BoundingBox<V>
where
    F: BaseFloat,
    V: MetricSpace<Metric = F> + Copy + Index<usize, Output = F> + Bounded<F>,
{
    type Output = BoundingBox<V>;
    /// Returns the intersection of `self` and `other`.
    /// # Examples
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let bdd_box0 = BoundingBox::from_iter(&[
    ///     Vector2::new(3.0, 2.0), Vector2::new(5.0,  6.0),
    /// ]);
    /// let bdd_box1 = BoundingBox::from_iter(&[
    ///     Vector2::new(4.0, 1.0), Vector2::new(7.0,  4.0),
    /// ]);
    /// let bdd_box = &bdd_box0 ^ &bdd_box1;
    /// assert_eq!(bdd_box.min(), &Vector2::new(4.0,  2.0));
    /// assert_eq!(bdd_box.max(), &Vector2::new(5.0,  4.0));
    ///
    /// let new_empty = &bdd_box ^ &BoundingBox::new();
    /// assert!(new_empty.is_empty());
    /// ```
    #[inline(always)]
    fn bitxor(self, other: &BoundingBox<V>) -> BoundingBox<V> { self.clone() ^ other }
}

impl<F, V> std::ops::BitXor<&BoundingBox<V>> for BoundingBox<V>
where
    F: BaseFloat,
    V: MetricSpace<Metric = F> + Copy + Index<usize, Output = F> + Bounded<F>,
{
    type Output = BoundingBox<V>;
    /// Returns the intersection of `self` and `other`.
    /// # Examples
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let bdd_box0 = BoundingBox::from_iter(&[
    ///     Vector2::new(3.0,  2.0), Vector2::new(5.0,  6.0),
    /// ]);
    /// let bdd_box1 = BoundingBox::from_iter(&[
    ///     Vector2::new(4.0,  1.0), Vector2::new(7.0,  4.0),
    /// ]);
    /// let bdd_box = bdd_box0 ^ &bdd_box1;
    /// assert_eq!(bdd_box.min(), &Vector2::new(4.0,  2.0));
    /// assert_eq!(bdd_box.max(), &Vector2::new(5.0,  4.0));
    ///
    /// let new_empty = bdd_box ^ &BoundingBox::new();
    /// assert!(new_empty.is_empty());
    /// ```
    #[inline(always)]
    fn bitxor(mut self, other: &BoundingBox<V>) -> BoundingBox<V> {
        self ^= other;
        self
    }
}

impl<F, V> std::ops::BitXor<BoundingBox<V>> for &BoundingBox<V>
where
    F: BaseFloat,
    V: MetricSpace<Metric = F> + Copy + Index<usize, Output = F> + Bounded<F>,
{
    type Output = BoundingBox<V>;
    /// Returns the intersection of `self` and `other`.
    /// # Examples
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let bdd_box0 = BoundingBox::from_iter(&[
    ///     Vector2::new(3.0,  2.0), Vector2::new(5.0,  6.0),
    /// ]);
    /// let bdd_box1 = BoundingBox::from_iter(&[
    ///     Vector2::new(4.0,  1.0), Vector2::new(7.0,  4.0),
    /// ]);
    /// let bdd_box = &bdd_box0 ^ bdd_box1;
    /// assert_eq!(bdd_box.min(), &Vector2::new(4.0,  2.0));
    /// assert_eq!(bdd_box.max(), &Vector2::new(5.0,  4.0));
    ///
    /// let new_empty = &bdd_box ^ BoundingBox::new();
    /// assert!(new_empty.is_empty());
    /// ```
    #[inline(always)]
    fn bitxor(self, other: BoundingBox<V>) -> BoundingBox<V> { other ^ self }
}

impl<F, V> std::ops::BitXor<BoundingBox<V>> for BoundingBox<V>
where
    F: BaseFloat,
    V: MetricSpace<Metric = F> + Copy + Index<usize, Output = F> + Bounded<F>,
{
    type Output = BoundingBox<V>;
    /// Returns the intersection of `self` and `other`.
    /// # Examples
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*, tolerance::*};
    /// let bdd_box0 = BoundingBox::from_iter(&[
    ///     Vector2::new(3.0,  2.0), Vector2::new(5.0,  6.0),
    /// ]);
    /// let bdd_box1 = BoundingBox::from_iter(&[
    ///     Vector2::new(4.0,  1.0), Vector2::new(7.0,  4.0),
    /// ]);
    /// let bdd_box = bdd_box0 ^ bdd_box1;
    /// assert_eq!(bdd_box.min(), &Vector2::new(4.0,  2.0));
    /// assert_eq!(bdd_box.max(), &Vector2::new(5.0,  4.0));
    ///
    /// let new_empty = bdd_box ^ BoundingBox::new();
    /// assert!(new_empty.is_empty());
    /// ```
    #[inline(always)]
    fn bitxor(self, other: BoundingBox<V>) -> BoundingBox<V> { self ^ &other }
}

impl<F, V> PartialOrd for BoundingBox<V>
where
    F: BaseFloat,
    V: MetricSpace<Metric = F> + Copy + Index<usize, Output = F> + Bounded<F> + PartialEq,
{
    /// Inclusion relationship
    /// # Examples
    /// ```
    /// use truck_base::{cgmath64::*, bounding_box::*};
    /// let bbx0 = BoundingBox::from_iter(&[
    ///     Point2::new(0.0, 0.0),
    ///     Point2::new(1.0, 1.0),
    /// ]);
    /// let bbx1 = BoundingBox::from_iter(&[
    ///     Point2::new(0.25, 0.25),
    ///     Point2::new(0.75, 0.75),
    /// ]);
    /// // bbx0 includes bbx1.
    /// assert!(bbx0 > bbx1);
    ///
    /// let bbx2 = BoundingBox::from_iter(&[
    ///     Point2::new(-1.0, -1.0),
    ///     Point2::new(0.75, 0.75),
    /// ]);
    /// // bbx0 does not include bbx2, and bbx2 does not include bbx0.
    /// assert!(!(bbx0 > bbx2));
    /// assert!(!(bbx0 < bbx2));
    /// assert!(!(bbx0 == bbx2));
    /// ```
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let max = self + other;
        match (self == &max, other == &max) {
            (true, true) => Some(Ordering::Equal),
            (true, false) => Some(Ordering::Greater),
            (false, true) => Some(Ordering::Less),
            (false, false) => None,
        }
    }
}
