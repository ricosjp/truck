use std::collections::{HashMap, HashSet};

use monstertruck_geometry::prelude::{NurbsSurface, Point3, Vector4};
use monstertruck_topology::shell::ShellCondition;
use monstertruck_traits::{
    BoundedCurve, ParametricCurve, ParametricSurface, SearchNearestParameter,
};
use smallvec::SmallVec;

use super::convert::{FilletableCurve, FilletableSurface, convert_shell_in, convert_shell_out};
use super::error::FilletError;
use super::ops;
use super::params::{FilletOptions, RadiusSpec};
use super::types::*;

type Result<T> = std::result::Result<T, FilletError>;

/// Face list for an edge. Manifold edges always have exactly 2 faces,
/// so `SmallVec<[usize; 2]>` avoids heap allocation.
type FaceList = SmallVec<[usize; 2]>;

fn is_manifold_edge(edge_face_map: &HashMap<EdgeId, FaceList>, edge_id: EdgeId) -> bool {
    edge_face_map
        .get(&edge_id)
        .is_some_and(|faces| faces.len() == 2)
}

/// Builds a map from [`EdgeId`] to the face indices that contain it.
fn build_edge_face_map(shell: &Shell) -> HashMap<EdgeId, FaceList> {
    let mut map: HashMap<EdgeId, FaceList> = HashMap::new();
    shell.iter().enumerate().for_each(|(face_idx, face)| {
        face.edge_iter().for_each(|edge| {
            map.entry(edge.id())
                .and_modify(|v| {
                    if !v.contains(&face_idx) {
                        v.push(face_idx);
                    }
                })
                .or_insert_with(|| smallvec::smallvec![face_idx]);
        });
    });
    map
}

fn sampled_curve_distance_squared(curve: &Curve, point: Point3, sample_count: usize) -> f64 {
    let (t0, t1) = curve.range_tuple();
    (0..=sample_count)
        .map(|i| {
            let t = t0 + (t1 - t0) * (i as f64) / (sample_count as f64);
            squared_distance(curve.subs(t), point)
        })
        .fold(f64::INFINITY, f64::min)
}

fn sampled_variable_radius_upper_bound(radius: &dyn Fn(f64) -> f64, sample_count: usize) -> f64 {
    (0..=sample_count)
        .map(|i| radius(i as f64 / sample_count as f64))
        .fold(f64::NEG_INFINITY, f64::max)
}

fn rematch_selected_edge_id(
    shell: &Shell,
    original_curve: &Curve,
    used_ids: &HashSet<EdgeId>,
) -> Option<EdgeId> {
    let tolerance = 1.0e-6;
    let tolerance_squared = tolerance * tolerance;
    shell
        .edge_iter()
        .filter(|edge| !used_ids.contains(&edge.id()))
        .filter(|edge| {
            let (front, back) = edge.ends();
            sampled_curve_distance_squared(original_curve, front.point(), 24) < tolerance_squared
                && sampled_curve_distance_squared(original_curve, back.point(), 24)
                    < tolerance_squared
        })
        .max_by(|left, right| {
            approximate_edge_length(left, 8).total_cmp(&approximate_edge_length(right, 8))
        })
        .map(|edge| edge.id())
}

fn sampled_curve_surface_max_distance(
    curve: &Curve,
    surface: &NurbsSurface<Vector4>,
    sample_count: usize,
) -> f64 {
    let (t0, t1) = curve.range_tuple();
    (0..=sample_count)
        .map(|i| t0 + (t1 - t0) * (i as f64) / (sample_count as f64))
        .map(|t| curve.evaluate(t))
        .filter_map(|point| {
            surface
                .search_nearest_parameter(point, None, 50)
                .map(|(u, v)| squared_distance(surface.evaluate(u, v), point).sqrt())
        })
        .fold(0.0, f64::max)
}

