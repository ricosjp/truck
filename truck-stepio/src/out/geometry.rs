use super::{Result, *};
use truck_geometry::*;
use truck_modeling::{Curve as ModelingCurve, Surface as ModelingSurface};

impl Display for StepDisplay<Point2> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write_fmt(format_args!(
            "#{idx} = CARTESIAN_POINT('', {coordinates});\n",
            idx = self.idx,
            coordinates = SliceDisplay(AsRef::<[f64; 2]>::as_ref(&self.entity)),
        ))
    }
}
impl_step_length!(Point2, 1);

impl<'a> Display for StepDisplay<Point3> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write_fmt(format_args!(
            "#{idx} = CARTESIAN_POINT('', {coordinates});\n",
            idx = self.idx,
            coordinates = SliceDisplay(AsRef::<[f64; 3]>::as_ref(&self.entity)),
        ))
    }
}
impl_step_length!(Point3, 1);

/// class for display `DIRECTION`.
#[derive(Clone, Debug, Copy)]
pub struct VectorAsDirection<V>(V);

impl Display for StepDisplay<VectorAsDirection<Vector2>> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write_fmt(format_args!(
            "#{idx} = DIRECTION('', {direction_ratios});\n",
            idx = self.idx,
            direction_ratios = SliceDisplay(AsRef::<[f64; 2]>::as_ref(&self.entity.0)),
        ))
    }
}

impl Display for StepDisplay<VectorAsDirection<Vector3>> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write_fmt(format_args!(
            "#{idx} = DIRECTION('', {direction_ratios});\n",
            idx = self.idx,
            direction_ratios = SliceDisplay(AsRef::<[f64; 3]>::as_ref(&self.entity.0)),
        ))
    }
}

impl<V> StepLength for VectorAsDirection<V> {
    #[inline]
    fn step_length(&self) -> usize { 1 }
}

impl<V> Display for StepDisplay<V>
where
    V: InnerSpace<Scalar = f64>,
    StepDisplay<VectorAsDirection<V>>: Display,
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        let magnitude = self.entity.magnitude();
        let direction_idx = self.idx + 1;
        f.write_fmt(format_args!(
            "#{idx} = VECTOR('', #{direction_idx}, {magnitude:?});\n{direction}",
            idx = self.idx,
            direction = StepDisplay::new(VectorAsDirection(self.entity / magnitude), direction_idx),
        ))
    }
}
impl_step_length!(Vector2, 2);
impl_step_length!(Vector3, 2);

impl<'a, P> Display for StepDisplay<&'a BSplineCurve<P>>
where
    P: Copy,
    StepDisplay<P>: Display,
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        let curve = self.entity;
        let idx = self.idx;
        let (knots, multi) = curve.knot_vec().to_single_multi();
        let control_points_instances = curve
            .control_points()
            .iter()
            .enumerate()
            .map(|(i, p)| StepDisplay::new(*p, idx + i + 1))
            .collect::<Vec<_>>();
        f.write_fmt(format_args!(
			"#{idx} = B_SPLINE_CURVE_WITH_KNOTS('', {degree}, {control_points_list}, .UNSPECIFIED., .U., .U., {knot_multiplicities}, {knots}, .UNSPECIFIED.);\n{control_points_instances}",
            degree = curve.degree(),
            control_points_list = IndexSliceDisplay(self.idx + 1..=self.idx + curve.control_points().len()),
			knot_multiplicities = SliceDisplay(&multi),
            knots = SliceDisplay(&knots),
            control_points_instances = SliceDisplay(&control_points_instances),
		))
    }
}

impl<P> Display for StepDisplay<BSplineCurve<P>>
where
    P: Copy,
    StepDisplay<P>: Display,
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        Display::fmt(&StepDisplay::new(&self.entity, self.idx), f)
    }
}

impl<P> StepLength for BSplineCurve<P> {
    fn step_length(&self) -> usize { self.control_points().len() + 1 }
}

impl<'a, V> Display for StepDisplay<&'a NURBSCurve<V>>
where
    V: Homogeneous<f64>,
    V::Point: Copy,
    StepDisplay<V::Point>: Display,
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        let curve = self.entity;
        let idx = self.idx;
        let (knots, multi) = curve.knot_vec().to_single_multi();
        let control_points_instances = curve
            .control_points()
            .iter()
            .enumerate()
            .map(|(i, v)| StepDisplay::new(v.to_point(), idx + i + 1))
            .collect::<Vec<_>>();
        let weights = curve
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
            degree = curve.degree(),
            control_points_list =
                IndexSliceDisplay(self.idx + 1..=self.idx + curve.control_points().len()),
            knot_multiplicities = SliceDisplay(&multi),
            knots = SliceDisplay(&knots),
            weights = SliceDisplay(&weights),
            control_points_instances = SliceDisplay(&control_points_instances),
        ))
    }
}

