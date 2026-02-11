use super::*;
use std::fmt;

impl TmeshDirection {
    /// Returns an iterable object which rotates clockwise starting from UP.
    pub fn iter() -> impl Iterator<Item = TmeshDirection> {
        [
            TmeshDirection::Up,
            TmeshDirection::Right,
            TmeshDirection::Down,
            TmeshDirection::Left,
        ]
        .iter()
        .copied()
    }

    /// Rotates `self` a quarter of a rotation clockwise.
    pub fn clockwise(self) -> Self {
        match self {
            TmeshDirection::Up => TmeshDirection::Right,
            TmeshDirection::Right => TmeshDirection::Down,
            TmeshDirection::Down => TmeshDirection::Left,
            TmeshDirection::Left => TmeshDirection::Up,
        }
    }

    /// Rotates `self` a quarter of a turn anti-clockwise.
    pub fn anti_clockwise(self) -> Self {
        match self {
            TmeshDirection::Up => TmeshDirection::Left,
            TmeshDirection::Left => TmeshDirection::Down,
            TmeshDirection::Down => TmeshDirection::Right,
            TmeshDirection::Right => TmeshDirection::Up,
        }
    }

    /// Flips `self`, rotating it half a turn.
    pub fn flip(self) -> Self {
        match self {
            TmeshDirection::Up => TmeshDirection::Down,
            TmeshDirection::Down => TmeshDirection::Up,
            TmeshDirection::Left => TmeshDirection::Right,
            TmeshDirection::Right => TmeshDirection::Left,
        }
    }

    /// Returns true if the direction is UP or DOWN
    pub fn vertical(self) -> bool {
        match self {
            TmeshDirection::Up => true,
            TmeshDirection::Down => true,
            TmeshDirection::Left => false,
            TmeshDirection::Right => false,
        }
    }

    /// Returns true if the direction is RIGHT or LEFT
    pub fn horizontal(self) -> bool {
        match self {
            TmeshDirection::Up => false,
            TmeshDirection::Down => false,
            TmeshDirection::Left => true,
            TmeshDirection::Right => true,
        }
    }

    /// Returns true if a knot interval in the direction is a positive delta.
    pub fn knot_delta_positive(self) -> bool {
        match self {
            TmeshDirection::Up => true,
            TmeshDirection::Down => false,
            TmeshDirection::Left => false,
            TmeshDirection::Right => true,
        }
    }

    /// Returns true if a knot interval in the direction is a negative delta.
    pub fn knot_delta_negative(self) -> bool {
        match self {
            TmeshDirection::Up => false,
            TmeshDirection::Down => true,
            TmeshDirection::Left => true,
            TmeshDirection::Right => false,
        }
    }

    /// Adds `delta` to an existing set of knot `coords`, taking into account the direction `self`
    /// such that `delta` is correctly added or subtracted from one of `coords`'s members in order
    /// to move `coords` in the direction `self` by a distance `delta`.
    pub fn mutate_knot_coordinates(self, coords: (f64, f64), delta: f64) -> (f64, f64) {
        match self {
            TmeshDirection::Up => (coords.0, coords.1 + delta),
            TmeshDirection::Down => (coords.0, coords.1 - delta),
            TmeshDirection::Left => (coords.0 - delta, coords.1),
            TmeshDirection::Right => (coords.0 + delta, coords.1),
        }
    }
}

impl fmt::Display for TmeshDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dir_string = match self {
            TmeshDirection::Up => "up",
            TmeshDirection::Down => "down",
            TmeshDirection::Left => "left",
            TmeshDirection::Right => "right",
        };

        write!(f, "{}", dir_string)
    }
}
