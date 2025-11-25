#![allow(clippy::mutable_key_type)]
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::cell::{Ref, RefCell, RefMut};
use typed_arena::Arena;

#[derive(Clone, Debug)]
struct NodeData<'a, T> {
    parents: RefCell<Vec<Node<'a, T>>>,
    children: RefCell<Vec<Node<'a, T>>>,
    entity: RefCell<T>,
}

impl<'a, T> NodeData<'a, T> {
    #[inline]
    fn new(entity: T) -> Self {
        Self {
            parents: Default::default(),
            children: Default::default(),
            entity: entity.into(),
        }
    }
}

/// Node of a dag.
#[derive(Debug)]
pub struct Node<'a, T>(&'a NodeData<'a, T>);

impl<'a, T> Clone for Node<'a, T> {
    #[inline]
    fn clone(&self) -> Self { *self }
}

impl<'a, T> Copy for Node<'a, T> {}

impl<'a, T> PartialEq for Node<'a, T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool { std::ptr::eq(self.0, other.0) }
}

impl<'a, T> Eq for Node<'a, T> {}

impl<'a, T> std::hash::Hash for Node<'a, T> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { std::ptr::hash(self.0, state); }
}

/// Path of the graph.
#[derive(Debug)]
pub struct Path<'a, T>(Vec<Node<'a, T>>);

impl<'a, T> Clone for Path<'a, T> {
    #[inline]
    fn clone(&self) -> Self { Path(self.0.clone()) }
}

impl<'a, T> PartialEq for Path<'a, T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool { self.0 == other.0 }
}

impl<'a, T> std::ops::Deref for Path<'a, T> {
    type Target = [Node<'a, T>];
    #[inline]
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<'a, T> AsRef<[Node<'a, T>]> for Path<'a, T> {
    #[inline]
    fn as_ref(&self) -> &[Node<'a, T>] { &self.0 }
}

impl<'a, T> From<Path<'a, T>> for Vec<Node<'a, T>> {
    #[inline]
    fn from(value: Path<'a, T>) -> Self { value.0 }
}

impl<'a, T> IntoIterator for Path<'a, T> {
    type Item = Node<'a, T>;
    type IntoIter = std::vec::IntoIter<Node<'a, T>>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl<'a, 'b, T> IntoIterator for &'b Path<'a, T> {
    type Item = Node<'a, T>;
    type IntoIter = std::iter::Copied<std::slice::Iter<'b, Node<'a, T>>>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.0.iter().copied() }
}

impl<'a, T> Path<'a, T> {
    /// Extracts a slice containing the entire path.
    #[inline]
    pub fn as_slice(&self) -> &[Node<'a, T>] { &self.0 }
    /// Returns the terminal node.
    #[inline]
    pub fn terminal_node(&self) -> Node<'a, T> { *self.last().unwrap() }
}

/// DAG: Directed acyclic graph
#[allow(missing_debug_implementations)]
pub struct Dag<'a, T> {
    arena: Arena<NodeData<'a, T>>,
    pub(super) nodes: RefCell<Vec<Node<'a, T>>>,
}

impl<'a, T> Default for Dag<'a, T> {
    fn default() -> Self {
        Self {
            arena: Default::default(),
            nodes: Default::default(),
        }
    }
}

