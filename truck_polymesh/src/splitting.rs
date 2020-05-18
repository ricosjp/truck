use crate::MeshHandler;
use crate::errors::Error;
use std::collections::HashMap;

impl MeshHandler {
    pub fn clustering_face(&self) -> Vec<Vec<usize>> {
        if self.mesh.normals.is_empty() {
            panic!("{}", Error::NoNormal);
        }
        let face_adjacency = self.create_face_adjacency();
        get_components(&face_adjacency)
    }

    fn create_face_adjacency(&self) -> Vec<Vec<usize>> {
        let mesh = &self.mesh;
        let mut face_adjacency = vec![Vec::<usize>::new(); mesh.tri_faces.len()];

        let mut edge_face_map: HashMap<[[usize; 2]; 2], usize> = HashMap::new();
        for (i, face) in mesh.tri_faces.iter().enumerate() {
            signup_adjacency(i, face[0], face[1], &mut face_adjacency, &mut edge_face_map);
            signup_adjacency(i, face[1], face[2], &mut face_adjacency, &mut edge_face_map);
            signup_adjacency(i, face[2], face[0], &mut face_adjacency, &mut edge_face_map);
        }
        face_adjacency
    }
}

fn signup_adjacency(
    i: usize,
    face0: [usize; 3],
    face1: [usize; 3],
    face_adjacency: &mut Vec<Vec<usize>>,
    edge_face_map: &mut HashMap<[[usize; 2]; 2], usize>,
) {
    let edge = if face0[0] < face1[0] {
        [[face0[0], face0[2]], [face1[0], face1[2]]]
    } else {
        [[face1[0], face1[2]], [face0[0], face0[2]]]
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

fn get_components(adjacency: &Vec<Vec<usize>>) -> Vec<Vec<usize>> {
    let mut unchecked = vec![true; adjacency.len()];
    let mut components = Vec::new();
    loop {
        let first = match unchecked.iter().position(|x| *x) {
            Some(idx) => idx,
            None => return components,
        };
        let mut stack = vec![first];
        let mut component = vec![first];
        unchecked[first] = false;
        while !stack.is_empty() {
            let cursor = stack.pop().unwrap();
            for i in &adjacency[cursor] {
                if unchecked[*i] {
                    unchecked[*i] = false;
                    component.push(*i);
                    stack.push(*i);
                }
            }
        }
        components.push(component);
    }
}
