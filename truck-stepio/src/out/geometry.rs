use super::{Result, *};
use truck_geometry::prelude::*;
use truck_modeling::{Curve as ModelingCurve, Surface as ModelingSurface};
use truck_polymesh::PolylineCurve;

impl DisplayByStep for Point2 {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        f.write_fmt(format_args!(
            "#{idx} = CARTESIAN_POINT('', {coordinates});\n",
            coordinates = SliceDisplay(AsRef::<[f64; 2]>::as_ref(self)),
        ))
    }
}
impl_const_step_length!(Point2, 1);

impl DisplayByStep for Point3 {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        f.write_fmt(format_args!(
            "#{idx} = CARTESIAN_POINT('', {coordinates});\n",
            coordinates = SliceDisplay(AsRef::<[f64; 3]>::as_ref(self)),
        ))
    }
}
impl_const_step_length!(Point3, 1);

/// class for display `DIRECTION`.
#[derive(Clone, Debug, Copy)]
pub struct VectorAsDirection<V>(pub V);

impl DisplayByStep for VectorAsDirection<Vector2> {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        f.write_fmt(format_args!(
            "#{idx} = DIRECTION('', {direction_ratios});\n",
            direction_ratios = SliceDisplay(AsRef::<[f64; 2]>::as_ref(&self.0)),
        ))
    }
}

impl DisplayByStep for VectorAsDirection<Vector3> {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        f.write_fmt(format_args!(
            "#{idx} = DIRECTION('', {direction_ratios});\n",
            direction_ratios = SliceDisplay(AsRef::<[f64; 3]>::as_ref(&self.0)),
        ))
    }
}
impl_const_step_length!(VectorAsDirection<V>, 1, <V>);

impl<V> DisplayByStep for V
where
    V: InnerSpace<Scalar = f64>,
    VectorAsDirection<V>: DisplayByStep,
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let magnitude = self.magnitude();
        let direction_idx = idx + 1;
        f.write_fmt(format_args!(
            "#{idx} = VECTOR('', #{direction_idx}, {magnitude});\n{direction}",
            direction = StepDisplay::new(VectorAsDirection(*self / magnitude), direction_idx),
            magnitude = FloatDisplay(magnitude),
        ))
    }
}
impl_const_step_length!(Vector2, 2);
impl_const_step_length!(Vector3, 2);

impl<P> DisplayByStep for Line<P>
where
    P: EuclideanSpace + ConstStepLength + DisplayByStep,
    P::Diff: DisplayByStep,
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let pnt_idx = idx + 1;
        let dir_idx = idx + 1 + P::LENGTH;
        f.write_fmt(format_args!(
            "#{idx} = LINE('', #{pnt_idx}, #{dir_idx});\n{pnt}{dir}",
            pnt = StepDisplay::new(self.0, pnt_idx),
            dir = StepDisplay::new(self.1 - self.0, dir_idx),
        ))
    }
}

impl<P> StepLength for Line<P>
where
    P: EuclideanSpace + ConstStepLength,
    P::Diff: ConstStepLength,
{
    #[inline(always)]
    fn step_length(&self) -> usize { <Self as ConstStepLength>::LENGTH }
}

impl<P> ConstStepLength for Line<P>
where
    P: EuclideanSpace + ConstStepLength,
    P::Diff: ConstStepLength,
{
    const LENGTH: usize = 1 + P::LENGTH + P::Diff::LENGTH;
}

impl<P> StepCurve for Line<P> {}

impl<P> DisplayByStep for PolylineCurve<P>
where P: Copy + ConstStepLength + DisplayByStep
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        f.write_fmt(format_args!(
            "#{idx} = POLYLINE('', {range});\n",
            range = IndexSliceDisplay(idx + 1..=idx + self.0.len())
        ))?;
        let closure = |(i, p): (usize, &P)| p.fmt(idx + 1 + i * P::LENGTH, f);
        self.0.iter().enumerate().try_for_each(closure)
    }
}

impl<P: ConstStepLength> StepLength for PolylineCurve<P> {
    #[inline(always)]
    fn step_length(&self) -> usize { 1 + self.0.len() * P::LENGTH }
}

