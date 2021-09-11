use crate::*;
pub use vertex::VertexDisplay;
pub use edge::EdgeDisplay;
pub use wire::WireDisplay;

#[derive(Clone)]
pub(super) struct MutexFmt<'a, T>(pub &'a Mutex<T>);

impl<'a, T: Debug> Debug for MutexFmt<'a, T> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		use std::sync::TryLockError;
		match self.0.try_lock() {
			Ok(guard) => f.write_fmt(format_args!("{:?}", &&*guard)),
			Err(TryLockError::Poisoned(err)) => {
				f.write_fmt(format_args!("{:?}", &&**err.get_ref()))
			}
			Err(TryLockError::WouldBlock) => f.pad("<locked>"),
		}
	}
}

impl FormatConfiguation {
	/// displays all elements' id and entity.
	#[inline(always)]
	pub fn all() -> Self {
		Self {
			vertex_id: true,
			vertex_entity: true,
			edge_id: true,
			edge_entity: true,
			face_id: true,
			face_entity: true,
		}
	}
	/// displays no elements' id and entity.
	#[inline(always)]
	pub fn empty() -> Self {
		Self {
			vertex_id: false,
			vertex_entity: false,
			edge_id: false,
			edge_entity: false,
			face_id: false,
			face_entity: false,
		}
	}
	/// displays all elements' id, does not display entity.
	#[inline(always)]
	pub fn id() -> Self {
		Self {
			vertex_id: true,
			vertex_entity: false,
			edge_id: true,
			edge_entity: false,
			face_id: true,
			face_entity: false,
		}
	}
	/// displays all elements' entity, does not display id.
	#[inline(always)]
	pub fn entity() -> Self {
		Self {
			vertex_id: false,
			vertex_entity: true,
			edge_id: false,
			edge_entity: true,
			face_id: false,
			face_entity: true,
		}
	}
	/// displays vertices' entity, edges' id and faces' id.
	#[inline(always)]
	pub fn vertex_entity_others_id() -> Self {
		Self {
			vertex_id: false,
			vertex_entity: true,
			edge_id: true,
			edge_entity: false,
			face_id: true,
			face_entity: false,
		}
	}
}

macro_rules! derive_ops {
	($fnct: tt) => {
		type Output = Self;
		fn $fnct(self, other: Self) -> Self {
			Self {
				vertex_id: self.vertex_id.$fnct(other.vertex_id),
				vertex_entity: self.vertex_entity.$fnct(other.vertex_entity),
				edge_id: self.edge_id.$fnct(other.edge_id),
				edge_entity: self.edge_entity.$fnct(other.edge_entity),
				face_id: self.face_id.$fnct(other.face_id),
				face_entity: self.face_entity.$fnct(other.face_entity),
			}
		}
	};
}

impl std::ops::BitAnd for FormatConfiguation {
	derive_ops!(bitand);
}
impl std::ops::BitOr for FormatConfiguation {
	derive_ops!(bitor);
}
impl std::ops::BitXor for FormatConfiguation {
	derive_ops!(bitxor);
}

#[test]
fn mutex_fmt() {
	let mutex = Arc::new(Mutex::new([0.0, 1.0]));
	let mf = MutexFmt(&mutex);
	assert_eq!(&format!("{:?}", mf), "[0.0, 1.0]");
	let a = mutex.lock();
	assert_eq!(&format!("{:?}", mf), "<locked>");
	drop(a);
	let mutex0 = Arc::clone(&mutex);
	let _ = std::thread::spawn(move || {
		let mut a = mutex0.lock().unwrap();
		*a = [1.0, 2.0];
		panic!();
	})
	.join();
	assert!(mutex.is_poisoned());
	assert_eq!(&format!("{:?}", mf), "[1.0, 2.0]");
}
