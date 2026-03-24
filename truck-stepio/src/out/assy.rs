use super::{Result, *};
use crate::common::PartAttrs;
use truck_assembly::assy::*;

const GLOBAL_APPLICATION_CONTEXT_INDEX: usize = 1;
const COMMON_REPRESENTATION_CONTEXT_INDEX: usize = 2;
const GLOBAL_IDENTITY_MATRIX: usize =
    COMMON_REPRESENTATION_CONTEXT_INDEX + TruckRepresentationContext::LENGTH;

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
#{tolerance_idx} = UNCERTAINTY_MEASURE_WITH_UNIT(1.0E-6, \
#{length_unit_idx}, 'distance_accuracy_value', 'confusion accuracy');\n"
        ))
    }
}
impl_const_step_length!(TruckRepresentationContext, 5);

#[derive(Clone, Debug)]
enum MatrixOrModel<Matrix, Model> {
    Matrix(Matrix),
    Model(Model),
}

impl<Matrix, Model> DisplayByStep for MatrixOrModel<Matrix, Model>
where
    Model: DisplayByStep + StepLength,
    Matrix: DisplayByStep + ConstStepLength,
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Matrix(mat) => DisplayByStep::fmt(mat, idx, f),
            Self::Model(model) => DisplayByStep::fmt(model, idx, f),
        }
    }
}

impl<'a, Model, Models, Matrix> DisplayByStep
    for Node<'a, NodeEntity<Models, PartAttrs>, EdgeEntity<Matrix, PartAttrs>>
where
    Model: DisplayByStep + StepLength,
    for<'b> &'b Models: IntoIterator<Item = &'b Model>,
    Matrix: Copy,
    MatrixAsAxis<Matrix>: DisplayByStep + ConstStepLength,
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let NodeEntity {
            shape,
            attrs:
                PartAttrs {
                    id,
                    name,
                    description,
                },
        } = self.entity();
        let sdr_idx = idx;
        let pds_idx = idx + 1;
        let pd_idx = idx + 2;
        let pdf_idx = idx + 3;
        let p_idx = idx + 4;
        let pdc_idx = idx + 5;
        let pc_idx = idx + 6;
        let sr_idx = idx + 7;

        let mut shape_indices = vec![GLOBAL_IDENTITY_MATRIX];
        let mut cursor = idx + 8;
        let mut displays = Vec::new();
        for edge in self.edges() {
            let mat = MatrixAsAxis(*edge.matrix());
            shape_indices.push(cursor);
            displays.push(StepDataDisplay::new(MatrixOrModel::Matrix(mat), cursor));
            cursor += MatrixAsAxis::LENGTH;
        }
        for shape in shape {
            shape_indices.push(cursor);
            displays.push(StepDataDisplay::new(MatrixOrModel::Model(shape), cursor));
            cursor += shape.step_length();
        }

        let shape_indices = IndexSliceDisplay(shape_indices);
        f.write_fmt(format_args!(
            "#{sdr_idx} = SHAPE_DEFINITION_REPRESENTATION(#{pds_idx}, #{sr_idx});
#{pds_idx} = PRODUCT_DEFINITION_SHAPE('', '', #{pd_idx});
#{pd_idx} = PRODUCT_DEFINITION('design', '', #{pdf_idx}, #{pdc_idx});
#{pdf_idx} = PRODUCT_DEFINITION_FORMATION('', '', #{p_idx});
#{p_idx} = PRODUCT('{id}', '{name}', '{description}', (#{pc_idx}));
#{pdc_idx} = DESIGN_CONTEXT('', #{GLOBAL_APPLICATION_CONTEXT_INDEX}, 'design');
#{pc_idx} = MECHANICAL_CONTEXT('', #{GLOBAL_APPLICATION_CONTEXT_INDEX}, 'mechanical');
#{sr_idx} = SHAPE_REPRESENTATION('', {shape_indices}, #{COMMON_REPRESENTATION_CONTEXT_INDEX});\n"
        ))?;

        for display in &displays {
            Display::fmt(display, f)?;
        }
        Ok(())
    }
}

impl<'a, Model, Models, Matrix> StepLength
    for Node<'a, NodeEntity<Models, PartAttrs>, EdgeEntity<Matrix, PartAttrs>>
where
    Model: StepLength,
    for<'b> &'b Models: IntoIterator<Item = &'b Model>,
    Matrix: Copy,
    MatrixAsAxis<Matrix>: ConstStepLength,
{
    fn step_length(&self) -> usize {
        8 + MatrixAsAxis::LENGTH * self.edges().len()
            + (&self.entity().shape)
                .into_iter()
                .map(|shape| shape.step_length())
                .sum::<usize>()
    }
}

#[derive(Clone, Copy, Debug)]
struct NodeInEdge {
    product_definition_idx: usize,
    shape_representation_idx: usize,
}

