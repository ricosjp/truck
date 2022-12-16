use super::{Result, *};

#[derive(Clone, Debug)]
pub(super) struct StepShell<'a, P, C, S> {
    entity: &'a CompressedShell<P, C, S>,
    idx: usize,
    face_indices: Vec<usize>,
    ep_edges: usize,
    ep_vertices: usize,
    surface_indices: Vec<usize>,
    curve_indices: Vec<usize>,
    ep_points: usize,
    is_open: bool,
}

impl<'a, P, C, S> StepDisplay<&'a CompressedShell<P, C, S>>
where
    P: Copy,
    C: StepLength,
    S: StepLength,
{
    fn to_step_shell(&self, is_open: bool) -> StepShell<'a, P, C, S> {
        let shell = self.entity;
        let faces = &shell.faces;
        let edges = &shell.edges;
        let vertices = &shell.vertices;
        let mut cursor = self.idx + 1;
        let face_indices = faces
            .iter()
            .map(|f| {
                let res = cursor;
                cursor += 1 + f.boundaries.iter().map(|b| 2 + b.len()).sum::<usize>();
                res
            })
            .collect::<Vec<_>>();
        let ep_edges = cursor;
        let ep_vertices = ep_edges + edges.len();
        cursor = ep_vertices + vertices.len();
        let surface_indices = faces
            .iter()
            .map(|f| {
                let res = cursor;
                cursor += f.surface.step_length();
                res
            })
            .collect::<Vec<_>>();
        let curve_indices = edges
            .iter()
            .map(|e| {
                let res = cursor;
                cursor += e.curve.step_length();
                res
            })
            .collect::<Vec<_>>();
        let ep_points = cursor;
        StepShell {
            entity: self.entity,
            idx: self.idx,
            face_indices,
            ep_edges,
            ep_vertices,
            surface_indices,
            curve_indices,
            ep_points,
            is_open,
        }
    }
}

impl<'a, P: Copy, C, S> Display for StepShell<'a, P, C, S>
where
    StepDisplay<P>: Display,
    StepDisplay<&'a C>: Display,
    StepDisplay<&'a S>: Display,
{
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        let StepShell {
            entity,
            idx,
            face_indices,
            ep_edges,
            ep_vertices,
            surface_indices,
            curve_indices,
            ep_points,
            is_open,
        } = self;
        let faces = &entity.faces;
        let edges = &entity.edges;
        let vertices = &entity.vertices;
        let shell_kind = match is_open {
            true => "OPEN_SHELL",
            false => "CLOSED_SHELL",
        };
        formatter.write_fmt(format_args!(
            "#{idx} = {shell_kind}('', {face_indices});\n",
            face_indices = IndexSliceDisplay(self.face_indices.iter().copied()),
        ))?;
        faces.iter().enumerate().try_for_each(|(i, f)| {
            let idx = face_indices[i];
            let same_sence = if f.orientation { ".T." } else { ".F." };
            let mut cursor = idx + 1;
            let face_bounds = f
                .boundaries
                .iter()
                .map(|b| {
                    let res = cursor;
                    cursor += 2 + b.len();
                    res
                })
                .collect::<Vec<_>>();
            formatter.write_fmt(format_args!(
                "#{idx} = FACE_SURFACE('', {face_bound}, #{face_geometry}, {same_sence});\n",
                face_bound = IndexSliceDisplay(face_bounds.iter().copied()),
                face_geometry = surface_indices[i],
            ))?;
            cursor = idx + 1;
            f.boundaries.iter().try_for_each(|b| {
                let face_bound_idx = cursor;
                let edge_loop_idx = cursor + 1;
                let ep_oriented_edges = cursor + 2;
                cursor += 2 + b.len();
                formatter.write_fmt(format_args!(
                    "#{face_bound_idx} = FACE_BOUND('', #{edge_loop_idx}, {same_sence});
#{edge_loop_idx} = EDGE_LOOP('', {oriented_edge_indices});\n",
                    oriented_edge_indices =
                        IndexSliceDisplay(ep_oriented_edges..ep_oriented_edges + b.len()),
                ))?;
                b.iter().enumerate().try_for_each(|(j, ce)| {
                    formatter.write_fmt(format_args!(
                        "#{idx} = ORIENTED_EDGE('', *, *, #{edge_element}, {orientation});\n",
                        idx = ep_oriented_edges + j,
                        edge_element = ep_edges + ce.index,
                        orientation = if ce.orientation { ".T." } else { ".F." },
                    ))
                })
            })
        })?;
        edges.iter().enumerate().try_for_each(|(i, e)| {
            formatter.write_fmt(format_args!(
                "#{idx} = EDGE_CURVE('', #{edge_start}, #{edge_end}, #{edge_geometry}, .T.);\n",
                idx = ep_edges + i,
                edge_start = ep_vertices + e.vertices.0,
                edge_end = ep_vertices + e.vertices.1,
                edge_geometry = curve_indices[i],
            ))
        })?;
        (0..vertices.len()).try_for_each(|i| {
            formatter.write_fmt(format_args!(
                "#{idx} = VERTEX_POINT('', #{vertex_geometry});\n",
                idx = ep_vertices + i,
                vertex_geometry = ep_points + i,
            ))
        })?;
        faces.iter().zip(surface_indices).try_for_each(|(f, idx)| {
            Display::fmt(&StepDisplay::new(&f.surface, *idx), formatter)
        })?;
        edges
            .iter()
            .zip(curve_indices)
            .try_for_each(|(e, idx)| Display::fmt(&StepDisplay::new(&e.curve, *idx), formatter))?;
        vertices
            .iter()
            .enumerate()
            .try_for_each(|(i, v)| Display::fmt(&StepDisplay::new(*v, ep_points + i), formatter))
    }
}

