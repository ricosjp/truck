#[macro_export]
macro_rules! impl_surface {
	($mod: tt, $mod_impl_surface: ident) => {
		mod $mod_impl_surface {
			use super::$mod;
			//use std::convert::TryFrom;
			//use std::result::Result;
			use $crate::truck_geometry::*;
			$crate::sub_impl_surface!($mod);
		}
	};
}

#[doc(hidden)]
#[macro_export]
macro_rules! sub_impl_surface {
	($mod: tt) => {
		impl From<&$mod::Plane> for Plane {
			fn from(plane: &$mod::Plane) -> Self {
				let mat = Matrix4::from(&plane.elementary_surface.position);
				let o = Point3::from_homogeneous(mat[3]);
				let p = o + mat[0].truncate();
				let q = o + mat[1].truncate();
				Self::new(o, p, q)
			}
		}
		impl From<&$mod::CylindricalSurface> for RevolutedCurve<Line<Point3>> {
			fn from(cs: &$mod::CylindricalSurface) -> Self {
				let mat = Matrix4::from(&cs.elementary_surface.position);
				let radius = **cs.radius;
				let o = Point3::from_homogeneous(mat[3]);
				let (x, z) = (mat[0].truncate(), mat[2].truncate());
				let p = o + radius * x;
				let q = p + z;
				Self::by_revolution(Line(q, p), o, z)
			}
		}
		impl From<&$mod::ConicalSurface> for RevolutedCurve<Line<Point3>> {
			fn from(cs: &$mod::ConicalSurface) -> Self {
				let mat = Matrix4::from(&cs.elementary_surface.position);
				let (radius, angle) = (*cs.radius, *cs.semi_angle);
				let o = Point3::from_homogeneous(mat[3]);
				let (x, z) = (mat[0].truncate(), mat[2].truncate());
				let p = o + radius * x;
				let q = p + f64::sin(angle) * x + f64::cos(angle) * z;
				Self::by_revolution(Line(q, p), o, z)
			}
		}
	};
}