impl<V> Display for StepDisplay<NURBSCurve<V>>
where
    V: Homogeneous<f64>,
    V::Point: Copy,
    StepDisplay<V::Point>: Display,
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        Display::fmt(&StepDisplay::new(&self.entity, self.idx), f)
    }
}

impl<'a> Display for StepDisplay<&'a ModelingCurve> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self.entity {
            ModelingCurve::BSplineCurve(x) => Display::fmt(&StepDisplay::new(x, self.idx), f),
            ModelingCurve::NURBSCurve(x) => Display::fmt(&StepDisplay::new(x, self.idx), f),
            ModelingCurve::IntersectionCurve(_) => unimplemented!(),
        }
    }
}

impl Display for StepDisplay<ModelingCurve> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        Display::fmt(&StepDisplay::new(&self.entity, self.idx), f)
    }
}

impl StepLength for ModelingCurve {
    fn step_length(&self) -> usize {
        match self {
            ModelingCurve::BSplineCurve(x) => x.step_length(),
            ModelingCurve::NURBSCurve(x) => x.step_length(),
            ModelingCurve::IntersectionCurve(_) => unimplemented!(),
        }
    }
}

impl<P> StepLength for NURBSCurve<P> {
    fn step_length(&self) -> usize { self.control_points().len() + 1 }
}

impl Display for StepDisplay<Plane> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let idx = self.idx;
        let axis2_placement_idx = idx + 1;
        let location_idx = idx + 2;
        let z_axis_idx = idx + 3;
        let x_axis_idx = idx + 4;
        f.write_fmt(format_args!(
            "#{idx} = PLANE('', #{axis2_placement_idx});
#{axis2_placement_idx} = AXIS2_PLACEMENT_3D('', #{location_idx}, #{z_axis_idx}, #{x_axis_idx});
{location}{z_axis}{x_axis}",
            location = StepDisplay::new(self.entity.origin(), location_idx),
            z_axis = StepDisplay::new(VectorAsDirection(self.entity.normal()), z_axis_idx),
            x_axis = StepDisplay::new(
                VectorAsDirection(self.entity.u_axis().normalize()),
                x_axis_idx
            )
        ))
    }
}

impl StepLength for Plane {
    fn step_length(&self) -> usize { 5 }
}

