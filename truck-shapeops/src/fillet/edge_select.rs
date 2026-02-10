use std::collections::HashMap;

use truck_geometry::prelude::Point3;

use super::convert::{convert_shell_in, convert_shell_out, FilletableCurve, FilletableSurface};
use super::error::FilletError;
use super::ops;
use super::params::FilletOptions;
use super::types::*;

type Result<T> = std::result::Result<T, FilletError>;

/// Builds a map from [`EdgeID`] to the face indices that contain it.
fn build_edge_face_map(shell: &Shell) -> HashMap<EdgeID, Vec<usize>> {
    let mut map: HashMap<EdgeID, Vec<usize>> = HashMap::new();
    shell.iter().enumerate().for_each(|(face_idx, face)| {
        face.edge_iter().for_each(|edge| {
            map.entry(edge.id())
                .and_modify(|v| {
                    if !v.contains(&face_idx) {
                        v.push(face_idx);
                    }
                })
                .or_insert_with(|| vec![face_idx]);
        });
    });
    map
}

/// An edge chain: a sequence of edge IDs sharing the same face pair.
///
/// Face indices are resolved at processing time, not at grouping time,
/// because earlier chains may mutate the shell and invalidate indices.
#[derive(Debug)]
struct Chain {
    edge_ids: Vec<EdgeID>,
}

/// Groups requested edges into chains of connected edges sharing the same face pair.
fn group_edges_into_chains(
    shell: &Shell,
    edge_ids: &[EdgeID],
    edge_face_map: &HashMap<EdgeID, Vec<usize>>,
) -> Result<Vec<Chain>> {
    // Group edges by their (sorted) face pair.
    let mut pair_groups: HashMap<(usize, usize), Vec<EdgeID>> = HashMap::new();
    for &eid in edge_ids {
        let faces = edge_face_map.get(&eid).ok_or(FilletError::EdgeNotFound)?;
        if faces.len() != 2 {
            return Err(FilletError::NonManifoldEdge(faces.len()));
        }
        let key = if faces[0] < faces[1] {
            (faces[0], faces[1])
        } else {
            (faces[1], faces[0])
        };
        pair_groups.entry(key).or_default().push(eid);
    }

    // For each face pair group, build connected chains by vertex connectivity.
    let mut chains = Vec::new();
    for ((_, _), group_eids) in pair_groups {
        // Build adjacency: vertex_id -> list of (edge_id, other_vertex_id).
        let mut vert_adj: HashMap<VertexID, Vec<(EdgeID, VertexID)>> = HashMap::new();
        let mut edge_map: HashMap<EdgeID, Edge> = HashMap::new();
        for &eid in &group_eids {
            // Find the actual edge in the shell.
            let edge = shell
                .edge_iter()
                .find(|e| e.id() == eid)
                .ok_or(FilletError::EdgeNotFound)?;
            let (front, back) = edge.ends();
            let (fid, bid) = (front.id(), back.id());
            vert_adj.entry(fid).or_default().push((eid, bid));
            vert_adj.entry(bid).or_default().push((eid, fid));
            edge_map.insert(eid, edge);
        }

        // Walk chains: find endpoints (degree 1) or arbitrary start.
        let mut used: HashMap<EdgeID, bool> = group_eids.iter().map(|&e| (e, false)).collect();
        while let Some(&start_eid) = used.iter().find(|(_, &v)| !v).map(|(k, _)| k) {
            // Find an endpoint to start from (vertex with degree 1 in our subgraph).
            let start_edge = &edge_map[&start_eid];
            let (front, back) = start_edge.ends();
            // Prefer starting at a vertex with degree 1 in the remaining unused edges.
            let unused_degree = |vid: VertexID| {
                vert_adj.get(&vid).map_or(0, |v| {
                    v.iter()
                        .filter(|(e, _)| !*used.get(e).unwrap_or(&true))
                        .count()
                })
            };
            let start_vid = if unused_degree(front.id()) <= unused_degree(back.id()) {
                front.id()
            } else {
                back.id()
            };

            let mut chain_eids = Vec::new();
            let mut current_vid = start_vid;
            loop {
                let next = vert_adj
                    .get(&current_vid)
                    .and_then(|neighbors| neighbors.iter().find(|(eid, _)| !used[eid]).copied());
                match next {
                    Some((eid, next_vid)) => {
                        *used.get_mut(&eid).unwrap() = true;
                        chain_eids.push(eid);
                        current_vid = next_vid;
                    }
                    None => break,
                }
            }
            if !chain_eids.is_empty() {
                chains.push(Chain {
                    edge_ids: chain_eids,
                });
            }
        }
    }
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
    edge_face_map: &HashMap<EdgeID, Vec<usize>>,
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
/// [`simple_fillet`](super::simple_fillet)/[`fillet_with_side`](super::fillet_with_side)
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

    // Group edges into chains (by face pair topology). Chain stores only edge IDs.
    let chains = {
        let edge_face_map = build_edge_face_map(shell);
        group_edges_into_chains(shell, edge_ids, &edge_face_map)?
    };

    // Process each chain with a fresh edge-face map, so that mutations from
    // earlier chains don't cause stale face indices.
    for chain in &chains {
        let edge_face_map = build_edge_face_map(shell);

        // Resolve face_a and face_b from the first edge in this chain.
        let first_eid = chain.edge_ids[0];
        let faces = edge_face_map
            .get(&first_eid)
            .ok_or(FilletError::EdgeNotFound)?;
        if faces.len() != 2 {
            return Err(FilletError::NonManifoldEdge(faces.len()));
        }
        let (face_a_idx, face_b_idx) = if faces[0] < faces[1] {
            (faces[0], faces[1])
        } else {
            (faces[1], faces[0])
        };

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

            let (new_face_a, new_face_b, fillet, new_side0, new_side1) = ops::fillet_with_side(
                face_a,
                face_b,
                eid,
                side0.as_ref(),
                side1.as_ref(),
                options,
            )?;

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
            // Multi-edge chain: build a Wire and use fillet_along_wire.
            let wire: Wire = chain
                .edge_ids
                .iter()
                .filter_map(|&eid| shell.edge_iter().find(|e| e.id() == eid))
                .collect();

            ops::fillet_along_wire(shell, &wire, options)?;
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
