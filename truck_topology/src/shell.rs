use crate::*;
use std::collections::{HashMap, HashSet};
use std::vec::Vec;

impl<P, C, S> Shell<P, C, S> {
    /// Creates the empty shell.
    #[inline(always)]
    pub const fn new() -> Shell<P, C, S> {
        Shell {
            face_list: Vec::new(),
        }
    }

    /// Creates the empty shell with space for at least `capacity` faces.
    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Shell<P, C, S> {
        Shell {
            face_list: Vec::with_capacity(capacity),
        }
    }

    /// Returns an iterator over the faces. Practically, an alias of `iter()`.
    #[inline(always)]
    pub fn face_iter(&self) -> FaceIter<P, C, S> { self.iter() }

    /// Returns a mutable iterator over the faces. Practically, an alias of `iter_mut()`.
    #[inline(always)]
    pub fn face_iter_mut(&mut self) -> FaceIterMut<P, C, S> { self.iter_mut() }

    /// Creates a consuming iterator. Practically, an alias of `into_iter()`.
    #[inline(always)]
    pub fn face_into_iter(self) -> FaceIntoIter<P, C, S> { self.face_list.into_iter() }

    /// Moves all the faces of `other` into `self`, leaving `other` empty.
    #[inline(always)]
    pub fn append(&mut self, other: &mut Shell<P, C, S>) { self.face_list.append(&mut other.face_list); }

