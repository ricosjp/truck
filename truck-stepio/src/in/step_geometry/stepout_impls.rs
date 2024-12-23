use crate::out::FloatDisplay;

use super::{*, truck_stepio::out};

impl out::ConstStepLength for Processor<Sphere, Matrix4> {
    const LENGTH: usize = Processor::<truck_geometry::prelude::Sphere, Matrix4>::LENGTH;
}
impl out::StepLength for Processor<Sphere, Matrix4> {
    fn step_length(&self) -> usize { <Self as out::ConstStepLength>::LENGTH }
}
impl out::DisplayByStep for Processor<Sphere, Matrix4> {
    fn fmt(&self, idx: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Processor::new(self.entity().0)
            .transformed(*self.transform())
            .fmt(idx, f)
    }
}

impl out::DisplayByStep for ElementarySurface {
    fn fmt(&self, idx: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plane(x) => x.fmt(idx, f),
            Self::Sphere(x) => x.fmt(idx, f),
            Self::ToroidalSurface(x) => x.fmt(idx, f),
            Self::CylindricalSurface(processor) => {
                let position_idx = idx + 1;
                let location_idx = idx + 2;
                let axis_idx = idx + 3;
                let ref_direction_idx = idx + 4;

                let revo = processor.entity();
                let o = revo.origin();
                let Line(p, _) = revo.entity_curve();
                let location = out::StepDisplay::new(o, location_idx);
                let raw_axis = out::VectorAsDirection(revo.axis());
                let axis = out::StepDisplay::new(raw_axis, axis_idx);
                let raw_ref_direction = out::VectorAsDirection((p - o).normalize());
                let ref_direction = out::StepDisplay::new(raw_ref_direction, ref_direction_idx);
                let radius = (p - o).magnitude();

                f.write_fmt(format_args!(
                    "#{idx} = CYLINDRICAL_SURFACE('', #{position_idx}, {radius});
#{position_idx} = AXIS2_PLACEMENT_3D('', #{location_idx}, #{axis_idx}, #{ref_direction_idx});
{location}{axis}{ref_direction}"
                ))
            }
            Self::ConicalSurface(processor) => {
                let revo = processor.entity();
                let transform = processor.transform();
                let line = revo.entity_curve();
                let p = line.0;
                let v = line.1 - p;

                let radius = FloatDisplay(p.x);
                let semi_angle = FloatDisplay(f64::atan(v.x));

                let position_idx = idx + 1;
                let location_idx = idx + 2;
                let axis_idx = idx + 3;
                let ref_direction_idx = idx + 4;

                let location = out::StepDisplay::new(transform[3].to_point(), location_idx);
                let raw_axis = out::VectorAsDirection(transform[2].truncate());
                let axis = out::StepDisplay::new(raw_axis, axis_idx);
                let raw_ref_direction = out::VectorAsDirection(transform[0].truncate());
                let ref_direction = out::StepDisplay::new(raw_ref_direction, ref_direction_idx);

                f.write_fmt(format_args!(
                    "#{idx} = CONICAL_SURFACE('', #{position_idx}, {radius}, {semi_angle});
#{position_idx} = AXIS2_PLACEMENT_3D('', #{location_idx}, #{axis_idx}, #{ref_direction_idx});
{location}{axis}{ref_direction}"
                ))
            }
        }
    }
}