impl<P> StepCurve for PolylineCurve<P> {}

impl<P> DisplayByStep for BSplineCurve<P>
where P: Copy + ConstStepLength + DisplayByStep
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let (knots, multi) = self.knot_vec().to_single_multi();
        let control_points_instances = self
            .control_points()
            .iter()
            .enumerate()
            .map(|(i, p)| StepDisplay::new(*p, idx + 1 + i * P::LENGTH))
            .collect::<Vec<_>>();
        f.write_fmt(format_args!(
            "#{idx} = B_SPLINE_CURVE_WITH_KNOTS('', {degree}, {control_points_list}, .UNSPECIFIED., .U., .U., {knot_multiplicities}, {knots}, .UNSPECIFIED.);\n{control_points_instances}",
            degree = self.degree(),
            control_points_list = IndexSliceDisplay((idx + 1..=idx + self.control_points().len() * P::LENGTH).step_by(P::LENGTH)),
			knot_multiplicities = SliceDisplay(&multi),
            knots = SliceDisplay(&knots),
            control_points_instances = SliceDisplay(&control_points_instances),
		))
    }
}

impl<P> StepLength for BSplineCurve<P> {
    #[inline(always)]
    fn step_length(&self) -> usize { self.control_points().len() + 1 }
}

impl<P> StepCurve for BSplineCurve<P> {}

impl<V> DisplayByStep for NurbsCurve<V>
where
    V: Homogeneous<Scalar = f64>,
    V::Point: ConstStepLength + DisplayByStep,
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let (knots, multi) = self.knot_vec().to_single_multi();
        let control_points_instances = self
            .control_points()
            .iter()
            .enumerate()
            .map(|(i, v)| StepDisplay::new(v.to_point(), idx + 1 + i * V::Point::LENGTH))
            .collect::<Vec<_>>();
        let weights = self
            .control_points()
            .iter()
            .map(|v| v.weight())
            .collect::<Vec<_>>();
        f.write_fmt(format_args!(
            "#{idx} = (
    BOUNDED_CURVE()
    B_SPLINE_CURVE({degree}, {control_points_list}, .UNSPECIFIED., .U., .U.)
    B_SPLINE_CURVE_WITH_KNOTS({knot_multiplicities}, {knots}, .UNSPECIFIED.)
    CURVE()
    GEOMETRIC_REPRESENTATION_ITEM()
    RATIONAL_B_SPLINE_CURVE({weights})
    REPRESENTATION_ITEM('')
);\n{control_points_instances}",
            degree = self.degree(),
            control_points_list = IndexSliceDisplay(
                (idx + 1..=idx + self.control_points().len() * V::Point::LENGTH)
                    .step_by(V::Point::LENGTH)
            ),
            knot_multiplicities = SliceDisplay(&multi),
            knots = SliceDisplay(&knots),
            weights = SliceDisplay(&weights),
            control_points_instances = SliceDisplay(&control_points_instances),
        ))
    }
}

impl<V> StepLength for NurbsCurve<V> {
    #[inline(always)]
    fn step_length(&self) -> usize { self.control_points().len() + 1 }
}

impl<V> StepCurve for NurbsCurve<V> {}

impl DisplayByStep for Processor<TrimmedCurve<UnitCircle<Point2>>, Matrix3> {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let transform = *self.transform();
        let position_idx = idx + 1;
        let location_idx = idx + 2;
        let ref_direction_idx = idx + 3;
        let r0 = transform[0].magnitude();
        let r1 = transform[1].magnitude();
        let ref_direction = VectorAsDirection(transform[0].truncate() / r0);
        let location = transform[2].to_point();
        if r0.near(&r1) {
            let r = FloatDisplay(r0);
            f.write_fmt(format_args!("#{idx} = CIRCLE('', #{position_idx}, {r});\n"))?;
        } else {
            let (r0, r1) = (FloatDisplay(r0), FloatDisplay(r1));
            f.write_fmt(format_args!(
                "#{idx} = ELLIPSE('', #{position_idx}, {r0}, {r1});\n"
            ))?;
        }
        f.write_fmt(format_args!(
            "#{position_idx} = AXIS2_PLACEMENT_2D('', #{location_idx}, #{ref_direction_idx});\n",
        ))?;
        DisplayByStep::fmt(&location, location_idx, f)?;
        DisplayByStep::fmt(&ref_direction, ref_direction_idx, f)
    }
}
impl_const_step_length!(Processor<TrimmedCurve<UnitCircle<Point2>>, Matrix3>, 4);

