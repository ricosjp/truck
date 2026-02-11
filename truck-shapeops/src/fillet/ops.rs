use algo::curve::search_closest_parameter;
use truck_geometry::prelude::*;

use super::error::FilletError;
use super::geometry::*;
use super::params::{FilletOptions, FilletProfile, RadiusSpec};
use super::topology::*;
use super::types::*;

type Result<T> = std::result::Result<T, FilletError>;

/// Fillets a single shared edge between two faces.
///
/// Returns `(new_face0, new_face1, fillet_face)`.
pub fn simple_fillet(
    face0: &Face,
    face1: &Face,
    filleted_edge_id: EdgeID,
    options: &FilletOptions,
) -> Result<(Face, Face, Face)> {
    let is_filleted_edge = move |edge: &Edge| edge.id() == filleted_edge_id;
    let filleted_edge =
        face0
            .edge_iter()
            .find(is_filleted_edge)
            .ok_or(FilletError::GeometryFailed {
                context: "filleted edge not found in face0",
            })?;

    let division = options.division.get();
    let fillet_surface = {
        let surface0 = face0.oriented_surface();
        let surface1 = face1.oriented_surface();
        let curve = filleted_edge.oriented_curve();
        let make = |radius: &dyn Fn(f64) -> f64| match &options.profile {
            FilletProfile::Round => {
                rolling_ball_fillet_surface(&surface0, &surface1, &curve, division, radius, true)
            }
            FilletProfile::Chamfer => {
                chamfer_fillet_surface(&surface0, &surface1, &curve, division, radius, true)
            }
            FilletProfile::Ridge => {
                ridge_fillet_surface(&surface0, &surface1, &curve, division, radius, true)
            }
            FilletProfile::Custom(profile) => custom_fillet_surface(
                &surface0, &surface1, &curve, division, radius, true, profile,
            ),
        };
        match &options.radius {
            RadiusSpec::Constant(r) => {
                let r = *r;
                make(&|_| r)
            }
            RadiusSpec::Variable(f) => make(f.as_ref()),
            RadiusSpec::PerEdge(radii) => match radii.first() {
                Some(&r) => make(&|_| r),
                None => None,
            },
        }
        .ok_or(FilletError::GeometryFailed {
            context: "fillet surface computation",
        })?
    };

    let (new_face0, fillet_edge0) = {
        let bezier = fillet_surface.column_curve(0);
        cut_face_by_bezier(face0, bezier, filleted_edge.id()).ok_or(
            FilletError::GeometryFailed {
                context: "cut face0 by bezier",
            },
        )?
    };
    let (new_face1, fillet_edge1) = {
        let bezier = fillet_surface.column_curve(fillet_surface.control_points().len() - 1);
        cut_face_by_bezier(face1, bezier.inverse(), filleted_edge.id()).ok_or(
            FilletError::GeometryFailed {
                context: "cut face1 by bezier",
            },
        )?
    };

    let ((v0, v1), (v2, v3)) = (fillet_edge0.ends(), fillet_edge1.ends());
    let edge0 = create_pcurve_edge((v0, (0.0, 0.0)), (v3, (1.0, 0.0)), &fillet_surface).ok_or(
        FilletError::GeometryFailed {
            context: "create pcurve edge0",
        },
    )?;
    let edge1 = create_pcurve_edge((v2, (1.0, 1.0)), (v1, (0.0, 1.0)), &fillet_surface).ok_or(
        FilletError::GeometryFailed {
            context: "create pcurve edge1",
        },
    )?;
    let fillet = {
        let fillet_boundary = [fillet_edge0.inverse(), edge0, fillet_edge1.inverse(), edge1];
        Face::new(vec![fillet_boundary.into()], fillet_surface)
    };

    Ok((new_face0, new_face1, fillet))
}

