use super::*;
use std::collections::HashMap;

pub trait FaceAdjacency {
    /// create the adjacency list of the faces
    fn face_adjacency(&self) -> Vec<Vec<usize>>;
}

impl FaceAdjacency for Faces {
    fn face_adjacency(&self) -> Vec<Vec<usize>> {
        let len = self.len();
        let mut face_adjacency = vec![Vec::<usize>::new(); len];
        let mut edge_face_map: HashMap<[(usize, Option<usize>); 2], usize> = HashMap::new();
        for (i, face) in self.face_iter().enumerate() {
            face.windows(2)
                .chain(std::iter::once([face[face.len() - 1], face[0]].as_ref()))
                .for_each(|v| {
                    signup_adjacency(i, v[0], v[1], &mut face_adjacency, &mut edge_face_map)
                })
        }
        face_adjacency
    }
}

fn signup_adjacency(
    i: usize,
    v0: Vertex,
    v1: Vertex,
    face_adjacency: &mut Vec<Vec<usize>>,
    edge_face_map: &mut HashMap<[(usize, Option<usize>); 2], usize>,
) {
    let edge = if v0.pos < v1.pos {
        [(v0.pos, v0.nor), (v1.pos, v1.nor)]
    } else {
        [(v1.pos, v1.nor), (v0.pos, v0.nor)]
    };
    match edge_face_map.get(&edge) {
        Some(j) => {
            face_adjacency[i].push(*j);
            face_adjacency[*j].push(i);
        }
        None => {
            edge_face_map.insert(edge, i);
        }
    }
}
