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
		impl From<&$mod::CylindricalSurface>
			for truck_geometry::RevolutedCurve<truck_geometry::Line<Point3>>
		{
			fn from(cs: &$mod::CylindricalSurface) -> Self {
				let mat = Matrix4::from(&cs.elementary_surface.position);
				let radius = **cs.radius;
				let o = Point3::from_homogeneous(mat[3]);
				let p = o + radius * mat[0].truncate();
				let q = p + mat[2].truncate();
				Self::by_revolution(truck_geometry::Line(p, q), o, mat[2].truncate())
			}
		}
		impl From<&$mod::ConicalSurface> for truck_geometry::RevolutedCurve<truck_geometry::Line<Point3>> {
			fn from(cs: &$mod::ConicalSurface) -> Self {
				let mat = Matrix4::from(&cs.elementary_surface.position);
				let radius = *cs.radius;
				let o = Point3::from_homogeneous(mat[3]);
				let p = o + radius * mat[0].truncate();
			}
		}
	};
}
