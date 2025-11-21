use rustc_hash::FxHashSet as HashSet;
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
        let occurs_loop = child.contains_as_descendant(self);
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

    fn contains_as_descendant(self, node: Self) -> bool {
        self.sub_contains_as_descendant(node, &mut Default::default())
    }

    #[allow(clippy::mutable_key_type)]
    fn sub_contains_as_descendant(self, node: Self, seen: &mut HashSet<Self>) -> bool {
        if self == node {
            return true;
        }
        for &child in &*self.children_ref() {
            if seen.contains(&child) {
                continue;
            } else if child.sub_contains_as_descendant(node, seen) {
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
