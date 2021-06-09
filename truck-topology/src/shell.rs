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

    /// Returns an iterator over the edges.
    #[inline(always)]
    pub fn edge_iter<'a>(&'a self) -> impl Iterator<Item = Edge<P, C>> + 'a {
        self.face_iter().flat_map(Face::boundaries).flatten()
    }

    /// Returns an iterator over the vertices.
    #[inline(always)]
    pub fn vertex_iter<'a>(&'a self) -> impl Iterator<Item = Vertex<P>> + 'a {
        self.edge_iter().map(|edge| edge.front().clone())
    }

    /// Moves all the faces of `other` into `self`, leaving `other` empty.
    #[inline(always)]
    pub fn append(&mut self, other: &mut Shell<P, C, S>) {
        self.face_list.append(&mut other.face_list);
    }

    /// Determines the shell conditions: non-regular, regular, oriented, or closed.  
    /// The complexity increases in proportion to the number of edges.
    ///
    /// Examples for each condition can be found on the page of
    /// [`ShellCondition`](./shell/enum.ShellCondition.html).
    pub fn shell_condition(&self) -> ShellCondition {
        self.face_iter()
            .flat_map(Face::boundary_iters)
            .flatten()
            .collect::<Boundaries<C>>()
            .condition()
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
    /// let shell: Shell<_, _, _> = wire.into_iter().map(|w| Face::new(vec![w], ())).collect();
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
        let boundaries: Boundaries<C> = self
            .face_iter()
            .flat_map(Face::boundary_iters)
            .flatten()
            .collect();
        let mut boundary_edges = Vec::new();
        let mut vemap: HashMap<Vertex<P>, Edge<P, C>> = HashMap::new();
        let edge_iter = self.face_iter().flat_map(Face::boundary_iters).flatten();
        for edge in edge_iter {
            if boundaries.boundaries.get(&edge.id()).is_some() {
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
    /// let shell: Shell<_, _, _> = wire.into_iter().map(|w| Face::new(vec![w], ())).collect();
    /// let adjacency = shell.vertex_adjacency();
    /// let v0_ads_vec = adjacency.get(&v[0].id()).unwrap();
    /// let v0_ads: HashSet<&VertexID<()>> = HashSet::from_iter(v0_ads_vec);
    /// assert_eq!(v0_ads, HashSet::from_iter(vec![&v[2].id(), &v[3].id()]));
    /// ```
    pub fn vertex_adjacency(&self) -> HashMap<VertexID<P>, Vec<VertexID<P>>> {
        let mut adjacency: HashMap<VertexID<P>, Vec<VertexID<P>>> = HashMap::new();
        let mut done_edge: HashSet<EdgeID<C>> = HashSet::new();
        let edge_iter = self.face_iter().flat_map(|face| {
            face.absolute_boundaries()
                .iter()
                .flat_map(|wire| wire.edge_iter())
        });
        for edge in edge_iter {
            if !done_edge.insert(edge.id()) {
                continue;
            }
            let v0 = edge.front().id();
            let v1 = edge.back().id();
            adjacency.entry(v0).or_insert(Vec::new()).push(v1);
            adjacency.entry(v1).or_insert(Vec::new()).push(v0);
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
    /// let shell: Shell<_, _, _> = wire.into_iter().map(|w| Face::new(vec![w], ())).collect();
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
            let edge_iter = face
                .absolute_boundaries()
                .iter()
                .flat_map(|wire| wire.edge_iter());
            for edge in edge_iter {
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
    /// let face0 = Face::new(vec![wire0], ());
    /// let wire1 = Wire::from_iter(vec![
    ///     &Edge::new(&v[3], &v[1], ()),
    ///     &shared_edge,
    ///     &Edge::new(&v[2], &v[3], ()),
    /// ]);
    /// let face1 = Face::new(vec![wire1], ());
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
    /// let face0 = Face::new(vec![wire0], ());
    /// let wire1 = Wire::from_iter(vec![
    ///     &Edge::new(&v[3], &v[4], ()),
    ///     &Edge::new(&v[4], &v[5], ()),
    ///     &Edge::new(&v[5], &v[3], ())
    /// ]);
    /// let face1 = Face::new(vec![wire1], ());
    /// let shell: Shell<_, _, _> = vec![face0, face1].into();
    /// assert!(!shell.is_connected());
    /// ```
    pub fn is_connected(&self) -> bool {
        let mut adjacency = self.vertex_adjacency();
        for face in self {
            for wire in face.boundaries.windows(2) {
                let v0 = wire[0].front_vertex().unwrap();
                let v1 = wire[1].front_vertex().unwrap();
                adjacency.get_mut(&v0.id()).unwrap().push(v1.id());
                adjacency.get_mut(&v1.id()).unwrap().push(v0.id());
            }
        }
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
    ///     Face::new(vec![wire0], ()),
    ///     Face::new(vec![wire1], ()),
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
    /// let shell: Shell<_, _, _> = wire.into_iter().map(|w| Face::new(vec![w], ())).collect();
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
    /// let shell: Shell<_, _, _> = wire.into_iter().map(|w| Face::new(vec![w], ())).collect();
    /// assert_eq!(shell.shell_condition(), ShellCondition::Closed);
    /// assert!(shell.is_connected());
    /// assert_eq!(shell.singular_vertices(), vec![v[0].clone()]);
    /// ```
    pub fn singular_vertices(&self) -> Vec<Vertex<P>> {
        let mut vert_wise_adjacency: HashMap<Vertex<P>, HashMap<EdgeID<C>, Vec<EdgeID<C>>>> =
            HashMap::new();
        for face in self.face_iter() {
            let first_edge = &face.absolute_boundaries()[0][0];
            let mut edge_iter = face
                .absolute_boundaries()
                .iter()
                .flat_map(|wire| wire.edge_iter())
                .peekable();
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
    /// Returns the consistence of the geometry of end vertices
    /// and the geometry of edge.
    #[inline(always)]
    pub fn is_geometric_consistent(&self) -> bool
    where
        P: Tolerance,
        C: ParametricCurve<Point = P>,
        S: IncludeCurve<C>, {
        self.iter().all(|face| face.is_geometric_consistent())
    }

    /// Cuts one edge into two edges at vertex.
    /// # Failures
    /// Returns `false` and not edit `self` if:
    /// - `vertex` is already included in `self`, or
    /// - cutting of edge fails.
    #[inline(always)]
    pub fn cut_edge(&mut self, edge_id: EdgeID<C>, vertex: &Vertex<P>) -> bool
    where
        P: Clone,
        C: Cut<Point = P> + SearchParameter<Point = P, Parameter = f64>, {
        if self.vertex_iter().any(|v| &v == vertex) {
            return false;
        }
        let mut edges = None;
        self.iter_mut()
            .flat_map(|face| face.boundaries.iter_mut())
            .try_for_each(|wire| {
                let find_res = wire
                    .iter()
                    .enumerate()
                    .find(|(_, edge)| edge.id() == edge_id);
                let (idx, edge) = match find_res {
                    Some(got) => got,
                    None => return Some(()),
                };
                if edges.is_none() {
                    let absedge = match edge.orientation() {
                        true => edge.clone(),
                        false => edge.inverse(),
                    };
                    edges = Some(absedge.cut(vertex)?);
                }
                let edges = edges.as_ref().unwrap();
                let new_wire = match edge.orientation() {
                    true => Wire::from(vec![edges.0.clone(), edges.1.clone()]),
                    false => Wire::from(vec![edges.1.inverse(), edges.0.inverse()]),
                };
                let flag = wire.swap_edge_into_wire(idx, new_wire);
                debug_assert!(flag);
                Some(())
            })
            .is_some()
    }
}

impl<P, C, S> Clone for Shell<P, C, S> {
    #[inline(always)]
    fn clone(&self) -> Shell<P, C, S> {
        Shell {
            face_list: self.face_list.clone(),
        }
    }
}

impl<P, C, S> From<Shell<P, C, S>> for Vec<Face<P, C, S>> {
    #[inline(always)]
    fn from(shell: Shell<P, C, S>) -> Vec<Face<P, C, S>> { shell.face_list }
}

impl<P, C, S> From<Vec<Face<P, C, S>>> for Shell<P, C, S> {
    #[inline(always)]
    fn from(faces: Vec<Face<P, C, S>>) -> Shell<P, C, S> { Shell { face_list: faces } }
}

impl<P, C, S> std::iter::FromIterator<Face<P, C, S>> for Shell<P, C, S> {
    #[inline(always)]
    fn from_iter<I: IntoIterator<Item = Face<P, C, S>>>(iter: I) -> Shell<P, C, S> {
        Shell {
            face_list: iter.into_iter().collect(),
        }
    }
}

impl<P, C, S> IntoIterator for Shell<P, C, S> {
    type Item = Face<P, C, S>;
    type IntoIter = std::vec::IntoIter<Face<P, C, S>>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter { self.face_list.into_iter() }
}

impl<'a, P, C, S> IntoIterator for &'a Shell<P, C, S> {
    type Item = &'a Face<P, C, S>;
    type IntoIter = std::slice::Iter<'a, Face<P, C, S>>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter { self.face_list.iter() }
}

impl<P, C, S> std::ops::Deref for Shell<P, C, S> {
    type Target = Vec<Face<P, C, S>>;
    #[inline(always)]
    fn deref(&self) -> &Vec<Face<P, C, S>> { &self.face_list }
}

impl<P, C, S> std::ops::DerefMut for Shell<P, C, S> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Vec<Face<P, C, S>> { &mut self.face_list }
}

/// The reference iterator over all faces in shells
pub type FaceIter<'a, P, C, S> = std::slice::Iter<'a, Face<P, C, S>>;
/// The mutable reference iterator over all faces in shells
pub type FaceIterMut<'a, P, C, S> = std::slice::IterMut<'a, Face<P, C, S>>;
/// The into iterator over all faces in shells
pub type FaceIntoIter<P, C, S> = std::vec::IntoIter<Face<P, C, S>>;

/// The shell conditions being determined by the half-edge model.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
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
    /// let shell: Shell<_, _, _> = wire.into_iter().map(|w| Face::new(vec![w], ())).collect();
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
    /// let shell: Shell<_, _, _> = wire.into_iter().map(|w| Face::new(vec![w], ())).collect();
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
    /// let shell: Shell<_, _, _> = wire.into_iter().map(|w| Face::new(vec![w], ())).collect();
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
    /// let mut shell: Shell<_, _, _> = wire.into_iter().map(|w| Face::new(vec![w], ())).collect();
    /// shell[5].invert();
    /// assert_eq!(shell.shell_condition(), ShellCondition::Closed);
    /// ```
    Closed,
}

impl std::ops::BitAnd for ShellCondition {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        match (self, other) {
            (Self::Irregular, _) => Self::Irregular,
            (_, Self::Irregular) => Self::Irregular,
            (Self::Regular, _) => Self::Regular,
            (_, Self::Regular) => Self::Regular,
            (Self::Oriented, _) => Self::Oriented,
            (_, Self::Oriented) => Self::Oriented,
            (Self::Closed, Self::Closed) => Self::Closed,
        }
    }
}

#[derive(Debug, Clone)]
struct Boundaries<C> {
    checked: HashSet<EdgeID<C>>,
    boundaries: HashMap<EdgeID<C>, bool>,
    condition: ShellCondition,
}

impl<C> Boundaries<C> {
    #[inline(always)]
    fn new() -> Self {
        Self {
            checked: Default::default(),
            boundaries: Default::default(),
            condition: ShellCondition::Oriented,
        }
    }

    #[inline(always)]
    fn insert<P>(&mut self, edge: &Edge<P, C>) {
        self.condition = self.condition
            & match (
                self.checked.insert(edge.id()),
                self.boundaries.insert(edge.id(), edge.orientation()),
            ) {
                (true, None) => ShellCondition::Oriented,
                (false, None) => ShellCondition::Irregular,
                (true, Some(_)) => panic!("unexpected case!"),
                (false, Some(ori)) => {
                    self.boundaries.remove(&edge.id());
                    match edge.orientation() == ori {
                        true => ShellCondition::Regular,
                        false => ShellCondition::Oriented,
                    }
                }
            }
    }

    #[inline(always)]
    fn condition(&self) -> ShellCondition {
        if self.condition == ShellCondition::Oriented && self.boundaries.is_empty() {
            ShellCondition::Closed
        } else {
            self.condition
        }
    }
}

impl<P, C> std::iter::FromIterator<Edge<P, C>> for Boundaries<C> {
    #[inline(always)]
    fn from_iter<I: IntoIterator<Item = Edge<P, C>>>(iter: I) -> Self {
        let mut boundaries = Boundaries::new();
        iter.into_iter().for_each(|edge| boundaries.insert(&edge));
        boundaries
    }
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