/// Scans all face boundaries and extracts contiguous runs of selected edges.
///
/// Returns `(face_idx, ordered edge IDs)` for each maximal run.
fn collect_boundary_runs(shell: &Shell, selected: &HashSet<EdgeId>) -> Vec<(usize, Vec<EdgeId>)> {
    let mut candidate_runs = Vec::new();

    for (face_idx, face) in shell.iter().enumerate() {
        for boundary in face.boundary_iters() {
            let boundary_edges: Vec<Edge> = boundary.collect();
            let n = boundary_edges.len();
            if n == 0 {
                continue;
            }

            let is_selected: Vec<bool> = boundary_edges
                .iter()
                .map(|e| selected.contains(&e.id()))
                .collect();

            let boundary_closed =
                n >= 2 && boundary_edges[0].front() == boundary_edges[n - 1].back();

            if boundary_closed {
                // Find a non-selected edge to start from so runs don't
                // get split at the wrap-around seam.
                if let Some(start) = is_selected.iter().position(|&s| !s) {
                    let mut run: Vec<EdgeId> = Vec::new();
                    for offset in 1..=n {
                        let idx = (start + offset) % n;
                        if is_selected[idx] {
                            run.push(boundary_edges[idx].id());
                        } else if !run.is_empty() {
                            candidate_runs.push((face_idx, std::mem::take(&mut run)));
                        }
                    }
                    if !run.is_empty() {
                        candidate_runs.push((face_idx, run));
                    }
                } else {
                    // ALL edges selected → one closed run covering entire boundary.
                    let ids: Vec<EdgeId> = boundary_edges.iter().map(|e| e.id()).collect();
                    candidate_runs.push((face_idx, ids));
                }
            } else {
                // Open boundary: linear scan, split on non-selected gaps.
                let mut run: Vec<EdgeId> = Vec::new();
                for (i, &sel) in is_selected.iter().enumerate() {
                    if sel {
                        run.push(boundary_edges[i].id());
                    } else if !run.is_empty() {
                        candidate_runs.push((face_idx, std::mem::take(&mut run)));
                    }
                }
                if !run.is_empty() {
                    candidate_runs.push((face_idx, run));
                }
            }
        }
    }

    candidate_runs
}

/// An edge chain: a contiguous run of selected edges on a single face boundary.
///
/// `shared_face_idx` is the face whose boundary contains these edges.
/// The other face index is resolved at processing time (it may shift as
/// earlier chains mutate the shell).
#[derive(Debug)]
struct Chain {
    edge_ids: Vec<EdgeId>,
    shared_face_idx: usize,
}

/// Groups requested edges into chains by finding contiguous runs of selected
/// edges along each face boundary.
///
/// For each face boundary, maximal contiguous runs of selected edges are
/// extracted. Each selected edge appears in exactly two candidate runs (one per
/// adjacent face). The edge is assigned to the longer run (tiebreak: lower face
/// index). Final chains are sorted longest-first for processing stability.
fn group_edges_into_chains(
    shell: &Shell,
    edge_ids: &[EdgeId],
    edge_face_map: &HashMap<EdgeId, FaceList>,
) -> Result<Vec<Chain>> {
    // 1. Validate all edges are manifold.
    for &eid in edge_ids {
        let faces = edge_face_map.get(&eid).ok_or(FilletError::EdgeNotFound)?;
        if faces.len() != 2 {
            return Err(FilletError::NonManifoldEdge(faces.len()));
        }
    }

    let selected: HashSet<EdgeId> = edge_ids.iter().copied().collect();

    // 2. For each face boundary, find contiguous runs of selected edges.
    let candidate_runs = collect_boundary_runs(shell, &selected);

    // 3. Each selected edge appears in exactly 2 candidate runs (one per face).
    //    Assign each edge to its longest run (tiebreak: lower face_idx).
    let mut edge_best_run: HashMap<EdgeId, usize> = HashMap::new();
    for (run_idx, (face_idx, run_edges)) in candidate_runs.iter().enumerate() {
        for &eid in run_edges {
            let replace = match edge_best_run.get(&eid) {
                None => true,
                Some(&prev_idx) => {
                    let prev_len = candidate_runs[prev_idx].1.len();
                    let this_len = run_edges.len();
                    this_len > prev_len
                        || (this_len == prev_len && *face_idx < candidate_runs[prev_idx].0)
                }
            };
            if replace {
                edge_best_run.insert(eid, run_idx);
            }
        }
    }

    // 4. Collect final chains: process runs longest-first, claiming unclaimed
    //    edges and splitting on gaps where edges were already claimed.
    let mut run_order: Vec<usize> = (0..candidate_runs.len()).collect();
    run_order.sort_by(|&a, &b| {
        candidate_runs[b]
            .1
            .len()
            .cmp(&candidate_runs[a].1.len())
            .then(candidate_runs[a].0.cmp(&candidate_runs[b].0))
    });

    let mut globally_claimed: HashSet<EdgeId> = HashSet::new();
    let mut chains: Vec<Chain> = Vec::new();

    for run_idx in run_order {
        let (face_idx, ref run_edges) = candidate_runs[run_idx];
        // Split this run on already-claimed edges, producing sub-chains.
        let mut current_run: Vec<EdgeId> = Vec::new();
        for &eid in run_edges {
            if globally_claimed.contains(&eid) {
                if !current_run.is_empty() {
                    chains.push(Chain {
                        edge_ids: std::mem::take(&mut current_run),
                        shared_face_idx: face_idx,
                    });
                }
            } else {
                current_run.push(eid);
            }
        }
        if !current_run.is_empty() {
            chains.push(Chain {
                edge_ids: current_run,
                shared_face_idx: face_idx,
            });
        }
        // Mark all original run edges as globally claimed.
        globally_claimed.extend(run_edges);
    }

    // 5. Sort final chains: longest first, then shared_face_idx for determinism.
    chains.sort_by(|a, b| {
        b.edge_ids
            .len()
            .cmp(&a.edge_ids.len())
            .then(a.shared_face_idx.cmp(&b.shared_face_idx))
    });

    Ok(chains)
}

