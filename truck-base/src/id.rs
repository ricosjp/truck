use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// Id structure with `Copy`, `Hash` and `Eq` using raw pointers
pub struct Id<T>(usize, PhantomData<T>);

impl<T> Id<T> {
    /// Creates the Id by a raw pointer.
    #[inline(always)]
    pub fn new(ptr: *const T) -> Id<T> { Id(ptr as usize, PhantomData) }
}

impl<T> Clone for Id<T> {
    #[inline(always)]
    fn clone(&self) -> Id<T> { *self }
}

impl<T> Copy for Id<T> {}

impl<T> Hash for Id<T> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) { self.0.hash(state) }
}

impl<T> PartialEq for Id<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool { self.0 == other.0 }
}

impl<T> Eq for Id<T> {}

/// Renamed to [`Id`] per RFC 430 (C-CASE).
#[deprecated(note = "renamed to Id per RFC 430 (C-CASE)")]
pub type ID<T> = Id<T>;

impl<T> Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_fmt(format_args!("0x{:x}", self.0))
    }
}

#[test]
fn debug_backward_compatibility() {
    let x: f64 = 3.0;
    let id = Id::new(&x);
    let a = format!("{id:?}");
    let b = format!("{:p}", &x);
    assert_eq!(a, b);
}