impl DisplayByStep for Processor<TrimmedCurve<UnitCircle<Point3>>, Matrix4> {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let transform = self.transform();
        let position_idx = idx + 1;
        let location_idx = idx + 2;
        let axis_idx = idx + 3;
        let ref_direction_idx = idx + 4;
        let location = transform[3].to_point();
        let axis = VectorAsDirection(transform[2].truncate().normalize());
        let r0 = transform[0].magnitude();
        let r1 = transform[1].magnitude();
        let ref_direction = VectorAsDirection(transform[0].truncate() / r0);
        if r0.near(&r1) {
            let r = FloatDisplay(r0);
            f.write_fmt(format_args!("#{idx} = CIRCLE('', #{position_idx}, {r});\n"))?;
        } else {
            let (r0, r1) = (FloatDisplay(r0), FloatDisplay(r1));
            f.write_fmt(format_args!(
                "#{idx} = ELLIPSE('', #{position_idx}, {r0}, {r1});\n"
            ))?;
        }
        f.write_fmt(format_args!(
            "#{position_idx} = AXIS2_PLACEMENT_3D('', #{location_idx}, #{axis_idx}, #{ref_direction_idx});\n",
        ))?;
        DisplayByStep::fmt(&location, location_idx, f)?;
        DisplayByStep::fmt(&axis, axis_idx, f)?;
        DisplayByStep::fmt(&ref_direction, ref_direction_idx, f)
    }
}
impl_const_step_length!(Processor<TrimmedCurve<UnitCircle<Point3>>, Matrix4>, 5);

impl DisplayByStep for Processor<TrimmedCurve<UnitHyperbola<Point2>>, Matrix3> {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let transform = *self.transform();
        let position_idx = idx + 1;
        let location_idx = idx + 2;
        let ref_direction_idx = idx + 3;
        let r0 = transform[0].magnitude();
        let r1 = transform[1].magnitude();
        let ref_direction_raw = VectorAsDirection(transform[0].truncate() / r0);
        let ref_direction = StepDisplay::new(ref_direction_raw, ref_direction_idx);
        let location = StepDisplay::new(transform[2].to_point(), location_idx);
        let (r0, r1) = (FloatDisplay(r0), FloatDisplay(r1));
        f.write_fmt(format_args!(
            "#{idx} = HYPERBOLA('', #{position_idx}, {r0}, {r1});
#{position_idx} = AXIS2_PLACEMENT_2D('', #{location_idx}, #{ref_direction_idx});
{location}{ref_direction}"
        ))
    }
}
impl_const_step_length!(Processor<TrimmedCurve<UnitHyperbola<Point2>>, Matrix3>, 4);

impl DisplayByStep for Processor<TrimmedCurve<UnitHyperbola<Point3>>, Matrix4> {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let transform = self.transform();
        let position_idx = idx + 1;
        let location_idx = idx + 2;
        let axis_idx = idx + 3;
        let ref_direction_idx = idx + 4;
        let location = StepDisplay::new(transform[3].to_point(), location_idx);
        let axis_raw = VectorAsDirection(transform[2].truncate().normalize());
        let axis = StepDisplay::new(axis_raw, axis_idx);
        let r0 = transform[0].magnitude();
        let r1 = transform[1].magnitude();
        let ref_direction_raw = VectorAsDirection(transform[0].truncate() / r0);
        let ref_direction = StepDisplay::new(ref_direction_raw, ref_direction_idx);
        let (r0, r1) = (FloatDisplay(r0), FloatDisplay(r1));
        f.write_fmt(format_args!(
            "#{idx} = HYPERBOLA('', #{position_idx}, {r0}, {r1});
#{position_idx} = AXIS2_PLACEMENT_3D('', #{location_idx}, #{axis_idx}, #{ref_direction_idx});
{location}{axis}{ref_direction}"
        ))
    }
}
impl_const_step_length!(Processor<TrimmedCurve<UnitHyperbola<Point3>>, Matrix4>, 5);

