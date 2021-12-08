use truck_base::cgmath64::*;

pub mod geometry;
mod impl_curve;
mod parse_primitives;

mod tentative_ {
    use crate::*;
    #[path = "../tentative.rs"]
    pub mod tentative;
    parse_primitives!(tentative);
    impl_curve!(tentative);
}
