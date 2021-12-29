#[macro_export]
macro_rules! parse_primitives {
    ($mod: tt, $mod_parse_primitives: ident) => {
        mod $mod_parse_primitives {
            use super::$mod;
            use std::convert::TryFrom;
            use std::result::Result;
            use $crate::alias::*;
            $crate::sub_parse_primitives!($mod);
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! sub_parse_primitives {
    ($mod: tt) => {
        impl Empty for $mod::RepresentationItem {
            fn empty() -> Self { Self::new(String::new().into()) }
        }
        impl Empty for $mod::GeometricRepresentationItem {
            fn empty() -> Self { Self::new(Empty::empty()) }
        }
        impl Empty for $mod::Point {
            fn empty() -> $mod::Point { Self::new(Empty::empty()) }
        }

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
        impl From<Point2> for $mod::CartesianPoint {
            fn from(p: Point2) -> Self {
                Self {
                    point: Empty::empty(),
                    coordinates: vec![p.x.into(), p.y.into()],
                }
            }
        }
        impl From<Point3> for $mod::CartesianPoint {
            fn from(p: Point3) -> Self {
                Self {
                    point: Empty::empty(),
                    coordinates: vec![p.x.into(), p.y.into(), p.z.into()],
                }
            }
        }
        impl From<&$mod::CartesianPointAny> for Point2 {
            fn from(pt: &$mod::CartesianPointAny) -> Self {
                Self::from(AsRef::<$mod::CartesianPoint>::as_ref(pt))
            }
        }
        impl From<$mod::CartesianPointAny> for Point2 {
            fn from(pt: $mod::CartesianPointAny) -> Self { Point2::from(&pt) }
        }
        impl From<&$mod::CartesianPointAny> for Point3 {
            fn from(pt: &$mod::CartesianPointAny) -> Self {
                Self::from(AsRef::<$mod::CartesianPoint>::as_ref(pt))
            }
        }
        impl From<$mod::CartesianPointAny> for Point3 {
            fn from(pt: $mod::CartesianPointAny) -> Self { Point3::from(&pt) }
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
        impl From<Vector2> for $mod::Direction {
            fn from(dir: Vector2) -> Self {
                Self::new(Empty::empty(), vec![dir.x.into(), dir.y.into()])
            }
        }
        impl From<Vector3> for $mod::Direction {
            fn from(dir: Vector3) -> Self {
                Self::new(
                    Empty::empty(),
                    vec![dir.x.into(), dir.y.into(), dir.z.into()],
                )
            }
        }
        impl From<&$mod::Vector> for Vector2 {
            fn from(vec: &$mod::Vector) -> Self {
                Vector2::from(&vec.orientation) * vec.magnitude.0
            }
        }
        impl From<$mod::Vector> for Vector2 {
            fn from(vec: $mod::Vector) -> Self { Vector2::from(&vec) }
        }
        impl From<&$mod::Vector> for Vector3 {
            fn from(vec: &$mod::Vector) -> Self {
                Vector3::from(&vec.orientation) * vec.magnitude.0
            }
        }
        impl From<$mod::Vector> for Vector3 {
            fn from(vec: $mod::Vector) -> Self { Vector3::from(&vec) }
        }
        impl From<Vector2> for $mod::Vector {
            fn from(dir: Vector2) -> Self {
                let mag = dir.magnitude();
                let ori = dir / mag;
                Self::new(Empty::empty(), ori.into(), mag.into())
            }
        }
        impl From<Vector3> for $mod::Vector {
            fn from(dir: Vector3) -> Self {
                let mag = dir.magnitude();
                let ori = dir / mag;
                Self::new(Empty::empty(), ori.into(), mag.into())
            }
        }
        impl From<Point2> for $mod::Placement {
            fn from(p: Point2) -> $mod::Placement {
                Self::new(Empty::empty(), $mod::CartesianPoint::from(p).into())
            }
        }
        impl From<Point3> for $mod::Placement {
            fn from(p: Point3) -> $mod::Placement {
                Self::new(Empty::empty(), $mod::CartesianPoint::from(p).into())
            }
        }
        impl From<&$mod::Axis2Placement2D> for Matrix3 {
            fn from(axis: &$mod::Axis2Placement2D) -> Self {
                let z = Point2::from(&axis.placement.location);
                let x = match &axis.ref_direction {
                    Some(axis) => Vector2::from(axis),
                    None => Vector2::unit_x(),
                };
                Matrix3::new(x.x, x.y, 0.0, -x.y, x.x, 0.0, z.x, z.y, 1.0)
            }
        }
        impl From<$mod::Axis2Placement2D> for Matrix3 {
            fn from(axis: $mod::Axis2Placement2D) -> Self { Matrix3::from(&axis) }
        }
        impl From<Matrix3> for $mod::Axis2Placement2D {
            fn from(mat: Matrix3) -> Self {
                Self {
                    placement: Homogeneous::to_point(mat[2]).into(),
                    ref_direction: Some(mat[0].truncate().into()),
                }
            }
        }
        impl From<&$mod::Axis2Placement3D> for Matrix4 {
            fn from(axis: &$mod::Axis2Placement3D) -> Matrix4 {
                let w = Point3::from(&axis.placement.location);
                let z = match &axis.axis {
                    Some(axis) => Vector3::from(axis),
                    None => Vector3::unit_z(),
                };
                let x = match &axis.ref_direction {
                    Some(axis) => Vector3::from(axis),
                    None => Vector3::unit_x(),
                };
                let x = (x - x.dot(z) * z).normalize();
                let y = z.cross(x);
                Matrix4::new(
                    x.x, x.y, x.z, 0.0, y.x, y.y, y.z, 0.0, z.x, z.y, z.z, 0.0, w.x, w.y, w.z, 1.0,
                )
            }
        }
        impl From<$mod::Axis2Placement3D> for Matrix4 {
            fn from(axis: $mod::Axis2Placement3D) -> Self { Matrix4::from(&axis) }
        }
        impl From<Matrix4> for $mod::Axis2Placement3D {
            fn from(mat: Matrix4) -> Self {
                Self {
                    placement: Homogeneous::to_point(mat[3]).into(),
                    axis: Some(mat[2].truncate().into()),
                    ref_direction: Some(mat[0].truncate().into()),
                }
            }
        }
        impl TryFrom<&$mod::Axis2Placement> for Matrix3 {
            type Error = String;
            fn try_from(axis: &$mod::Axis2Placement) -> Result<Self, String> {
                use $mod::Axis2Placement::*;
                match axis {
                    Axis2Placement2D(axis) => Ok(Matrix3::from(axis.as_ref())),
                    Axis2Placement3D(_) => Err("This is not a 2D axis placement.".to_string()),
                }
            }
        }
        impl TryFrom<&$mod::Axis2Placement> for Matrix4 {
            type Error = String;
            fn try_from(axis: &$mod::Axis2Placement) -> Result<Self, String> {
                use $mod::Axis2Placement::*;
                match axis {
                    Axis2Placement2D(_) => Err("This is not a 3D axis placement.".to_string()),
                    Axis2Placement3D(axis) => Ok(Matrix4::from(axis.as_ref())),
                }
            }
        }
    };
}
