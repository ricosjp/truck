use crate::{Tolerance, Origin};

impl Tolerance for f64 {
    #[inline(always)]
    fn near(&self, other: &f64) -> bool {
        if self - other > Self::TOLERANCE {
            false
        } else if other - self > Self::TOLERANCE {
            false
        } else {
            true
        }
    }

    #[inline(always)]
    fn near2(&self, other: &f64) -> bool {
        if self - other > Self::TOLERANCE2 {
            false
        } else if other - self > Self::TOLERANCE2 {
            false
        } else {
            true
        }
    }
}

impl Tolerance for &[f64] {
    fn near(&self, other: &&[f64]) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for (a, b) in self.iter().zip(other.iter()) {
             if !a.near(b) {
                 return false;
             }
        }
        true
    }
    
    fn near2(&self, other: &&[f64]) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for (a, b) in self.iter().zip(other.iter()) {
             if !a.near2(b) {
                 return false;
             }
        }
        true
    }
}

impl Origin for f64 {
    const ORIGIN : f64 = 0.0;
}

#[inline(always)]
pub fn inv_or_zero(delta : f64) -> f64 {
    if delta.so_small() {
        0.0
    } else {
        1.0 / delta
    }
}

/// round the value by tolerance
/// # Example
/// ```
/// # use truck_geometry::tolerance::round_by_tolerance;
/// assert_eq!(round_by_tolerance(1.23456789), 1.2345678);
/// ```
pub fn round_by_tolerance(val: f64) -> f64 {
    (val / f64::TOLERANCE).floor() * f64::TOLERANCE
}
