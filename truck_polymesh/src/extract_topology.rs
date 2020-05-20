use crate::MeshHandler;
use std::collections::HashMap;
use topology::*;

impl MeshHandler {
    pub fn extract_topology(&self) -> Shell {
        let mesh = &self.mesh;
        let v = Vertex::news(mesh.vertices.len());

        let mut shell = Shell::new();
        let mut edges: HashMap<(Vertex, Vertex), Edge> = HashMap::new();
        for face in &mesh.tri_faces {
            let i = face[0][0];
            let j = face[1][0];
            let k = face[2][0];
            let mut wire = Wire::new();
            wire.push_back(create_edge(v[i], v[j], &mut edges));
            wire.push_back(create_edge(v[j], v[k], &mut edges));
            wire.push_back(create_edge(v[k], v[i], &mut edges));
            shell.push(Face::new(wire));
        }
        for face in &mesh.quad_faces {
            let i = face[0][0];
            let j = face[1][0];
            let k = face[2][0];
            let l = face[3][0];
            let mut wire = Wire::new();
            wire.push_back(create_edge(v[i], v[j], &mut edges));
            wire.push_back(create_edge(v[j], v[k], &mut edges));
            wire.push_back(create_edge(v[k], v[l], &mut edges));
            wire.push_back(create_edge(v[l], v[i], &mut edges));
            shell.push(Face::new(wire));
        }

        shell
    }
}

fn create_edge(v0: Vertex, v1: Vertex, edges: &mut HashMap<(Vertex, Vertex), Edge>) -> Edge {
    let min = std::cmp::min(v0, v1);
    let max = std::cmp::max(v0, v1);
    let edge = match edges.get(&(min, max)) {
        Some(edge) => edge.clone(),
        None => {
            let edge = Edge::new(min, max);
            edges.insert((min, max), edge);
            edge
        }
    };

    if v0 == min {
        edge.clone()
    } else {
        edge.inverse()
    }
}
