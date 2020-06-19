use crate::{Edge, Face, Shell, Vertex, Wire};
use std::collections::{HashMap, HashSet};
use std::vec::Vec;

impl Shell {
    /// Creates the empty shell.
    #[inline(always)]
    pub const fn new() -> Shell {
        Shell {
            face_list: Vec::new(),
        }
    }

    /// Creates the empty shell with space for at least `capacity` faces.
    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Shell {
        Shell {
            face_list: Vec::with_capacity(capacity),
        }
    }

    /// Returns an iterator over the faces. Practically, an alias of `iter()`.
    #[inline(always)]
    pub fn face_iter(&self) -> FaceIter { self.iter() }

    /// Returns a mutable iterator over the faces. Practically, an alias of `iter_mut()`.
    #[inline(always)]
    pub fn face_iter_mut(&mut self) -> FaceIterMut { self.iter_mut() }

    /// Creates a consuming iterator. Practically, an alias of `into_iter()`.
    #[inline(always)]
    pub fn face_into_iter(self) -> FaceIntoIter { self.face_list.into_iter() }

    /// Moves all the faces of `other` into `self`, leaving `other` empty.
    #[inline(always)]
    pub fn append(&mut self, other: &mut Shell) { self.face_list.append(&mut other.face_list); }

    /// Returns a tuple.
    /// * The 0th component is whether the shell is regular or not.
    /// * The 1st component is whether the shell is oriented or not.
    /// * The 2nd component is whether the shell is closed or not.
    /// * The 3rd component is the set of all ids of the inner edge of the shell.
    fn inner_edge_extraction(&self) -> (bool, bool, bool, HashSet<usize>) {
        let mut all_edges: HashMap<usize, bool> = HashMap::with_capacity(self.face_list.len());
        let mut inner_edges: HashSet<usize> = HashSet::with_capacity(self.face_list.len());

        let mut regular = true;
        let mut oriented = true;
        for edge in self.face_iter().flat_map(Face::boundary_iter) {
            let new_ori = edge.absolute_front() == edge.front();
            if let Some(ori) = all_edges.insert(edge.id(), new_ori) {
                regular = regular && inner_edges.insert(edge.id());
                oriented = oriented && (new_ori != ori)
            }
        }

        let closed = all_edges.len() == inner_edges.len();
        (regular, oriented, closed, inner_edges)
    }
    /// Determines the shell conditions: non-regular, regular, oriented, or closed.  
    /// The complexity increases in proportion to the number of edges.
    ///
    /// Examples for each condition can be found on the page of
    /// [`ShellCondition`](./shell/enum.ShellCondition.html).
    pub fn shell_condition(&self) -> ShellCondition {
        let (regular, oriented, closed, _) = self.inner_edge_extraction();
        match (regular, oriented, closed) {
            (false, _, _) => ShellCondition::Irregular,
            (true, false, _) => ShellCondition::Regular,
            (true, true, false) => ShellCondition::Oriented,
            (true, true, true) => ShellCondition::Closed,
        }
    }

