use truck_base::cgmath64::*;

mod tentative;

pub mod geometry;
mod impl_curve;
mod parse_primitives;

mod tentative_ {
    use crate::*;
    parse_primitives!(tentative);
    impl_curve!(tentative);
}