impl<'a, T> Dag<'a, T> {
    /// Constructor
    #[inline]
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
            nodes: Vec::new().into(),
        }
    }
    /// Constructs a new dag with capacity for `n` values pre-allocated.
    #[inline]
    pub fn with_capacity(n: usize) -> Self {
        Self {
            arena: Arena::with_capacity(n),
            nodes: Vec::with_capacity(n).into(),
        }
    }
    /// Creates one node
    #[inline]
    pub fn create_node(&'a self, entity: T) -> Node<'a, T> {
        let node = Node(self.arena.alloc(NodeData::new(entity)));
        self.nodes.borrow_mut().push(node);
        node
    }
    /// Creates nodes from iterator
    #[inline]
    pub fn create_nodes<I>(&'a self, iter: I) -> Vec<Node<'a, T>>
    where I: IntoIterator<Item = T> {
        let mut nodes = self.nodes.borrow_mut();
        let closure = move |entity: T| {
            let node = Node(self.arena.alloc(NodeData::new(entity)));
            nodes.push(node);
            node
        };
        iter.into_iter().map(closure).collect()
    }

    /// Returns `true` if the graph contains no nodes.
    #[inline]
    pub fn is_empty(&self) -> bool { self.arena.len() == 0 }
    /// Returns the number of the nodes.
    /// # Examples
    /// ```
    /// use truck_assembly::dag::*;
    /// let dag = Dag::<()>::new();
    /// let _ = dag.create_nodes([(); 5]);
    /// assert_eq!(dag.len(), 5);
    /// ```
    #[inline]
    pub fn len(&self) -> usize { self.arena.len() }
    /// Returns all top level nodes.
    /// # Examples
    /// ```
    /// use truck_assembly::dag::*;
    /// let dag = Dag::<()>::new();
    /// let a = dag.create_nodes([(); 5]);
    /// a[0].add_child(a[1]);
    /// a[0].add_child(a[2]);
    /// a[2].add_child(a[3]);
    /// a[4].add_child(a[3]);
    /// let nodes = dag.all_nodes();
    /// assert_eq!(nodes.len(), 5);
    /// ```
    #[inline]
    pub fn all_nodes(&'a self) -> Vec<Node<'a, T>> { self.nodes.borrow().clone() }
    /// Returns all top level nodes.
    /// # Examples
    /// ```
    /// use truck_assembly::dag::*;
    /// let dag = Dag::<()>::new();
    /// let a = dag.create_nodes([(); 5]);
    /// a[0].add_child(a[1]);
    /// a[0].add_child(a[2]);
    /// a[2].add_child(a[3]);
    /// a[4].add_child(a[3]);
    /// let tops = dag.top_nodes();
    /// assert_eq!(tops.len(), 2);
    /// assert!(tops[0] != tops[1]);
    /// ```
    pub fn top_nodes(&'a self) -> Vec<Node<'a, T>> {
        self.nodes
            .borrow()
            .iter()
            .filter_map(|&node| match node.num_of_parents() == 0 {
                true => Some(node),
                false => None,
            })
            .collect()
    }
    /// Takes a closure and creates `dst`'s nodes which calls that closure on each entity.
    /// # Examples
    /// ```
    /// use truck_assembly::dag::*;
    /// let src_dag = Dag::<i32>::new();
    /// let dst_dag = Dag::<f64>::new();
    ///
    /// let a = src_dag.create_nodes(0..3);
    /// a[0].add_child(a[1]);
    /// a[0].add_child(a[2]);
    /// a[1].add_child(a[2]);
    /// src_dag.map_into(&dst_dag, |n: &i32| 2f64.powi(*n));
    ///
    /// assert_eq!(dst_dag.len(), 3);
    /// let dst_top = dst_dag.top_nodes()[0];
    /// let top_children = dst_top.children();
    /// let b = [dst_top, top_children[0], top_children[1]];
    ///
    /// assert_eq!(b[0].children(), vec![b[1], b[2]]);
    /// assert_eq!(*b[0].entity().borrow(), 1.0);
    /// assert_eq!(b[1].children(), vec![b[2]]);
    /// assert_eq!(*b[1].entity().borrow(), 2.0);
    /// assert!(b[2].children().is_empty());
    /// assert_eq!(*b[2].entity().borrow(), 4.0);
    /// ```
    pub fn map_into<'b, F, U>(&'a self, dst: &'b Dag<'b, U>, f: F)
    where
        F: Fn(&T) -> U,
        U: 'b, {
        // create node
        let ref_nodes = self.nodes.borrow();
        let nodes_map: HashMap<_, _> = ref_nodes
            .iter()
            .map(|&src_node| (src_node, dst.create_node(f(&*src_node.entity().borrow()))))
            .collect();

        // copy relationships
        nodes_map.iter().for_each(|(&src_node, &dst_node)| {
            // copy parents
            let ref_src_parents = src_node.parents_ref();
            let parents_iter = ref_src_parents
                .iter()
                .map(|parent| *nodes_map.get(parent).unwrap());
            dst_node.parents_mut().extend(parents_iter);

            // copy children
            let ref_src_children = src_node.children_ref();
            let children_iter = ref_src_children
                .iter()
                .map(|child| *nodes_map.get(child).unwrap());
            dst_node.children_mut().extend(children_iter);
        });
    }
    /// Extends the DAG based on the adjacency lists, and returns added nodes.
    /// # Examples
    /// ```
    /// use truck_assembly::dag::*;
    /// let dag = Dag::<usize>::new();
    ///
    /// let adjacency = [vec![1, 2], vec![3, 4], vec![], vec![], vec![2]];
    /// let nodes = dag.extend_by_adjacency(0..5, &adjacency).unwrap();
    ///
    /// let parents = nodes[2].parents();
    /// assert_eq!(parents.len(), 2);
    /// assert_eq!(parents[0], nodes[0]);
    /// assert_eq!(parents[1], nodes[4]);
    /// ```
    /// # Failure
    /// Returns `None` and does not change `self` if:
    /// - the length of `entities` does not equals to `adjacency`,
    /// - there exists an index in `adjacency` which is more than the length of `entities`, or,
    /// - the added graph has a cycle.
    ///
    /// ```
    /// use truck_assembly::dag::*;
    /// let dag = Dag::<usize>::new();
    /// let adjacency = [vec![1, 2], vec![3, 4], vec![], vec![], vec![1, 2]];
    /// assert!(dag.extend_by_adjacency(0..5, &adjacency).is_none());
    /// ```
    pub fn extend_by_adjacency<EntityIter>(
        &'a self,
        entities: EntityIter,
        adjacency: &[Vec<usize>],
    ) -> Option<Vec<Node<'a, T>>>
    where
        EntityIter: IntoIterator<Item = T>,
        EntityIter::IntoIter: ExactSizeIterator,
    {
        let entity_iter = entities.into_iter();
        let settled = |&idx: &usize| idx < entity_iter.len();
        if !adjacency.iter().flatten().all(settled) {
            return None;
        }
        if entity_iter.len() != adjacency.len() || has_cycle(adjacency) {
            return None;
        }

        let nodes = entity_iter
            .map(|entity| Node(&*self.arena.alloc(NodeData::new(entity))))
            .collect::<Vec<_>>();

        for (&node, children) in nodes.iter().zip(adjacency) {
            for &child_idx in children {
                let child = nodes[child_idx];
                node.children_mut().push(child);
                child.parents_mut().push(node);
            }
        }

        self.nodes.borrow_mut().extend(&nodes);
        Some(nodes)
    }
}

fn has_cycle(adjacency: &[Vec<usize>]) -> bool {
    let mut seen = vec![false; adjacency.len()];
    let mut finished = vec![false; adjacency.len()];

    while let Some(first) = finished.iter().position(|&x| !x) {
        if sub_has_cycle(first, adjacency, &mut seen, &mut finished) {
            return true;
        }
    }
    false
}

fn sub_has_cycle(
    cursor: usize,
    adjacency: &[Vec<usize>],
    seen: &mut [bool],
    finished: &mut [bool],
) -> bool {
    seen[cursor] = true;
    for &child in &adjacency[cursor] {
        if finished[child] {
            continue;
        } else if seen[child] || sub_has_cycle(child, adjacency, seen, finished) {
            return true;
        }
    }
    finished[cursor] = true;
    false
}

impl<'a, T> Node<'a, T> {
    #[inline]
    pub(super) fn parents_ref(self) -> Ref<'a, Vec<Self>> { self.0.parents.borrow() }
    #[inline]
    pub(super) fn parents_mut(self) -> RefMut<'a, Vec<Self>> { self.0.parents.borrow_mut() }
    /// Returns a clone of the parents list.
    /// # Examples
    /// ```
    /// use truck_assembly::dag::*;
    /// let dag = Dag::<()>::new();
    /// let a = dag.create_nodes([(); 3]);
    /// a[0].add_parent(a[1]);
    /// a[2].add_child(a[0]);
    /// assert_eq!(a[0].parents(), vec![a[1], a[2]]);
    /// ```
    #[inline]
    pub fn parents(self) -> Vec<Self> { self.parents_ref().clone() }
    /// Returns the number of the parents.
    /// # Examples
    /// ```
    /// use truck_assembly::dag::*;
    /// let dag = Dag::<()>::new();
    /// let a = dag.create_nodes([(); 5]);
    /// for &n in &a[1..] {
    ///     n.add_child(a[0]);
    /// }
    /// assert_eq!(a[0].num_of_parents(), 4);
    /// ```
    #[inline]
    pub fn num_of_parents(self) -> usize { self.parents_ref().len() }
    /// Adds a parent `parent` to `self`. Returns `false` and do nothing if and only if:
    /// - `self` already `parent` as a parent, or
    /// - adding a edge causes a loop to occur.
    /// # Example
    /// ```
    /// use truck_assembly::dag::*;
    /// let dag = Dag::<()>::new();
    /// let a = dag.create_nodes([(); 2]);
    /// assert!(a[0].add_parent(a[1]));
    /// assert_eq!(a[0].parents(), vec![a[1]]);
    /// assert!(!a[0].add_parent(a[1]));
    /// assert!(!a[1].add_parent(a[0]));
    /// ```
    #[inline]
    pub fn add_parent(self, parent: Self) -> bool { parent.add_child(self) }
    /// Returns `true` and removes `node` from the parents of `self` if and only if `node` is a parent of `self`.
    /// # Examples
    /// ```
    /// use truck_assembly::dag::*;
    /// let dag = Dag::<()>::new();
    /// let a = dag.create_nodes([(); 2]);
    /// assert!(!a[0].remove_parent(a[1]));
    /// assert!(a[1].add_child(a[0]));
    /// assert!(a[0].remove_parent(a[1]));
    /// assert!(!a[0].remove_parent(a[1]));
    /// ```
    #[inline]
    pub fn remove_parent(self, node: Self) -> bool { node.remove_child(self) }
    /// Remove all parents.
    /// # Examples
    /// ```
    /// use truck_assembly::dag::*;
    /// let dag = Dag::<()>::new();
    /// let a = dag.create_nodes([(); 3]);
    /// a[0].add_parent(a[1]);
    /// a[0].add_parent(a[2]);
    /// a[1].add_parent(a[2]);
    ///
    /// a[0].clear_parents();
    ///
    /// assert!(a[0].parents().is_empty());
    /// assert!(a[1].children().is_empty());
    /// assert_eq!(a[2].children(), vec![a[1]]);
    /// ```
    #[inline]
    pub fn clear_parents(self) {
        for node in &*self.parents_ref() {
            let mut node_children = node.children_mut();
            let some_j = node_children.iter().position(|&child| child == self);
            let _ = node_children.remove(some_j.unwrap());
        }
        self.parents_mut().clear();
    }

    #[inline]
    pub(super) fn children_ref(self) -> Ref<'a, Vec<Self>> { self.0.children.borrow() }
    #[inline]
    pub(super) fn children_mut(self) -> RefMut<'a, Vec<Self>> { self.0.children.borrow_mut() }
    /// Returns a clone of the children list.
    /// # Examples
    /// ```
    /// use truck_assembly::dag::*;
    /// let dag = Dag::<()>::new();
    /// let a = dag.create_nodes([(); 5]);
    /// a[0].add_child(a[1]);
    /// a[2].add_parent(a[0]);
    /// assert_eq!(a[0].children(), vec![a[1], a[2]]);
    /// ```
    #[inline]
    pub fn children(self) -> Vec<Self> { self.children_ref().clone() }
    /// Returns the number of the children.
    /// # Examples
    /// ```
    /// use truck_assembly::dag::*;
    /// let dag = Dag::<()>::new();
    /// let a = dag.create_nodes([(); 5]);
    /// for &n in &a[1..] {
    ///     n.add_parent(a[0]);
    /// }
    /// assert_eq!(a[0].num_of_children(), 4);
    /// ```
    #[inline]
    pub fn num_of_children(self) -> usize { self.children_ref().len() }
    /// Adds a child `child` to `self`. Returns `false` and do nothing if and only if:
    /// - `self` already `child` as a child, or
    /// - adding a edge causes a loop to occur.
    /// # Example
    /// ```
    /// use truck_assembly::dag::*;
    /// let dag = Dag::<()>::new();
    /// let a = dag.create_nodes([(); 2]);
    /// assert!(a[0].add_child(a[1]));
    /// assert_eq!(a[0].children(), vec![a[1]]);
    /// assert!(!a[0].add_child(a[1]));
    /// assert!(!a[1].add_child(a[0]));
    /// ```
    #[inline]
    pub fn add_child(self, child: Self) -> bool {
        let already_child = self.children_ref().contains(&child);
        let occurs_loop = child.contains_as_descendant(self, &mut Default::default());
        if !already_child && !occurs_loop {
            self.children_mut().push(child);
            child.parents_mut().push(self);
        }
        !already_child && !occurs_loop
    }
    /// Returns `true` and removes `node` from the children of `self` if and only if `node` is a child of `self`.
    /// # Examples
    /// ```
    /// use truck_assembly::dag::*;
    /// let dag = Dag::<()>::new();
    /// let a = dag.create_nodes([(); 2]);
    /// assert!(!a[0].remove_child(a[1]));
    /// assert!(a[1].add_parent(a[0]));
    /// assert!(a[0].remove_child(a[1]));
    /// assert!(!a[0].remove_child(a[1]));
    /// ```
    #[inline]
    pub fn remove_child(self, node: Self) -> bool {
        let mut self_children = self.children_mut();
        let Some(i) = self_children.iter().position(|&child| child == node) else {
            return false;
        };
        let _ = self_children.remove(i);
        let mut node_parents = node.parents_mut();
        let some_j = node_parents.iter().position(|&parent| parent == self);
        let _ = node_parents.remove(some_j.unwrap());
        true
    }
    /// Remove all children.
    /// # Examples
    /// ```
    /// use truck_assembly::dag::*;
    /// let dag = Dag::<()>::new();
    /// let a = dag.create_nodes([(); 3]);
    /// a[0].add_child(a[1]);
    /// a[0].add_child(a[2]);
    /// a[1].add_child(a[2]);
    ///
    /// a[0].clear_children();
    ///
    /// assert!(a[0].children().is_empty());
    /// assert!(a[1].parents().is_empty());
    /// assert_eq!(a[2].parents(), vec![a[1]]);
    /// ```
    #[inline]
    pub fn clear_children(self) {
        for node in &*self.children_ref() {
            let mut node_parents = node.parents_mut();
            let some_j = node_parents.iter().position(|&child| child == self);
            let _ = node_parents.remove(some_j.unwrap());
        }
        self.children_mut().clear();
    }

    /// Returns the `RefCell` of entity.
    /// # Remarks
    /// Each Node corresponds to only one RefCell for an entity.
    /// # Examples
    /// ```
    /// use truck_assembly::dag::*;
    /// let dag = Dag::<usize>::new();
    /// let a = dag.create_nodes(0..5);
    /// assert_eq!(*a[3].entity().borrow(), 3);
    /// assert!(a[0].entity().try_borrow().is_ok());
    /// let _x = a[0].entity().borrow_mut();
    /// assert!(a[0].entity().try_borrow().is_err());
    /// assert!(a[1].entity().try_borrow().is_ok());
    /// ```
    #[inline]
    pub fn entity(self) -> &'a RefCell<T> { &self.0.entity }

    /// Returns iterator for all paths with the top = `self`.
    /// # Examples
    /// ```
    /// use truck_assembly::dag::*;
    /// let dag = Dag::<()>::new();
    /// let a = dag.create_nodes([(); 5]);
    /// a[0].add_child(a[1]);
    /// a[0].add_child(a[2]);
    /// a[0].add_child(a[3]);
    /// a[2].add_child(a[3]);
    /// a[2].add_child(a[4]);
    /// let paths = a[0].paths_iter().collect::<Vec<_>>();
    /// let slices = paths.iter().map(Path::as_slice).collect::<Vec<_>>();
    /// assert_eq!(slices.len(), 6);
    /// assert_eq!(slices[0], &[a[0]]);
    /// assert_eq!(slices[1], &[a[0], a[1]]);
    /// assert_eq!(slices[2], &[a[0], a[2]]);
    /// assert_eq!(slices[3], &[a[0], a[2], a[3]]);
    /// assert_eq!(slices[4], &[a[0], a[2], a[4]]);
    /// assert_eq!(slices[5], &[a[0], a[3]]);
    /// ```
    #[inline]
    pub fn paths_iter(self) -> impl Iterator<Item = Path<'a, T>> {
        PathsIter {
            current: Path(vec![self]),
            indices: vec![],
            end_iter: false,
        }
    }

    fn first_maximal_path(self) -> Path<'a, T> {
        let (mut cursor, mut vec) = (self, vec![self]);
        while let Some(&child) = cursor.children_ref().first() {
            cursor = child;
            vec.push(cursor);
        }
        Path(vec)
    }

    /// Returns iterator for all maximul paths with the top = `self`.
    /// # Examples
    /// ```
    /// use truck_assembly::dag::*;
    /// let dag = Dag::<()>::new();
    /// let a = dag.create_nodes([(); 5]);
    /// a[0].add_child(a[1]);
    /// a[0].add_child(a[2]);
    /// a[0].add_child(a[3]);
    /// a[2].add_child(a[3]);
    /// a[2].add_child(a[4]);
    /// let paths = a[0].maximul_paths_iter().collect::<Vec<_>>();
    /// let slices = paths.iter().map(Path::as_slice).collect::<Vec<_>>();
    /// assert_eq!(slices.len(), 4);
    /// assert_eq!(slices[0], &[a[0], a[1]]);
    /// assert_eq!(slices[1], &[a[0], a[2], a[3]]);
    /// assert_eq!(slices[2], &[a[0], a[2], a[4]]);
    /// assert_eq!(slices[3], &[a[0], a[3]]);
    /// ```
    #[inline]
    pub fn maximul_paths_iter(self) -> impl Iterator<Item = Path<'a, T>> {
        let current = self.first_maximal_path();
        MaximulPathsIter {
            indices: vec![0; current.len() - 1],
            current,
            end_iter: false,
        }
    }

    fn contains_as_descendant(self, node: Self, seen: &mut HashSet<Self>) -> bool {
        if self == node {
            return true;
        }
        for &child in &*self.children_ref() {
            if seen.contains(&child) {
                continue;
            } else if child.contains_as_descendant(node, seen) {
                return true;
            }
        }
        seen.insert(node);
        false
    }
}

