use super::{Result, *};
use truck_topology::{*, compress::*};

#[derive(Clone, Debug)]
struct StepShell<'a, P, C, S> {
    entity: &'a CompressedShell<P, C, S>,
    face_indices: Vec<usize>,
    ep_edges: usize,
    ep_vertices: usize,
    surface_indices: Vec<usize>,
    curve_indices: Vec<usize>,
    ep_points: usize,
}

impl<'a, P, C, S> StepDisplay<&'a CompressedShell<P, C, S>>
where
    P: Copy,
    C: StepLength,
    S: StepLength,
{
    fn to_step_shell(&self) -> StepShell<'a, P, C, S> {
        let shell = self.entity;
        let faces = shell.faces();
        let edges = shell.edges();
        let vertices = shell.vertices();
        let mut cursor = self.idx + 1;
        let face_indices = faces
            .iter()
            .map(|f| {
                let res = cursor;
                cursor += 1 + f.boundaries().iter().map(|b| 2 + b.len()).sum::<usize>();
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
                cursor += f.surface().step_length();
                res
            })
            .collect::<Vec<_>>();
        let curve_indices = edges
            .iter()
            .map(|e| {
                let res = cursor;
                cursor += e.curve().step_length();
                res
            })
            .collect::<Vec<_>>();
        let ep_points = cursor;
        StepShell {
            entity: self.entity,
            face_indices,
            ep_edges,
            ep_vertices,
            surface_indices,
            curve_indices,
            ep_points,
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
            face_indices,
            ep_edges,
            ep_vertices,
            surface_indices,
            curve_indices,
            ep_points,
        } = self;
        let faces = entity.faces();
        let edges = entity.edges();
        let vertices = entity.vertices();
        faces.iter().enumerate().try_for_each(|(i, f)| {
            let idx = face_indices[i];
            let same_sence = if f.orientation() { ".T." } else { ".F." };
            let mut cursor = idx + 1;
            let face_bounds = f
                .boundaries()
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
            f.boundaries().iter().try_for_each(|b| {
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
                b.iter().enumerate().try_for_each(|(j, (e, ori))| {
                    formatter.write_fmt(format_args!(
                        "#{idx} = ORIENTED_EDGE('', *, *, #{edge_element}, {orientation});\n",
                        idx = ep_oriented_edges + j,
                        edge_element = ep_edges + e,
                        orientation = if *ori { ".T." } else { ".F." },
                    ))
                })
            })
        })?;
        edges.iter().enumerate().try_for_each(|(i, e)| {
            formatter.write_fmt(format_args!(
                "#{idx} = EDGE_CURVE('', #{edge_start}, #{edge_end}, #{edge_geometry}, .T.);\n",
                idx = ep_edges + i,
                edge_start = ep_vertices + e.vertices().0,
                edge_end = ep_vertices + e.vertices().1,
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
            Display::fmt(&StepDisplay::new(f.surface(), *idx), formatter)
        })?;
        edges
            .iter()
            .zip(curve_indices)
            .try_for_each(|(e, idx)| Display::fmt(&StepDisplay::new(e.curve(), *idx), formatter))?;
        vertices
            .iter()
            .enumerate()
            .try_for_each(|(i, v)| Display::fmt(&StepDisplay::new(*v, ep_points + i), formatter))
    }
}

impl<'a, P, C, S> StepLength for StepShell<'a, P, C, S> {
    fn step_length(&self) -> usize {
        self.ep_points + self.entity.vertices().len() - self.face_indices[0]
    }
}

impl<'a, P, C, S> Display for StepDisplay<&'a CompressedSolid<P, C, S>>
where
    P: Copy,
    C: StepLength,
    S: StepLength,
    StepDisplay<P>: Display,
    StepDisplay<&'a C>: Display,
    StepDisplay<&'a S>: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let StepDisplay { entity: solid, idx } = self;
        if solid.boundaries().is_empty() {
            f.pad("empty solid!")?;
            Err(std::fmt::Error)
        } else if solid.boundaries().len() == 1 {
            let shell_idx = idx + 1;
            f.write_fmt(format_args!(
                "#{idx} = MANIFOLD_SOLID_BREP('', #{shell_idx});\n"
            ))?;
            let shell = &solid.boundaries()[0];
            let step_shell = StepDisplay::new(shell, shell_idx).to_step_shell();
            f.write_fmt(format_args!(
                "#{shell_idx} = CLOSED_SHELL('', {face_indices});\n",
                face_indices = IndexSliceDisplay(step_shell.face_indices.iter().copied()),
            ))?;
            Display::fmt(&step_shell, f)
        } else {
            let mut cursor = self.idx + 1;
            let step_shells = solid
                .boundaries()
                .iter()
                .map(|shell| {
                    let res = StepDisplay::new(shell, cursor).to_step_shell();
                    cursor += 1 + res.step_length();
                    res
                })
                .collect::<Vec<_>>();
            let first_shell_idx = step_shells[0].face_indices[0] - 1;
            f.write_fmt(format_args!(
                "#{idx} = BREP_WITH_VOIDS('', #{first_shell_idx}, {other_shells});\n",
                other_shells = IndexSliceDisplay(
                    step_shells[1..]
                        .iter()
                        .map(|step_shell| step_shell.face_indices[0] - 1)
                ),
            ))?;
            f.write_fmt(format_args!(
                "#{first_shell_idx} = CLOSED_SHELL('', {face_indices});\n",
                face_indices = IndexSliceDisplay(step_shells[0].face_indices.iter().copied()),
            ))?;
            Display::fmt(&step_shells[0], f)?;
            step_shells[1..].iter().try_for_each(|step_shell| {
                f.write_fmt(format_args!(
                    "#{shell_idx} = ORIENTED_CLOSED_SHELL('', {face_indices}, .T.);\n",
                    shell_idx = step_shell.face_indices[0] - 1,
                    face_indices = IndexSliceDisplay(step_shell.face_indices.iter().copied()),
                ))?;
                Display::fmt(step_shell, f)
            })
        }
    }
}

impl<'a, P, C, S> Display for StepDisplay<CompressedSolid<P, C, S>>
where
    P: Copy,
    C: StepLength + 'a,
    S: StepLength + 'a,
    StepDisplay<P>: Display,
    StepDisplay<&'a C>: Display,
    StepDisplay<&'a S>: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Display::fmt(&self, f)
    }
}

impl<'a, P, C, S> Display for StepDisplay<&'a Solid<P, C, S>>
where
    P: Copy,
    C: StepLength + Clone,
    S: StepLength + Clone,
    StepDisplay<P>: Display,
    StepDisplay<&'a C>: Display + 'a,
    StepDisplay<&'a S>: Display + 'a,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let compressed = self.entity.compress();
        Display::fmt(&StepDisplay::new(compressed, self.idx), f)
    }
}