    /// Returns a vector of all boundaries as wires.
    /// # Examples
    /// ```
    /// # use truck_topology::*;
    /// # use truck_topology::shell::ShellCondition;
    /// let v = Vertex::news(6);
    /// let edge = [
    ///     Edge::new(v[0], v[1]),
    ///     Edge::new(v[0], v[2]),
    ///     Edge::new(v[1], v[2]),
    ///     Edge::new(v[1], v[3]),
    ///     Edge::new(v[1], v[4]),
    ///     Edge::new(v[2], v[4]),
    ///     Edge::new(v[2], v[5]),
    ///     Edge::new(v[3], v[4]),
    ///     Edge::new(v[4], v[5]),
    /// ];
    /// let wire = vec![
    ///     Wire::from(vec![edge[0], edge[2], edge[1].inverse()]),
    ///     Wire::from(vec![edge[3], edge[7], edge[4].inverse()]),
    ///     Wire::from(vec![edge[5], edge[8], edge[6].inverse()]),
    ///     Wire::from(vec![edge[2].inverse(), edge[4], edge[5].inverse()]),
    /// ];
    /// let shell: Shell = wire.into_iter().map(|w| Face::new(w)).collect();
    /// let boundary = shell.extract_boundaries()[0].clone();
    /// assert_eq!(
    ///     boundary,
    ///     Wire::from(vec![edge[0], edge[3], edge[7], edge[8], edge[6].inverse(), edge[1].inverse()])
    /// );
    /// ```
    /// # Remarks
    /// This method is optimized when the shell is oriented.
    /// Even if the shell is not oriented, all the edges of the boundary are extracted.
    /// However, the connected components of the boundary are split into several wires.
    pub fn extract_boundaries(&self) -> Vec<Wire> {
        let (_, _, _, inner_edges) = self.inner_edge_extraction();
        let mut boundary_edges = Vec::new();
        let mut vemap: HashMap<Vertex, Edge> = HashMap::new();
        let edge_iter = self.face_iter().flat_map(Face::boundary_iter);
        for edge in edge_iter {
            if inner_edges.get(&edge.id()).is_none() {
                boundary_edges.push(edge);
                vemap.insert(edge.front(), edge);
            }
        }
        let mut res = Vec::new();
        for edge in boundary_edges {
            if let Some(mut cursor) = vemap.remove(&edge.front()) {
                let mut wire = Wire::from(vec![cursor]);
                loop {
                    cursor = match vemap.remove(&cursor.back()) {
                        None => break,
                        Some(got) => {
                            wire.push_back(got);
                            got
                        }
                    };
                }
                res.push(wire);
            }
        }
        res
    }

    /// Returns the adjacency matrix of vertices in the shell.
    ///
    /// Now, we denote the returned pair by `(verts, adjacency)`. Then,
    ///
    /// * `verts` is the vector containing all vertices in the shell.
    /// * `adjacency` is the adjacency matrix.
    ///
    /// The indices in the adjacency matrix represents the ones in `verts`, not the id of vertices.
    /// For example, `adjacency[0] == vec![1, 2, 3]` means `verts[0]` is adjacent to `verts[1]`,
    /// `verts[2]`, and `verts[3]`. Since the graph is non-oriented, `j` is included in `adjacency[i]`
    /// if and only if `i` is included in `adjacency[j]`.
    pub fn create_vertex_adjacency(&self) -> (Vec<Vertex>, Vec<Vec<usize>>) {
        let mut vvec: Vec<Vertex> = Vec::new();
        let mut vmap: HashMap<Vertex, usize> = HashMap::new();
        let mut adjacency = Vec::new();
        let edge_iter = self
            .face_iter()
            .flat_map(|face| face.absolute_boundary().edge_iter());
        for edge in edge_iter {
            let v0 = edge.front();
            let v1 = edge.back();
            let i = *vmap.entry(v0).or_insert_with(|| {
                vvec.push(v0);
                adjacency.push(Vec::new());
                vvec.len() - 1
            });
            let j = *vmap.entry(v1).or_insert_with(|| {
                vvec.push(v1);
                adjacency.push(Vec::new());
                vvec.len() - 1
            });
            adjacency[i].push(j);
            adjacency[j].push(i);
        }
        (vvec, adjacency)
    }