impl DisplayByStep for Processor<TrimmedCurve<UnitParabola<Point2>>, Matrix3> {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let transform = *self.transform();
        let position_idx = idx + 1;
        let location_idx = idx + 2;
        let ref_direction_idx = idx + 3;
        let r0 = transform[0].magnitude();
        let r1 = transform[1].magnitude();
        let focal_dist = FloatDisplay(r1 * r1 / r0);
        let ref_direction_raw = VectorAsDirection(transform[0].truncate() / r0);
        let ref_direction = StepDisplay::new(ref_direction_raw, ref_direction_idx);
        let location = StepDisplay::new(transform[2].to_point(), location_idx);
        f.write_fmt(format_args!(
            "#{idx} = PARABOLA('', #{position_idx}, {focal_dist});
#{position_idx} = AXIS2_PLACEMENT_2D('', #{location_idx}, #{ref_direction_idx});
{location}{ref_direction}"
        ))
    }
}
impl_const_step_length!(Processor<TrimmedCurve<UnitParabola<Point2>>, Matrix3>, 4);

impl DisplayByStep for Processor<TrimmedCurve<UnitParabola<Point3>>, Matrix4> {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let transform = self.transform();
        let position_idx = idx + 1;
        let location_idx = idx + 2;
        let axis_idx = idx + 3;
        let ref_direction_idx = idx + 4;
        let location = StepDisplay::new(transform[3].to_point(), location_idx);
        let axis_raw = VectorAsDirection(transform[2].truncate().normalize());
        let axis = StepDisplay::new(axis_raw, axis_idx);
        let r0 = transform[0].magnitude();
        let r1 = transform[1].magnitude();
        let focal_dist = FloatDisplay(r1 * r1 / r0);
        let ref_direction_raw = VectorAsDirection(transform[0].truncate() / r0);
        let ref_direction = StepDisplay::new(ref_direction_raw, ref_direction_idx);
        f.write_fmt(format_args!(
            "#{idx} = PARABOLA('', #{position_idx}, {focal_dist});
#{position_idx} = AXIS2_PLACEMENT_3D('', #{location_idx}, #{axis_idx}, #{ref_direction_idx});
{location}{axis}{ref_direction}"
        ))
    }
}
impl_const_step_length!(Processor<TrimmedCurve<UnitParabola<Point3>>, Matrix4>, 5);

impl<C, M: One> StepCurve for Processor<C, M> {
    #[inline(always)]
    fn same_sense(&self) -> bool { self.orientation() }
}

impl<C, S0, S1> DisplayByStep for IntersectionCurve<C, S0, S1>
where
    C: StepLength + DisplayByStep,
    S0: StepLength + DisplayByStep,
    S1: StepLength + DisplayByStep,
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let curve_idx = idx + 1;
        let surface0_idx = curve_idx + self.leader().step_length();
        let surface1_idx = surface0_idx + self.surface0().step_length();
        f.write_fmt(format_args!(
            "#{idx} = INTERSECTION_CURVE('', #{curve_idx}, (#{surface0_idx}, #{surface1_idx}), .CURVE_3D.);\n"
        ))?;
        self.leader().fmt(curve_idx, f)?;
        self.surface0().fmt(surface0_idx, f)?;
        self.surface1().fmt(surface0_idx, f)
    }
}

impl<C: StepLength, S0: StepLength, S1: StepLength> StepLength for IntersectionCurve<C, S0, S1> {
    #[inline(always)]
    fn step_length(&self) -> usize {
        1 + self.leader().step_length()
            + self.surface0().step_length()
            + self.surface1().step_length()
    }
}

impl<C, S0, S1> ConstStepLength for IntersectionCurve<C, S0, S1>
where
    C: ConstStepLength,
    S0: ConstStepLength,
    S1: ConstStepLength,
{
    const LENGTH: usize = 1 + C::LENGTH + S0::LENGTH + S1::LENGTH;
}

