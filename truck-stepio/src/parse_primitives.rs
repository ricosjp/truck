#[macro_export]
macro_rules! parse_primitives {
	($mod: tt) => {
		impl From<&$mod::CartesianPoint> for Point2 {
			fn from(pt: &$mod::CartesianPoint) -> Self {
				let mut pt = pt.coordinates.clone();
				pt.resize(2, 0.0_f64.into());
				Point2::new(*pt[0], *pt[1])
			}
		}
		impl From<$mod::CartesianPoint> for Point2 {
			fn from(pt: $mod::CartesianPoint) -> Self { Point2::from(&pt) }
		}
		impl From<&$mod::CartesianPoint> for Point3 {
			fn from(pt: &$mod::CartesianPoint) -> Self {
				let mut pt = pt.coordinates.clone();
				pt.resize(3, 0.0_f64.into());
				Point3::new(*pt[0], *pt[1], *pt[2])
			}
		}
		impl From<$mod::CartesianPoint> for Point3 {
			fn from(pt: $mod::CartesianPoint) -> Self { Point3::from(&pt) }
		}
		impl From<&$mod::Direction> for Vector2 {
			fn from(dir: &$mod::Direction) -> Self {
				let mut dir = dir.direction_ratios.clone();
				dir.resize(2, 0.0);
				Vector2::new(dir[0], dir[1])
			}
		}
		impl From<$mod::Direction> for Vector2 {
			fn from(dir: $mod::Direction) -> Self { Vector2::from(&dir) }
		}
		impl From<&$mod::Direction> for Vector3 {
			fn from(dir: &$mod::Direction) -> Self {
				let mut dir = dir.direction_ratios.clone();
				dir.resize(3, 0.0);
				Vector3::new(dir[0], dir[1], dir[2])
			}
		}
		impl From<$mod::Direction> for Vector3 {
			fn from(dir: $mod::Direction) -> Self { Vector3::from(&dir) }
		}
		impl From<&$mod::Vector> for Vector3 {
			fn from(vec: &$mod::Vector) -> Self { Vector3::from(&vec.orientation) * vec.magnitude.0 }
		}
		impl From<$mod::Vector> for Vector3 {
			fn from(vec: $mod::Vector) -> Self { Vector3::from(&vec) }
		}
	};
}