/// Finds the side face adjacent to a given vertex in a chain, if any.
///
/// A side face is one that shares the vertex and an edge with one of the two
/// main faces but is not itself one of the main faces.
fn find_side_face(
    shell: &Shell,
    face_a_idx: usize,
    face_b_idx: usize,
    vertex_id: VertexId,
    edge_face_map: &HashMap<EdgeId, FaceList>,
) -> Option<usize> {
    // Find edges incident to vertex_id in face_a or face_b.
    [face_a_idx, face_b_idx]
        .iter()
        .flat_map(|&fi| {
            shell[fi].edge_iter().filter_map(move |edge| {
                let (front, back) = edge.ends();
                if front.id() == vertex_id || back.id() == vertex_id {
                    edge_face_map.get(&edge.id()).and_then(|faces| {
                        faces
                            .iter()
                            .find(|&&f| f != face_a_idx && f != face_b_idx)
                            .copied()
                    })
                } else {
                    None
                }
            })
        })
        .next()
}

fn apply_single_edge_fillet(
    shell: &mut Shell,
    edge_id: EdgeId,
    preferred_shared_face_idx: Option<usize>,
    options: &FilletOptions,
) -> Result<()> {
    let edge_face_map = build_edge_face_map(shell);
    let faces = edge_face_map
        .get(&edge_id)
        .ok_or(FilletError::EdgeNotFound)?;
    if faces.len() != 2 {
        if std::env::var_os("MT_FILLET_DEBUG").is_some() {
            eprintln!(
                "debug fillet edge {:?}: skipped non-manifold candidate ({} faces).",
                edge_id,
                faces.len()
            );
        }
        return Ok(());
    }

    let face_a_idx = preferred_shared_face_idx
        .filter(|idx| faces.contains(idx))
        .unwrap_or(faces[0]);
    let face_b_idx = *faces
        .iter()
        .find(|&&f| f != face_a_idx)
        .ok_or(FilletError::EdgeNotFound)?;

    let face_a = shell[face_a_idx].clone();
    let face_b = shell[face_b_idx].clone();
    let edge = face_a
        .edge_iter()
        .find(|edge| edge.id() == edge_id)
        .ok_or(FilletError::EdgeNotFound)?;
    let (front_vertex_id, back_vertex_id) = {
        let (front, back) = edge.ends();
        (front.id(), back.id())
    };

    let side0_idx = find_side_face(
        shell,
        face_a_idx,
        face_b_idx,
        front_vertex_id,
        &edge_face_map,
    );
    let side1_idx = find_side_face(
        shell,
        face_a_idx,
        face_b_idx,
        back_vertex_id,
        &edge_face_map,
    );

    let side0 = side0_idx.map(|idx| shell[idx].clone());
    let side1 = side1_idx.map(|idx| shell[idx].clone());

    let fillet_result = ops::fillet_with_side(
        &face_a,
        &face_b,
        edge_id,
        side0.as_ref(),
        side1.as_ref(),
        options,
    );
    let (new_face_a, new_face_b, fillet, new_side0, new_side1) = match fillet_result {
        Ok(result) => result,
        Err(error) => {
            if std::env::var_os("MT_FILLET_DEBUG").is_some() {
                let curve = edge.oriented_curve();
                let surface_a = face_a.oriented_surface();
                let surface_b = face_b.oriented_surface();
                let drift_a = sampled_curve_surface_max_distance(&curve, &surface_a, 16);
                let drift_b = sampled_curve_surface_max_distance(&curve, &surface_b, 16);
                eprintln!(
                    "debug fillet edge {:?}: drift_a={:.3e}, drift_b={:.3e}, error={error}",
                    edge_id, drift_a, drift_b
                );
            }
            return Err(error);
        }
    };

    shell[face_a_idx] = new_face_a;
    shell[face_b_idx] = new_face_b;
    if let Some(new_side) = new_side0
        && let Some(idx) = side0_idx
    {
        shell[idx] = new_side;
    }
    if let Some(new_side) = new_side1
        && let Some(idx) = side1_idx
    {
        shell[idx] = new_side;
    }
    shell.push(fillet);
    Ok(())
}