#[derive(Clone, Debug)]
struct PathsIter<'a, T> {
    current: Path<'a, T>,
    indices: Vec<usize>,
    end_iter: bool,
}

impl<'a, T> Iterator for PathsIter<'a, T> {
    type Item = Path<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.end_iter {
            return None;
        }
        let res = self.current.clone();

        let leaf = *self.current.last().unwrap();
        if leaf.children_ref().is_empty() {
            let len = self.indices.len();
            let some_idx = (0..len)
                .map(|i| len - i - 1)
                .find(|&idx| self.indices[idx] + 1 < self.current[idx].num_of_children());
            if let Some(idx) = some_idx {
                self.current.0.truncate(idx + 2);
                self.indices.truncate(idx + 1);
                self.indices[idx] += 1;
                self.current.0[idx + 1] = self.current[idx].children_ref()[self.indices[idx]];
            } else {
                self.end_iter = true;
            }
        } else {
            self.current.0.push(leaf.children_ref()[0]);
            self.indices.push(0);
        }

        Some(res)
    }
}

#[derive(Clone, Debug)]
struct MaximulPathsIter<'a, T> {
    current: Path<'a, T>,
    indices: Vec<usize>,
    end_iter: bool,
}

impl<'a, T> Iterator for MaximulPathsIter<'a, T> {
    type Item = Path<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.end_iter {
            return None;
        }
        let res = self.current.clone();

        let len = self.indices.len();
        let some_idx = (0..len)
            .map(|i| len - i - 1)
            .find(|&idx| self.indices[idx] + 1 < self.current[idx].num_of_children());
        let Some(idx) = some_idx else {
            self.end_iter = true;
            return Some(res);
        };

        self.current.0.truncate(idx + 1);
        self.indices.truncate(idx + 1);
        self.indices[idx] += 1;

        let node = self.current[idx].children_ref()[self.indices[idx]];
        let path = node.first_maximal_path();
        let zeros = std::iter::repeat_n(0, path.len() - 1);
        self.indices.extend(zeros);
        self.current.0.extend(path.0);

        Some(res)
    }
}