    /// Returns the adjacency matrix of faces in the shell.
    ///
    /// The indices in the adjacency matrix represents the ones in the shell, not the id of faces.
    /// For example, the `0`th vector in the adjacency matrix is `vec![1, 2, 3]`, it means
    /// `self[0]` is adjacent to `shell[1]`, `shell[2]`, and `shell[3]`.
    /// Since the graph is non-oriented, `j` is included in `adjacency[i]` if and only if
    /// `i` is included in `adjacency[j]`.
    pub fn create_face_adjacency(&self) -> Vec<Vec<usize>> {
        let mut adjacency = vec![Vec::new(); self.len()];
        let mut edge_face_map: HashMap<usize, Vec<usize>> = HashMap::new();
        for (i, face) in self.face_iter().enumerate() {
            for edge in face.absolute_boundary().edge_iter() {
                if let Some(vec) = edge_face_map.get_mut(&edge.id()) {
                    for j in vec {
                        adjacency[i].push(*j);
                        adjacency[*j].push(i);
                    }
                } else {
                    edge_face_map.insert(edge.id(), vec![i]);
                }
            }
        }
        adjacency
    }

    /// Returns whether the shell is connected or not.
    /// # Examples
    /// ## The empty shell
    /// ```
    /// # use truck_topology::*;
    /// assert!(Shell::new().is_connected());
    /// ```
    /// ## A connected shell
    /// ```
    /// # use truck_topology::*;
    /// let v = Vertex::news(4);
    /// let shared_edge = Edge::new(v[1], v[2]);
    /// let face0 = Face::new(
    ///     Wire::from(vec![Edge::new(v[0], v[1]), shared_edge, Edge::new(v[2], v[0])])
    /// );
    /// let face1 = Face::new(
    ///     Wire::from(vec![Edge::new(v[3], v[1]), shared_edge, Edge::new(v[2], v[3])])
    /// );
    /// let shell: Shell = vec![face0, face1].into();
    /// assert!(shell.is_connected());
    /// ```
    /// ## A non-connected shell
    /// ```
    /// # use truck_topology::*;
    /// let v = Vertex::news(6);
    /// let face0 = Face::new(
    ///     Wire::from(vec![Edge::new(v[0], v[1]), Edge::new(v[1], v[2]), Edge::new(v[2], v[0])])
    /// );
    /// let face1 = Face::new(
    ///     Wire::from(vec![Edge::new(v[3], v[4]), Edge::new(v[4], v[5]), Edge::new(v[5], v[3])])
    /// );
    /// let shell: Shell = vec![face0, face1].into();
    /// assert!(!shell.is_connected());
    /// ```
    pub fn is_connected(&self) -> bool {
        let (_, adjacency) = self.create_vertex_adjacency();
        check_connectivity(&adjacency)
    }

    /// Returns a vector consisting of shells of each connected components.
    /// # Remarks
    /// Since this method uses the face adjacency matrix, multiple components
    /// are perhaps generated even if the shell is connected. In that case,
    /// there is a pair of faces such that share vertices but not edges.
    /// ```
    /// # use truck_topology::*;
    /// let v = Vertex::news(5);
    /// let wire0 = Wire::from(vec![
    ///     Edge::new(v[0], v[1]),
    ///     Edge::new(v[1], v[2]),
    ///     Edge::new(v[2], v[0]),
    /// ]);
    /// let wire1 = Wire::from(vec![
    ///     Edge::new(v[0], v[3]),
    ///     Edge::new(v[3], v[4]),
    ///     Edge::new(v[4], v[0]),
    /// ]);
    /// let shell = Shell::from(vec![
    ///     Face::new(wire0),
    ///     Face::new(wire1),
    /// ]);
    /// assert!(shell.is_connected());
    /// assert_eq!(shell.connected_components().len(), 2);
    /// ```
    pub fn connected_components(&self) -> Vec<Shell> {
        let adjacency = self.create_face_adjacency();
        let components = create_components(&adjacency);
        components
            .iter()
            .map(|vec| vec.iter().map(|i| self[*i].clone()).collect())
            .collect()
    }

