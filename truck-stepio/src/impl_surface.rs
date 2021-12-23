#[macro_export]
macro_rules! impl_surface {
	($mod: tt) => {
		impl From<&$mod::Plane> for truck_geometry::Plane {
			fn from(plane: &$mod::Plane) -> Self {
				let mat = Matrix4::from(&plane.elementary_surface.position);
				let o = Point3::from_homogeneous(mat[3]);
				let p = o + mat[0].truncate();
				let q = o + mat[1].truncate();
				Self::new(o, p, q)
			}
		}
	};
}