/// Fillets a shared edge between two faces, optionally updating adjacent side faces.
///
/// Returns `(new_face0, new_face1, fillet_face, new_side0, new_side1)`.
#[allow(clippy::type_complexity)]
pub fn fillet_with_side(
    face0: &Face,
    face1: &Face,
    filleted_edge_id: EdgeID,
    side0: Option<&Face>,
    side1: Option<&Face>,
    options: &FilletOptions,
) -> Result<(Face, Face, Face, Option<Face>, Option<Face>)> {
    let (new_face0, new_face1, fillet) = simple_fillet(face0, face1, filleted_edge_id, options)?;

    let (front_edge0, back_edge0) = {
        let fillet_edge_id = fillet.absolute_boundaries()[0][0].id();
        find_adjacent_edge(&new_face0, fillet_edge_id).ok_or(FilletError::GeometryFailed {
            context: "find adjacent edge in new_face0",
        })?
    };
    let (front_edge1, back_edge1) = {
        let fillet_edge_id = fillet.absolute_boundaries()[0][2].id();
        find_adjacent_edge(&new_face1, fillet_edge_id).ok_or(FilletError::GeometryFailed {
            context: "find adjacent edge in new_face1",
        })?
    };

    let is_filleted_edge = |edge: &Edge| edge.id() == filleted_edge_id;
    let filleted_edge =
        face0
            .edge_iter()
            .find(is_filleted_edge)
            .ok_or(FilletError::GeometryFailed {
                context: "filleted edge not found in face0",
            })?;
    let (v0, v1) = filleted_edge.ends();

    let new_side0 = side0.and_then(|side0| {
        let fillet_edge = &fillet.absolute_boundaries()[0][1];
        create_new_side(side0, fillet_edge, v0.id(), &front_edge0, &back_edge1)
    });
    let new_side1 = side1.and_then(|side1| {
        let fillet_edge = &fillet.absolute_boundaries()[0][3];
        create_new_side(side1, fillet_edge, v1.id(), &front_edge1, &back_edge0)
    });
    Ok((new_face0, new_face1, fillet, new_side0, new_side1))
}

/// Fillets along a wire of edges sharing a common face in the shell.
///
/// Supports both open and closed wires. Modifies `shell` in place by replacing
/// filleted faces and adding new fillet faces.
pub fn fillet_along_wire(shell: &mut Shell, wire: &Wire, options: &FilletOptions) -> Result<()> {
    let division = options.division.get();

    // Validate variable radius constraint for closed wire fillets.
    // Open wires don't wrap around, so f(0) ≈ f(1) is only needed for closed wires.
    if wire.is_closed() {
        if let RadiusSpec::Variable(f) = &options.radius {
            if !f(0.0).near2(&f(1.0)) {
                return Err(FilletError::VariableRadiusUnsupported);
            }
        }
    }
    if !wire.is_continuous() {
        return Err(FilletError::DiscontinuousWire);
    }

    let closed = wire.is_closed();

    let shared_face_index =
        find_shared_face_with_front_edge(shell, wire).ok_or(FilletError::SharedFaceNotFound)?;
    let adjacent_faces = enumerate_adjacent_faces(shell, wire, shared_face_index)
        .ok_or(FilletError::AdjacentFacesNotFound)?;

    let mut fillet_surfaces = match &options.radius {
        RadiusSpec::Constant(r) => {
            let r = *r;
            fillet_surfaces_along_wire(
                shell,
                wire,
                shared_face_index,
                &adjacent_faces,
                move |_| r,
                division,
                &options.profile,
            )
        }
        RadiusSpec::Variable(f) => fillet_surfaces_along_wire(
            shell,
            wire,
            shared_face_index,
            &adjacent_faces,
            f.as_ref(),
            division,
            &options.profile,
        ),
        RadiusSpec::PerEdge(radii) => match radii.first() {
            Some(&r) => fillet_surfaces_along_wire(
                shell,
                wire,
                shared_face_index,
                &adjacent_faces,
                move |_| r,
                division,
                &options.profile,
            ),
            None => None,
        },
    }
    .ok_or(FilletError::FilletSurfaceComputationFailed)?;

    // Interior seam averaging.
    (1..fillet_surfaces.len()).for_each(|i| {
        let len = fillet_surfaces[i].control_points().len();
        (0..len).for_each(|j| {
            let len = fillet_surfaces[i - 1].control_points()[j].len();
            let p = *fillet_surfaces[i - 1].control_point(j, len - 1);
            let q = *fillet_surfaces[i].control_point(j, 0);
            let c = (p + q) / 2.0;
            *fillet_surfaces[i - 1].control_point_mut(j, len - 1) = c;
            *fillet_surfaces[i].control_point_mut(j, 0) = c;
        });
    });

    // Wrap-around seam averaging for closed wires.
    if closed {
        let last = fillet_surfaces.len() - 1;
        for j in 0..fillet_surfaces[last].control_points().len() {
            let len = fillet_surfaces[last].control_points()[j].len();
            let p = *fillet_surfaces[last].control_point(j, len - 1);
            let q = *fillet_surfaces[0].control_point(j, 0);
            let c = (p + q) / 2.0;
            *fillet_surfaces[last].control_point_mut(j, len - 1) = c;
            *fillet_surfaces[0].control_point_mut(j, 0) = c;
        }
    }

    if closed {
        fillet_along_wire_closed(
            shell,
            wire,
            shared_face_index,
            &adjacent_faces,
            &fillet_surfaces,
        )
    } else {
        fillet_along_wire_open(
            shell,
            wire,
            shared_face_index,
            &adjacent_faces,
            &fillet_surfaces,
        )
    }
}

