use crate::*;
use std::collections::HashMap;

type Vertex = topology::Vertex<Point3>;
type Edge = topology::Edge<Point3, ()>;
type Wire = topology::Wire<Point3, ()>;
type Face = topology::Face<Point3, (), ()>;
type Shell = topology::Shell<Point3, (), ()>;

/// create a shell from the mesh
impl MeshHandler {
    /// create a shell from the mesh
    pub fn extract_topology(&self) -> Shell {
        let mesh = &self.mesh;
        let v = Vertex::news(&mesh.positions);

        let mut shell = Shell::new();
        let mut edges: HashMap<(&Vertex, &Vertex), Edge> = HashMap::new();
        for face in &mesh.tri_faces {
            let i = face[0][0];
            let j = face[1][0];
            let k = face[2][0];
            let mut wire = Wire::new();
            wire.push_back(create_edge(&v[i], &v[j], &mut edges));
            wire.push_back(create_edge(&v[j], &v[k], &mut edges));
            wire.push_back(create_edge(&v[k], &v[i], &mut edges));
            shell.push(Face::new(wire, ()));
        }
        for face in &mesh.quad_faces {
            let i = face[0][0];
            let j = face[1][0];
            let k = face[2][0];
            let l = face[3][0];
            let mut wire = Wire::new();
            wire.push_back(create_edge(&v[i], &v[j], &mut edges));
            wire.push_back(create_edge(&v[j], &v[k], &mut edges));
            wire.push_back(create_edge(&v[k], &v[l], &mut edges));
            wire.push_back(create_edge(&v[l], &v[i], &mut edges));
            shell.push(Face::new(wire, ()));
        }
        for face in &mesh.other_faces {
            let idx: Vec<_> = face.iter().map(|x| x[0]).collect();
            let mut wire = Wire::new();
            for i in 0..=idx.len() {
                let idx0 = idx[i];
                let idx1 = idx[(i + 1) % idx.len()];
                wire.push_back(create_edge(&v[idx0], &v[idx1], &mut edges));
            }
            shell.push(Face::new(wire, ()));
        }

        shell
    }
}

fn create_edge<'a>(v0: &'a Vertex, v1: &'a Vertex, edges: &mut HashMap<(&'a Vertex, &'a Vertex), Edge>) -> Edge {
    let (min, max) = if compare_vertex(v0, v1) { (v0, v1) } else { (v1, v0) };
    let edge = match edges.get(&(min, max)) {
        Some(edge) => edge.clone(),
        None => {
            let edge = Edge::new(min, max, ());
            edges.insert((min, max), edge.clone());
            edge
        }
    };

    if v0 == min {
        edge
    } else {
        edge.inverse()
    }
}

fn compare_vertex(v0: &Vertex, v1: &Vertex) -> bool {
    let pt0 = v0.try_lock_point().unwrap();
    let pt1 = v1.try_lock_point().unwrap();
    if pt0[0] > pt1[0] {
        true
    } else if pt0[0] < pt1[0] {
        false
    } else if pt0[1] > pt1[1] {
        true
    } else if pt0[1] < pt1[1] {
        false
    } else if pt0[2] > pt1[2] {
        true
    } else {
        false
    }
}