    /// Returns whether the shell has a singular vertex or not.
    ///
    /// Here, we say that a vertex is singular if, for a sufficiently small neighborhood U of
    /// the vertex, the set U - {the vertex} is not connected.
    ///
    /// A regular, oriented, or closed shell becomes a manifold if and only if the shell has
    /// no singular vertices.
    pub fn has_singular_vertex(&self) -> bool {
        let mut vert_wise_adjacency: HashMap<Vertex, (HashMap<usize, usize>, Vec<Vec<usize>>)> =
            HashMap::new();
        for face in self.face_iter() {
            let first_edge = &face.absolute_boundary()[0];
            let mut edge_iter = face.absolute_boundary().edge_iter().peekable();
            while let Some(edge) = edge_iter.next() {
                let (emap, adjacency) = vert_wise_adjacency
                    .entry(edge.back())
                    .or_insert((HashMap::new(), Vec::new()));
                let next_edge = *edge_iter.peek().unwrap_or(&first_edge);
                let idx0 = *emap.entry(edge.id()).or_insert_with(|| {
                    adjacency.push(Vec::new());
                    adjacency.len() - 1
                });
                let idx1 = *emap.entry(next_edge.id()).or_insert_with(|| {
                    adjacency.push(Vec::new());
                    adjacency.len() - 1
                });
                adjacency[idx0].push(idx1);
                adjacency[idx1].push(idx0);
            }
        }
        for (_, adjacency) in vert_wise_adjacency.values() {
            if !check_connectivity(adjacency) {
                return true;
            }
        }
        false
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

impl std::ops::Deref for Shell {
    type Target = Vec<Face>;
    fn deref(&self) -> &Vec<Face> { &self.face_list }
}

impl std::ops::DerefMut for Shell {
    fn deref_mut(&mut self) -> &mut Vec<Face> { &mut self.face_list }
}

pub type FaceIter<'a> = std::slice::Iter<'a, Face>;
pub type FaceIterMut<'a> = std::slice::IterMut<'a, Face>;
pub type FaceIntoIter = std::vec::IntoIter<Face>;

#[derive(PartialEq, Eq, Debug)]
pub enum ShellCondition {
    /// This shell is not regular.
    /// # Examples
    /// ```
    /// # use truck_topology::*;
    /// # use truck_topology::shell::ShellCondition;
    /// let v = Vertex::news(5);
    /// let edge = [
    ///    Edge::new(v[0], v[1]),
    ///    Edge::new(v[0], v[2]),
    ///    Edge::new(v[0], v[3]),
    ///    Edge::new(v[0], v[4]),
    ///    Edge::new(v[1], v[2]),
    ///    Edge::new(v[1], v[3]),
    ///    Edge::new(v[1], v[4]),
    /// ];
    /// let wire = vec![
    ///    Wire::from(vec![edge[0], edge[4], edge[1].inverse()]),
    ///    Wire::from(vec![edge[0], edge[5], edge[2].inverse()]),
    ///    Wire::from(vec![edge[0], edge[6], edge[3].inverse()]),
    /// ];
    /// let shell: Shell = wire.into_iter().map(|w| Face::new(w)).collect();
    /// // The shell is irregular because three faces share edge[0].
    /// assert_eq!(shell.shell_condition(), ShellCondition::Irregular);
    /// ```
    Irregular,
    /// All edges are shared by at most two faces.
    /// # Examples
    /// ```
    /// # use truck_topology::*;
    /// # use truck_topology::shell::ShellCondition;
    /// let v = Vertex::news(6);
    /// let edge = [
    ///     Edge::new(v[0], v[1]),
    ///     Edge::new(v[0], v[2]),
    ///     Edge::new(v[1], v[2]),
    ///     Edge::new(v[1], v[3]),
    ///     Edge::new(v[1], v[4]),
    ///     Edge::new(v[2], v[4]),
    ///     Edge::new(v[2], v[5]),
    ///     Edge::new(v[3], v[4]),
    ///     Edge::new(v[4], v[5]),
    /// ];
    /// let wire = vec![
    ///     Wire::from(vec![edge[0], edge[2], edge[1].inverse()]),
    ///     Wire::from(vec![edge[3], edge[7], edge[4].inverse()]),
    ///     Wire::from(vec![edge[5], edge[8], edge[6].inverse()]),
    ///     Wire::from(vec![edge[2], edge[5], edge[4].inverse()]),
    /// ];
    /// let shell: Shell = wire.into_iter().map(|w| Face::new(w)).collect();
    /// // This shell is regular, but not oriented.
    /// // It is because the orientations of shell[0] and shell[3] are incompatible on edge[2].
    /// assert_eq!(shell.shell_condition(), ShellCondition::Regular);
    /// ```
    Regular,
    /// The orientations of faces are compatible.
    /// # Examples
    /// ```
    /// # use truck_topology::*;
    /// # use truck_topology::shell::ShellCondition;
    /// let v = Vertex::news(6);
    /// let edge = [
    ///     Edge::new(v[0], v[1]),
    ///     Edge::new(v[0], v[2]),
    ///     Edge::new(v[1], v[2]),
    ///     Edge::new(v[1], v[3]),
    ///     Edge::new(v[1], v[4]),
    ///     Edge::new(v[2], v[4]),
    ///     Edge::new(v[2], v[5]),
    ///     Edge::new(v[3], v[4]),
    ///     Edge::new(v[4], v[5]),
    /// ];
    /// let wire = vec![
    ///     Wire::from(vec![edge[0], edge[2], edge[1].inverse()]),
    ///     Wire::from(vec![edge[3], edge[7], edge[4].inverse()]),
    ///     Wire::from(vec![edge[5], edge[8], edge[6].inverse()]),
    ///     Wire::from(vec![edge[2].inverse(), edge[4], edge[5].inverse()]),
    /// ];
    /// let shell: Shell = wire.into_iter().map(|w| Face::new(w)).collect();
    /// // The orientations of all faces in the shell are compatible on the shared edges.
    /// // This shell is not closed because edge[0] is included in only the 0th face.
    /// assert_eq!(shell.shell_condition(), ShellCondition::Oriented);
    /// ```
    Oriented,
    /// All edges are shared by two faces.
    /// # Examples
    /// ```
    /// # use truck_topology::*;
    /// # use truck_topology::shell::ShellCondition;
    /// let v = Vertex::news(8);
    /// let edge = [
    ///     Edge::new(v[0], v[1]),
    ///     Edge::new(v[1], v[2]),
    ///     Edge::new(v[2], v[3]),
    ///     Edge::new(v[3], v[0]),
    ///     Edge::new(v[0], v[4]),
    ///     Edge::new(v[1], v[5]),
    ///     Edge::new(v[2], v[6]),
    ///     Edge::new(v[3], v[7]),
    ///     Edge::new(v[4], v[5]),
    ///     Edge::new(v[5], v[6]),
    ///     Edge::new(v[6], v[7]),
    ///     Edge::new(v[7], v[4]),
    /// ];
    /// let wire = vec![
    ///     Wire::from(vec![edge[0], edge[1], edge[2], edge[3]]),
    ///     Wire::from(vec![edge[0].inverse(), edge[4], edge[8], edge[5].inverse()]),
    ///     Wire::from(vec![edge[1].inverse(), edge[5], edge[9], edge[6].inverse()]),
    ///     Wire::from(vec![edge[2].inverse(), edge[6], edge[10], edge[7].inverse()]),
    ///     Wire::from(vec![edge[3].inverse(), edge[7], edge[11], edge[4].inverse()]),
    ///     Wire::from(vec![edge[8], edge[9], edge[10], edge[11]]),
    /// ];
    /// let mut shell: Shell = wire.into_iter().map(|w| Face::new(w)).collect();
    /// shell[5].invert();
    /// assert_eq!(shell.shell_condition(), ShellCondition::Closed);
    /// ```
    Closed,
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
