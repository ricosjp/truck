use rustc_hash::FxHashSet as HashSet;
use std::cell::{Ref, RefCell, RefMut};
use typed_arena::Arena;

/// Graph: arena of the nodes
pub struct Graph<'a, T>(Arena<NodeData<'a, T>>);

#[derive(Clone, Debug)]
struct NodeData<'a, T> {
    parents: RefCell<Vec<Node<'a, T>>>,
    children: RefCell<Vec<Node<'a, T>>>,
    entity: T,
}

/// Node of a graph.
#[derive(Debug)]
pub struct Node<'a, T>(&'a NodeData<'a, T>);

impl<'a, T> Clone for Node<'a, T> {
    fn clone(&self) -> Self { Node(self.0) }
}

impl<'a, T> Copy for Node<'a, T> {}

impl<'a, T> PartialEq for Node<'a, T> {
    fn eq(&self, other: &Self) -> bool { std::ptr::eq(self.0, other.0) }
}

impl<'a, T> Eq for Node<'a, T> {}

impl<'a, T> std::hash::Hash for Node<'a, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { std::ptr::hash(self.0, state); }
}

impl<'a, T> Graph<'a, T> {
    /// Constructor
    #[inline]
    pub fn new() -> Self { Self(Arena::new()) }
    /// Constructs a new graph with capacity for `n` values pre-allocated.
    #[inline]
    pub fn with_capacity(n: usize) -> Self { Self(Arena::with_capacity(n)) }
    /// Creates one node
    #[inline]
    pub fn create_node(&'a self, entity: T) -> Node<'a, T> {
        Node(self.0.alloc(NodeData {
            parents: RefCell::new(Vec::new()),
            children: RefCell::new(Vec::new()),
            entity,
        }))
    }
    /// Creates nodes from iterator
    #[inline]
    pub fn create_nodes<I>(&'a self, iter: I) -> Vec<Node<'a, T>>
    where I: IntoIterator<Item = T> {
        iter.into_iter()
            .map(|entity| {
                Node(self.0.alloc(NodeData {
                    parents: RefCell::new(Vec::new()),
                    children: RefCell::new(Vec::new()),
                    entity,
                }))
            })
            .collect()
    }
    /// Returns the number of the nodes
    /// # Examples
    /// ```
    /// use truck_assembly::graph::*;
    /// let graph = Graph::new();
    /// let _ = graph.create_nodes([(); 5]);
    /// assert_eq!(graph.len(), 5);
    /// ```
    #[inline]
    pub fn len(&self) -> usize { self.0.len() }
}

impl<'a, T> Node<'a, T> {
    #[inline]
    fn parents_ref(self) -> Ref<'a, Vec<Self>> { self.0.parents.borrow() }
    #[inline]
    fn parents_mut(self) -> RefMut<'a, Vec<Self>> { self.0.parents.borrow_mut() }
    /// Returns a clone of the parents list.
    /// # Examples
    /// ```
    /// use truck_assembly::graph::*;
    /// let graph = Graph::new();
    /// let a = graph.create_nodes([(); 3]);
    /// a[0].add_parent(a[1]);
    /// a[2].add_child(a[0]);
    /// assert_eq!(a[0].parents(), vec![a[1], a[2]]);
    /// ```
    #[inline]
    pub fn parents(self) -> Vec<Self> { self.parents_ref().clone() }
    /// Returns the number of the parents.
    /// # Examples
    /// ```
    /// use truck_assembly::graph::*;
    /// let graph = Graph::new();
    /// let a = graph.create_nodes([(); 5]);
    /// for &n in &a[1..] {
    ///     n.add_child(a[0]);
    /// }
    /// assert_eq!(a[0].num_of_parents(), 4);
    /// ```
    #[inline]
    pub fn num_of_parents(self) -> usize { self.parents_ref().len() }
    /// Adds a parent `parent` to `self`.
    /// Returns `false` if and only if `parent` is already a parent of `self`.
    /// # Example
    /// ```
    /// use truck_assembly::graph::*;
    /// let graph = Graph::new();
    /// let a = graph.create_nodes([(); 2]);
    /// assert!(a[0].add_parent(a[1]));
    /// assert_eq!(a[0].parents(), vec![a[1]]);
    /// assert!(!a[0].add_parent(a[1]));
    /// ```
    #[inline]
    pub fn add_parent(self, parent: Self) -> bool {
        let already = self.parents_ref().iter().any(|&node| node == parent);
        if !already {
            self.parents_mut().push(parent);
            parent.children_mut().push(self);
        }
        !already
    }
    /// Returns `true` and removes `node` from the parents of `self` if and only if `node` is a parent of `self`.
    /// # Examples
    /// ```
    /// use truck_assembly::graph::*;
    /// let graph = Graph::new();
    /// let a = graph.create_nodes([(); 2]);
    /// assert!(!a[0].remove_parent(a[1]));
    /// assert!(a[1].add_child(a[0]));
    /// assert!(a[0].remove_parent(a[1]));
    /// assert!(!a[0].remove_parent(a[1]));
    /// ```
    #[inline]
    pub fn remove_parent(self, node: Self) -> bool {
        let mut self_parents = self.parents_mut();
        let Some(i) = self_parents.iter().position(|&parent| parent == node) else {
            return false;
        };
        let _ = self_parents.remove(i);
        let mut node_children = node.children_mut();
        let some_j = node_children.iter().position(|&child| child == self);
        let _ = node_children.remove(some_j.unwrap());
        true
    }
    /// Remove all parents.
    /// # Examples
    /// ```
    /// use truck_assembly::graph::*;
    /// let graph = Graph::new();
    /// let a = graph.create_nodes([(); 3]);
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
    fn children_ref(self) -> Ref<'a, Vec<Self>> { self.0.children.borrow() }
    #[inline]
    fn children_mut(self) -> RefMut<'a, Vec<Self>> { self.0.children.borrow_mut() }
    /// Returns a clone of the children list.
    /// # Examples
    /// ```
    /// use truck_assembly::graph::*;
    /// let graph = Graph::new();
    /// let a = graph.create_nodes([(); 5]);
    /// a[0].add_child(a[1]);
    /// a[2].add_parent(a[0]);
    /// assert_eq!(a[0].children(), vec![a[1], a[2]]);
    /// ```
    #[inline]
    pub fn children(self) -> Vec<Self> { self.children_ref().clone() }
    /// Returns the number of the children.
    /// # Examples
    /// ```
    /// use truck_assembly::graph::*;
    /// let graph = Graph::new();
    /// let a = graph.create_nodes([(); 5]);
    /// for &n in &a[1..] {
    ///     n.add_parent(a[0]);
    /// }
    /// assert_eq!(a[0].num_of_children(), 4);
    /// ```
    #[inline]
    pub fn num_of_children(self) -> usize { self.children_ref().len() }
    /// Adds a child `child` to `self`.
    /// Returns `false` if and only if `child` is already a child of `self`.
    /// # Example
    /// ```
    /// use truck_assembly::graph::*;
    /// let graph = Graph::new();
    /// let a = graph.create_nodes([(); 2]);
    /// assert!(a[0].add_child(a[1]));
    /// assert_eq!(a[0].children(), vec![a[1]]);
    /// assert!(!a[0].add_child(a[1]));
    /// ```
    #[inline]
    pub fn add_child(self, child: Self) -> bool {
        let already = self.children_ref().iter().any(|&node| node == child);
        if !already {
            self.children_mut().push(child);
            child.parents_mut().push(self);
        }
        !already
    }
    /// Returns `true` and removes `node` from the children of `self` if and only if `node` is a child of `self`.
    /// # Examples
    /// ```
    /// use truck_assembly::graph::*;
    /// let graph = Graph::new();
    /// let a = graph.create_nodes([(); 2]);
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
    /// use truck_assembly::graph::*;
    /// let graph = Graph::new();
    /// let a = graph.create_nodes([(); 3]);
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

    /// Returns the reference of entity of the graph
    #[inline]
    pub fn entity(self) -> &'a T { &self.0.entity }

    /// Returns iterator for all paths with the top = `self`.
    /// # Examples
    /// ```
    /// use truck_assembly::graph::*;
    /// let graph = Graph::new();
    /// let a = graph.create_nodes([(); 5]);
    /// a[0].add_child(a[1]);
    /// a[0].add_child(a[2]);
    /// a[0].add_child(a[3]);
    /// a[2].add_child(a[3]);
    /// a[2].add_child(a[4]);
    /// let paths = a[0].paths_iter().collect::<Vec<_>>();
    /// assert_eq!(paths[0], vec![a[0]]);
    /// assert_eq!(paths[1], vec![a[0], a[1]]);
    /// assert_eq!(paths[2], vec![a[0], a[2]]);
    /// assert_eq!(paths[3], vec![a[0], a[2], a[3]]);
    /// assert_eq!(paths[4], vec![a[0], a[2], a[4]]);
    /// assert_eq!(paths[5], vec![a[0], a[3]]);
    /// ```
    #[inline]
    pub fn paths_iter(self) -> impl Iterator<Item = Vec<Self>> {
        PathsIter {
            current: vec![self],
            indices: vec![],
            end_iter: false,
        }
    }

    fn first_maximal_path(self) -> Vec<Self> {
        let (mut cursor, mut path) = (self, vec![self]);
        while let Some(&child) = cursor.children_ref().get(0) {
            cursor = child;
            path.push(cursor);
        }
        path
    }

    /// Returns iterator for all maximul paths with the top = `self`.
    /// # Examples
    /// ```
    /// use truck_assembly::graph::*;
    /// let graph = Graph::new();
    /// let a = graph.create_nodes([(); 5]);
    /// a[0].add_child(a[1]);
    /// a[0].add_child(a[2]);
    /// a[0].add_child(a[3]);
    /// a[2].add_child(a[3]);
    /// a[2].add_child(a[4]);
    /// let paths = a[0].maximul_paths_iter().collect::<Vec<_>>();
    /// assert_eq!(paths[0], vec![a[0], a[1]]);
    /// assert_eq!(paths[1], vec![a[0], a[2], a[3]]);
    /// assert_eq!(paths[2], vec![a[0], a[2], a[4]]);
    /// assert_eq!(paths[3], vec![a[0], a[3]]);
    /// ```
    #[inline]
    pub fn maximul_paths_iter(self) -> impl Iterator<Item = Vec<Self>> {
        let current = self.first_maximal_path();
        MaximulPathsIter {
            indices: vec![0; current.len() - 1],
            current,
            end_iter: false,
        }
    }

    /// Returns `true` if there is a path with the top = `self` which contains a cycle.
    /// # Examples
    /// ```
    /// use truck_assembly::graph::*;
    /// let graph = Graph::new();
    /// let a = graph.create_nodes([(); 5]);
    /// a[0].add_child(a[1]);
    /// a[0].add_child(a[2]);
    /// a[0].add_child(a[3]);
    /// a[1].add_child(a[2]);
    /// a[2].add_child(a[3]);
    /// assert!(!a[0].has_cycle());
    /// 
    /// a[3].add_child(a[1]);
    /// assert!(a[0].has_cycle());
    /// ```
    pub fn has_cycle(self) -> bool {
        self.sub_has_cycle(&mut Default::default(), &mut Default::default())
    }

    fn sub_has_cycle(self, seen: &mut HashSet<Self>, finished: &mut HashSet<Self>) -> bool {
        seen.insert(self);
        for &child in &*self.children_ref() {
            if finished.contains(&child) {
                continue;
            } else if seen.contains(&child) || child.sub_has_cycle(seen, finished) {
                return true;
            }
        }
        finished.insert(self);
        false
    }
}

#[derive(Clone, Debug)]
struct PathsIter<'a, T> {
    current: Vec<Node<'a, T>>,
    indices: Vec<usize>,
    end_iter: bool,
}

impl<'a, T> Iterator for PathsIter<'a, T> {
    type Item = Vec<Node<'a, T>>;
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
                self.current.truncate(idx + 2);
                self.indices.truncate(idx + 1);
                self.indices[idx] += 1;
                self.current[idx + 1] = self.current[idx].children_ref()[self.indices[idx]];
            } else {
                self.end_iter = true;
            }
        } else {
            self.current.push(leaf.children_ref()[0]);
            self.indices.push(0);
        }

        Some(res)
    }
}

#[derive(Clone, Debug)]
struct MaximulPathsIter<'a, T> {
    current: Vec<Node<'a, T>>,
    indices: Vec<usize>,
    end_iter: bool,
}

impl<'a, T> Iterator for MaximulPathsIter<'a, T> {
    type Item = Vec<Node<'a, T>>;
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

        self.current.truncate(idx + 1);
        self.indices.truncate(idx + 1);
        self.indices[idx] += 1;

        let node = self.current[idx].children_ref()[self.indices[idx]];
        let path = node.first_maximal_path();
        let zeros = std::iter::repeat(0).take(path.len() - 1);
        self.indices.extend(zeros);
        self.current.extend(path);

        Some(res)
    }
}