impl<C: StepCurve, S0, S1> StepCurve for IntersectionCurve<C, S0, S1> {
    #[inline(always)]
    fn same_sense(&self) -> bool { self.leader().same_sense() }
}

impl<C, S> DisplayByStep for PCurve<C, S>
where
    C: DisplayByStep,
    S: DisplayByStep + StepLength,
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let surface_idx = idx + 1;
        let repr_idx = surface_idx + self.surface().step_length();
        let context_idx = repr_idx + 1;
        let curve_idx = repr_idx + 2;
        let curve = StepDisplay::new(self.curve(), curve_idx);
        let surface = StepDisplay::new(self.surface(), surface_idx);
        f.write_fmt(format_args!(
            "#{idx} = PCURVE('', #{surface_idx}, #{repr_idx});
{surface}#{repr_idx} = DEFINITIONAL_REPRESENTATION('', (#{curve_idx}), #{context_idx});
#{context_idx} = (
    GEOMETRIC_REPRESENTATION_CONTEXT(2)
    PARAMETRIC_REPRESENTATION_CONTEXT()
    REPRESENTATION_CONTEXT('2D SPACE', '')
);
{curve}"
        ))
    }
}

impl<C: StepLength, S: StepLength> StepLength for PCurve<C, S> {
    fn step_length(&self) -> usize { 3 + self.curve().step_length() + self.surface().step_length() }
}

impl<C, S> ConstStepLength for PCurve<C, S>
where
    C: ConstStepLength,
    S: ConstStepLength,
{
    const LENGTH: usize = 3 + C::LENGTH + S::LENGTH;
}

impl<C: StepCurve, S> StepCurve for PCurve<C, S> {
    #[inline(always)]
    fn same_sense(&self) -> bool { self.curve().same_sense() }
}

impl DisplayByStep for ModelingCurve {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        match self {
            ModelingCurve::Line(x) => DisplayByStep::fmt(x, idx, f),
            ModelingCurve::BSplineCurve(x) => DisplayByStep::fmt(x, idx, f),
            ModelingCurve::NurbsCurve(x) => DisplayByStep::fmt(x, idx, f),
            ModelingCurve::IntersectionCurve(x) => DisplayByStep::fmt(x, idx, f),
        }
    }
}

impl StepLength for ModelingCurve {
    fn step_length(&self) -> usize {
        match self {
            ModelingCurve::Line(_) => Line::<Point3>::LENGTH,
            ModelingCurve::BSplineCurve(x) => x.step_length(),
            ModelingCurve::NurbsCurve(x) => x.step_length(),
            ModelingCurve::IntersectionCurve(x) => x.step_length(),
        }
    }
}

impl StepCurve for ModelingCurve {}

impl DisplayByStep for Plane {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let axis2_placement_idx = idx + 1;
        let location_idx = idx + 2;
        let z_axis_idx = idx + 3;
        let x_axis_idx = idx + 4;
        f.write_fmt(format_args!(
            "#{idx} = PLANE('', #{axis2_placement_idx});
#{axis2_placement_idx} = AXIS2_PLACEMENT_3D('', #{location_idx}, #{z_axis_idx}, #{x_axis_idx});
{location}{z_axis}{x_axis}",
            location = StepDisplay::new(self.origin(), location_idx),
            z_axis = StepDisplay::new(VectorAsDirection(self.normal()), z_axis_idx),
            x_axis = StepDisplay::new(VectorAsDirection(self.u_axis().normalize()), x_axis_idx)
        ))
    }
}
impl_const_step_length!(Plane, 5);

impl StepSurface for Plane {}

