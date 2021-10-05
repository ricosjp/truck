use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct CompressedEdge<C> {
    vertices: (usize, usize),
    curve: C,
}

impl<C> CompressedEdge<C> {
    fn create_edge<P>(self, v: &[Vertex<P>]) -> Result<Edge<P, C>> {
        let front = &v[self.vertices.0];
        let back = &v[self.vertices.1];
        Edge::try_new(front, back, self.curve)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CompressedFace<S> {
    boundaries: Vec<Vec<(usize, bool)>>,
    orientation: bool,
    surface: S,
}

impl<S> CompressedFace<S> {
    fn create_face<P, C>(self, edges: &[Edge<P, C>]) -> Result<Face<P, C, S>> {
        let wires: Vec<Wire<P, C>> = self
            .boundaries
            .into_iter()
            .map(|wire| {
                wire.into_iter()
                    .map(|(idx, ori)| match ori {
                        true => edges[idx].clone(),
                        false => edges[idx].inverse(),
                    })
                    .collect()
            })
            .collect();
        let mut face = Face::try_new(wires, self.surface)?;
        if !self.orientation {
            face.invert();
        }
        Ok(face)
    }
}

/// Serialized compressed shell
#[derive(Debug, Serialize, Deserialize)]
pub struct CompressedShell<P, C, S> {
    vertices: Vec<P>,
    edges: Vec<CompressedEdge<C>>,
    faces: Vec<CompressedFace<S>>,
}

/// Serialized compressed solid
#[derive(Debug, Serialize, Deserialize)]
pub struct CompressedSolid<P, C, S> {
    boundaries: Vec<CompressedShell<P, C, S>>,
}

struct CompressDirector<P, C> {
    vmap: HashMap<VertexID<P>, (usize, P)>,
    emap: HashMap<EdgeID<C>, (usize, CompressedEdge<C>)>,
}

impl<P: Clone, C: Clone> CompressDirector<P, C> {
    #[inline(always)]
    fn new() -> Self {
        Self {
            vmap: HashMap::new(),
            emap: HashMap::new(),
        }
    }
    #[inline(always)]
    fn get_vid(&mut self, vertex: &Vertex<P>) -> usize {
        let id = self.vmap.len();
        self.vmap
            .get_or_insert(vertex.id(), || (id, vertex.get_point()))
            .0
    }

    #[inline(always)]
    fn get_eid(&mut self, edge: &Edge<P, C>) -> (usize, bool) {
        match self.emap.get(&edge.id()) {
            Some(got) => (got.0, edge.orientation()),
            None => {
                let id = self.emap.len();
                let front_id = self.get_vid(edge.absolute_front());
                let back_id = self.get_vid(edge.absolute_back());
                let curve = edge.get_curve();
                let cedge = CompressedEdge {
                    vertices: (front_id, back_id),
                    curve,
                };
                self.emap.insert(edge.id(), (id, cedge));
                (id, edge.orientation())
            }
        }
    }

    #[inline(always)]
    fn create_boundary(&mut self, boundary: &Wire<P, C>) -> Vec<(usize, bool)> {
        boundary.iter().map(|edge| self.get_eid(edge)).collect()
    }

    #[inline(always)]
    fn create_cface<S: Clone>(&mut self, face: &Face<P, C, S>) -> CompressedFace<S> {
        CompressedFace {
            boundaries: face
                .boundaries
                .iter()
                .map(|wire| self.create_boundary(wire))
                .collect(),
            orientation: face.orientation(),
            surface: face.get_surface(),
        }
    }

    #[inline(always)]
    fn map2vec<K, T>(map: HashMap<K, (usize, T)>) -> Vec<T> {
        let mut vec: Vec<_> = map.into_iter().map(|entry| entry.1).collect();
        vec.sort_by(|x, y| x.0.cmp(&y.0));
        vec.into_iter().map(|x| x.1).collect()
    }

    #[inline(always)]
    fn vertices_edges(self) -> (Vec<P>, Vec<CompressedEdge<C>>) {
        (Self::map2vec(self.vmap), Self::map2vec(self.emap))
    }
}

impl<P: Clone, C: Clone, S: Clone> Shell<P, C, S> {
    /// Compresses the shell into the serialized compressed shell.
    pub fn compress(&self) -> CompressedShell<P, C, S> {
        let mut director = CompressDirector::new();
        let mut face_closure = |face: &Face<P, C, S>| director.create_cface(face);
        let faces = self.iter().map(&mut face_closure).collect();
        let (vertices, edges) = director.vertices_edges();
        CompressedShell {
            vertices,
            edges,
            faces,
        }
    }

    /// Extracts the serialized compressed shell into the shell.
    pub fn extract(cshell: CompressedShell<P, C, S>) -> Result<Self> {
        let CompressedShell {
            vertices,
            edges,
            faces,
        } = cshell;
        let vertices: Vec<_> = vertices.into_iter().map(Vertex::new).collect();
        let edges = edges
            .into_iter()
            .map(move |edge| edge.create_edge(&vertices))
            .collect::<Result<Vec<_>>>()?;
        faces
            .into_iter()
            .map(move |face| face.create_face(&edges))
            .collect()
    }
}

impl<P: Clone, C: Clone, S: Clone> Solid<P, C, S> {
    /// Compresses the solid into the serialized compressed solid.
    pub fn compress(&self) -> CompressedSolid<P, C, S> {
        CompressedSolid {
            boundaries: self
                .boundaries()
                .iter()
                .map(|shell| shell.compress())
                .collect(),
        }
    }

    /// Extracts the serialized compressed shell into the shell.
    pub fn extract(csolid: CompressedSolid<P, C, S>) -> Result<Self> {
        let shells: Result<Vec<Shell<P, C, S>>> = csolid
            .boundaries
            .into_iter()
            .map(|cshell| Shell::extract(cshell))
            .collect();
        Solid::try_new(shells?)
    }
}

// -------------------------- test -------------------------- //

#[test]
fn compress_extract() {
    let cube = solid::cube();
    let shell0 = &cube.boundaries()[0];
    let shell1 = Shell::extract(shell0.compress()).unwrap();
    assert!(same_topology(shell0, &shell1));
}

#[allow(dead_code)]
fn vmap_subroutin<P, Q>(
    v0: &Vertex<P>,
    v1: &Vertex<Q>,
    vmap: &mut HashMap<VertexID<P>, VertexID<Q>>,
) -> bool {
    match vmap.get(&v0.id()) {
        Some(got) => *got == v1.id(),
        None => {
            vmap.insert(v0.id(), v1.id());
            true
        }
    }
}

#[allow(dead_code)]
fn emap_subroutin<P, Q, C, D>(
    edge0: &Edge<P, C>,
    edge1: &Edge<Q, D>,
    vmap: &mut HashMap<VertexID<P>, VertexID<Q>>,
    emap: &mut HashMap<EdgeID<C>, EdgeID<D>>,
) -> bool {
    match emap.get(&edge0.id()) {
        Some(got) => *got == edge1.id(),
        None => {
            emap.insert(edge0.id(), edge1.id());
            vmap_subroutin(edge0.front(), edge1.front(), vmap)
                && vmap_subroutin(edge0.back(), edge1.back(), vmap)
        }
    }
}

#[allow(dead_code)]
fn same_topology<P, C, S, Q, D, T>(one: &Shell<P, C, S>, other: &Shell<Q, D, T>) -> bool {
    let mut vmap = HashMap::<VertexID<P>, VertexID<Q>>::new();
    let mut emap = HashMap::<EdgeID<C>, EdgeID<D>>::new();
    if one.len() != other.len() {
        return false;
    }
    for (face0, face1) in one.iter().zip(other.iter()) {
        let biters0 = face0.boundary_iters();
        let biters1 = face1.boundary_iters();
        if biters0.len() != biters1.len() {
            return false;
        }
        for (biter0, biter1) in biters0.into_iter().zip(biters1) {
            if biter0.len() != biter1.len() {
                return false;
            }
            for (edge0, edge1) in biter0.zip(biter1) {
                if !emap_subroutin(&edge0, &edge1, &mut vmap, &mut emap) {
                    return false;
                }
            }
        }
    }
    true
}