impl<'a, P, C, S> StepLength for StepShell<'a, P, C, S> {
    fn step_length(&self) -> usize {
        1 + self.ep_points + self.entity.vertices.len() - self.face_indices[0]
    }
}

#[derive(Clone, Debug)]
pub(super) struct StepSolid<'a, P, C, S> {
    idx: usize,
    boundaries: Vec<StepShell<'a, P, C, S>>,
}

impl<'a, P, C, S> StepDisplay<&'a CompressedSolid<P, C, S>>
where
    P: Copy,
    C: StepLength,
    S: StepLength,
{
    fn to_step_solid(&self) -> StepSolid<'a, P, C, S> {
        let StepDisplay { entity: solid, idx } = self;
        let mut cursor = idx + 1;
        let boundaries = solid
            .boundaries
            .iter()
            .map(|shell| {
                let res = StepDisplay::new(shell, cursor).to_step_shell(false);
                cursor += 1 + res.step_length();
                res
            })
            .collect::<Vec<_>>();
        StepSolid {
            idx: *idx,
            boundaries,
        }
    }
}

impl<'a, P, C, S> Display for StepSolid<'a, P, C, S>
where
    P: Copy,
    C: StepLength,
    S: StepLength,
    StepDisplay<P>: Display,
    StepDisplay<&'a C>: Display,
    StepDisplay<&'a S>: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let StepSolid { idx, boundaries } = self;
        match boundaries.len() {
            0 => {
                f.pad("empty solid!")?;
                Err(std::fmt::Error)
            }
            1 => {
                let shell_idx = idx + 1;
                let step_shell = &boundaries[0];
                f.write_fmt(format_args!(
                    "#{idx} = MANIFOLD_SOLID_BREP('', #{shell_idx});\n"
                ))?;
                Display::fmt(step_shell, f)
            }
            _ => {
                let first_shell_idx = boundaries[0].face_indices[0] - 1;
                f.write_fmt(format_args!(
                    "#{idx} = BREP_WITH_VOIDS('', #{first_shell_idx}, {other_shells});\n",
                    other_shells = IndexSliceDisplay(
                        boundaries[1..]
                            .iter()
                            .map(|step_shell| step_shell.face_indices[0] - 2)
                    ),
                ))?;
                Display::fmt(&boundaries[0], f)?;
                boundaries[1..].iter().try_for_each(|step_shell| {
                    let oriented_shell_idx = step_shell.face_indices[0] - 2;
                    let shell_idx = step_shell.face_indices[0] - 1;
                    f.write_fmt(format_args!(
                    "#{oriented_shell_idx} = ORIENTED_CLOSED_SHELL('', *, #{shell_idx}, .T.);\n",
                ))?;
                    Display::fmt(step_shell, f)
                })
            }
        }
    }
}

