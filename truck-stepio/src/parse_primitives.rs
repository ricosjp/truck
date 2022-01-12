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
        impl Empty for $mod::Curve {
            fn empty() -> Self { Self::new(Empty::empty()) }
        }
        impl Empty for $mod::Surface {
            fn empty() -> Self { Self::new(Empty::empty()) }
        }

        $crate::impl_from!(
            impl From<&$mod::CartesianPoint> for Point2 {
                fn from(pt: &$mod::CartesianPoint) -> Self {
                    let mut pt = pt.coordinates.clone();
                    pt.resize(2, 0.0_f64.into());
                    Point2::new(*pt[0], *pt[1])
                }
            }
            impl From<&$mod::CartesianPoint> for Point3 {
                fn from(pt: &$mod::CartesianPoint) -> Self {
                    let mut pt = pt.coordinates.clone();
                    pt.resize(3, 0.0_f64.into());
                    Point3::new(*pt[0], *pt[1], *pt[2])
                }
            }
            /*
            impl From<&$mod::CartesianPointAny> for Point2 {
                fn from(pt: &$mod::CartesianPointAny) -> Self {
                    Self::from(AsRef::<$mod::CartesianPoint>::as_ref(pt))
                }
            }
            impl From<&$mod::CartesianPointAny> for Point3 {
                fn from(pt: &$mod::CartesianPointAny) -> Self {
                    Self::from(AsRef::<$mod::CartesianPoint>::as_ref(pt))
                }
            }
            */
            impl From<&$mod::EvaluatedDegeneratePcurve> for Point3 {
                fn from(pt: &$mod::EvaluatedDegeneratePcurve) -> Self {
                    Point3::from(&pt.equivalent_point)
                }
            }
            impl From<&$mod::Direction> for Vector2 {
                fn from(dir: &$mod::Direction) -> Self {
                    let mut dir = dir.direction_ratios.clone();
                    dir.resize(2, 0.0);
                    Vector2::new(dir[0], dir[1])
                }
            }
            impl From<&$mod::Direction> for Vector3 {
                fn from(dir: &$mod::Direction) -> Self {
                    let mut dir = dir.direction_ratios.clone();
                    dir.resize(3, 0.0);
                    Vector3::new(dir[0], dir[1], dir[2])
                }
            }
            impl From<&$mod::Vector> for Vector2 {
                fn from(vec: &$mod::Vector) -> Self {
                    Vector2::from(&vec.orientation) * vec.magnitude.0
                }
            }
            impl From<&$mod::Vector> for Vector3 {
                fn from(vec: &$mod::Vector) -> Self {
                    Vector3::from(&vec.orientation) * vec.magnitude.0
                }
            }
            impl From<&$mod::Placement> for Point2 {
                fn from(placement: &$mod::Placement) -> Self { Self::from(&placement.location) }
            }
            impl From<&$mod::Placement> for Point3 {
                fn from(placement: &$mod::Placement) -> Self { Self::from(&placement.location) }
            }
            impl From<&$mod::Axis2Placement2D> for Matrix3 {
                fn from(axis: &$mod::Axis2Placement2D) -> Self {
                    let z = Point2::from(&axis.placement);
                    let x = match &axis.ref_direction {
                        Some(axis) => Vector2::from(axis),
                        None => Vector2::unit_x(),
                    };
                    Matrix3::new(x.x, x.y, 0.0, -x.y, x.x, 0.0, z.x, z.y, 1.0)
                }
            }
            impl From<&$mod::Axis2Placement3D> for Matrix4 {
                fn from(axis: &$mod::Axis2Placement3D) -> Matrix4 {
                    let w = Point3::from(&axis.placement);
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
            impl From<&$mod::CartesianTransformationOperator> for Matrix3 {
                fn from(trans: &$mod::CartesianTransformationOperator) -> Self {
                    let x = match &trans.axis1 {
                        Some(x) => Vector2::from(x),
                        None => Vector2::unit_x(),
                    };
                    let y = match &trans.axis2 {
                        Some(y) => Vector2::from(y),
                        None => Vector2::unit_y(),
                    };
                    let z = Point2::from(&trans.local_origin);
                    let scale = trans.scale.unwrap_or(1.0);
                    Self::new(x[0], x[1], 0.0, y[0], y[1], 0.0, z[0], z[1], 1.0) * scale
                }
            }
            impl From<&$mod::CartesianTransformationOperator3D> for Matrix4 {
                fn from(trans: &$mod::CartesianTransformationOperator3D) -> Self {
                    let trans2d = &trans.cartesian_transformation_operator;
                    let x = match &trans2d.axis1 {
                            Some(x) => Vector3::from(x),
                        None => Vector3::unit_x(),
                    };
                    let y = match &trans2d.axis2 {
                        Some(y) => Vector3::from(y),
                        None => Vector3::unit_y(),
                    };
                    let z = match &trans.axis3 {
                        Some(z) => Vector3::from(z),
                        None => Vector3::unit_z(),
                    };
                    let w = Point3::from(&trans2d.local_origin);
                    let scale = trans2d.scale.unwrap_or(1.0);
                    Self::new(
                        x[0], x[1], x[2], 0.0, y[0], y[1], y[2], 0.0, z[0], z[1], z[2], 0.0, w[0],
                        w[1], w[2], 1.0,
                    ) * scale
                }
            }
        );
        $crate::impl_try_from!(
            impl TryFrom<&$mod::DegeneratePcurveAny> for Point3 {
                fn try_from(pt: &$mod::DegeneratePcurveAny) -> Result<Self, ExpressParseError> {
                    use $mod::DegeneratePcurveAny as DP;
                    match pt {
                        DP::DegeneratePcurve(_) => Err("not enough data!".to_string()),
                        DP::EvaluatedDegeneratePcurve(x) => Ok(Point3::from(&**x)),
                    }
                }
            }
            impl TryFrom<&$mod::PointOnCurve> for Point2 {
                fn try_from(pt: &$mod::PointOnCurve) -> Result<Self, ExpressParseError> {
                    let curve = Curve::<Point2, Vector3, Matrix3>::try_from(&pt.basis_curve)?;
                    let t: f64 = *pt.point_parameter.as_ref();
                    Ok(curve.subs(t))
                }
            }
            impl TryFrom<&$mod::PointOnCurve> for Point3 {
                fn try_from(pt: &$mod::PointOnCurve) -> Result<Self, ExpressParseError> {
                    let curve = Curve::<Point3, Vector4, Matrix4>::try_from(&pt.basis_curve)?;
                    let t: f64 = *pt.point_parameter.as_ref();
                    Ok(curve.subs(t))
                }
            }
            impl TryFrom<&$mod::PointOnSurface> for Point3 {
                fn try_from(pt: &$mod::PointOnSurface) -> Result<Self, ExpressParseError> {
                    let surface = Surface::try_from(&pt.basis_surface)?;
                    let u: f64 = *pt.point_parameter_u.as_ref();
                    let v: f64 = *pt.point_parameter_v.as_ref();
                    Ok(surface.subs(u, v))
                }
            }
            impl TryFrom<&$mod::PointReplica> for Point2 {
                fn try_from(pt: &$mod::PointReplica) -> Result<Self, ExpressParseError> {
                    Ok(Matrix3::try_from(&pt.transformation)?
                        .transform_point(Point2::try_from(&pt.parent_pt)?))
                }
            }
            impl TryFrom<&$mod::PointReplica> for Point3 {
                fn try_from(pt: &$mod::PointReplica) -> Result<Self, ExpressParseError> {
                    Ok(Matrix4::try_from(&pt.transformation)?
                        .transform_point(Point3::try_from(&pt.parent_pt)?))
                }
            }
            impl TryFrom<&$mod::PointAny> for Point2 {
                fn try_from(pt: &$mod::PointAny) -> Result<Self, ExpressParseError> {
                    use $mod::PointAny as P;
                    match pt {
                        P::Point(_) => Err("not enough data!".to_string()),
                        P::CartesianPoint(x) => Ok(Self::from(&**x)),
                        P::DegeneratePcurve(x) => Err("this point is not 2d".to_string()),
                        P::PointOnCurve(x) => Self::try_from(&**x),
                        P::PointOnSurface(_) => Err("this point is not 2d.".to_string()),
                        P::PointReplica(x) => Self::try_from(&**x),
                    }
                }
            }
            impl TryFrom<&$mod::PointAny> for Point3 {
                fn try_from(pt: &$mod::PointAny) -> Result<Self, ExpressParseError> {
                    use $mod::PointAny as P;
                    match pt {
                        P::Point(_) => Err("not enough data!".to_string()),
                        P::CartesianPoint(x) => Ok(Self::from(&**x)),
                        P::DegeneratePcurve(x) => Self::try_from(&**x),
                        P::PointOnCurve(x) => Self::try_from(&**x),
                        P::PointOnSurface(x) => Self::try_from(&**x),
                        P::PointReplica(x) => Self::try_from(&**x),
                    }
                }
            }
            impl TryFrom<&$mod::Axis2Placement> for Matrix3 {
                fn try_from(axis: &$mod::Axis2Placement) -> Result<Self, ExpressParseError> {
                    use $mod::Axis2Placement::*;
                    match axis {
                        Axis2Placement2D(axis) => Ok(Matrix3::from(axis.as_ref())),
                        Axis2Placement3D(_) => Err("This is not a 2D axis placement.".to_string()),
                    }
                }
            }
            impl TryFrom<&$mod::Axis2Placement> for Matrix4 {
                fn try_from(axis: &$mod::Axis2Placement) -> Result<Self, ExpressParseError> {
                    use $mod::Axis2Placement::*;
                    match axis {
                        Axis2Placement2D(_) => Err("This is not a 3D axis placement.".to_string()),
                        Axis2Placement3D(axis) => Ok(Matrix4::from(axis.as_ref())),
                    }
                }
            }
            impl TryFrom<&$mod::CartesianTransformationOperatorAny> for Matrix3 {
                fn try_from(
                    trans: &$mod::CartesianTransformationOperatorAny,
                ) -> Result<Self, ExpressParseError> {
                        use $mod::CartesianTransformationOperatorAny as CTO;
                    match trans {
                        CTO::CartesianTransformationOperator(x) => Ok(Self::from(&**x)),
                        _ => Err("This is not 2d transformation.".to_string()),
                    }
                }
            }
            impl TryFrom<&$mod::CartesianTransformationOperatorAny> for Matrix4 {
                fn try_from(
                    trans: &$mod::CartesianTransformationOperatorAny,
                ) -> Result<Self, ExpressParseError> {
                    use $mod::CartesianTransformationOperatorAny as CTO;
                    match trans {
                        CTO::CartesianTransformationOperator3D(x) => Ok(Self::from(&**x)),
                        _ => Err("This is not 3d transformation.".to_string()),
                    }
                }
            }
        );
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
        impl From<Matrix3> for $mod::Axis2Placement2D {
            fn from(mat: Matrix3) -> Self {
                Self {
                    placement: Homogeneous::to_point(mat[2]).into(),
                    ref_direction: Some(mat[0].truncate().into()),
                }
            }
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

    };
}