/// Open-wire fillet face construction (original logic).
fn fillet_along_wire_open(
    shell: &mut Shell,
    wire: &Wire,
    shared_face_index: FaceBoundaryEdgeIndex,
    adjacent_faces: &[FaceBoundaryEdgeIndex],
    fillet_surfaces: &[NurbsSurface<Vector4>],
) -> Result<()> {
    type CffTuple<'a> = (&'a [NurbsSurface<Vector4>], &'a FaceBoundaryEdgeIndex);
    let create_fillet_face = |(surfaces, face_index): CffTuple<'_>| {
        let fillet_surface = concat_fillet_surface(surfaces);
        let edge0 = create_free_edge(surfaces[1].column_curve(0).into());

        let edge1 = cut_face_by_last_bezier(shell, *face_index, &fillet_surface)?;

        let edge2 = {
            let (v0, v1) = (edge0.front(), edge1.back());
            let (u, v) = fillet_surface.search_parameter(v1.point(), (1.0, 1.0), 100)?;
            let param_line = Line((0.0, 1.0).into(), (u, v).into());
            let pcurve = ParamCurveLinear::new(param_line, fillet_surface.clone());
            Edge::new(v0, v1, pcurve.into())
        };

        let edge3 = {
            let (v0, v1) = (edge0.back(), edge1.front());
            let (u, v) = fillet_surface.search_parameter(v1.point(), (1.0, 2.0), 100)?;
            let param_line = Line((0.0, 2.0).into(), (u, v).into());
            let pcurve = ParamCurveLinear::new(param_line, fillet_surface.clone());
            Edge::new(v0, v1, pcurve.into())
        };

        let boundary = [edge0.inverse(), edge2, edge1.inverse(), edge3.inverse()].into();
        Some(Face::new(vec![boundary], fillet_surface))
    };

    let mut fillet_faces = fillet_surfaces
        .windows(3)
        .zip(adjacent_faces.iter().skip(1))
        .map(create_fillet_face)
        .collect::<Option<Shell>>()
        .ok_or(FilletError::GeometryFailed {
            context: "create fillet faces",
        })?;

    let first_fillet = {
        let fillet_surface = concat_fillet_surface(&fillet_surfaces[0..=1]);

        let (front_edge, _) =
            find_adjacent_edge(&shell[shared_face_index.face_index], wire[0].id()).ok_or(
                FilletError::GeometryFailed {
                    context: "find front edge for first fillet",
                },
            )?;

        let edge0 = {
            let mut bezier = fillet_surfaces[0].column_curve(0);
            let curve = front_edge.oriented_curve();
            let (t0, _) = search_closest_parameter(&bezier, &curve, (0.0, 1.0), 100).ok_or(
                FilletError::GeometryFailed {
                    context: "search closest parameter for first fillet edge0",
                },
            )?;
            bezier = bezier.cut(t0);
            let v0 = Vertex::new(bezier.front());
            let v1 = Vertex::new(bezier.back());
            Edge::new(&v0, &v1, bezier.into())
        };

        let edge1 = cut_face_by_last_bezier(shell, adjacent_faces[0], &fillet_surface).ok_or(
            FilletError::GeometryFailed {
                context: "cut face by last bezier for first fillet",
            },
        )?;

        let edge2 = {
            let (v0, v1) = (edge0.front(), edge1.back());
            let t0 = edge0.curve().range_tuple().0;
            let (u, v) = fillet_surface
                .search_parameter(v1.point(), (1.0, 0.0), 100)
                .ok_or(FilletError::GeometryFailed {
                    context: "search parameter for first fillet edge2",
                })?;
            let param_line = Line((0.0, t0).into(), (u, v).into());
            let pcurve = ParamCurveLinear::new(param_line, fillet_surface.clone());
            Edge::new(v0, v1, pcurve.into())
        };

        let edge3 = {
            let (v0, v1) = (edge0.back(), edge1.front());
            let (u, v) = fillet_surface
                .search_parameter(v1.point(), (1.0, 1.0), 100)
                .ok_or(FilletError::GeometryFailed {
                    context: "search parameter for first fillet edge3",
                })?;
            let param_line = Line((0.0, 1.0).into(), (u, v).into());
            let pcurve = ParamCurveLinear::new(param_line, fillet_surface.clone());
            Edge::new(v0, v1, pcurve.into())
        };

        let wire = [edge0.inverse(), edge2, edge1.inverse(), edge3.inverse()].into();
        Face::new(vec![wire], fillet_surface)
    };
    fillet_faces.insert(0, first_fillet);

    let last_fillet = {
        let len = wire.len();

        let (_, last_edge) =
            find_adjacent_edge(&shell[shared_face_index.face_index], wire[len - 1].id()).ok_or(
                FilletError::GeometryFailed {
                    context: "find last edge for last fillet",
                },
            )?;

        let edge0 = {
            let mut bezier = fillet_surfaces[len - 1].column_curve(0);
            let curve = last_edge.oriented_curve();
            let (t0, _) = search_closest_parameter(&bezier, &curve, (1.0, 2.0), 100).ok_or(
                FilletError::GeometryFailed {
                    context: "search closest parameter for last fillet edge0",
                },
            )?;
            bezier.cut(t0);
            let v0 = Vertex::new(bezier.front());
            let v1 = Vertex::new(bezier.back());
            Edge::new(&v0, &v1, bezier.into())
        };

        let fillet_surface = concat_fillet_surface(&fillet_surfaces[len - 2..len]);

        let edge1 = cut_face_by_last_bezier(shell, adjacent_faces[len - 1], &fillet_surface)
            .ok_or(FilletError::GeometryFailed {
                context: "cut face by last bezier for last fillet",
            })?;

        let edge2 = {
            let (v0, v1) = (edge0.front(), edge1.back());
            let (u, v) = fillet_surface
                .search_parameter(v1.point(), (1.0, 1.0), 100)
                .ok_or(FilletError::GeometryFailed {
                    context: "search parameter for last fillet edge2",
                })?;
            let param_line = Line((0.0, 1.0).into(), (u, v).into());
            let pcurve = ParamCurveLinear::new(param_line, fillet_surface.clone());
            Edge::new(v0, v1, pcurve.into())
        };

        let edge3 = {
            let (v0, v1) = (edge0.back(), edge1.front());
            let t0 = edge0.curve().range_tuple().1;
            let (u, v) = fillet_surface
                .search_parameter(v1.point(), (1.0, 2.0), 100)
                .ok_or(FilletError::GeometryFailed {
                    context: "search parameter for last fillet edge3",
                })?;
            let param_line = Line((0.0, t0 + 1.0).into(), (u, v).into());
            let pcurve = ParamCurveLinear::new(param_line, fillet_surface.clone());
            Edge::new(v0, v1, pcurve.into())
        };

        let wire = [edge0.inverse(), edge2, edge1.inverse(), edge3.inverse()].into();
        Face::new(vec![wire], fillet_surface)
    };
    fillet_faces.push(last_fillet);

    {
        let mut previous_vertex = None;
        let mut new_wire = fillet_faces
            .face_iter()
            .map(|face| {
                let edge = &face.boundaries()[0][0];
                let v0 = Vertex::new(edge.front().point());
                let v1 = match &previous_vertex {
                    Some(v) => Vertex::clone(v),
                    None => Vertex::new(edge.back().point()),
                };
                let new_edge = Edge::new(&v0, &v1, edge.oriented_curve());
                previous_vertex = Some(v0);
                new_edge.inverse()
            })
            .collect::<Wire>();

        let shared_face = &mut shell[shared_face_index.face_index];
        let (front_edge, _) =
            find_adjacent_edge(shared_face, wire[0].id()).ok_or(FilletError::GeometryFailed {
                context: "find front edge on shared face",
            })?;
        let (_, back_edge) = find_adjacent_edge(shared_face, wire[wire.len() - 1].id()).ok_or(
            FilletError::GeometryFailed {
                context: "find back edge on shared face",
            },
        )?;

        let mut boundaries = shared_face.boundaries();

        if front_edge == back_edge {
            let pre_new_edge = front_edge
                .not_strictly_cut(new_wire.front_vertex().unwrap())
                .ok_or(FilletError::GeometryFailed {
                    context: "cut front edge (same as back)",
                })?
                .0;
            let new_edge = pre_new_edge
                .not_strictly_cut(new_wire.back_vertex().unwrap())
                .ok_or(FilletError::GeometryFailed {
                    context: "cut back edge (same as front)",
                })?
                .1;
            new_wire.push_front(new_edge);
        } else {
            let new_front_edge = front_edge
                .not_strictly_cut(new_wire.front_vertex().unwrap())
                .ok_or(FilletError::GeometryFailed {
                    context: "cut front edge",
                })?
                .0;
            let new_back_edge = back_edge
                .not_strictly_cut(new_wire.back_vertex().unwrap())
                .ok_or(FilletError::GeometryFailed {
                    context: "cut back edge",
                })?
                .1;
            new_wire.push_front(new_front_edge);
            new_wire.push_back(new_back_edge);

            let boundary = &boundaries[shared_face_index.boundary_index];
            let len = boundary.len() - new_wire.len();
            let top_index = shared_face_index.edge_index + new_wire.len() - 1;
            (0..len).for_each(|i| {
                new_wire.push_back(boundary[(top_index + i) % boundary.len()].clone());
            });
        }
        boundaries[shared_face_index.boundary_index] = new_wire;
        *shared_face = Face::new(boundaries, shared_face.oriented_surface())
    }

    shell.extend(fillet_faces);

    Ok(())
}

