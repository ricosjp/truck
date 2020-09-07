use crate::Error;
use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Read, Write};
use topology::*;
use std::result::Result;

/// the wrapped up topological shell data.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct WrappedUpShell {
    pub number_of_vertices: usize,
    pub edges: Vec<(usize, usize)>,
    pub faces: Vec<(bool, bool, Vec<usize>)>,
}

fn get_vertex<'a, P>(
    vertex: &Vertex<P>,
    vmap: &mut HashMap<VertexID<P>, usize>,
    vertices: &mut Vec<VertexID<P>>,
) -> usize
{
    if let Some(idx) = vmap.get(&vertex.id()) {
        *idx
    } else {
        let idx = vmap.len();
        vmap.insert(vertex.id(), idx);
        vertices.push(vertex.id());
        idx
    }
}

fn get_edge<'a, P, C>(
    edge: &'a Edge<P, C>,
    vmap: &mut HashMap<VertexID<P>, usize>,
    vertices: &mut Vec<VertexID<P>>,
    emap: &mut HashMap<EdgeID<C>, usize>,
    edges: &mut Vec<(usize, usize)>,
) -> usize
{
    if let Some(idx) = emap.get(&edge.id()) {
        *idx
    } else {
        let front = get_vertex(edge.absolute_front(), vmap, vertices);
        let back = get_vertex(edge.absolute_back(), vmap, vertices);
        let idx = emap.len();
        emap.insert(edge.id(), idx);
        edges.push((front, back));
        idx
    }
}

fn wrap_up_face<P, C, S>(
    face: &Face<P, C, S>,
    vmap: &mut HashMap<VertexID<P>, usize>,
    vertices: &mut Vec<VertexID<P>>,
    emap: &mut HashMap<EdgeID<C>, usize>,
    edges: &mut Vec<(usize, usize)>,
) -> (bool, bool, Vec<usize>)
{
    let orientation = face.orientation();
    let edge = &face.absolute_boundary()[0];
    let first_orientation = edge.absolute_front() == edge.front();
    let vec = face
        .absolute_boundary()
        .edge_iter()
        .map(|edge| get_edge(edge, vmap, vertices, emap, edges))
        .collect();
    (orientation, first_orientation, vec)
}

fn wrap_up<P, C, S>(shell: &Shell<P, C, S>) -> WrappedUpShell {
    let mut vmap: HashMap<VertexID<P>, usize> = HashMap::new();
    let mut emap: HashMap<EdgeID<C>, usize> = HashMap::new();
    let mut vertices: Vec<VertexID<P>> = Vec::new();
    let mut edges: Vec<(usize, usize)> = Vec::new();
    let faces: Vec<(bool, bool, Vec<usize>)> = shell
        .face_iter()
        .map(|face| wrap_up_face(face, &mut vmap, &mut vertices, &mut emap, &mut edges))
        .collect();
    WrappedUpShell {
        number_of_vertices: vertices.len(),
        edges,
        faces,
    }
}

/// extract topology data
fn extract(topodata: &WrappedUpShell) -> Shell<(), (), ()> {
    let v = Vertex::news(&vec![(); topodata.number_of_vertices]);
    let edges: Vec<_> = topodata
        .edges
        .iter()
        .map(|(i, j)| Edge::new(&v[*i], &v[*j], ()))
        .collect();
    topodata.faces.iter().map(|(orientation, first_orientation, boundary_info)| {
        let mut wire = Wire::new();
        let mut iter = boundary_info.iter();
        let idx = *iter.next().unwrap();
        match *first_orientation {
            true => wire.push_back(edges[idx].clone()),
            false => wire.push_back(edges[idx].inverse()),
        }

        for idx in iter {
            match wire.back_vertex().unwrap() == edges[*idx].front() {
                true => wire.push_back(edges[*idx].clone()),
                false => wire.push_back(edges[*idx].inverse()),
            }
        }

        let mut face = Face::new(wire, ());
        if !orientation {
            face.invert();
        }
        face
    }).collect()
}

pub fn read<R: Read>(reader: R) -> Result<Shell<(), (), ()>, Error> {
    let wrappedup = serde_json::from_reader(BufReader::new(reader))?;
    let shell = extract(&wrappedup);
    Ok(shell)
}

pub fn write<W: Write, P, C, S>(shell: &Shell<P, C, S>, writer: W) -> Result<(), Error> {
    let writer = BufWriter::new(writer);
    let wrappedup = wrap_up(shell);
    Ok(serde_json::to_writer_pretty(writer, &wrappedup)?)
}