impl DisplayByStep for Processor<Sphere, Matrix4> {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let sphere = *self.entity();
        let transform = self.transform();
        let position_idx = idx + 1;
        let location_idx = idx + 2;
        let axis_idx = idx + 3;
        let ref_direction_idx = idx + 4;
        let location = transform[3].to_point() + sphere.center().to_vec();
        let axis = VectorAsDirection(transform[2].truncate().normalize());
        let r0 = transform[0].magnitude();
        let r1 = transform[1].magnitude();
        if !r0.near(&r1) {
            f.write_str("The transform of sphere includes non-uniform scale.")?;
            return ERR;
        }
        let ref_direction = VectorAsDirection(transform[0].truncate() / r0);
        let r = FloatDisplay(r0 * sphere.radius());
        f.write_fmt(format_args!(
            "#{idx} = SPHERICAL_SURFACE('', #{position_idx}, {r});
#{position_idx} = AXIS2_PLACEMENT_3D('', #{location_idx}, #{axis_idx}, #{ref_direction_idx});\n"
        ))?;
        DisplayByStep::fmt(&location, location_idx, f)?;
        DisplayByStep::fmt(&axis, axis_idx, f)?;
        DisplayByStep::fmt(&ref_direction, ref_direction_idx, f)
    }
}
impl_const_step_length!(Processor<Sphere, Matrix4>, 5);

impl DisplayByStep for Sphere {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        DisplayByStep::fmt(&Processor::new(*self), idx, f)
    }
}
impl_const_step_length!(Sphere, 5);
impl StepSurface for Sphere {}

impl DisplayByStep for Processor<Torus, Matrix4> {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let torus = *self.entity();
        let transform = self.transform();
        let position_idx = idx + 1;
        let location_idx = idx + 2;
        let axis_idx = idx + 3;
        let ref_direction_idx = idx + 4;
        let location = transform[3].to_point() + torus.center().to_vec();
        let axis = VectorAsDirection(transform[2].truncate().normalize());
        let r0 = transform[0].magnitude();
        let r1 = transform[1].magnitude();
        if !r0.near(&r1) {
            f.write_str("The transform of sphere includes non-uniform scale.")?;
            return ERR;
        }
        let ref_direction = VectorAsDirection(transform[0].truncate() / r0);
        let greater = FloatDisplay(r0 * torus.large_radius());
        let lesser = FloatDisplay(r0 * torus.small_radius());
        f.write_fmt(format_args!(
            "#{idx} = TOROIDAL_SURFACE('', #{position_idx}, {greater}, {lesser});
#{position_idx} = AXIS2_PLACEMENT_3D('', #{location_idx}, #{axis_idx}, #{ref_direction_idx});\n",
        ))?;
        DisplayByStep::fmt(&location, location_idx, f)?;
        DisplayByStep::fmt(&axis, axis_idx, f)?;
        DisplayByStep::fmt(&ref_direction, ref_direction_idx, f)
    }
}
impl_const_step_length!(Processor<Torus, Matrix4>, 5);

impl StepSurface for Processor<Torus, Matrix4> {
    #[inline(always)]
    fn same_sense(&self) -> bool { self.orientation() }
}

impl DisplayByStep for Torus {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        DisplayByStep::fmt(&Processor::new(*self), idx, f)
    }
}
impl_const_step_length!(Torus, 5);
impl StepSurface for Torus {}

impl<P> DisplayByStep for BSplineSurface<P>
where P: Copy + DisplayByStep
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let control_points = self.control_points();
        let control_points_instances = self
            .control_points()
            .iter()
            .flatten()
            .enumerate()
            .map(|(i, p)| StepDisplay::new(*p, idx + i + 1))
            .collect::<Vec<_>>();
        let mut counter = 0;
        let control_points_list = control_points
            .iter()
            .map(|slice| {
                counter += slice.len();
                IndexSliceDisplay(idx + counter - slice.len() + 1..=idx + counter)
            })
            .collect::<Vec<_>>();
        let (uknots, umulti) = self.uknot_vec().to_single_multi();
        let (vknots, vmulti) = self.vknot_vec().to_single_multi();
        f.write_fmt(format_args!(
            "#{idx} = B_SPLINE_SURFACE_WITH_KNOTS('', {u_degree}, {v_degree}, {control_points_list}, .UNSPECIFIED., .U., .U., .U., \
{u_multiplicities}, {v_multiplicities}, {u_knots}, {v_knots}, .UNSPECIFIED.);\n{control_points_instances}",
            u_degree = self.udegree(),
            v_degree = self.vdegree(),
            control_points_list = SliceDisplay(&control_points_list),
            u_multiplicities = SliceDisplay(&umulti),
            v_multiplicities = SliceDisplay(&vmulti),
            u_knots = SliceDisplay(&uknots),
            v_knots = SliceDisplay(&vknots),
            control_points_instances = SliceDisplay(&control_points_instances),
        ))
    }
}

