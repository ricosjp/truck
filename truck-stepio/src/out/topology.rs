use super::{Result, *};
use truck_topology::*;

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
		let shell = &self.entity;
		let faces = shell.faces();
		let edges = shell.edges();
		let vertices = shell.vertices();
		let mut cursor = self.idx + 1;
		let face_indices = faces
			.iter()
			.map(|f| {
				let res = cursor;
				cursor += 1 + f.boundaries().len();
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
			entity: &self.entity,
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
	fn fmt(&self, formatter: &mut Formatter) -> Result {
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
			formatter.write_fmt(format_args!(
				"#{idx} = FACE_SURFACE('', {face_bound}, {face_geometry}, {same_sence});\n",
				face_bound = IndexSliceDisplay(idx + 1..=idx + f.boundaries().len()),
				face_geometry = surface_indices[i],
				same_sence = if f.orientation() { ".T." } else { ".F." },
			))?;
			f.boundaries().iter().enumerate().try_for_each(|(i, b)| {
				formatter.write_fmt(format_args!(
					"#{idx} = FACE_BOUND('', EDGE_LOOP('', (",
					idx = idx + i
				))?;
				b.iter().enumerate().try_for_each(|(j, (e, ori))| {
					if j != 0 {
						formatter.write_str(", ")?;
					}
					formatter.write_fmt(format_args!(
						"ORIENTED_EDGE('', *, *, {edge_element}, {orientation})",
						edge_element = ep_edges + e,
						orientation = if *ori { ".T." } else { ".F." },
					))
				})?;
				formatter.write_str(")), .T.);\n")
			})
		})?;
		edges.iter().enumerate().try_for_each(|(i, e)| {
			formatter.write_fmt(format_args!(
				"#{idx} = EDGE_CURVE('', {edge_start}, {edge_end}, {edge_geometry}, .T.);\n",
				idx = ep_edges + i,
				edge_start = ep_vertices + e.vertices().0,
				edge_end = ep_vertices + e.vertices().1,
				edge_geometry = curve_indices[i],
			))
		})?;
		(0..vertices.len()).try_for_each(|i| {
			formatter.write_fmt(format_args!(
				"#{idx} = VERTEX_POINT('', {vertex_geometry});\n",
				idx = ep_vertices + i,
				vertex_geometry = ep_points + i,
			))
		})?;
		faces
			.into_iter()
			.zip(surface_indices)
			.try_for_each(|(f, idx)| {
				Display::fmt(&StepDisplay::new(f.surface(), *idx), formatter)
			})?;
		edges
			.into_iter()
			.zip(curve_indices)
			.try_for_each(|(e, idx)| Display::fmt(&StepDisplay::new(e.curve(), *idx), formatter))?;
		vertices
			.into_iter()
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
	fn fmt(&self, f: &mut Formatter) -> Result {
		let StepDisplay { entity: solid, idx } = self;
		if solid.boundaries().is_empty() {
			eprintln!("empty solid!");
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
