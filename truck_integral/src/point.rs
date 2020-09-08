use crate::*;
use geometry::*;

macro_rules! impl_point_for_integers {
    ($int: ty) => {
        impl Point for $int {
            fn near(&self, other: &Self) -> bool { self == other }
        }
    };
    ($a: ty, $($b: ty), *) => {
        impl_point_for_integers!($a);
        impl_point_for_integers!($($b), *);
    };
}

impl_point_for_integers!(
    usize, u8, u16, u32, u64, u128, isize, i8, i16, i32, i64, i128
);

macro_rules! impl_point_for_tolerance {
    ($point: ty) => {
        impl Point for $point {
            fn near(&self, other: &Self) -> bool { Tolerance::near(self, other) }
        }
    };
    ($a: ty, $($b: ty), *) => {
        impl_point_for_tolerance!($a);
        impl_point_for_tolerance!($($b), *);
    };
}

impl_point_for_tolerance!(
    f64, Point1, Point2, Point3, Vector1, Vector2, Vector3
);

impl Point for Vector4 {
    fn near(&self, other: &Self) -> bool {
        let point0 = Point3::from_homogeneous(*self);
        let point1 = Point3::from_homogeneous(*other);
        Tolerance::near(&point0, &point1)
    }
}

