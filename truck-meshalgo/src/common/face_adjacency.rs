use super::*;
use rustc_hash::FxHashMap as HashMap;

pub trait FaceAdjacency {
    /// create the adjacency list of the faces
    fn face_adjacency(&self, use_normal: bool) -> Vec<Vec<usize>>;
}

impl FaceAdjacency for Faces {
    fn face_adjacency(&self, use_normal: bool) -> Vec<Vec<usize>> {
        let len = self.len();
        let mut face_adjacency = vec![Vec::<usize>::new(); len];
        let mut edge_face_map = HashMap::default();
        for (i, face) in self.face_iter().enumerate() {
            face.windows(2)
                .chain(std::iter::once([face[face.len() - 1], face[0]].as_ref()))
                .for_each(|v| {
                    signup_adjacency(
                        i,
                        v[0],
                        v[1],
                        &mut face_adjacency,
                        &mut edge_face_map,
                        use_normal,
                    )
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
    use_normal: bool,
) {
    let edge = match (v0.pos < v1.pos, use_normal) {
        (true, true) => [(v0.pos, v0.nor), (v1.pos, v1.nor)],
        (false, true) => [(v1.pos, v1.nor), (v0.pos, v0.nor)],
        (true, false) => [(v0.pos, None), (v1.pos, None)],
        (false, false) => [(v1.pos, None), (v0.pos, None)],
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
