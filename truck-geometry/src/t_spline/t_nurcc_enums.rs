use super::*;
use std::fmt;

impl TnurccConnection {
    /// Converts a `usize` to a `TnurccConnection`. Modulo's `i` by `4`, resulting in a lossy
    /// conversion. Thus, the `From` trait has not been used.
    ///
    /// # Panics
    /// Panics if `i % 4 >= 4` (impossible?).
    pub fn from_usize(i: usize) -> TnurccConnection {
        match i % 4 {
            0 => TnurccConnection::LeftCw,
            1 => TnurccConnection::LeftAcw,
            2 => TnurccConnection::RightCw,
            3 => TnurccConnection::RightAcw,
            _ => panic!("Modulo 4 returned result greater than 3"),
        }
    }
}

impl fmt::Display for TnurccConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TnurccConnection::LeftAcw => write!(f, "Anti-clockwise Left"),
            TnurccConnection::RightAcw => write!(f, "Anti-clockwise Right"),
            TnurccConnection::LeftCw => write!(f, "Clockwise Left"),
            TnurccConnection::RightCw => write!(f, "Clockwise Right"),
        }
    }
}