/// Fillets the specified edges of a shell.
///
/// Resolves face adjacency automatically and dispatches to
/// [`fillet`](super::fillet)/[`fillet_with_side`](super::fillet_with_side)
/// for single edges or [`fillet_along_wire`](super::fillet_along_wire) for multi-edge chains.
pub fn fillet_edges(
    shell: &mut Shell,
    edge_ids: &[EdgeId],
    params: Option<&FilletOptions>,
) -> Result<()> {
    let default_options = FilletOptions::default();
    let options = params.unwrap_or(&default_options);

    // Validate all requested edges exist and are manifold up-front.
    {
        let edge_face_map = build_edge_face_map(shell);
        for &eid in edge_ids {
            let faces = edge_face_map.get(&eid).ok_or(FilletError::EdgeNotFound)?;
            if faces.len() != 2 {
                return Err(FilletError::NonManifoldEdge(faces.len()));
            }
        }
    }

    // Validate per-edge radius count.
    if let RadiusSpec::PerEdge(ref radii) = options.radius
        && radii.len() != edge_ids.len()
    {
        return Err(FilletError::PerEdgeRadiusMismatch {
            given: radii.len(),
            expected: edge_ids.len(),
        });
    }

    // Reject edges that are too short for the requested fillet radius.
    {
        let variable_radius_upper_bound = if let RadiusSpec::Variable(f) = &options.radius {
            sampled_variable_radius_upper_bound(f.as_ref(), 32)
        } else {
            0.0
        };
        for (i, &eid) in edge_ids.iter().enumerate() {
            let effective_radius = match &options.radius {
                RadiusSpec::Constant(r) => *r,
                RadiusSpec::Variable(_) => variable_radius_upper_bound,
                RadiusSpec::PerEdge(radii) => radii[i],
            };
            let edge = shell
                .edge_iter()
                .find(|e| e.id() == eid)
                .ok_or(FilletError::EdgeNotFound)?;
            let length = approximate_edge_length(&edge, 16);
            if length < 2.0 * effective_radius {
                return Err(FilletError::DegenerateEdge);
            }
        }
    }

    // Group edges into chains (by face pair topology). Chain stores only edge IDs.
    let chains = {
        let edge_face_map = build_edge_face_map(shell);
        group_edges_into_chains(shell, edge_ids, &edge_face_map)?
    };

    // Build EdgeId→index map for PerEdge radius lookup.
    let edge_id_to_idx: HashMap<EdgeId, usize> = edge_ids
        .iter()
        .enumerate()
        .map(|(i, &eid)| (eid, i))
        .collect();
    let original_edge_curves: HashMap<EdgeId, Curve> = edge_ids
        .iter()
        .copied()
        .map(|eid| {
            shell
                .edge_iter()
                .find(|edge| edge.id() == eid)
                .map(|edge| (eid, edge.oriented_curve()))
                .ok_or(FilletError::EdgeNotFound)
        })
        .collect::<Result<_>>()?;

    // Process each chain with a fresh edge-face map, so that mutations from
    // earlier chains don't cause stale face indices.
    for chain in &chains {
        let shell_checkpoint = shell.clone();
        let edge_face_map = build_edge_face_map(shell);
        let mut used_ids = HashSet::<EdgeId>::new();
        let mut resolved_pairs: Vec<(EdgeId, EdgeId)> = Vec::new();
        for original_eid in chain.edge_ids.iter().copied() {
            let resolved_eid = if is_manifold_edge(&edge_face_map, original_eid)
                && !used_ids.contains(&original_eid)
            {
                Some(original_eid)
            } else {
                original_edge_curves
                    .get(&original_eid)
                    .and_then(|original_curve| {
                        rematch_selected_edge_id(shell, original_curve, &used_ids)
                    })
            };
            if let Some(resolved_eid) = resolved_eid {
                used_ids.insert(resolved_eid);
                resolved_pairs.push((original_eid, resolved_eid));
            } else if std::env::var_os("MT_FILLET_DEBUG").is_some() {
                eprintln!(
                    "debug fillet chain {:?}: unresolved edge {:?}, skipping.",
                    chain.edge_ids, original_eid
                );
            }
        }
        if resolved_pairs.is_empty() {
            continue;
        }
        let resolved_edge_ids: Vec<EdgeId> = resolved_pairs
            .iter()
            .map(|(_, resolved_eid)| *resolved_eid)
            .collect();
        let resolved_to_original: HashMap<EdgeId, EdgeId> = resolved_pairs
            .iter()
            .map(|(original_eid, resolved_eid)| (*resolved_eid, *original_eid))
            .collect();

        // Resolve face_a (shared face from grouping) and face_b from current map.
        let Some(first_eid) = resolved_edge_ids.first().copied() else {
            continue;
        };
        let Some(faces) = edge_face_map.get(&first_eid) else {
            if std::env::var_os("MT_FILLET_DEBUG").is_some() {
                eprintln!(
                    "debug fillet chain {:?}: missing seed edge {:?}, skipping.",
                    chain.edge_ids, first_eid
                );
            }
            continue;
        };
        if faces.len() != 2 {
            if std::env::var_os("MT_FILLET_DEBUG").is_some() {
                eprintln!(
                    "debug fillet chain {:?}: skipped non-manifold seed edge {:?} ({} faces).",
                    chain.edge_ids,
                    first_eid,
                    faces.len()
                );
            }
            continue;
        }
        // face_a is the shared face from boundary-run grouping; face_b is the other.
        let face_a_idx = if faces.contains(&chain.shared_face_idx) {
            chain.shared_face_idx
        } else {
            faces[0]
        };

        if chain.edge_ids.len() == 1 {
            let chain_opts;
            let opts = if let RadiusSpec::PerEdge(radii) = &options.radius {
                let original_eid = chain.edge_ids[0];
                chain_opts = FilletOptions {
                    radius: RadiusSpec::Constant(radii[edge_id_to_idx[&original_eid]]),
                    divisions: options.divisions,
                    profile: options.profile.clone(),
                };
                &chain_opts
            } else {
                options
            };

            match apply_single_edge_fillet(shell, first_eid, Some(face_a_idx), opts) {
                Ok(()) => {}
                Err(FilletError::GeometryFailed { .. }) => continue,
                Err(error) => return Err(error),
            }
        } else {
            // Multi-edge chain: build a Wire from the shared face's boundary
            // to get edges in the correct orientation for that face.
            let chain_id_set: HashSet<EdgeId> = resolved_edge_ids.iter().copied().collect();
            let Some(wire): Option<Wire> =
                shell[face_a_idx]
                    .boundary_iters()
                    .into_iter()
                    .find_map(|boundary| {
                        let edges = boundary
                            .filter(|edge| chain_id_set.contains(&edge.id()))
                            .collect::<Vec<_>>();
                        (edges.len() == resolved_edge_ids.len()).then_some(edges.into())
                    })
            else {
                if std::env::var_os("MT_FILLET_DEBUG").is_some() {
                    eprintln!(
                        "debug fillet chain {:?}: failed to construct wire, skipping.",
                        chain.edge_ids
                    );
                }
                continue;
            };

            let chain_opts;
            let opts = if let RadiusSpec::PerEdge(radii) = &options.radius {
                let chain_radii: Vec<f64> = wire
                    .edge_iter()
                    .map(|edge| {
                        resolved_to_original
                            .get(&edge.id())
                            .copied()
                            .ok_or(FilletError::EdgeNotFound)
                            .map(|eid| radii[edge_id_to_idx[&eid]])
                    })
                    .collect::<Result<_>>()?;
                chain_opts = FilletOptions {
                    radius: RadiusSpec::PerEdge(chain_radii),
                    divisions: options.divisions,
                    profile: options.profile.clone(),
                };
                &chain_opts
            } else {
                options
            };

            let wire_fillet_failed = match ops::fillet_along_wire(shell, &wire, opts) {
                Ok(()) => false,
                Err(FilletError::FilletSurfaceComputationFailed)
                | Err(FilletError::GeometryFailed { .. })
                | Err(FilletError::SharedFaceNotFound)
                | Err(FilletError::AdjacentFacesNotFound)
                | Err(FilletError::DiscontinuousWire) => true,
                Err(error) => return Err(error),
            };

            if wire_fillet_failed {
                let original_to_resolved: HashMap<EdgeId, EdgeId> =
                    resolved_pairs.iter().copied().collect();
                let mut fallback_used_ids = HashSet::<EdgeId>::new();
                let mut chain_failed = false;
                for original_eid in chain.edge_ids.iter().copied() {
                    let edge_face_map = build_edge_face_map(shell);
                    let initial_eid = original_to_resolved
                        .get(&original_eid)
                        .copied()
                        .unwrap_or(original_eid);
                    let resolved_eid = if is_manifold_edge(&edge_face_map, initial_eid)
                        && !fallback_used_ids.contains(&initial_eid)
                    {
                        initial_eid
                    } else {
                        let Some(original_curve) = original_edge_curves.get(&original_eid) else {
                            chain_failed = true;
                            break;
                        };
                        let Some(rematched) =
                            rematch_selected_edge_id(shell, original_curve, &fallback_used_ids)
                        else {
                            chain_failed = true;
                            break;
                        };
                        rematched
                    };
                    fallback_used_ids.insert(resolved_eid);

                    let single_edge_opts;
                    let opts = if let RadiusSpec::PerEdge(radii) = &options.radius {
                        single_edge_opts = FilletOptions {
                            radius: RadiusSpec::Constant(radii[edge_id_to_idx[&original_eid]]),
                            divisions: options.divisions,
                            profile: options.profile.clone(),
                        };
                        &single_edge_opts
                    } else {
                        options
                    };
                    match apply_single_edge_fillet(
                        shell,
                        resolved_eid,
                        Some(chain.shared_face_idx),
                        opts,
                    ) {
                        Ok(()) => {}
                        Err(FilletError::GeometryFailed { .. }) => {
                            chain_failed = true;
                            break;
                        }
                        Err(error) => return Err(error),
                    }
                }
                if chain_failed {
                    *shell = shell_checkpoint;
                    continue;
                }
            }
        }
    }
    Ok(())
}