impl NodeInEdge {
    fn new(node_idx: usize) -> Self {
        NodeInEdge {
            product_definition_idx: node_idx + 2,
            shape_representation_idx: node_idx + 7,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct EdgeDisplay<'a> {
    matrix_idx: usize,
    attrs: &'a PartAttrs,
    nodes: (NodeInEdge, NodeInEdge),
}

impl<'a> DisplayByStep for EdgeDisplay<'a> {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let cdsr_idx = idx;
        let rr_idx = cdsr_idx + 1;
        let idt_idx = rr_idx + 1;
        let mat_idx = self.matrix_idx;
        let pds_idx = idt_idx + 1;
        let nauo_idx = pds_idx + 1;

        let (
            NodeInEdge {
                product_definition_idx: pd_idx0,
                shape_representation_idx: sr_idx0,
            },
            NodeInEdge {
                product_definition_idx: pd_idx1,
                shape_representation_idx: sr_idx1,
            },
        ) = self.nodes;

        let PartAttrs {
            name,
            id,
            description,
        } = &self.attrs;

        f.write_fmt(format_args!(
            "#{cdsr_idx} = CONTEXT_DEPENDENT_SHAPE_REPRESENTATION(#{rr_idx}, #{pds_idx});
#{rr_idx} = (
    REPRESENTATION_RELATIONSHIP('', '', #{sr_idx0}, #{sr_idx1})
    REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION(#{idt_idx})
    SHAPE_REPRESENTATION_RELATIONSHIP()
);
#{idt_idx} = ITEM_DEFINED_TRANSFORMATION('', '', #{GLOBAL_IDENTITY_MATRIX}, #{mat_idx});
#{pds_idx} = PRODUCT_DEFINITION_SHAPE('', '', #{nauo_idx});
#{nauo_idx} = NEXT_ASSEMBLY_USAGE_OCCURRENCE('{id}', '{name}', '{description}', #{pd_idx0}, #{pd_idx1}, $);\n"
        ))
    }
}

impl ConstStepLength for EdgeDisplay<'_> {
    const LENGTH: usize = 5;
}

impl StepLength for EdgeDisplay<'_>
{
    #[inline]
    fn step_length(&self) -> usize { Self::LENGTH }
}

impl<Model, Models, Matrix> Display for StepDesign<Model, Models, Matrix>
where
    Model: DisplayByStep + StepLength,
    for<'a> &'a Models: IntoIterator<Item = &'a Model>,
    Matrix: Copy + truck_modeling::base::One,
    MatrixAsAxis<Matrix>: DisplayByStep + ConstStepLength,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        use std::collections::HashMap;
        let crc_display = StepDataDisplay::new(
            TruckRepresentationContext,
            COMMON_REPRESENTATION_CONTEXT_INDEX,
        );
        let app_ctx = &self.app_ctx;
        let id_display = StepDataDisplay::new(MatrixAsAxis(Matrix::one()), GLOBAL_IDENTITY_MATRIX);
        f.write_fmt(format_args!(
            "#{GLOBAL_APPLICATION_CONTEXT_INDEX} = APPLICATION_CONTEXT('{app_ctx}');
{crc_display}{id_display}",
        ))?;

        let mut idx = GLOBAL_IDENTITY_MATRIX + MatrixAsAxis::LENGTH;
        let mut node_map = HashMap::new();
        let mut mat_map = HashMap::new();
        for node in self.assy.all_nodes() {
            node_map.insert(node.index(), NodeInEdge::new(idx));

            let mut cursor = idx + 8;
            for (idx, _) in node.edges().enumerate() {
                mat_map.insert((node.index(), idx), cursor);
                cursor += MatrixAsAxis::LENGTH;
            }

            DisplayByStep::fmt(&node, idx, f)?;
            idx += node.step_length();
        }

        for node in self.assy.all_nodes() {
            for (i, edge) in node.edges().enumerate() {
                let (node_idx0, node_idx1) = edge.nodes();
                let nodes = (
                    *node_map.get(&node_idx0).unwrap(),
                    *node_map.get(&node_idx1).unwrap(),
                );

                let mat_key = (node.index(), i);
                let matrix_idx = *mat_map.get(&mat_key).unwrap();

                let edge_display = EdgeDisplay {
                    nodes,
                    matrix_idx,
                    attrs: edge.attrs(),
                };
                DisplayByStep::fmt(&edge_display, idx, f)?;
                idx += EdgeDisplay::LENGTH;
            }
        }

        Ok(())
    }
}

impl<Model, Models, Matrix> StepDesign<Model, Models, Matrix> {
    /// constructor
    #[inline]
    pub fn new(assy: Assembly<Models, PartAttrs, Matrix, PartAttrs>) -> Self {
        Self {
            assy,
            app_ctx: "generated shape data".to_string(),
            _model_ty: Default::default(),
        }
    }
    /// constructor with custom application context
    #[inline]
    pub fn with_app_context(
        assy: Assembly<Models, PartAttrs, Matrix, PartAttrs>,
        app_ctx: String,
    ) -> Self {
        Self {
            assy,
            app_ctx,
            _model_ty: Default::default(),
        }
    }
}

impl<Model> StepDesign<Model, Option<Model>, Matrix4> {
    /// generate representation from one model
    pub fn from_model(model: Model) -> Self {
        let mut assy = Assembly::new();
        assy.create_node(NodeEntity {
            shape: Some(model),
            attrs: Default::default(),
        });
        Self::new(assy)
    }
}
