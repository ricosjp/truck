use super::{Result, *};
use truck_geometry::prelude::*;
use truck_modeling::{Curve as ModelingCurve, Leader, Surface as ModelingSurface};
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
            "#{idx} = VECTOR('', #{direction_idx}, {magnitude:?});\n{direction}",
            direction = StepDisplay::new(VectorAsDirection(*self / magnitude), direction_idx),
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

impl<P> DisplayByStep for PolylineCurve<P>
where P: Copy + ConstStepLength + DisplayByStep
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        f.write_fmt(format_args!(
            "#{idx} = POLYLINE('', {range});\n",
            range = IndexSliceDisplay(idx + 1..=idx + self.0.len())
        ))?;
        self.0
            .iter()
            .enumerate()
            .try_for_each(|(i, p)| Display::fmt(&StepDisplay::new(*p, idx + 1 + i * P::LENGTH), f))
    }
}

impl<P: ConstStepLength> StepLength for PolylineCurve<P> {
    fn step_length(&self) -> usize { 1 + self.0.len() * P::LENGTH }
}

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
    fn step_length(&self) -> usize { self.control_points().len() + 1 }
}

impl<V> DisplayByStep for NurbsCurve<V>
where
    V: Homogeneous<f64>,
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

impl<P> StepLength for NurbsCurve<P> {
    fn step_length(&self) -> usize { self.control_points().len() + 1 }
}

impl<C, S> DisplayByStep for IntersectionCurve<C, S>
where
    C: StepLength + DisplayByStep,
    S: StepLength + DisplayByStep,
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let curve_idx = idx + 1;
        let surface0_idx = curve_idx + self.leader().step_length();
        let surface1_idx = surface0_idx + self.surface0().step_length();
        f.write_fmt(format_args!(
            "#{idx} = INTERSECTION_CURVE('', #{curve_idx}, (#{surface0_idx}, #{surface1_idx}), .CURVE_3D.);\n"
        ))?;
        Display::fmt(&StepDisplay::new(self.leader(), curve_idx), f)?;
        Display::fmt(&StepDisplay::new(self.surface0(), surface0_idx), f)?;
        Display::fmt(&StepDisplay::new(self.surface1(), surface1_idx), f)
    }
}

impl<C, S> StepLength for IntersectionCurve<C, S>
where
    C: StepLength,
    S: StepLength,
{
    fn step_length(&self) -> usize {
        1 + self.leader().step_length()
            + self.surface0().step_length()
            + self.surface1().step_length()
    }
}

impl DisplayByStep for Leader {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        match self {
            Leader::Polyline(x) => Display::fmt(&StepDisplay::new(x, idx), f),
            Leader::BSpline(x) => Display::fmt(&StepDisplay::new(x, idx), f),
        }
    }
}

impl StepLength for Leader {
    fn step_length(&self) -> usize {
        match self {
            Leader::Polyline(x) => x.step_length(),
            Leader::BSpline(x) => x.step_length(),
        }
    }
}

impl DisplayByStep for ModelingCurve {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        match self {
            ModelingCurve::Line(x) => Display::fmt(&StepDisplay::new(x, idx), f),
            ModelingCurve::BSplineCurve(x) => Display::fmt(&StepDisplay::new(x, idx), f),
            ModelingCurve::NurbsCurve(x) => Display::fmt(&StepDisplay::new(x, idx), f),
            ModelingCurve::IntersectionCurve(x) => Display::fmt(&StepDisplay::new(x, idx), f),
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
    fn step_length(&self) -> usize { 1 + self.control_points().iter().map(Vec::len).sum::<usize>() }
}

impl<V> DisplayByStep for NurbsSurface<V>
where
    V: Homogeneous<f64>,
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
    fn step_length(&self) -> usize { 1 + self.control_points().iter().map(Vec::len).sum::<usize>() }
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
    fn step_length(&self) -> usize { 4 + self.entity_curve().step_length() }
}

impl<C> DisplayByStep for Processor<RevolutedCurve<C>, Matrix4>
where C: StepLength + Transformed<Matrix4> + DisplayByStep
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let surface = self.entity();
        let transform = self.transform();
        let (k, a, _) = transform
            .iwasawa_decomposition()
            .expect("Transform is not regular.");
        assert_near!(a[0][0], a[1][1], "Transform contains non-uniform scale.");
        assert_near!(a[1][1], a[2][2], "Transform contains non-uniform scale.");
        let curve = surface.entity_curve().transformed(*transform);
        let axis = k.transform_vector(surface.axis());
        let origin = transform.transform_point(surface.origin());
        let surface = RevolutedCurve::by_revolution(curve, origin, axis);
        Display::fmt(&StepDisplay::new(surface, idx), f)
    }
}

impl DisplayByStep for ModelingSurface {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        match self {
            ModelingSurface::Plane(x) => Display::fmt(&StepDisplay::new(x, idx), f),
            ModelingSurface::BSplineSurface(x) => Display::fmt(&StepDisplay::new(x, idx), f),
            ModelingSurface::NurbsSurface(x) => Display::fmt(&StepDisplay::new(x, idx), f),
            ModelingSurface::RevolutedCurve(x) => Display::fmt(&StepDisplay::new(x, idx), f),
        }
    }
}

impl StepLength for ModelingSurface {
    fn step_length(&self) -> usize {
        match self {
            ModelingSurface::Plane(x) => x.step_length(),
            ModelingSurface::BSplineSurface(x) => x.step_length(),
            ModelingSurface::NurbsSurface(x) => x.step_length(),
            ModelingSurface::RevolutedCurve(x) => x.entity().step_length(),
        }
    }
}