/// Fillets the specified edges of a shell with arbitrary curve/surface types.
///
/// Converts the shell to internal NURBS types, runs [`fillet_edges`], and
/// converts back. This is the main entry point for external callers whose
/// shells use types like `monstertruck_modeling::Curve` / `monstertruck_modeling::Surface`.
pub fn fillet_edges_generic<C, S>(
    shell: &mut monstertruck_topology::Shell<Point3, C, S>,
    edges: &[monstertruck_topology::Edge<Point3, C>],
    params: Option<&FilletOptions>,
) -> Result<()>
where
    C: FilletableCurve,
    S: FilletableSurface,
{
    let default_options = FilletOptions::default();
    let options = params.unwrap_or(&default_options);
    let (mut internal_shell, internal_edge_ids) = convert_shell_in(shell, edges)?;
    let original_shell = internal_shell.clone();
    fillet_edges(&mut internal_shell, &internal_edge_ids, Some(options))?;
    if internal_shell.shell_condition() != ShellCondition::Closed {
        if std::env::var_os("MT_FILLET_DEBUG").is_some() {
            eprintln!("debug fillet generic: rollback to original shell (non-closed result).");
        }
        internal_shell = original_shell;
    }
    *shell = convert_shell_out(&internal_shell)?;
    Ok(())
}
