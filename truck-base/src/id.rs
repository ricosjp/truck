use std::fmt::Debug;
use std::hash::{Hash, Hasher};

/// ID structure with `Copy`, `Hash` and `Eq` using raw pointers
pub struct ID<T>(*const T);

impl<T> ID<T> {
    /// Creates the ID by a raw pointer.
    #[inline(always)]
    pub fn new(ptr: *const T) -> ID<T> { ID(ptr) }
}

impl<T> Clone for ID<T> {
    #[inline(always)]
    fn clone(&self) -> ID<T> { ID(self.0) }
}

impl<T> Copy for ID<T> {}

impl<T> Hash for ID<T> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) { std::ptr::hash(self.0, state); }
}

impl<T> PartialEq for ID<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool { std::ptr::eq(self.0, other.0) }
}

impl<T> Eq for ID<T> {}

impl<T> Debug for ID<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_fmt(format_args!("{:p}", self.0))
    }
}
