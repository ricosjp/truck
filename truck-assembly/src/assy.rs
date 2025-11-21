use crate::dag;
pub use dag::{Node, Path};
use itertools::Itertools;
use truck_base::cgmath64::One;

/// Entity of node of assembly
#[derive(Clone, Debug, PartialEq)]
pub struct Entity<Matrix, Shape, Attrs> {
    /// transform matrix
    pub matrix: Matrix,
    /// shapes in node
    pub shapes: Vec<Shape>,
    /// extra attributes (e.g. label, material, and other properties
    pub attrs: Attrs,
}

/// Assembly
pub type Assembly<'a, Matrix, Shape, Attrs> = dag::Dag<'a, Entity<Matrix, Shape, Attrs>>;

impl<'a, Matrix, Shape, Attrs> Path<'a, Entity<Matrix, Shape, Attrs>> {
    /// Returns transform matrix of `self`.
    /// # Examples
    /// ```
    /// use truck_assembly::assy::*;
    /// let assy = Assembly::<f64, (), ()>::new();
    /// let iter_closure = |i: usize| Entity {
    ///     matrix: i as f64,
    ///     shapes: Vec::new(),
    ///     attrs: (),
    /// };
    /// let node = assy.create_nodes((1..=10).map(iter_closure));
    /// for i in 0..9 {
    ///     node[i].add_child(node[i + 1]);
    /// }
    /// let path = node[0].maximul_paths_iter().next().unwrap();
    /// assert_eq!(path.matrix(), 3628800.0);
    /// ```
    /// # Panics
    /// Panic occurs if a node within the path already holds a mutable reference to an entity.
    /// ```should_panic
    /// use truck_assembly::assy::*;
    /// let assy = Assembly::<f64, (), ()>::new();
    /// let iter_closure = |i: usize| Entity {
    ///     matrix: i as f64,
    ///     shapes: Vec::new(),
    ///     attrs: (),
    /// };
    /// let node = assy.create_nodes((1..=10).map(iter_closure));
    /// for i in 0..9 {
    ///     node[i].add_child(node[i + 1]);
    /// }
    /// let _x = node[0].entity().borrow_mut();
    /// let path = node[0].maximul_paths_iter().next().unwrap();
    /// path.matrix();
    /// ```
    pub fn matrix(&self) -> Matrix
    where Matrix: One + Copy + std::ops::Mul<Output = Matrix> {
        self.iter().fold(Matrix::one(), |matrix, node| {
            matrix * node.entity().borrow().matrix
        })
    }
}

impl<'a, Matrix, Shape, Attrs> Assembly<'a, Matrix, Shape, Attrs> {
    /// Modify the assembly to achieve the following state.
    /// - Each node has at most one shape.
    /// - Nodes with child nodes do not have a shape.
    ///
    /// Assign default attributes to newly added nodes.
    /// # Examples
    /// ```
    /// use truck_assembly::assy::*;
    /// let assy = Assembly::<f64, usize, usize>::new();
    /// let a = assy.create_nodes([
    ///     Entity {
    ///         matrix: 2.0,
    ///         shapes: vec![],
    ///         attrs: 1,
    ///     },
    ///     Entity {
    ///         matrix: -1.0,
    ///         shapes: vec![1, 2, 3],
    ///         attrs: 2,
    ///     },
    ///     Entity {
    ///         matrix: 4.0,
    ///         shapes: vec![4, 5],
    ///         attrs: 3,
    ///     },
    ///     Entity {
    ///         matrix: -3.0,
    ///         shapes: vec![6],
    ///         attrs: 1,
    ///     },
    /// ]);
    /// a[0].add_child(a[1]);
    /// a[1].add_child(a[2]);
    /// a[0].add_child(a[3]);
    /// let third_entity = a[3].entity().borrow().clone();
    /// 
    /// assy.extract_nodes_as_children();
    /// 
    /// assert_eq!(assy.len(), 8);
    /// assert_eq!(a[0].children(), vec![a[1], a[3]]);
    /// assert_eq!(a[1].num_of_children(), 5);
    /// assert!(a[1].entity().borrow().shapes.is_empty());
    /// let child_entities = a[1]
    ///     .children()
    ///     .into_iter()
    ///     .map(|child| child.entity().borrow().clone())
    ///     .collect::<Vec<_>>();
    /// assert_eq!(child_entities[0], Entity {
    ///     matrix: 4.0,
    ///     shapes: vec![4],
    ///     attrs: 3,
    /// });
    /// assert_eq!(child_entities[1], Entity {
    ///     matrix: 1.0,
    ///     shapes: vec![1],
    ///     attrs: 0,
    /// });
    /// assert_eq!(child_entities[2], Entity {
    ///     matrix: 1.0,
    ///     shapes: vec![2],
    ///     attrs: 0,
    /// });
    /// assert_eq!(child_entities[3], Entity {
    ///     matrix: 1.0,
    ///     shapes: vec![3],
    ///     attrs: 0,
    /// });
    /// assert_eq!(child_entities[4], Entity {
    ///     matrix: 4.0,
    ///     shapes: vec![5],
    ///     attrs: 0,
    /// });
    /// 
    /// assert_eq!(*a[3].entity().borrow(), third_entity);
    /// ```
    pub fn extract_nodes_as_children(&'a self)
    where
        Matrix: Copy + One,
        Attrs: Default, {
        let mut added_node = Vec::<Node<'a, _>>::new();
        self.all_nodes().into_iter().for_each(|node| {
            if !node.children_ref().is_empty() {
                let mut entity_ref = node.entity().borrow_mut();
                let entity_iter = entity_ref.shapes.drain(..).map(|shape| Entity {
                    matrix: One::one(),
                    shapes: vec![shape],
                    attrs: Default::default(),
                });
                let new_nodes = self.create_nodes(entity_iter);
                new_nodes.iter().for_each(|&new_node| {
                    node.add_child(new_node);
                });
                added_node.extend(new_nodes);
            }
        });
        self.nodes.borrow_mut().extend(added_node);

        let mut added_node = Vec::<Node<'a, _>>::new();
        self.all_nodes().into_iter().for_each(|node| {
            if node.entity().borrow().shapes.len() > 1 {
                let mut entity_ref = node.entity().borrow_mut();
                let matrix = entity_ref.matrix;
                let entity_iter = entity_ref.shapes.drain(1..).map(|shape| Entity {
                    matrix,
                    shapes: vec![shape],
                    attrs: Default::default(),
                });
                let new_nodes = self.create_nodes(entity_iter);
                node.children_ref()
                    .iter()
                    .cartesian_product(&new_nodes)
                    .for_each(|(&child, &new_node)| {
                        new_node.add_child(child);
                    });
                node.parents_ref()
                    .iter()
                    .cartesian_product(&new_nodes)
                    .for_each(|(&parent, &new_node)| {
                        new_node.add_parent(parent);
                    });
                added_node.extend(new_nodes);
            }
        });
        self.nodes.borrow_mut().extend(added_node);
    }
}