    /// Returns a tuple.
    /// * The 0th component is whether the shell is regular or not.
    /// * The 1st component is whether the shell is oriented or not.
    /// * The 2nd component is whether the shell is closed or not.
    /// * The 3rd component is the set of all ids of the inner edge of the shell.
    fn inner_edge_extraction(&self) -> (bool, bool, bool, HashSet<EdgeID<C>>) {
        let mut all_edges: HashMap<EdgeID<C>, bool> = HashMap::with_capacity(self.face_list.len());
        let mut inner_edges: HashSet<EdgeID<C>> = HashSet::with_capacity(self.face_list.len());

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
    /// use truck_topology::*;
    /// use truck_topology::shell::ShellCondition;
    /// use std::iter::FromIterator;
    /// let v = Vertex::news(&[(); 6]);
    /// let edge = [
    ///     Edge::new(&v[0], &v[1], ()),
    ///     Edge::new(&v[0], &v[2], ()),
    ///     Edge::new(&v[1], &v[2], ()),
    ///     Edge::new(&v[1], &v[3], ()),
    ///     Edge::new(&v[1], &v[4], ()),
    ///     Edge::new(&v[2], &v[4], ()),
    ///     Edge::new(&v[2], &v[5], ()),
    ///     Edge::new(&v[3], &v[4], ()),
    ///     Edge::new(&v[4], &v[5], ()),
    /// ];
    /// let wire = vec![
    ///     Wire::from_iter(vec![&edge[0], &edge[2], &edge[1].inverse()]),
    ///     Wire::from_iter(vec![&edge[3], &edge[7], &edge[4].inverse()]),
    ///     Wire::from_iter(vec![&edge[5], &edge[8], &edge[6].inverse()]),
    ///     Wire::from_iter(vec![&edge[2].inverse(), &edge[4], &edge[5].inverse()]),
    /// ];
    /// let shell: Shell<_, _, _> = wire.into_iter().map(|w| Face::new(w, ())).collect();
    /// let boundary = shell.extract_boundaries()[0].clone();
    /// assert_eq!(
    ///     boundary,
    ///     Wire::from_iter(vec![&edge[0], &edge[3], &edge[7], &edge[8], &edge[6].inverse(), &edge[1].inverse()]),
    /// );
    /// ```
    /// # Remarks
    /// This method is optimized when the shell is oriented.
    /// Even if the shell is not oriented, all the edges of the boundary are extracted.
    /// However, the connected components of the boundary are split into several wires.
    pub fn extract_boundaries(&self) -> Vec<Wire<P, C>> {
        let (_, _, _, inner_edges) = self.inner_edge_extraction();
        let mut boundary_edges = Vec::new();
        let mut vemap: HashMap<Vertex<P>, Edge<P, C>> = HashMap::new();
        let edge_iter = self.face_iter().flat_map(Face::boundary_iter);
        for edge in edge_iter {
            if inner_edges.get(&edge.id()).is_none() {
                boundary_edges.push(edge.clone());
                vemap.insert(edge.front().clone(), edge.clone());
            }
        }
        let mut res = Vec::new();
        for edge in boundary_edges {
            if let Some(mut cursor) = vemap.remove(&edge.front()) {
                let mut wire = Wire::from(vec![cursor.clone()]);
                loop {
                    cursor = match vemap.remove(&cursor.back()) {
                        None => break,
                        Some(got) => {
                            wire.push_back(got.clone());
                            got.clone()
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
    /// For the returned hashmap `map` and each vertex `v`,
    /// the vector `map[&v]` cosists all vertices which is adjacent to `v`.
    /// # Exmaples
    /// ```
    /// use truck_topology::*;
    /// use std::collections::HashSet;
    /// use std::iter::FromIterator;
    /// let v = Vertex::news(&[(); 4]);
    /// let edge = [
    ///     Edge::new(&v[0], &v[2], ()),
    ///     Edge::new(&v[0], &v[3], ()),
    ///     Edge::new(&v[1], &v[2], ()),
    ///     Edge::new(&v[1], &v[3], ()),
    ///     Edge::new(&v[2], &v[3], ()),
    /// ];
    /// let wire = vec![
    ///     Wire::from_iter(vec![&edge[0], &edge[4], &edge[1].inverse()]),
    ///     Wire::from_iter(vec![&edge[2], &edge[4], &edge[3].inverse()]),
    /// ];
    /// let shell: Shell<_, _, _> = wire.into_iter().map(|w| Face::new(w, ())).collect();
    /// let adjacency = shell.vertex_adjacency();
    /// let v0_ads_vec = adjacency.get(&v[0]).unwrap();
    /// let v0_ads: HashSet<&Vertex<()>> = HashSet::from_iter(v0_ads_vec);
    /// assert_eq!(v0_ads, HashSet::from_iter(vec![&v[2], &v[3]]));
    /// ```
    pub fn vertex_adjacency(&self) -> HashMap<Vertex<P>, Vec<Vertex<P>>> {
        let mut adjacency: HashMap<Vertex<P>, Vec<Vertex<P>>> = HashMap::new();
        let mut done_edge: HashSet<EdgeID<C>> = HashSet::new();
        let edge_iter = self
            .face_iter()
            .flat_map(|face| face.absolute_boundary().edge_iter());
        for edge in edge_iter {
            if !done_edge.insert(edge.id()) {
                continue;
            }
            let v0 = edge.front();
            let v1 = edge.back();
            adjacency.entry(v0.clone()).or_insert(Vec::new()).push(v1.clone());
            adjacency.entry(v1.clone()).or_insert(Vec::new()).push(v0.clone());
        }
        adjacency
    }

    /// Returns the adjacency matrix of faces in the shell.
    ///
    /// For the returned hashmap `map` and each face `face`,
    /// the vector `map[&face]` consists all faces adjacent to `face`.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// use truck_topology::shell::ShellCondition;
    /// use std::iter::FromIterator;
    /// let v = Vertex::news(&[(); 6]);
    /// let edge = [
    ///     Edge::new(&v[0], &v[1], ()),
    ///     Edge::new(&v[0], &v[2], ()),
    ///     Edge::new(&v[1], &v[2], ()),
    ///     Edge::new(&v[1], &v[3], ()),
    ///     Edge::new(&v[1], &v[4], ()),
    ///     Edge::new(&v[2], &v[4], ()),
    ///     Edge::new(&v[2], &v[5], ()),
    ///     Edge::new(&v[3], &v[4], ()),
    ///     Edge::new(&v[4], &v[5], ()),
    /// ];
    /// let wire = vec![
    ///     Wire::from_iter(vec![&edge[0], &edge[2], &edge[1].inverse()]),
    ///     Wire::from_iter(vec![&edge[3], &edge[7], &edge[4].inverse()]),
    ///     Wire::from_iter(vec![&edge[5], &edge[8], &edge[6].inverse()]),
    ///     Wire::from_iter(vec![&edge[2].inverse(), &edge[4], &edge[5].inverse()]),
    /// ];
    /// let shell: Shell<_, _, _> = wire.into_iter().map(|w| Face::new(w, ())).collect();
    /// let face_adjacency = shell.face_adjacency();
    /// assert_eq!(face_adjacency[&shell[0]].len(), 1);
    /// assert_eq!(face_adjacency[&shell[1]].len(), 1);
    /// assert_eq!(face_adjacency[&shell[2]].len(), 1);
    /// assert_eq!(face_adjacency[&shell[3]].len(), 3);
    /// ```
    pub fn face_adjacency(&self) -> HashMap<&Face<P, C, S>, Vec<&Face<P, C, S>>> {
        let mut adjacency: HashMap<&Face<P, C, S>, Vec<&Face<P, C, S>>> = HashMap::new();
        let mut edge_face_map: HashMap<EdgeID<C>, Vec<&Face<P, C, S>>> = HashMap::new();
        for face in self.face_iter() {
            for edge in face.absolute_boundary().edge_iter() {
                if let Some(vec) = edge_face_map.get_mut(&edge.id()) {
                    for tmp in vec {
                        adjacency.entry(face).or_insert(Vec::new()).push(tmp);
                        adjacency.entry(tmp).or_insert(Vec::new()).push(face);
                    }
                } else {
                    adjacency.entry(face).or_insert(Vec::new());
                    edge_face_map.insert(edge.id(), vec![face]);
                }
            }
        }
        adjacency
    }

    /// Returns whether the shell is connected or not.
    /// # Examples
    /// ```
    /// // The empty shell is connected.
    /// use truck_topology::*;
    /// assert!(Shell::<(), (), ()>::new().is_connected());
    /// ```
    /// ```
    /// // An example of a connected shell
    /// use truck_topology::*;
    /// use std::iter::FromIterator;
    /// let v = Vertex::news(&[(); 4]);
    /// let shared_edge = Edge::new(&v[1], &v[2], ());
    /// let wire0 = Wire::from_iter(vec![
    ///     &Edge::new(&v[0], &v[1], ()),
    ///     &shared_edge,
    ///     &Edge::new(&v[2], &v[0], ()),
    /// ]);
    /// let face0 = Face::new(wire0, ());
    /// let wire1 = Wire::from_iter(vec![
    ///     &Edge::new(&v[3], &v[1], ()),
    ///     &shared_edge,
    ///     &Edge::new(&v[2], &v[3], ()),
    /// ]);
    /// let face1 = Face::new(wire1, ());
    /// let shell: Shell<_, _, _> = vec![face0, face1].into();
    /// assert!(shell.is_connected());
    /// ```
    /// ```
    /// // An example of a non-connected shell
    /// use truck_topology::*;
    /// use std::iter::FromIterator;
    /// let v = Vertex::news(&[(); 6]);
    /// let wire0 = Wire::from_iter(vec![
    ///     Edge::new(&v[0], &v[1], ()),
    ///     Edge::new(&v[1], &v[2], ()),
    ///     Edge::new(&v[2], &v[0], ())
    /// ]);
    /// let face0 = Face::new(wire0, ());
    /// let wire1 = Wire::from_iter(vec![
    ///     &Edge::new(&v[3], &v[4], ()),
    ///     &Edge::new(&v[4], &v[5], ()),
    ///     &Edge::new(&v[5], &v[3], ())
    /// ]);
    /// let face1 = Face::new(wire1, ());
    /// let shell: Shell<_, _, _> = vec![face0, face1].into();
    /// assert!(!shell.is_connected());
    /// ```
    pub fn is_connected(&self) -> bool {
        let mut adjacency = self.vertex_adjacency();
        check_connectivity(&mut adjacency)
    }

    /// Returns a vector consisting of shells of each connected components.
    /// # Examples
    /// ```
    /// use truck_topology::Shell;
    /// // The empty shell has no connected component.
    /// assert!(Shell::<(), (), ()>::new().connected_components().is_empty());
    /// ```
    /// # Remarks
    /// Since this method uses the face adjacency matrix, multiple components
    /// are perhaps generated even if the shell is connected. In that case,
    /// there is a pair of faces such that share vertices but not edges.
    /// ```
    /// use truck_topology::*;
    /// use std::iter::FromIterator;
    /// let v = Vertex::news(&[(); 5]);
    /// let wire0 = Wire::from_iter(vec![
    ///     Edge::new(&v[0], &v[1], ()),
    ///     Edge::new(&v[1], &v[2], ()),
    ///     Edge::new(&v[2], &v[0], ()),
    /// ]);
    /// let wire1 = Wire::from_iter(vec![
    ///     Edge::new(&v[0], &v[3], ()),
    ///     Edge::new(&v[3], &v[4], ()),
    ///     Edge::new(&v[4], &v[0], ()),
    /// ]);
    /// let shell = Shell::from(vec![
    ///     Face::new(wire0, ()),
    ///     Face::new(wire1, ()),
    /// ]);
    /// assert!(shell.is_connected());
    /// assert_eq!(shell.connected_components().len(), 2);
    /// ```
    pub fn connected_components(&self) -> Vec<Shell<P, C, S>> {
        let mut adjacency = self.face_adjacency();
        let components = create_components(&mut adjacency);
        components
            .into_iter()
            .map(|vec| vec.into_iter().map(|face| face.clone()).collect())
            .collect()
    }

    /// Returns the vector of all singular vertices.
    ///
    /// Here, we say that a vertex is singular if, for a sufficiently small neighborhood U of
    /// the vertex, the set U - {the vertex} is not connected.
    ///
    /// A regular, oriented, or closed shell becomes a manifold if and only if the shell has
    /// no singular vertices.
    /// # Examples
    /// ```
    /// // A regular manifold: Mobius bundle
    /// use truck_topology::*;
    /// use truck_topology::shell::ShellCondition;
    /// use std::iter::FromIterator;
    ///
    /// let v = Vertex::news(&[(), (), (), ()]);
    /// let edge = [
    ///     Edge::new(&v[0], &v[1], ()),
    ///     Edge::new(&v[1], &v[2], ()),
    ///     Edge::new(&v[2], &v[0], ()),
    ///     Edge::new(&v[1], &v[3], ()),
    ///     Edge::new(&v[3], &v[2], ()),
    ///     Edge::new(&v[0], &v[3], ()),
    /// ];
    /// let wire = vec![
    ///     Wire::from_iter(vec![&edge[0], &edge[3], &edge[4], &edge[2]]),
    ///     Wire::from_iter(vec![&edge[1], &edge[2], &edge[5], &edge[3].inverse()]),
    /// ];
    /// let shell: Shell<_, _, _> = wire.into_iter().map(|w| Face::new(w, ())).collect();
    /// assert_eq!(shell.shell_condition(), ShellCondition::Regular);
    /// assert!(shell.singular_vertices().is_empty());
    /// ```
    /// ```
    /// // A closed and connected shell which has a singular vertex.
    /// use truck_topology::*;
    /// use truck_topology::shell::*;
    /// use std::iter::FromIterator;
    ///
    /// let v = Vertex::news(&[(); 7]);
    /// let edge = [
    ///     Edge::new(&v[0], &v[1], ()), // 0
    ///     Edge::new(&v[0], &v[2], ()), // 1
    ///     Edge::new(&v[0], &v[3], ()), // 2
    ///     Edge::new(&v[1], &v[2], ()), // 3
    ///     Edge::new(&v[2], &v[3], ()), // 4
    ///     Edge::new(&v[3], &v[1], ()), // 5
    ///     Edge::new(&v[0], &v[4], ()), // 6
    ///     Edge::new(&v[0], &v[5], ()), // 7
    ///     Edge::new(&v[0], &v[6], ()), // 8
    ///     Edge::new(&v[4], &v[5], ()), // 9
    ///     Edge::new(&v[5], &v[6], ()), // 10
    ///     Edge::new(&v[6], &v[4], ()), // 11
    /// ];
    /// let wire = vec![
    ///     Wire::from_iter(vec![&edge[0].inverse(), &edge[1], &edge[3].inverse()]),
    ///     Wire::from_iter(vec![&edge[1].inverse(), &edge[2], &edge[4].inverse()]),
    ///     Wire::from_iter(vec![&edge[2].inverse(), &edge[0], &edge[5].inverse()]),
    ///     Wire::from_iter(vec![&edge[3], &edge[4], &edge[5]]),
    ///     Wire::from_iter(vec![&edge[6].inverse(), &edge[7], &edge[9].inverse()]),
    ///     Wire::from_iter(vec![&edge[7].inverse(), &edge[8], &edge[10].inverse()]),
    ///     Wire::from_iter(vec![&edge[8].inverse(), &edge[6], &edge[11].inverse()]),
    ///     Wire::from_iter(vec![&edge[9], &edge[10], &edge[11]]),
    /// ];
    /// let shell: Shell<_, _, _> = wire.into_iter().map(|w| Face::new(w, ())).collect();
    /// assert_eq!(shell.shell_condition(), ShellCondition::Closed);
    /// assert!(shell.is_connected());
    /// assert_eq!(shell.singular_vertices(), vec![v[0].clone()]);
    /// ```
    pub fn singular_vertices(&self) -> Vec<Vertex<P>> {
        let mut vert_wise_adjacency: HashMap<Vertex<P>, HashMap<EdgeID<C>, Vec<EdgeID<C>>>> = HashMap::new();
        for face in self.face_iter() {
            let first_edge = &face.absolute_boundary()[0];
            let mut edge_iter = face.absolute_boundary().edge_iter().peekable();
            while let Some(edge) = edge_iter.next() {
                let adjacency = vert_wise_adjacency
                    .entry(edge.back().clone())
                    .or_insert(HashMap::new());
                let next_edge = *edge_iter.peek().unwrap_or(&first_edge);
                adjacency
                    .entry(edge.id())
                    .or_insert(Vec::new())
                    .push(next_edge.id());
                adjacency
                    .entry(next_edge.id())
                    .or_insert(Vec::new())
                    .push(edge.id());
            }
        }
        vert_wise_adjacency
            .into_iter()
            .filter_map(|(vertex, mut adjacency)| {
                Some(vertex).filter(|_| !check_connectivity(&mut adjacency))
            })
            .collect()
    }
}

impl<P, C, S> From<Shell<P, C, S>> for Vec<Face<P, C, S>> {
    fn from(shell: Shell<P, C, S>) -> Vec<Face<P, C, S>> { shell.face_list }
}

impl<P, C, S> From<Vec<Face<P, C, S>>> for Shell<P, C, S> {
    fn from(faces: Vec<Face<P, C, S>>) -> Shell<P, C, S> { Shell { face_list: faces } }
}

impl<P, C, S> std::iter::FromIterator<Face<P, C, S>> for Shell<P, C, S> {
    fn from_iter<I: IntoIterator<Item = Face<P, C, S>>>(iter: I) -> Shell<P, C, S> {
        Shell {
            face_list: iter.into_iter().collect(),
        }
    }
}

impl<P, C, S> std::ops::Deref for Shell<P, C, S> {
    type Target = Vec<Face<P, C, S>>;
    fn deref(&self) -> &Vec<Face<P, C, S>> { &self.face_list }
}

impl<P, C, S> std::ops::DerefMut for Shell<P, C, S> {
    fn deref_mut(&mut self) -> &mut Vec<Face<P, C, S>> { &mut self.face_list }
}

/// The reference iterator over all faces in shells
pub type FaceIter<'a, P, C, S> = std::slice::Iter<'a, Face<P, C, S>>;
/// The mutable reference iterator over all faces in shells
pub type FaceIterMut<'a, P, C, S> = std::slice::IterMut<'a, Face<P, C, S>>;
/// The into iterator over all faces in shells
pub type FaceIntoIter<P, C, S> = std::vec::IntoIter<Face<P, C, S>>;

/// The shell conditions being determined by the half-edge model.
#[derive(PartialEq, Eq, Debug)]
pub enum ShellCondition {
    /// This shell is not regular.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// use truck_topology::shell::ShellCondition;
    /// use std::iter::FromIterator;
    /// let v = Vertex::news(&[(); 5]);
    /// let edge = [
    ///    Edge::new(&v[0], &v[1], ()),
    ///    Edge::new(&v[0], &v[2], ()),
    ///    Edge::new(&v[0], &v[3], ()),
    ///    Edge::new(&v[0], &v[4], ()),
    ///    Edge::new(&v[1], &v[2], ()),
    ///    Edge::new(&v[1], &v[3], ()),
    ///    Edge::new(&v[1], &v[4], ()),
    /// ];
    /// let wire = vec![
    ///    Wire::from_iter(vec![&edge[0], &edge[4], &edge[1].inverse()]),
    ///    Wire::from_iter(vec![&edge[0], &edge[5], &edge[2].inverse()]),
    ///    Wire::from_iter(vec![&edge[0], &edge[6], &edge[3].inverse()]),
    /// ];
    /// let shell: Shell<_, _, _> = wire.into_iter().map(|w| Face::new(w, ())).collect();
    /// // The shell is irregular because three faces share edge[0].
    /// assert_eq!(shell.shell_condition(), ShellCondition::Irregular);
    /// ```
    Irregular,
    /// All edges are shared by at most two faces.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// use truck_topology::shell::ShellCondition;
    /// use std::iter::FromIterator;
    /// let v = Vertex::news(&[(); 6]);
    /// let edge = [
    ///     Edge::new(&v[0], &v[1], ()),
    ///     Edge::new(&v[0], &v[2], ()),
    ///     Edge::new(&v[1], &v[2], ()),
    ///     Edge::new(&v[1], &v[3], ()),
    ///     Edge::new(&v[1], &v[4], ()),
    ///     Edge::new(&v[2], &v[4], ()),
    ///     Edge::new(&v[2], &v[5], ()),
    ///     Edge::new(&v[3], &v[4], ()),
    ///     Edge::new(&v[4], &v[5], ()),
    /// ];
    /// let wire = vec![
    ///     Wire::from_iter(vec![&edge[0], &edge[2], &edge[1].inverse()]),
    ///     Wire::from_iter(vec![&edge[3], &edge[7], &edge[4].inverse()]),
    ///     Wire::from_iter(vec![&edge[5], &edge[8], &edge[6].inverse()]),
    ///     Wire::from_iter(vec![&edge[2], &edge[5], &edge[4].inverse()]),
    /// ];
    /// let shell: Shell<_, _, _> = wire.into_iter().map(|w| Face::new(w, ())).collect();
    /// // This shell is regular, but not oriented.
    /// // It is because the orientations of shell[0] and shell[3] are incompatible on edge[2].
    /// assert_eq!(shell.shell_condition(), ShellCondition::Regular);
    /// ```
    Regular,
    /// The orientations of faces are compatible.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// use truck_topology::shell::ShellCondition;
    /// use std::iter::FromIterator;
    /// let v = Vertex::news(&[(); 6]);
    /// let edge = [
    ///     Edge::new(&v[0], &v[1] ,()),
    ///     Edge::new(&v[0], &v[2] ,()),
    ///     Edge::new(&v[1], &v[2] ,()),
    ///     Edge::new(&v[1], &v[3] ,()),
    ///     Edge::new(&v[1], &v[4] ,()),
    ///     Edge::new(&v[2], &v[4] ,()),
    ///     Edge::new(&v[2], &v[5] ,()),
    ///     Edge::new(&v[3], &v[4] ,()),
    ///     Edge::new(&v[4], &v[5] ,()),
    /// ];
    /// let wire = vec![
    ///     Wire::from_iter(vec![&edge[0], &edge[2], &edge[1].inverse()]),
    ///     Wire::from_iter(vec![&edge[3], &edge[7], &edge[4].inverse()]),
    ///     Wire::from_iter(vec![&edge[5], &edge[8], &edge[6].inverse()]),
    ///     Wire::from_iter(vec![&edge[2].inverse(), &edge[4], &edge[5].inverse()]),
    /// ];
    /// let shell: Shell<_, _, _> = wire.into_iter().map(|w| Face::new(w, ())).collect();
    /// // The orientations of all faces in the shell are compatible on the shared edges.
    /// // This shell is not closed because edge[0] is included in only the 0th face.
    /// assert_eq!(shell.shell_condition(), ShellCondition::Oriented);
    /// ```
    Oriented,
    /// All edges are shared by two faces.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// use truck_topology::shell::ShellCondition;
    /// use std::iter::FromIterator;
    /// let v = Vertex::news(&[(); 8]);
    /// let edge = [
    ///     Edge::new(&v[0], &v[1] ,()),
    ///     Edge::new(&v[1], &v[2] ,()),
    ///     Edge::new(&v[2], &v[3] ,()),
    ///     Edge::new(&v[3], &v[0] ,()),
    ///     Edge::new(&v[0], &v[4] ,()),
    ///     Edge::new(&v[1], &v[5] ,()),
    ///     Edge::new(&v[2], &v[6] ,()),
    ///     Edge::new(&v[3], &v[7] ,()),
    ///     Edge::new(&v[4], &v[5] ,()),
    ///     Edge::new(&v[5], &v[6] ,()),
    ///     Edge::new(&v[6], &v[7] ,()),
    ///     Edge::new(&v[7], &v[4] ,()),
    /// ];
    /// let wire = vec![
    ///     Wire::from_iter(vec![&edge[0], &edge[1], &edge[2], &edge[3]]),
    ///     Wire::from_iter(vec![&edge[0].inverse(), &edge[4], &edge[8], &edge[5].inverse()]),
    ///     Wire::from_iter(vec![&edge[1].inverse(), &edge[5], &edge[9], &edge[6].inverse()]),
    ///     Wire::from_iter(vec![&edge[2].inverse(), &edge[6], &edge[10], &edge[7].inverse()]),
    ///     Wire::from_iter(vec![&edge[3].inverse(), &edge[7], &edge[11], &edge[4].inverse()]),
    ///     Wire::from_iter(vec![&edge[8], &edge[9], &edge[10], &edge[11]]),
    /// ];
    /// let mut shell: Shell<_, _, _> = wire.into_iter().map(|w| Face::new(w, ())).collect();
    /// shell[5].invert();
    /// assert_eq!(shell.shell_condition(), ShellCondition::Closed);
    /// ```
    Closed,
}

fn check_connectivity<T>(adjacency: &mut HashMap<T, Vec<T>>) -> bool
where T: Eq + Clone + Hash {
    create_one_component(adjacency);
    adjacency.is_empty()
}

fn create_components<T>(adjacency: &mut HashMap<T, Vec<T>>) -> Vec<Vec<T>>
where T: Eq + Clone + Hash {
    let mut res = Vec::new();
    loop {
        let component = create_one_component(adjacency);
        match component.is_empty() {
            true => break,
            false => res.push(component),
        }
    }
    res
}

fn create_one_component<T>(adjacency: &mut HashMap<T, Vec<T>>) -> Vec<T>
where T: Eq + Hash + Clone {
    let mut iter = adjacency.keys();
    let first = match iter.next() {
        Some(key) => key.clone(),
        None => return Vec::new(),
    };
    let mut stack = vec![first];
    let mut res = Vec::new();
    while !stack.is_empty() {
        let i = stack.pop().unwrap();
        if let Some(vec) = adjacency.remove(&i) {
            res.push(i);
            for j in vec {
                stack.push(j);
            }
        }
    }
    res
}
