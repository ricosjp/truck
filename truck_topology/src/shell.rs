use crate::{Edge, Face, Result, Shell, Vertex, Wire};
use crate::errors::Error;
use std::collections::{HashMap, HashSet};
use std::vec::Vec;
use std::convert::TryFrom;

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
    pub fn new() -> Shell {
        Shell {
            face_list: Vec::new(),
        }
    }

    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Shell {
        Shell {
            face_list: Vec::with_capacity(capacity),
        }
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize { self.face_list.capacity() }

    #[inline(always)]
    pub fn reserve(&mut self, additional: usize) { self.face_list.reserve(additional) }

    #[inline(always)]
    pub fn reserve_exact(&mut self, additional: usize) { self.face_list.reserve_exact(additional) }

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

    #[inline(always)]
    pub fn face_iter_mut(&mut self) -> FaceIterMut { self.face_list.iter_mut() }

    #[inline(always)]
    pub fn face_into_iter(self) -> FaceIntoIter { self.face_list.into_iter() }

    pub fn append(&mut self, other: &mut Shell) { self.face_list.append(&mut other.face_list); }

    pub fn remove(&mut self, idx: usize) { self.face_list.remove(idx); }

    /// return (is oriented or not, all edges, inner edge), If irregular, return None.
    fn inner_edge_extraction(
        &self,
    ) -> Option<(bool, HashMap<usize, &Edge>, HashSet<usize>)> {
        let mut all_edges = HashMap::with_capacity(self.face_list.len());
        let mut inner_edges = HashSet::with_capacity(self.face_list.len());

        let mut oriented = true;
        let edge_iter = self
            .face_iter()
            .flat_map(|face| face.boundary().edge_iter());
        for edge in edge_iter {
            if let Some(edge0) = all_edges.insert(edge.id(), edge) {
                if !inner_edges.insert(edge.id()) {
                    return None;
                } else if edge == edge0 {
                    oriented = false;
                }
            }
        }

        Some((oriented, all_edges, inner_edges))
    }

    pub fn extract_boundaries(&self) -> Result<Vec<Wire>> {
        let (_, all_edges, inner_edges) = match self.inner_edge_extraction() {
            Some(tuple) => tuple,
            None => return Err(Error::NotRegularShell),
        };
        let mut vemap: HashMap<Vertex, &Edge> = HashMap::new();
        for edge in all_edges.values() {
            if inner_edges.get(&edge.id()).is_none() {
                vemap.insert(edge.front(), edge);
            }
        }
        let mut res = Vec::new();
        while !vemap.is_empty() {
            let vertex = *vemap.keys().next().unwrap();
            let mut cursor = vemap.remove(&vertex).unwrap();
            let mut wire = Wire::try_from(vec![*cursor]).unwrap();
            loop {
                cursor = match vemap.remove(&cursor.back()) {
                    None => break,
                    Some(got) => {
                        wire.push_back(*got);
                        got
                    }
                };
            }
            res.push(wire);
        }
        Ok(res)
    }

    /// determine the shell conditions: non-regular, regular, oriented, or closed.  
    /// The complexity increases in proportion to the number of edges.
    pub fn shell_condition(&self) -> ShellCondition {
        if let Some((oriented, all_edges, inner_edges)) = self.inner_edge_extraction() {
            if oriented {
                if all_edges.len() == inner_edges.len() {
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

    pub fn create_vertex_adjacency(&self) -> (HashMap<Vertex, usize>, Vec<Vec<usize>>) {
        let mut vmap: HashMap<Vertex, usize> = HashMap::new();
        let mut adjacency = Vec::new();
        let edge_iter = self
            .face_iter()
            .flat_map(|face| face.boundary().edge_iter());
        for edge in edge_iter {
            let v0 = edge.front();
            let v1 = edge.back();
            let i = match vmap.get(&v0) {
                Some(i) => *i,
                None => {
                    vmap.insert(v0, vmap.len());
                    adjacency.push(Vec::new());
                    vmap.len() - 1
                }
            };
            let j = match vmap.get(&v1) {
                Some(j) => *j,
                None => {
                    vmap.insert(v1, vmap.len());
                    adjacency.push(Vec::new());
                    vmap.len() - 1
                }
            };
            adjacency[i].push(j);
            adjacency[j].push(i);
        }
        (vmap, adjacency)
    }

    pub fn create_face_adjacency(&self) -> Vec<Vec<usize>> {
        let mut adjacency = vec![Vec::new(); self.len()];
        let mut edge_face_map: HashMap<usize, usize> = HashMap::new();
        for (i, face) in self.face_iter().enumerate() {
            for edge in face.boundary().edge_iter() {
                if let Some(j) = edge_face_map.get(&edge.id()) {
                    adjacency[i].push(*j);
                    adjacency[*j].push(i);
                } else {
                    edge_face_map.insert(edge.id(), i);
                }
            }
        }
        adjacency
    }

    pub fn is_connected(&self) -> bool {
        if self.is_empty() {
            return true;
        }
        let (_, adjacency) = self.create_vertex_adjacency();
        check_connectivity(&adjacency)
    }

    pub fn connected_components(&self) -> Vec<Shell> {
        let adjacency = self.create_face_adjacency();
        let components = create_components(&adjacency);
        components
            .iter()
            .map(|vec| vec.iter().map(|i| self[*i].clone()).collect())
            .collect()
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
        Shell {
            face_list: iter.into_iter().collect(),
        }
    }
}

impl std::ops::Index<usize> for Shell {
    type Output = Face;
    fn index(&self, idx: usize) -> &Face { &self.face_list[idx] }
}

pub type FaceIter<'a> = std::slice::Iter<'a, Face>;
pub type FaceIterMut<'a> = std::slice::IterMut<'a, Face>;
pub type FaceIntoIter = std::vec::IntoIter<Face>;

impl ShellCondition {
    fn get_id(&self) -> usize {
        match *self {
            ShellCondition::Irregular => 0,
            ShellCondition::Regular => 1,
            ShellCondition::Oriented => 2,
            ShellCondition::Closed => 3,
        }
    }
}

impl std::cmp::PartialOrd for ShellCondition {
    fn partial_cmp(&self, other: &ShellCondition) -> Option<std::cmp::Ordering> {
        self.get_id().partial_cmp(&other.get_id())
    }
}

fn check_connectivity(adjacency: &Vec<Vec<usize>>) -> bool {
    let mut unchecked = vec![true; adjacency.len()];
    let component = create_one_component(adjacency, &mut unchecked);
    component.len() == adjacency.len()
}

fn create_components(adjacency: &Vec<Vec<usize>>) -> Vec<Vec<usize>> {
    let mut unchecked = vec![true; adjacency.len()];
    let mut res = Vec::new();
    loop {
        let component = create_one_component(adjacency, &mut unchecked);
        match component.is_empty() {
            true => break,
            false => res.push(component),
        }
    }
    res
}

fn create_one_component(adjacency: &Vec<Vec<usize>>, unchecked: &mut Vec<bool>) -> Vec<usize> {
    let first = match unchecked.iter().position(|x| *x) {
        Some(i) => i,
        None => return Vec::new(),
    };
    let mut stack = vec![first];
    let mut res = vec![first];
    unchecked[first] = false;
    while !stack.is_empty() {
        let i = stack.pop().unwrap();
        for j in &adjacency[i] {
            if unchecked[*j] {
                stack.push(*j);
                res.push(*j);
                unchecked[*j] = false;
            }
        }
    }
    res
}

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