impl<'a, P, C, S> StepLength for StepSolid<'a, P, C, S> {
    fn step_length(&self) -> usize {
        let b = &self.boundaries;
        match b.len() {
            0 => 0,
            1 => 1 + b[0].step_length(),
            _ => b.len() + b.iter().map(StepLength::step_length).sum::<usize>(),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) enum PreStepModel<'a, P, C, S> {
    /// shell based surface model
    Shell(StepShell<'a, P, C, S>),
    /// solid model
    Solid(StepSolid<'a, P, C, S>),
}

impl<'a, P, C, S> From<&'a CompressedShell<P, C, S>> for PreStepModel<'a, P, C, S>
where
    P: Copy,
    C: StepLength,
    S: StepLength,
{
    fn from(shell: &'a CompressedShell<P, C, S>) -> Self {
        Self::Shell(StepDisplay::new(shell, 17).to_step_shell(true))
    }
}

impl<'a, P, C, S> From<&'a CompressedSolid<P, C, S>> for PreStepModel<'a, P, C, S>
where
    P: Copy,
    C: StepLength,
    S: StepLength,
{
    fn from(solid: &'a CompressedSolid<P, C, S>) -> Self {
        Self::Solid(StepDisplay::new(solid, 16).to_step_solid())
    }
}

impl<'a, P, C, S> Display for PreStepModel<'a, P, C, S>
where
    P: Copy,
    C: StepLength,
    S: StepLength,
    StepDisplay<P>: Display,
    StepDisplay<&'a C>: Display,
    StepDisplay<&'a S>: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Shell(x) => {
                f.write_fmt(format_args!(
                    "#{idx} = SHELL_BASED_SURFACE_MODEL('', (#{shell_idx}));\n",
                    idx = x.idx - 1,
                    shell_idx = x.idx
                ))?;
                Display::fmt(&x, f)
            }
            Self::Solid(x) => Display::fmt(x, f),
        }
    }
}

impl<'a, P, C, S> StepLength for PreStepModel<'a, P, C, S> {
    fn step_length(&self) -> usize {
        match self {
            Self::Shell(x) => 1 + x.step_length(),
            Self::Solid(x) => x.step_length(),
        }
    }
}

impl<'a, P, C, S> From<&'a CompressedShell<P, C, S>> for StepModel<'a, P, C, S>
where
    P: Copy,
    C: StepLength,
    S: StepLength,
{
    fn from(shell: &'a CompressedShell<P, C, S>) -> Self { Self(shell.into()) }
}

impl<'a, P, C, S> From<&'a CompressedSolid<P, C, S>> for StepModel<'a, P, C, S>
where
    P: Copy,
    C: StepLength,
    S: StepLength,
{
    fn from(solid: &'a CompressedSolid<P, C, S>) -> Self { Self(solid.into()) }
}

impl<'a, P, C, S> Display for StepModel<'a, P, C, S>
where
    P: Copy,
    C: StepLength,
    S: StepLength,
    StepDisplay<P>: Display,
    StepDisplay<&'a C>: Display,
    StepDisplay<&'a S>: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.pad(
"#1 = APPLICATION_PROTOCOL_DEFINITION('international standard', 'automotive_design', 2000, #2);
#2 = APPLICATION_CONTEXT('core data for automotive mechanical design processes');
#3 = SHAPE_DEFINITION_REPRESENTATION(#4, #10);
#4 = PRODUCT_DEFINITION_SHAPE('','', #5);
#5 = PRODUCT_DEFINITION('design','', #6, #9);
#6 = PRODUCT_DEFINITION_FORMATION('','', #7);
#7 = PRODUCT('','','', (#8));
#8 = PRODUCT_CONTEXT('', #2, 'mechanical');
#9 = PRODUCT_DEFINITION_CONTEXT('part definition', #2, 'design');
#10 = ADVANCED_BREP_SHAPE_REPRESENTATION('', (#16), #11);
#11 = (
    GEOMETRIC_REPRESENTATION_CONTEXT(3) 
    GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#15))
    GLOBAL_UNIT_ASSIGNED_CONTEXT((#12, #13, #14))
    REPRESENTATION_CONTEXT('Context #1', '3D Context with UNIT and UNCERTAINTY')
);
#12 = ( LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.) );
#13 = ( NAMED_UNIT(*) PLANE_ANGLE_UNIT() SI_UNIT($,.RADIAN.) );
#14 = ( NAMED_UNIT(*) SI_UNIT($,.STERADIAN.) SOLID_ANGLE_UNIT() );
#15 = UNCERTAINTY_MEASURE_WITH_UNIT(1.0E-6, #12, 'distance_accuracy_value','confusion accuracy');\n"
        )?;
        Display::fmt(&self.0, f)
    }
}

impl<'a, P, C, S> Default for StepModels<'a, P, C, S> {
    fn default() -> Self {
        Self {
            models: Vec::new(),
            next_idx: 16,
        }
    }
}