impl<P> StepLength for BSplineSurface<P> {
    #[inline(always)]
    fn step_length(&self) -> usize { 1 + self.control_points().iter().map(Vec::len).sum::<usize>() }
}
impl<P> StepSurface for BSplineSurface<P> {}

impl<V> DisplayByStep for NurbsSurface<V>
where
    V: Homogeneous<Scalar = f64>,
    V::Point: Copy + DisplayByStep,
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let control_points_instances = self
            .control_points()
            .iter()
            .flatten()
            .enumerate()
            .map(|(i, v)| StepDisplay::new(v.to_point(), idx + i + 1))
            .collect::<Vec<_>>();
        let mut counter = 0;
        let control_points_list = self
            .control_points()
            .iter()
            .map(|slice| {
                counter += slice.len();
                IndexSliceDisplay(idx + counter - slice.len() + 1..=idx + counter)
            })
            .collect::<Vec<_>>();
        let weights = self
            .control_points()
            .iter()
            .map(|slice| slice.iter().map(|v| v.weight()).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let weights = weights
            .iter()
            .map(|slice| SliceDisplay(slice))
            .collect::<Vec<_>>();
        let (uknots, umulti) = self.uknot_vec().to_single_multi();
        let (vknots, vmulti) = self.vknot_vec().to_single_multi();
        f.write_fmt(format_args!(
            "#{idx} = (
    BOUNDED_SURFACE()
    B_SPLINE_SURFACE({u_degree}, {v_degree}, {control_points_list}, .UNSPECIFIED., .U., .U., .U.)
    B_SPLINE_SURFACE_WITH_KNOTS({u_multiplicities}, {v_multiplicities}, {u_knots}, {v_knots}, .UNSPECIFIED.)
    GEOMETRIC_REPRESENTATION_ITEM()
    RATIONAL_B_SPLINE_SURFACE({weights})
    REPRESENTATION_ITEM('')
    SURFACE()
);\n{control_points_instances}",
            u_degree = self.udegree(),
            v_degree = self.vdegree(),
            control_points_list = SliceDisplay(&control_points_list),
            u_multiplicities = SliceDisplay(&umulti),
            v_multiplicities = SliceDisplay(&vmulti),
            u_knots = SliceDisplay(&uknots),
            v_knots = SliceDisplay(&vknots),
            control_points_instances = SliceDisplay(&control_points_instances),
            weights = SliceDisplay(&weights),
        ))
    }
}

impl<V> StepLength for NurbsSurface<V> {
    #[inline(always)]
    fn step_length(&self) -> usize { 1 + self.control_points().iter().map(Vec::len).sum::<usize>() }
}
impl<V> StepSurface for NurbsSurface<V> {}

impl<C> DisplayByStep for ExtrudedCurve<C, Vector3>
where C: StepLength + DisplayByStep
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let curve = self.entity_curve();
        let curve_idx = idx + 1;
        let vector_idx = idx + 1 + curve.step_length();
        let vector = self.extruding_vector();
        f.write_fmt(format_args!(
            "#{idx} = SURFACE_OF_LINEAR_EXTRUSION('', #{curve_idx}, #{vector_idx});\n{}{}",
            StepDisplay::new(curve, curve_idx),
            StepDisplay::new(vector, vector_idx),
        ))
    }
}
impl<C: StepLength> StepLength for ExtrudedCurve<C, Vector3> {
    fn step_length(&self) -> usize { 1 + self.entity_curve().step_length() + Vector3::LENGTH }
}
impl<C: ConstStepLength> ConstStepLength for ExtrudedCurve<C, Vector3> {
    const LENGTH: usize = 1 + C::LENGTH + Vector3::LENGTH;
}
impl<C> StepSurface for ExtrudedCurve<C, Vector3> {}

