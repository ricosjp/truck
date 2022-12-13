use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// ID structure with `Copy`, `Hash` and `Eq` using raw pointers
pub struct ID<T>(usize, PhantomData<T>);

impl<T> ID<T> {
    /// Creates the ID by a raw pointer.
    #[inline(always)]
    pub fn new(ptr: *const T) -> ID<T> { ID(ptr as usize, PhantomData) }
}

impl<T> Clone for ID<T> {
    #[inline(always)]
    fn clone(&self) -> ID<T> { ID(self.0, PhantomData) }
}

impl<T> Copy for ID<T> {}

impl<T> Hash for ID<T> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) { self.0.hash(state) }
}

impl<T> PartialEq for ID<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool { self.0 == other.0 }
}

impl<T> Eq for ID<T> {}

impl<T> Debug for ID<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_fmt(format_args!("0x{:x}", self.0))
    }
}

#[test]
fn debug_backward_compatibility() {
    let x: f64 = 3.0;
    let id = ID::new(&x);
    let a = format!("{:?}", id);
    let b = format!("{:p}", &x);
    assert_eq!(a, b);
}