impl<'a, P> Display for StepDisplay<&'a BSplineSurface<P>>
where
    P: Copy,
    StepDisplay<P>: Display,
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        let StepDisplay {
            entity: surface,
            idx,
        } = self;
        let control_points = surface.control_points();
        let control_points_instances = surface
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
        let (uknots, umulti) = surface.uknot_vec().to_single_multi();
        let (vknots, vmulti) = surface.vknot_vec().to_single_multi();
        f.write_fmt(format_args!(
            "#{idx} = B_SPLINE_SURFACE_WITH_KNOTS('', {u_degree}, {v_degree}, {control_points_list}, .UNSPECIFIED., .U., .U., .U., \
{u_multiplicities}, {v_multiplicities}, {u_knots}, {v_knots}, .UNSPECIFIED.);\n{control_points_instances}",
            u_degree = surface.udegree(),
            v_degree = surface.vdegree(),
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

impl<'a, V> Display for StepDisplay<&'a NURBSSurface<V>>
where
    V: Homogeneous<f64>,
    V::Point: Copy,
    StepDisplay<V::Point>: Display,
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        let StepDisplay {
            entity: surface,
            idx,
        } = self;
        let control_points_instances = surface
            .control_points()
            .iter()
            .flatten()
            .enumerate()
            .map(|(i, v)| StepDisplay::new(v.to_point(), idx + i + 1))
            .collect::<Vec<_>>();
        let mut counter = 0;
        let control_points_list = surface
            .control_points()
            .iter()
            .map(|slice| {
                counter += slice.len();
                IndexSliceDisplay(idx + counter - slice.len() + 1..=idx + counter)
            })
            .collect::<Vec<_>>();
        let weights = surface
            .control_points()
            .iter()
            .map(|slice| slice.iter().map(|v| v.weight()).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let weights = weights
            .iter()
            .map(|slice| SliceDisplay(slice))
            .collect::<Vec<_>>();
        let (uknots, umulti) = surface.uknot_vec().to_single_multi();
        let (vknots, vmulti) = surface.vknot_vec().to_single_multi();
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
            u_degree = surface.udegree(),
            v_degree = surface.vdegree(),
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

impl<V> StepLength for NURBSSurface<V> {
    fn step_length(&self) -> usize { 1 + self.control_points().iter().map(Vec::len).sum::<usize>() }
}

impl<'a, C> Display for StepDisplay<&'a RevolutedCurve<C>>
where
    C: StepLength,
    StepDisplay<&'a C>: Display,
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        let StepDisplay {
            entity: surface,
            idx,
        } = self;
        let curve = surface.entity_curve();
        let curve_idx = idx + 1;
        let axis_idx = curve_idx + curve.step_length();
        let location_idx = axis_idx + 1;
        let dir_idx = location_idx + 1;
        f.write_fmt(format_args!(
            "#{idx} = SURFACE_OF_REVOLUTION('', #{curve_idx}, #{axis_idx});
{curve}#{axis_idx} = AXIS1_PLACEMENT('', #{location_idx}, #{dir_idx});\n{location}{dir}",
            curve = StepDisplay::new(curve, curve_idx),
            location = StepDisplay::new(surface.origin(), location_idx),
            dir = StepDisplay::new(VectorAsDirection(surface.axis()), dir_idx),
        ))
    }
}

impl<C> Display for StepDisplay<RevolutedCurve<C>>
where
    C: StepLength + Clone,
    StepDisplay<C>: Display,
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        let StepDisplay {
            entity: surface,
            idx,
        } = self;
        let curve = surface.entity_curve();
        let curve_idx = idx + 1;
        let axis_idx = curve_idx + curve.step_length();
        let location_idx = axis_idx + 1;
        let dir_idx = location_idx + 1;
        f.write_fmt(format_args!(
            "#{idx} = SURFACE_OF_REVOLUTION('', #{curve_idx}, #{axis_idx});
{curve}#{axis_idx} = AXIS1_PLACEMENT('', #{location_idx}, #{dir_idx});\n{location}{dir}",
            curve = StepDisplay::new(curve.clone(), curve_idx),
            location = StepDisplay::new(surface.origin(), location_idx),
            dir = StepDisplay::new(VectorAsDirection(surface.axis()), dir_idx),
        ))
    }
}

impl<C: StepLength> StepLength for RevolutedCurve<C> {
    fn step_length(&self) -> usize { 4 + self.entity_curve().step_length() }
}

impl<'a, C> Display for StepDisplay<&'a Processor<RevolutedCurve<C>, Matrix4>>
where
    C: StepLength + Transformed<Matrix4>,
    StepDisplay<C>: Display,
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        let StepDisplay { entity, idx } = self;
        let surface = entity.entity();
        let transform = entity.transform();
        let (k, a, _) = transform
            .iwasawa_decomposition()
            .expect("Transform is not regular.");
        assert_near!(a[0][0], a[1][1], "Transform contains non-uniform scale.");
        assert_near!(a[1][1], a[2][2], "Transform contains non-uniform scale.");
        let curve = surface.entity_curve().transformed(*transform);
        let axis = k.transform_vector(surface.axis());
        let origin = transform.transform_point(surface.origin());
        let surface = RevolutedCurve::by_revolution(curve, origin, axis);
        Display::fmt(&StepDisplay::new(surface, *idx), f)
    }
}

impl<'a> Display for StepDisplay<&'a ModelingSurface> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self.entity {
            ModelingSurface::Plane(x) => Display::fmt(&StepDisplay::new(*x, self.idx), f),
            ModelingSurface::BSplineSurface(x) => Display::fmt(&StepDisplay::new(x, self.idx), f),
            ModelingSurface::NURBSSurface(x) => Display::fmt(&StepDisplay::new(x, self.idx), f),
            ModelingSurface::RevolutedCurve(x) => Display::fmt(&StepDisplay::new(x, self.idx), f),
        }
    }
}

impl StepLength for ModelingSurface {
    fn step_length(&self) -> usize {
        match self {
            ModelingSurface::Plane(x) => x.step_length(),
            ModelingSurface::BSplineSurface(x) => x.step_length(),
            ModelingSurface::NURBSSurface(x) => x.step_length(),
            ModelingSurface::RevolutedCurve(x) => x.entity().step_length(),
        }
    }
}
