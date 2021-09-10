use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};

/// Trait for implement `get_or_insert`
pub trait GetOrInsert<K, V> {
	/// Try to insert the key and value if there is not the value corresponding to the key.
	///
	/// # Return
	/// - `Ok(true)`: There is not the value corresponding to the key and a new value is inserted.
	/// - `Ok(false)`: There is already the value corresponding to the key.
	/// - `Err(_)`: There is not the value corresponding to the key and failed to create new value.
	///
	/// # Examples
	/// ```
	/// use std::collections::HashMap;
	/// use truck_base::maputil::GetOrInsert;
	///
	/// let mut counter = 0;
	/// let mut try_get_index = move || -> Option<usize> {
	/// 	if counter % 2 == 0 {
	/// 		counter += 1;
	/// 		Some(counter / 2)
	/// 	} else {
	/// 		counter += 1;
	/// 		None
	/// 	}
	/// };
	///
	/// let mut map = HashMap::new();
	/// assert_eq!(map.try_insert_if_none(0, &mut try_get_index), Some(true));
	/// assert_eq!(map.try_insert_if_none(1, &mut try_get_index), None);
	/// assert_eq!(map.try_insert_if_none(2, &mut try_get_index), Some(true));
	/// assert_eq!(map.try_insert_if_none(3, &mut try_get_index), None);
	/// assert_eq!(map.try_insert_if_none(2, &mut try_get_index), Some(false));
	/// assert_eq!(map.get(&2), Some(&1));
	/// assert_eq!(map.get(&3), None);
	/// ```
	fn try_insert_if_none<F: FnMut() -> Option<V>>(&mut self, key: K, f: F) -> Option<bool>;
	/// Insert the key and value if there is not the value corresponding to the key.
	///
	/// # Return
	/// Returns `true` iff there is not the value correspoinding to the key.
	///
	/// # Examples
	/// ```
	/// use std::collections::HashMap;
	/// use truck_base::maputil::GetOrInsert;
	///
	/// let mut map = HashMap::new();
	/// assert!(map.insert_if_none(0, || 1));
	/// assert!(map.insert_if_none(1, || 2));
	/// assert!(map.insert_if_none(2, || 3));
	/// assert!(!map.insert_if_none(1, || 4));
	/// ```
	fn insert_if_none<F: FnMut() -> V>(&mut self, key: K, f: F) -> bool;
	/// Get the value corresponding to `key` or try to insert `key` value
	///
	/// # Return
	/// - `Ok(val)`: `val` is the value corresponding to `key`. Perhaps, this is the new inseted value.
	/// - `Err(_)`: There is not the value corresponding to the key and failed to create new value.
	///
	/// # Examples
	/// ```
	/// use std::collections::HashMap;
	/// use truck_base::maputil::GetOrInsert;
	///
	/// let mut counter = 0;
	/// let mut try_get_index = move || -> Option<usize> {
	/// 	if counter % 2 == 0 {
	/// 		counter += 1;
	/// 		Some(counter / 2)
	/// 	} else {
	/// 		counter += 1;
	/// 		None
	/// 	}
	/// };
	///
	/// let mut map = HashMap::new();
	/// assert_eq!(map.try_get_or_insert(0, &mut try_get_index), Some(&0));
	/// assert_eq!(map.try_get_or_insert(1, &mut try_get_index), None);
	/// assert_eq!(map.try_get_or_insert(2, &mut try_get_index), Some(&1));
	/// assert_eq!(map.try_get_or_insert(3, &mut try_get_index), None);
	/// assert_eq!(map.try_get_or_insert(2, &mut try_get_index), Some(&1));
	/// assert_eq!(map.get(&3), None);
	/// ```
	fn try_get_or_insert<F: FnMut() -> Option<V>>(&mut self, key: K, f: F) -> Option<&V>;
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
	fn get_or_insert<F: FnMut() -> V>(&mut self, key: K, f: F) -> &V;
}

impl<K, V, S> GetOrInsert<K, V> for HashMap<K, V, S>
where
	K: Copy + Eq + Hash,
	S: BuildHasher,
{
	#[inline(always)]
	fn try_insert_if_none<F: FnMut() -> Option<V>>(&mut self, key: K, mut f: F) -> Option<bool> {
		let flag = self.get(&key).is_none();
		if flag {
			self.insert(key, f()?);
		}
		Some(flag)
	}
	#[inline(always)]
	fn insert_if_none<F: FnMut() -> V>(&mut self, key: K, mut f: F) -> bool {
		let flag = self.get(&key).is_none();
		if flag {
			self.insert(key, f());
		}
		flag
	}
	fn try_get_or_insert<F: FnMut() -> Option<V>>(&mut self, key: K, f: F) -> Option<&V> {
		self.try_insert_if_none(key, f)?;
		Some(self.get(&key).unwrap())
	}
	fn get_or_insert<F: FnMut() -> V>(&mut self, key: K, f: F) -> &V {
		self.insert_if_none(key, f);
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

impl<K> IntoIterator for IDMap<K> {
	type Item = (K, usize);
	type IntoIter = <HashMap<K, usize> as IntoIterator>::IntoIter;
	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}
