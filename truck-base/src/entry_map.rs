use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::hash::*;
use std::marker::PhantomData;

/// A utility structure for chaining entry and or_insert_with by a fixed closure.
///
/// # Example
/// ```
/// type EntryMap<K, V, KF, VF, P> = truck_base::entry_map::EntryMap<K, V, KF, VF, P>;
///
/// let mut id_generator = 0;
/// let mut map = EntryMap::new(
///     |x: f64| x.floor() as i32,
///     |_| {
///         id_generator += 1;
///         id_generator
///     },
/// );
///
/// assert_eq!(*map.entry_or_insert(3.5), 1);
/// assert_eq!(*map.entry_or_insert(4.2), 2);
/// assert_eq!(*map.entry_or_insert(3.6), 1);
/// ```
#[derive(Clone, Debug)]
pub struct EntryMap<K, V, KF, VF, P, S = RandomState> {
    hashmap: HashMap<K, V, S>,
    k_closure: KF,
    v_closure: VF,
    _phantom: PhantomData<P>,
}

/// type alias for EntryMap with `FxHasher`.
pub type FxEntryMap<K, V, KF, VF, P> =
    EntryMap<K, V, KF, VF, P, BuildHasherDefault<rustc_hash::FxHasher>>;

impl<K, V, KF, VF, P, S> EntryMap<K, V, KF, VF, P, S>
where
    K: Eq + Hash,
    S: BuildHasher + Default,
    P: Copy,
    KF: FnMut(P) -> K,
    VF: FnMut(P) -> V,
{
    /// constructor
    #[inline]
    pub fn new(k_closure: KF, v_closure: VF) -> Self {
        Self {
            hashmap: HashMap::default(),
            k_closure,
            v_closure,
            _phantom: PhantomData,
        }
    }

    /// Run chaining `entry` and `or_insert_with`.
    #[inline]
    pub fn entry_or_insert(&mut self, p: P) -> &mut V {
        self.hashmap
            .entry((self.k_closure)(p))
            .or_insert_with(|| (self.v_closure)(p))
    }
}

impl<K, V, KF, VF, P, S> From<EntryMap<K, V, KF, VF, P, S>> for HashMap<K, V, S> {
    #[inline]
    fn from(x: EntryMap<K, V, KF, VF, P, S>) -> Self { x.hashmap }
}

impl<K, V, KF, VF, P, S> IntoIterator for EntryMap<K, V, KF, VF, P, S> {
    type Item = (K, V);
    type IntoIter = <HashMap<K, V, S> as IntoIterator>::IntoIter;
    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.hashmap.into_iter() }
}
