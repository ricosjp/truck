use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};

/// Trait for implement `get_or_insert`
pub trait GetOrInsert<K, V> {
	/// Get the value corresponding to `key` or insert `key` value
	///
	/// # Examples
	/// ```
	/// use std::collections::HashMap;
	/// use truck_base::maputil::GetOrInsert;
	/// 
	/// let mut map = HashMap::new();
	/// assert_eq!(map.get_or_insert(0, || 1), &1);
	/// assert_eq!(map.get_or_insert(1, || 2), &2);
	/// assert_eq!(map.get_or_insert(2, || 3), &3);
	/// assert_eq!(map.get_or_insert(1, || 4), &2);
	/// ```
	fn get_or_insert<F: Fn() -> V>(&mut self, key: K, f: F) -> &V;
}

impl<K, V, S> GetOrInsert<K, V> for HashMap<K, V, S>
where
	K: Copy + Eq + Hash,
	S: BuildHasher,
{
	fn get_or_insert<F: Fn() -> V>(&mut self, key: K, f: F) -> &V {
		if self.get(&key).is_none() {
			self.insert(key, f());
		}
		self.get(&key).unwrap()
	}
}

/// Records the registered number.
/// 
/// It can be used as a reverse lookup of an index when a separate array containing entities is available.
#[derive(Clone, Debug)]
pub struct IDMap<K>(HashMap<K, usize>);

impl<K: Copy + Eq + Hash> IDMap<K> {
	/// Creates a new `IDMap`.
	#[inline(always)]
	pub fn new() -> Self { Self(HashMap::new()) }
	/// Returns the registered number or insert key.
	///
	/// # Examples
	/// ```
	/// use truck_base::maputil::IDMap;
	/// let mut map = IDMap::new();
	/// 
	/// // new member: the first member so number = 0
	/// assert_eq!(map.get_number_or_insert("ohayo"), 0);
	/// // new member: the second member so number = 1
	/// assert_eq!(map.get_number_or_insert("arigato"), 1);
	/// // new member: the third member so number = 2
	/// assert_eq!(map.get_number_or_insert("oyasumi"), 2);
	/// // "arigato" is already inserted: the member of number = 1
	/// assert_eq!(map.get_number_or_insert("arigato"), 1);
	/// ```
	#[inline(always)]
	pub fn get_number_or_insert(&mut self, key: K) -> usize {
		let len = self.0.len();
		*self.0.get_or_insert(key, || len)
	}
}