/// Closed-wire fillet face construction.
///
/// All faces are created uniformly using circular indexing — no special
/// "first" or "last" face since the wire wraps around.
fn fillet_along_wire_closed(
    shell: &mut Shell,
    _wire: &Wire,
    shared_face_index: FaceBoundaryEdgeIndex,
    adjacent_faces: &[FaceBoundaryEdgeIndex],
    fillet_surfaces: &[NurbsSurface<Vector4>],
) -> Result<()> {
    let n = fillet_surfaces.len();

    // Build fillet faces with circular [prev, curr, next] windowing.
    let mut fillet_faces = Shell::new();
    for i in 0..n {
        let prev = (i + n - 1) % n;
        let next = (i + 1) % n;
        let surfaces = [
            fillet_surfaces[prev].clone(),
            fillet_surfaces[i].clone(),
            fillet_surfaces[next].clone(),
        ];
        let fillet_surface = concat_fillet_surface(&surfaces);
        let edge0 = create_free_edge(surfaces[1].column_curve(0).into());

        let edge1 = cut_face_by_last_bezier(shell, adjacent_faces[i], &fillet_surface).ok_or(
            FilletError::GeometryFailed {
                context: "cut face by last bezier for closed fillet",
            },
        )?;

        let edge2 = {
            let (v0, v1) = (edge0.front(), edge1.back());
            let (u, v) = fillet_surface
                .search_parameter(v1.point(), (1.0, 1.0), 100)
                .ok_or(FilletError::GeometryFailed {
                    context: "search parameter for closed fillet edge2",
                })?;
            let param_line = Line((0.0, 1.0).into(), (u, v).into());
            let pcurve = ParamCurveLinear::new(param_line, fillet_surface.clone());
            Edge::new(v0, v1, pcurve.into())
        };

        let edge3 = {
            let (v0, v1) = (edge0.back(), edge1.front());
            let (u, v) = fillet_surface
                .search_parameter(v1.point(), (1.0, 2.0), 100)
                .ok_or(FilletError::GeometryFailed {
                    context: "search parameter for closed fillet edge3",
                })?;
            let param_line = Line((0.0, 2.0).into(), (u, v).into());
            let pcurve = ParamCurveLinear::new(param_line, fillet_surface.clone());
            Edge::new(v0, v1, pcurve.into())
        };

        let boundary = [edge0.inverse(), edge2, edge1.inverse(), edge3.inverse()].into();
        fillet_faces.push(Face::new(vec![boundary], fillet_surface));
    }

    // Rebuild the shared face boundary: replace the entire boundary with
    // a closed wire built from the fillet face edges.
    {
        let face_edges: Vec<_> = fillet_faces
            .face_iter()
            .map(|face| face.boundaries()[0][0].clone())
            .collect();

        let first_vertex = Vertex::new(face_edges[0].back().point());
        let mut previous_vertex = first_vertex.clone();
        let mut wire_edges = Vec::with_capacity(n);
        for (i, edge) in face_edges.iter().enumerate() {
            let v1 = previous_vertex.clone();
            let v0 = if i == n - 1 {
                first_vertex.clone()
            } else {
                Vertex::new(edge.front().point())
            };
            let new_edge = Edge::new(&v0, &v1, edge.oriented_curve());
            wire_edges.push(new_edge.inverse());
            previous_vertex = v0;
        }

        let new_wire: Wire = wire_edges.into();

        let shared_face = &mut shell[shared_face_index.face_index];
        let mut boundaries = shared_face.boundaries();
        boundaries[shared_face_index.boundary_index] = new_wire;
        *shared_face = Face::new(boundaries, shared_face.oriented_surface());
    }

    shell.extend(fillet_faces);

    Ok(())
}