impl<C, T: One> StepSurface for Processor<ExtrudedCurve<C, Vector3>, T> {
    #[inline(always)]
    fn same_sense(&self) -> bool { self.orientation() }
}

impl<C> DisplayByStep for RevolutedCurve<C>
where C: StepLength + DisplayByStep
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let curve = self.entity_curve();
        let curve_idx = idx + 1;
        let axis_idx = curve_idx + curve.step_length();
        let location_idx = axis_idx + 1;
        let dir_idx = location_idx + 1;
        f.write_fmt(format_args!(
            "#{idx} = SURFACE_OF_REVOLUTION('', #{curve_idx}, #{axis_idx});
{curve}#{axis_idx} = AXIS1_PLACEMENT('', #{location_idx}, #{dir_idx});\n{location}{dir}",
            curve = StepDisplay::new(curve, curve_idx),
            location = StepDisplay::new(self.origin(), location_idx),
            dir = StepDisplay::new(VectorAsDirection(self.axis()), dir_idx),
        ))
    }
}

impl<C: StepLength> StepLength for RevolutedCurve<C> {
    #[inline(always)]
    fn step_length(&self) -> usize { 4 + self.entity_curve().step_length() }
}

impl<C: ConstStepLength> ConstStepLength for RevolutedCurve<C> {
    const LENGTH: usize = 4 + C::LENGTH;
}

impl<C> StepSurface for RevolutedCurve<C> {
    #[inline(always)]
    fn same_sense(&self) -> bool { false }
}

impl<C> DisplayByStep for Processor<RevolutedCurve<C>, Matrix4>
where C: StepLength + Transformed<Matrix4> + DisplayByStep
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let surface = self.entity();
        let transform = self.transform();
        let (k, a, _) = match transform.iwasawa_decomposition() {
            Some(x) => x,
            None => {
                f.write_str("Transform is not regular")?;
                return ERR;
            }
        };
        if !a[0][0].near(&a[1][1]) || !a[1][1].near(&a[2][2]) {
            f.write_str("Transform contains non-uniform scale.")?;
            return ERR;
        }
        let curve = surface.entity_curve().transformed(*transform);
        let axis = k.transform_vector(surface.axis());
        let origin = transform.transform_point(surface.origin());
        let surface = RevolutedCurve::by_revolution(curve, origin, axis);
        DisplayByStep::fmt(&surface, idx, f)
    }
}
impl<C: StepLength> StepLength for Processor<RevolutedCurve<C>, Matrix4> {
    fn step_length(&self) -> usize { self.entity().step_length() }
}

impl<C, T: One> StepSurface for Processor<RevolutedCurve<C>, T> {
    #[inline(always)]
    fn same_sense(&self) -> bool { !self.orientation() }
}

impl DisplayByStep for ModelingSurface {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        match self {
            ModelingSurface::Plane(x) => DisplayByStep::fmt(x, idx, f),
            ModelingSurface::BSplineSurface(x) => DisplayByStep::fmt(x, idx, f),
            ModelingSurface::NurbsSurface(x) => DisplayByStep::fmt(x, idx, f),
            ModelingSurface::RevolutedCurve(x) => DisplayByStep::fmt(x, idx, f),
            ModelingSurface::TSplineSurface(tmesh) => {
                let bsp = tmesh.to_bspline_surface(4);
                DisplayByStep::fmt(&bsp, idx, f)
            }
        }
    }
}

impl StepLength for ModelingSurface {
    fn step_length(&self) -> usize {
        match self {
            ModelingSurface::Plane(_) => Plane::LENGTH,
            ModelingSurface::BSplineSurface(x) => x.step_length(),
            ModelingSurface::NurbsSurface(x) => x.step_length(),
            ModelingSurface::RevolutedCurve(x) => x.entity().step_length(),
            ModelingSurface::TSplineSurface(tmesh) => {
                let bsp = tmesh.to_bspline_surface(4);
                bsp.step_length()
            }
        }
    }
}

impl StepSurface for ModelingSurface {}
