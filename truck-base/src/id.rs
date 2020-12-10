use std::hash::{Hash, Hasher};

pub struct ID<T>(*const T);

impl<T> ID<T> {
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

impl<T> std::fmt::Debug for ID<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        f.write_fmt(format_args!("{:p}", self.0))
    }
}