impl<'a, P, C, S> StepModels<'a, P, C, S>
where
    P: Copy,
    C: StepLength,
    S: StepLength,
{
    /// push a shell to step models
    pub fn push_shell(&mut self, shell: &'a CompressedShell<P, C, S>) {
        let model = PreStepModel::Shell(
            StepDisplay::new(shell, self.next_idx + 1)
                .to_step_shell(true)
                .into(),
        );
        self.next_idx += model.step_length();
        self.models.push(model)
    }
    /// push a solid to step models
    pub fn push_solid(&mut self, solid: &'a CompressedSolid<P, C, S>) {
        let model = PreStepModel::Solid(
            StepDisplay::new(solid, self.next_idx)
                .to_step_solid()
                .into(),
        );
        self.next_idx += model.step_length();
        self.models.push(model)
    }
}

impl<'a, P, C, S> FromIterator<&'a CompressedShell<P, C, S>> for StepModels<'a, P, C, S>
where
    P: Copy,
    C: StepLength,
    S: StepLength,
{
    fn from_iter<T: IntoIterator<Item = &'a CompressedShell<P, C, S>>>(iter: T) -> Self {
        let mut next_idx = 16;
        let models = iter
            .into_iter()
            .map(|shell| {
                let model = PreStepModel::Shell(
                    StepDisplay::new(shell, next_idx + 1)
                        .to_step_shell(true)
                        .into(),
                );
                next_idx += model.step_length();
                model
            })
            .collect();
        Self { models, next_idx }
    }
}

impl<'a, P, C, S> FromIterator<&'a CompressedSolid<P, C, S>> for StepModels<'a, P, C, S>
where
    P: Copy,
    C: StepLength,
    S: StepLength,
{
    fn from_iter<T: IntoIterator<Item = &'a CompressedSolid<P, C, S>>>(iter: T) -> Self {
        let mut next_idx = 16;
        let models = iter
            .into_iter()
            .map(|solid| {
                let model =
                    PreStepModel::Solid(StepDisplay::new(solid, next_idx).to_step_solid().into());
                next_idx += model.step_length();
                model
            })
            .collect();
        Self { models, next_idx }
    }
}

impl<'a, P, C, S> Display for StepModels<'a, P, C, S>
where
    P: Copy,
    C: StepLength,
    S: StepLength,
    StepDisplay<P>: Display,
    StepDisplay<&'a C>: Display,
    StepDisplay<&'a S>: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.pad(
"#1 = APPLICATION_PROTOCOL_DEFINITION('international standard', 'automotive_design', 2000, #2);
#2 = APPLICATION_CONTEXT('core data for automotive mechanical design processes');
#3 = SHAPE_DEFINITION_REPRESENTATION(#4, #10);
#4 = PRODUCT_DEFINITION_SHAPE('','', #5);
#5 = PRODUCT_DEFINITION('design','', #6, #9);
#6 = PRODUCT_DEFINITION_FORMATION('','', #7);
#7 = PRODUCT('','','', (#8));
#8 = PRODUCT_CONTEXT('', #2, 'mechanical');
#9 = PRODUCT_DEFINITION_CONTEXT('part definition', #2, 'design');\n")?;
        let models_slice = IndexSliceDisplay(self.models.iter().map(|model| match model {
            PreStepModel::Shell(x) => x.idx - 1,
            PreStepModel::Solid(x) => x.idx,
        }));
        f.write_fmt(format_args!(
            "#10 = ADVANCED_BREP_SHAPE_REPRESENTATION('', {models_slice}, #11);\n"
        ))?;
        f.pad("#11 = (
    GEOMETRIC_REPRESENTATION_CONTEXT(3) 
    GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#15))
    GLOBAL_UNIT_ASSIGNED_CONTEXT((#12, #13, #14))
    REPRESENTATION_CONTEXT('Context #1', '3D Context with UNIT and UNCERTAINTY')
);
#12 = ( LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.) );
#13 = ( NAMED_UNIT(*) PLANE_ANGLE_UNIT() SI_UNIT($,.RADIAN.) );
#14 = ( NAMED_UNIT(*) SI_UNIT($,.STERADIAN.) SOLID_ANGLE_UNIT() );
#15 = UNCERTAINTY_MEASURE_WITH_UNIT(1.0E-6, #12, 'distance_accuracy_value','confusion accuracy');\n"
        )?;
        self.models
            .iter()
            .try_for_each(|model| Display::fmt(model, f))
    }
}
