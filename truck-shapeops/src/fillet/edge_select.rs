use std::collections::{HashMap, HashSet};

use smallvec::SmallVec;
use truck_geometry::prelude::Point3;
use truck_geotrait::{BoundedCurve, ParametricCurve};

use super::convert::{convert_shell_in, convert_shell_out, FilletableCurve, FilletableSurface};
use super::error::FilletError;
use super::ops;
use super::params::{FilletOptions, RadiusSpec};
use super::types::*;

type Result<T> = std::result::Result<T, FilletError>;

/// Face list for an edge. Manifold edges always have exactly 2 faces,
/// so `SmallVec<[usize; 2]>` avoids heap allocation.
type FaceList = SmallVec<[usize; 2]>;

/// Builds a map from [`EdgeID`] to the face indices that contain it.
fn build_edge_face_map(shell: &Shell) -> HashMap<EdgeID, FaceList> {
    let mut map: HashMap<EdgeID, FaceList> = HashMap::new();
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

/// Scans all face boundaries and extracts contiguous runs of selected edges.
///
/// Returns `(face_idx, ordered edge IDs)` for each maximal run.
fn collect_boundary_runs(shell: &Shell, selected: &HashSet<EdgeID>) -> Vec<(usize, Vec<EdgeID>)> {
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
                    let mut run: Vec<EdgeID> = Vec::new();
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
                    let ids: Vec<EdgeID> = boundary_edges.iter().map(|e| e.id()).collect();
                    candidate_runs.push((face_idx, ids));
                }
            } else {
                // Open boundary: linear scan, split on non-selected gaps.
                let mut run: Vec<EdgeID> = Vec::new();
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
    edge_ids: Vec<EdgeID>,
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
    edge_ids: &[EdgeID],
    edge_face_map: &HashMap<EdgeID, FaceList>,
) -> Result<Vec<Chain>> {
    // 1. Validate all edges are manifold.
    for &eid in edge_ids {
        let faces = edge_face_map.get(&eid).ok_or(FilletError::EdgeNotFound)?;
        if faces.len() != 2 {
            return Err(FilletError::NonManifoldEdge(faces.len()));
        }
    }

    let selected: HashSet<EdgeID> = edge_ids.iter().copied().collect();

    // 2. For each face boundary, find contiguous runs of selected edges.
    let candidate_runs = collect_boundary_runs(shell, &selected);

    // 3. Each selected edge appears in exactly 2 candidate runs (one per face).
    //    Assign each edge to its longest run (tiebreak: lower face_idx).
    let mut edge_best_run: HashMap<EdgeID, usize> = HashMap::new();
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

    let mut globally_claimed: HashSet<EdgeID> = HashSet::new();
    let mut chains: Vec<Chain> = Vec::new();

    for run_idx in run_order {
        let (face_idx, ref run_edges) = candidate_runs[run_idx];
        // Split this run on already-claimed edges, producing sub-chains.
        let mut current_run: Vec<EdgeID> = Vec::new();
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
    vertex_id: VertexID,
    edge_face_map: &HashMap<EdgeID, FaceList>,
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

/// Fillets the specified edges of a shell.
///
/// Resolves face adjacency automatically and dispatches to
/// [`fillet`](super::fillet)/[`fillet_with_side`](super::fillet_with_side)
/// for single edges or [`fillet_along_wire`](super::fillet_along_wire) for multi-edge chains.
pub fn fillet_edges(
    shell: &mut Shell,
    edge_ids: &[EdgeID],
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
    if let RadiusSpec::PerEdge(ref radii) = options.radius {
        if radii.len() != edge_ids.len() {
            return Err(FilletError::PerEdgeRadiusMismatch {
                given: radii.len(),
                expected: edge_ids.len(),
            });
        }
    }

    // Reject edges that are too short for the requested fillet radius.
    {
        for (i, &eid) in edge_ids.iter().enumerate() {
            let effective_radius = match &options.radius {
                RadiusSpec::Constant(r) => *r,
                RadiusSpec::Variable(f) => f(0.5),
                RadiusSpec::PerEdge(radii) => radii[i],
            };
            let edge = shell
                .edge_iter()
                .find(|e| e.id() == eid)
                .ok_or(FilletError::EdgeNotFound)?;
            let curve = edge.curve();
            let (t0, t1) = curve.range_tuple();
            let n = 10usize;
            let mut length = 0.0f64;
            let mut prev = curve.subs(t0);
            for i in 1..=n {
                let t = t0 + (t1 - t0) * (i as f64) / (n as f64);
                let pt = curve.subs(t);
                let d = pt - prev;
                length += (d.x * d.x + d.y * d.y + d.z * d.z).sqrt();
                prev = pt;
            }
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

    // Build EdgeID→index map for PerEdge radius lookup.
    let edge_id_to_idx: HashMap<EdgeID, usize> = edge_ids
        .iter()
        .enumerate()
        .map(|(i, &eid)| (eid, i))
        .collect();

    // Process each chain with a fresh edge-face map, so that mutations from
    // earlier chains don't cause stale face indices.
    for chain in &chains {
        let edge_face_map = build_edge_face_map(shell);

        // Resolve face_a (shared face from grouping) and face_b from current map.
        let first_eid = chain.edge_ids[0];
        let faces = edge_face_map
            .get(&first_eid)
            .ok_or(FilletError::EdgeNotFound)?;
        if faces.len() != 2 {
            return Err(FilletError::NonManifoldEdge(faces.len()));
        }
        // face_a is the shared face from boundary-run grouping; face_b is the other.
        let face_a_idx = chain.shared_face_idx;
        let face_b_idx = *faces
            .iter()
            .find(|&&f| f != face_a_idx)
            .ok_or(FilletError::EdgeNotFound)?;

        if chain.edge_ids.len() == 1 {
            // Single-edge: use fillet_with_side.
            let eid = first_eid;
            let face_a = &shell[face_a_idx];
            let face_b = &shell[face_b_idx];

            // Find the actual edge to get endpoint vertices.
            let edge = face_a
                .edge_iter()
                .find(|e| e.id() == eid)
                .ok_or(FilletError::EdgeNotFound)?;
            let (v_front, v_back) = edge.ends();

            let side0_idx =
                find_side_face(shell, face_a_idx, face_b_idx, v_front.id(), &edge_face_map);
            let side1_idx =
                find_side_face(shell, face_a_idx, face_b_idx, v_back.id(), &edge_face_map);

            let side0 = side0_idx.map(|i| shell[i].clone());
            let side1 = side1_idx.map(|i| shell[i].clone());

            let chain_opts;
            let opts = if let RadiusSpec::PerEdge(radii) = &options.radius {
                chain_opts = FilletOptions {
                    radius: RadiusSpec::Constant(radii[edge_id_to_idx[&eid]]),
                    divisions: options.divisions,
                    profile: options.profile.clone(),
                };
                &chain_opts
            } else {
                options
            };

            let (new_face_a, new_face_b, fillet, new_side0, new_side1) =
                ops::fillet_with_side(face_a, face_b, eid, side0.as_ref(), side1.as_ref(), opts)?;

            shell[face_a_idx] = new_face_a;
            shell[face_b_idx] = new_face_b;
            if let Some(ns0) = new_side0 {
                if let Some(idx) = side0_idx {
                    shell[idx] = ns0;
                }
            }
            if let Some(ns1) = new_side1 {
                if let Some(idx) = side1_idx {
                    shell[idx] = ns1;
                }
            }
            shell.push(fillet);
        } else {
            // Multi-edge chain: build a Wire from the shared face's boundary
            // to get edges in the correct orientation for that face.
            let chain_id_set: HashSet<EdgeID> = chain.edge_ids.iter().copied().collect();
            let wire: Wire = shell[face_a_idx]
                .boundary_iters()
                .into_iter()
                .flatten()
                .filter(|e| chain_id_set.contains(&e.id()))
                .collect();

            let chain_opts;
            let opts = if let RadiusSpec::PerEdge(radii) = &options.radius {
                let chain_radii: Vec<f64> = chain
                    .edge_ids
                    .iter()
                    .map(|eid| radii[edge_id_to_idx[eid]])
                    .collect();
                let n = chain_radii.len();
                chain_opts = FilletOptions {
                    radius: RadiusSpec::Variable(Box::new(move |t: f64| {
                        let idx = ((t * n as f64).floor() as usize).min(n - 1);
                        chain_radii[idx]
                    })),
                    divisions: options.divisions,
                    profile: options.profile.clone(),
                };
                &chain_opts
            } else {
                options
            };

            ops::fillet_along_wire(shell, &wire, opts)?;
        }
    }
    Ok(())
}

/// Fillets the specified edges of a shell with arbitrary curve/surface types.
///
/// Converts the shell to internal NURBS types, runs [`fillet_edges`], and
/// converts back. This is the main entry point for external callers whose
/// shells use types like `truck_modeling::Curve` / `truck_modeling::Surface`.
pub fn fillet_edges_generic<C, S>(
    shell: &mut truck_topology::Shell<Point3, C, S>,
    edges: &[truck_topology::Edge<Point3, C>],
    params: Option<&FilletOptions>,
) -> Result<()>
where
    C: FilletableCurve,
    S: FilletableSurface,
{
    let default_options = FilletOptions::default();
    let options = params.unwrap_or(&default_options);
    let (mut internal_shell, internal_edge_ids) = convert_shell_in(shell, edges)?;
    fillet_edges(&mut internal_shell, &internal_edge_ids, Some(options))?;
    *shell = convert_shell_out(&internal_shell)?;
    Ok(())
}
