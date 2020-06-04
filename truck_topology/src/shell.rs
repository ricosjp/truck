use crate::{Vertex, Edge, Wire, Face, Shell, WrappedUpShell};
use std::collections::{HashMap, HashSet};
use std::vec::Vec;

#[derive(PartialEq, Eq, Debug)]
pub enum ShellCondition {
    /// This shell is not regular.
    Irregular,
    /// All edges are shared by at most two faces.
    Regular,
    /// The orientations of faces are compatible.
    Oriented,
    /// All edges are shared by two faces.
    Closed,
}

impl Shell {
    /// Create the empty shell.
    #[inline(always)]
    pub fn new() -> Shell { Shell { face_list: Vec::new() } }

    #[inline(always)]
    pub fn is_empty(&self) -> bool { self.face_list.is_empty() }

    #[inline(always)]
    pub fn len(&self) -> usize { self.face_list.len() }
    
    /// add an face
    #[inline(always)]
    pub fn push(&mut self, face: Face) { self.face_list.push(face); }
    
    /// get a face iterator
    #[inline(always)]
    pub fn face_iter(&self) -> FaceIter { self.face_list.iter() }

    /// return (oriented, boundary used), If irregular, return None.
    fn boundary_edge_extraction(&self) -> Option<(bool, HashMap<usize, &Edge>, HashMap<usize, &Edge>)> {
        let mut boundary = HashMap::with_capacity(self.face_list.len());
        let mut used = HashMap::with_capacity(self.face_list.len());

        let mut oriented = true;
        for face in self.face_list.iter() {
            for edge in face.boundary().edge_iter() {
                if let Some(edge0) = boundary.insert(edge.id(), edge) {
                    if used.insert(edge.id(), edge).is_some() {
                        return None;
                    } else if edge == edge0 {
                        oriented = false;
                    }
                }
            }
        }

        Some((oriented, boundary, used))
    }

    /// determine the shell conditions: non-regular, regular, oriented, or closed.  
    /// The complexity increases in proportion to the number of edges.
    pub fn shell_condition(&self) -> ShellCondition {
        if let Some((oriented, boundary, used)) = self.boundary_edge_extraction() {
            if oriented {
                if boundary.len() == used.len() {
                    ShellCondition::Closed
                } else {
                    ShellCondition::Oriented
                }
            } else {
                ShellCondition::Regular
            }
        } else {
            ShellCondition::Irregular
        }
    }

    /// return (the set of all vertices, vertices adjacency lists)
    fn create_vertex_adjacency(&self) -> (HashSet<Vertex>, HashMap<Vertex, HashSet<Vertex>>){
        let mut adjacency: HashMap<Vertex, HashSet<Vertex>> = HashMap::with_capacity(self.face_list.len());
        let mut vertices = HashSet::with_capacity(self.face_list.len());
        for face in self.face_list.iter() {
            for edge in face.boundary.edge_iter() {
                if let Some(ref mut set) = adjacency.get_mut(&edge.front()) {
                    set.insert(edge.back());
                } else {
                    let mut set = HashSet::new();
                    set.insert(edge.back());
                    adjacency.insert(edge.front(), set);
                    vertices.insert(edge.front());
                }
            }
        }

        (vertices, adjacency)
    }

    fn get_one_component(unchecked: &mut HashSet<Vertex>, adjacency: &HashMap<Vertex, HashSet<Vertex>>) {
        let mut stack = Vec::new();
        let first_vertex = *unchecked.iter().next().unwrap();
        stack.push(first_vertex);
        unchecked.remove(&first_vertex);
        while !stack.is_empty() && !unchecked.is_empty() {
            let cursor = stack.pop().unwrap();
            for vertex in adjacency.get(&cursor).unwrap() {
                if unchecked.remove(&vertex) {
                    stack.push(*vertex);
                }
            }
        }
    }

    /// determine whether this shell is connected or not.
    /// The complexity increases in proportion to the number of vertices and edges.
    pub fn is_connected(&self) -> bool {
        if self.is_empty() {
            return true;
        }

        let (mut unchecked, adjacency) = self.create_vertex_adjacency();
        Shell::get_one_component(&mut unchecked, &adjacency);

        unchecked.is_empty()
    }

    /// return the following hash maps:
    /// * from vertex to the local id.
    /// * from edge to (the local id, front vertex's local id, back vertex's local id)
    fn create_vertex_edge_map(&self) -> (HashMap<Vertex, usize>, HashMap<usize, (usize, usize, usize)>) {
        let mut vertices_map = HashMap::with_capacity(self.face_list.len());
        let mut edges_map = HashMap::with_capacity(self.face_list.len());

        let mut vertex_counter = 0;
        let mut edge_counter = 0;
        for face in self.face_iter() {
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
    pub fn wrap_up(&self) -> WrappedUpShell {
        let (vertices_map, edges_map) = self.create_vertex_edge_map();

        let faces: Vec<(bool, Vec<usize>)> = self.face_iter()
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
            }).collect();

        let mut edges_with_id: Vec<(usize, usize, usize)> = edges_map.into_iter().map(|(_, x)| x).collect();
        edges_with_id.sort_by(|(a, _, _), (b, _, _)| a.cmp(b));
        let edges: Vec<(usize, usize)> = edges_with_id.into_iter().map(|(_, f, b)| (f, b)).collect();

        WrappedUpShell {
            number_of_vertices: vertices_map.len(),
            edges: edges,
            faces: faces,
        }
    }

    /// extract topology data
    pub fn extract(topodata: &WrappedUpShell) -> Shell {
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
}

impl std::convert::From<Shell> for Vec<Face> {
    fn from(shell: Shell) -> Vec<Face> { shell.face_list }
}

impl std::convert::From<Vec<Face>> for Shell {
    fn from(faces: Vec<Face>) -> Shell { Shell { face_list: faces } }
}

impl std::iter::FromIterator<Face> for Shell {
    fn from_iter<I: IntoIterator<Item = Face>>(iter: I) -> Shell {
        Shell { face_list: iter.into_iter().collect() }
    }
}

pub type FaceIter<'a> = std::slice::Iter<'a, Face>;

#[test]
fn shell_test() {
    use crate::*;
    let v = Vertex::news(4);
    let edge = [
        Edge::new(v[0], v[3]),
        Edge::new(v[0], v[3]),
        Edge::new(v[0], v[1]),
        Edge::new(v[1], v[2]),
        Edge::new(v[1], v[2]),
        Edge::new(v[2], v[3]),
    ];

    let mut wire0 = Wire::new();
    let mut wire1 = Wire::new();

    wire0.push_back(edge[0]);
    wire0.push_back(edge[5].inverse());
    wire0.push_back(edge[3].inverse());
    wire0.push_back(edge[2].inverse());

    wire1.push_back(edge[2]);
    wire1.push_back(edge[4]);
    wire1.push_back(edge[5]);
    wire1.push_back(edge[1].inverse());

    let face0 = Face::new(wire0);
    let face1 = Face::new(wire1);

    let mut shell = Shell::new();
    shell.push(face0);
    shell.push(face1);
}
