use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::Debug;
use std::hash::{BuildHasher, Hash};

/// Temporary implementation until `std::ops::Try` will become stable.
pub trait Try {
	#[doc(hidden)]
	type Output;
	#[doc(hidden)]
	type Residual;
	#[doc(hidden)]
	fn from_output(output: Self::Output) -> Self;
	#[doc(hidden)]
	fn from_residual(residual: Self::Residual) -> Self;
	#[doc(hidden)]
	fn is_ok(&self) -> bool;
	#[doc(hidden)]
	fn unwrap(self) -> Self::Output;
	#[doc(hidden)]
	fn into_residual(self) -> Self::Residual;
}

impl<T> Try for Option<T> {
	type Output = T;
	type Residual = Option<Infallible>;
	#[inline(always)]
	fn from_output(output: T) -> Self { Some(output) }
	#[inline(always)]
	fn from_residual(_: Self::Residual) -> Self { None }
	#[inline(always)]
	fn is_ok(&self) -> bool { self.is_some() }
	#[inline(always)]
	fn unwrap(self) -> T { self.unwrap() }
	#[inline(always)]
	fn into_residual(self) -> Option<Infallible> { None }
}

impl<T: Debug, E: Debug> Try for Result<T, E> {
	type Output = T;
	type Residual = Result<Infallible, E>;
	#[inline(always)]
	fn from_output(output: T) -> Self { Ok(output) }
	#[inline(always)]
	fn from_residual(residual: Self::Residual) -> Self { Err(residual.unwrap_err()) }
	#[inline(always)]
	fn is_ok(&self) -> bool { self.is_ok() }
	#[inline(always)]
	fn unwrap(self) -> T { self.unwrap() }
	#[inline(always)]
	fn into_residual(self) -> Result<Infallible, E> { Err(self.unwrap_err()) }
}

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
	/// let mut try_get_index = move || -> Result<usize, ()> {
	/// 	if counter % 2 == 0 {
	/// 		counter += 1;
	/// 		Ok(counter / 2)
	/// 	} else {
	/// 		counter += 1;
	/// 		Err(())
	/// 	}
	/// };
	///
	/// let mut map = HashMap::new();
	/// assert_eq!(map.try_insert_if_none(0, &mut try_get_index), Ok(true));
	/// assert_eq!(map.try_insert_if_none(1, &mut try_get_index), Err(()));
	/// assert_eq!(map.try_insert_if_none(2, &mut try_get_index), Ok(true));
	/// assert_eq!(map.try_insert_if_none(3, &mut try_get_index), Err(()));
	/// assert_eq!(map.try_insert_if_none(2, &mut try_get_index), Ok(false));
	/// assert_eq!(map.get(&2), Some(&1));
	/// assert_eq!(map.get(&3), None);
	/// ```
	fn try_insert_if_none<E, F: FnMut() -> Result<V, E>>(
		&mut self,
		key: K,
		f: F,
	) -> Result<bool, E>;
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
	/// let mut try_get_index = move || -> Result<usize, ()> {
	/// 	if counter % 2 == 0 {
	/// 		counter += 1;
	/// 		Ok(counter / 2)
	/// 	} else {
	/// 		counter += 1;
	/// 		Err(())
	/// 	}
	/// };
	///
	/// let mut map = HashMap::new();
	/// assert_eq!(map.try_get_or_insert(0, &mut try_get_index), Ok(&0));
	/// assert_eq!(map.try_get_or_insert(1, &mut try_get_index), Err(()));
	/// assert_eq!(map.try_get_or_insert(2, &mut try_get_index), Ok(&1));
	/// assert_eq!(map.try_get_or_insert(3, &mut try_get_index), Err(()));
	/// assert_eq!(map.try_get_or_insert(2, &mut try_get_index), Ok(&1));
	/// assert_eq!(map.get(&3), None);
	/// ```
	fn try_get_or_insert<E, F: FnMut() -> Result<V, E>>(&mut self, key: K, f: F) -> Result<&V, E>;
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
	fn try_insert_if_none<E, F: FnMut() -> Result<V, E>>(
		&mut self,
		key: K,
		mut f: F,
	) -> Result<bool, E> {
		let flag = self.get(&key).is_none();
		if flag {
			self.insert(key, f()?);
		}
		Ok(flag)
	}
	#[inline(always)]
	fn insert_if_none<F: FnMut() -> V>(&mut self, key: K, mut f: F) -> bool {
		let flag = self.get(&key).is_none();
		if flag {
			self.insert(key, f());
		}
		flag
	}
	fn try_get_or_insert<E, F: FnMut() -> Result<V, E>>(&mut self, key: K, f: F) -> Result<&V, E> {
		self.try_insert_if_none(key, f)?;
		Ok(self.get(&key).unwrap())
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
