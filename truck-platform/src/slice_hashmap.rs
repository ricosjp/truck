#![allow(dead_code)]

use rustc_hash::FxHashMap as HashMap;
use std::hash::Hash;

/// Hashmap for optimizing iteration.
#[derive(Clone, Debug)]
pub struct SliceHashMap<K, V> {
	vec: Vec<(K, V)>,
	map: HashMap<K, usize>,
}

impl<K, V> Default for SliceHashMap<K, V> {
	fn default() -> Self {
		SliceHashMap {
			vec: Vec::new(),
			map: HashMap::default(),
		}
	}
}

impl<K: Copy + Eq + Hash, V> SliceHashMap<K, V> {
	pub fn new() -> Self {
		Self::default()
	}
	pub fn get(&self, key: &K) -> Option<&V> {
		self.map.get(key).map(|idx| &self.vec[*idx].1)
	}
	pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
		let idx = *self.map.get(key)?;
		Some(&mut self.vec[idx].1)
	}
	pub fn len(&self) -> usize { self.vec.len() }
	pub fn insert(&mut self, key: K, value: V) -> Option<V> {
		match self.map.get(&key) {
			Some(idx) => {
				self.vec.push((key, value));
				Some(self.vec.swap_remove(*idx).1)
			}
			None => {
				self.map.insert(key, self.vec.len());
				self.vec.push((key, value));
				None
			}
		}
	}
	pub fn remove(&mut self, key: &K) -> Option<V> {
		self.map.remove(key).map(|idx| {
			let output = self.vec.swap_remove(idx);
			if idx < self.vec.len() {
				let key = self.vec[idx].0;
				*self.map.get_mut(&key).unwrap() = idx;
			}
			output.1
		})
	}
	pub fn clear(&mut self) {
		self.vec.clear();
		self.map.clear();
	}
	pub fn as_slice(&self) -> &[(K, V)] {
		&self.vec
	}
}

impl<K, V> IntoIterator for SliceHashMap<K, V> {
	type Item = (K, V);
	type IntoIter = <Vec<(K, V)> as IntoIterator>::IntoIter;
	fn into_iter(self) -> Self::IntoIter {
		self.vec.into_iter()
	}
}

impl<'a, K, V> IntoIterator for &'a SliceHashMap<K, V> {
	type Item = &'a (K, V);
	type IntoIter = <&'a Vec<(K, V)> as IntoIterator>::IntoIter;
	fn into_iter(self) -> Self::IntoIter {
		self.vec.iter()
	}
}

impl<K: Copy + Eq + Hash, V> std::iter::FromIterator<(K, V)> for SliceHashMap<K, V> {
	fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
		let mut res = Self::default();
		iter.into_iter().enumerate().for_each(|(i, (key, val))| {
			res.vec.push((key, val));
			res.map.insert(key, i);
		});
		res
	}
}

#[test]
fn sliced_hashmap() {
	let mut map: SliceHashMap<_, _> = (0..100).map(|i| (i, 100 - i)).collect();
	assert_eq!(map.len(), 100);
	assert_eq!(map.vec.len(), map.map.len());
	for i in 0..100 {
		assert_eq!(map.insert(i, i + 200).unwrap(), 100 - i);
		assert_eq!(map.vec.len(), map.map.len());
	}
	for i in 0..100 {
		assert_eq!(*map.get(&i).unwrap(), i + 200);
		assert_eq!(map.vec.len(), map.map.len());
	}
	assert!(map.insert(100, 300).is_none());
	assert_eq!(map.len(), 101);
	for i in 0..100 {
		assert_eq!(map.remove(&i).unwrap(), i + 200);
		assert_eq!(map.vec.len(), map.map.len());
	}
	assert_eq!(map.len(), 1);
	map.clear();
	assert_eq!(map.len(), 0);
}
