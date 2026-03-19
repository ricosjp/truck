use super::{Result, *};
use crate::common::PartAttrs;
use truck_assembly::assy::*;

const GLOBAL_APPLICATION_CONTEXT_INDEX: usize = 2;
const COMMON_REPRESENTATION_CONTEXT_INDEX: usize = 3;

#[derive(Clone, Copy, Debug)]
struct TruckRepresentationContext;

impl DisplayByStep for TruckRepresentationContext {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let context_idx = idx;
        let length_unit_idx = idx + 1;
        let plane_angle_unit_idx = idx + 2;
        let solid_angle_unit_idx = idx + 3;
        let tolerance_idx = idx + 4;
        f.write_fmt(format_args!(
"#{context_idx} = (
    GEOMETRIC_REPRESENTATION_CONTEXT(3)
    GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#{tolerance_idx}))
    GLOBAL_UNIT_ASSIGNED_CONTEXT((#{length_unit_idx}, #{plane_angle_unit_idx}, #{solid_angle_unit_idx}))
    REPRESENTATION_CONTEXT('Context #1', '3D Context with UNIT and UNCERTAINTY')
);
#{length_unit_idx} = ( LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.));
#{plane_angle_unit_idx} = ( NAMED_UNIT(*) PLANE_ANGLE_UNIT() SI_UNIT($,.RADIAN.) );
#{solid_angle_unit_idx} = ( NAMED_UNIT(*) SI_UNIT($,.STERADIAN.) SOLID_ANGLE_UNIT() );
#{tolerance_idx} = UNCERTAINTY_MEASURE_WITH_UNIT(LENGTH_MEASURE(1.0E-6), #{length_unit_idx}, 'distance_accuracy_value','confusion accuracy');"
        ))
    }
}
impl_const_step_length!(TruckRepresentationContext, 5);

impl<Shape> DisplayByStep for NodeEntity<Shape, PartAttrs>
where Shape: DisplayByStep
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let Self {
            shape,
            attrs:
                PartAttrs {
                    id,
                    name,
                    description,
                },
        } = self;
        let sdr_idx = idx;
        let pds_idx = idx + 1;
        let pd_idx = idx + 2;
        let pdf_idx = idx + 3;
        let p_idx = idx + 4;
        let pdc_idx = idx + 5;
        let pc_idx = idx + 6;
        let sr_idx = idx + 7;
        let shape_idx = idx + 8;
        f.write_fmt(format_args!(
"#{sdr_idx} = SHAPE_DEFINITION_REPRESENTATION(#{pds_idx}, #{sr_idx});
#{pds_idx} = PRODUCT_DEFINITION_SHAPE('', '', #{pd_idx});
#{pd_idx} = PRODUCT_DEFINITION('design', '', #{pdf_idx}, #{pdc_idx});
#{pdf_idx} = PRODUCT_DEFINITION_FORMATION('', '', #{p_idx});
#{p_idx} = PRODUCT('{id}', '{name}', '{description}', (#{pc_idx}));
#{pdc_idx} = DESIGN_CONTEXT('', #{GLOBAL_APPLICATION_CONTEXT_INDEX}, 'design');
#{pc_idx} = MECHANICAL_CONTEXT('', #{GLOBAL_APPLICATION_CONTEXT_INDEX}, 'mechanical');
#{sr_idx} = SHAPE_REPRESENTATION('', (#{shape_idx}), #{COMMON_REPRESENTATION_CONTEXT_INDEX});
"
        ))?;
        DisplayByStep::fmt(shape, shape_idx, f)
    }
}
