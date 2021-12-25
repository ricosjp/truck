#[macro_export]
macro_rules! impl_surface {
    ($mod: tt, $mod_impl_surface: ident) => {
        mod $mod_impl_surface {
            use super::$mod;
            //use std::convert::TryFrom;
            //use std::result::Result;
            use $crate::alias::*;
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
        impl From<&$mod::CylindricalSurface> for RevolutedLine {
            fn from(cs: &$mod::CylindricalSurface) -> Self {
                let mat = Matrix4::from(&cs.elementary_surface.position);
                let radius = **cs.radius;
                let o = Point3::from_homogeneous(mat[3]);
                let (x, z) = (mat[0].truncate(), mat[2].truncate());
                let p = o + radius * x;
                let q = p + z;
                Processor::new(RevolutedCurve::by_revolution(Line(p, q), o, z)).inverse()
            }
        }
        impl From<&$mod::ConicalSurface> for RevolutedLine {
            fn from(cs: &$mod::ConicalSurface) -> Self {
                let mat = Matrix4::from(&cs.elementary_surface.position);
                let (radius, angle) = (*cs.radius, *cs.semi_angle);
                let o = Point3::from_homogeneous(mat[3]);
                let (x, z) = (mat[0].truncate(), mat[2].truncate());
                let p = o + radius * x;
                let q = p + f64::sin(angle) * x + f64::cos(angle) * z;
                Processor::new(RevolutedCurve::by_revolution(Line(p, q), o, z)).inverse()
            }
        }
        impl From<&$mod::SphericalSurface> for Processor<Sphere, Matrix3> {
            fn from(ss: &$mod::SphericalSurface) -> Self {
                let mat = Matrix4::from(&ss.elementary_surface.position);
                let center = Point3::from_homogeneous(mat[3]);
                let radius = **ss.radius;
                let mat =
                    Matrix3::from_cols(mat[0].truncate(), mat[1].truncate(), mat[2].truncate());
                Processor::new(Sphere::new(center, radius)).transformed(mat)
            }
        }
    };
}
