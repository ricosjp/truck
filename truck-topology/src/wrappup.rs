use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct CompressedEdge<C> {
    vertices: (usize, usize),
    curve: C,
}

impl<C> CompressedEdge<C> {
    fn create_edge<P>(self, v: &Vec<Vertex<P>>) -> Result<Edge<P, C>> {
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
    fn create_face<P, C>(self, edges: &Vec<Edge<P, C>>) -> Result<Face<P, C, S>> {
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
        match self.vmap.get(&vertex.id()) {
            Some(got) => got.0,
            None => {
                let id = self.vmap.len();
                let pt = vertex.lock_point().unwrap().clone();
                self.vmap.insert(vertex.id(), (id, pt));
                id
            }
        }
    }

    #[inline(always)]
    fn get_eid(&mut self, edge: &Edge<P, C>) -> (usize, bool) {
        match self.emap.get(&edge.id()) {
            Some(got) => (got.0, edge.orientation()),
            None => {
                let id = self.emap.len();
                let front_id = self.get_vid(edge.absolute_front());
                let back_id = self.get_vid(edge.absolute_back());
                let curve = edge.lock_curve().unwrap().clone();
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
    fn create_cface<S: Clone>(&mut self, face: &Face<P, C, S>) -> CompressedFace<S> {
        let mut edge_closure = |edge: &Edge<P, C>| self.get_eid(edge);
        let mut wire_closure = |wire: &Wire<P, C>| wire.iter().map(&mut edge_closure).collect();
        CompressedFace {
            boundaries: face.boundaries.iter().map(&mut wire_closure).collect(),
            orientation: face.orientation(),
            surface: face.lock_surface().unwrap().clone(),
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
