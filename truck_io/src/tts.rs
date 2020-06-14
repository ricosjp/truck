use crate::Error;
use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Read, Write};
use topology::{Vertex, Edge, Wire, Face, Shell};

/// the wrapped up topological shell data.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct WrappedUpShell {
    pub number_of_vertices: usize,
    pub edges: Vec<(usize, usize)>,
    pub faces: Vec<(bool, Vec<usize>)>,
}

/// return the following hash maps:
/// * from vertex to the local id.
/// * from edge to (the local id, front vertex's local id, back vertex's local id)
fn create_vertex_edge_map(
    shell: &Shell,
) -> (
    HashMap<Vertex, usize>,
    HashMap<usize, (usize, usize, usize)>,
) {
    let mut vertices_map = HashMap::with_capacity(shell.face_iter().len());
    let mut edges_map = HashMap::with_capacity(shell.face_iter().len());

    let mut vertex_counter = 0;
    let mut edge_counter = 0;
    for face in shell.face_iter() {
        for edge in face.boundary().edge_iter() {
            let front_info = if let Some(idx) = vertices_map.get(&edge.front()) {
                *idx
            } else {
                vertices_map.insert(edge.front(), vertex_counter);
                vertex_counter += 1;
                vertex_counter - 1
            };
            let back_info = if let Some(idx) = vertices_map.get(&edge.back()) {
                *idx
            } else {
                vertices_map.insert(edge.back(), vertex_counter);
                vertex_counter += 1;
                vertex_counter - 1
            };

            if edges_map.get(&edge.id()).is_none() {
                edges_map.insert(edge.id(), (edge_counter, front_info, back_info));
                edge_counter += 1;
            }
        }
    }

    (vertices_map, edges_map)
}

/// wrap up the shell data to the topology data.
fn wrap_up(shell: &Shell) -> WrappedUpShell {
    let (vertices_map, edges_map) = create_vertex_edge_map(shell);

    let faces: Vec<(bool, Vec<usize>)> = shell
        .face_iter()
        .map(|face| {
            let mut wire_info = Vec::new();

            let mut edge_iter = face.boundary().edge_iter();
            let first_edge = edge_iter.next().unwrap();
            let (edge_info_id, front_info, _) = edges_map.get(&first_edge.id()).unwrap();
            wire_info.push(*edge_info_id);

            let vertex_info = vertices_map.get(&first_edge.front()).unwrap();
            let ori = front_info == vertex_info;

            for edge in edge_iter {
                let (edge_info_id, _, _) = edges_map.get(&edge.id()).unwrap();
                wire_info.push(*edge_info_id);
            }

            (ori, wire_info)
        })
        .collect();

    let mut edges_with_id: Vec<(usize, usize, usize)> =
        edges_map.into_iter().map(|(_, x)| x).collect();
    edges_with_id.sort_by(|(a, _, _), (b, _, _)| a.cmp(b));
    let edges: Vec<(usize, usize)> = edges_with_id.into_iter().map(|(_, f, b)| (f, b)).collect();

    WrappedUpShell {
        number_of_vertices: vertices_map.len(),
        edges: edges,
        faces: faces,
    }
}

/// extract topology data
fn extract(topodata: &WrappedUpShell) -> Shell {
    let v = Vertex::news(topodata.number_of_vertices);
    let mut edges = Vec::new();
    for (i, j) in &topodata.edges {
        edges.push(Edge::new(v[*i], v[*j]));
    }

    let mut shell = Shell::new();
    for (orient, wire_info) in &topodata.faces {
        let mut wire = Wire::new();
        let mut iter = wire_info.iter();
        let idx = *iter.next().unwrap();
        if *orient {
            wire.push_back(edges[idx]);
        } else {
            wire.push_back(edges[idx].inverse());
        }

        for idx in iter {
            if wire.back_vertex().unwrap() == edges[*idx].front() {
                wire.push_back(edges[*idx]);
            } else {
                wire.push_back(edges[*idx].inverse());
            }
        }

        shell.push(Face::new(wire));
    }

    shell
}

pub fn read<R: Read>(reader: R) -> Result<Shell, Error> {
    let wrappedup = serde_json::from_reader(BufReader::new(reader))?;
    let shell = extract(&wrappedup);
    Ok(shell)
}

pub fn write<W: Write>(shell: &Shell, writer: W) -> Result<(), Error> {
    let writer = BufWriter::new(writer);
    let wrappedup = wrap_up(shell);
    Ok(serde_json::to_writer_pretty(writer, &wrappedup)?)
}
