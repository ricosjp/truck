use crate::dag;
pub use dag::{Edge, EdgeMut, Node, NodeMut, Path};
//use itertools::Itertools;
use truck_base::cgmath64::One;

/// Entity of the node
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NodeEntity<Shape, NodeAttrs> {
    /// shape of node
    pub shape: Shape,
    /// extra attributes (e.g. label, material, and other properties
    pub attrs: NodeAttrs,
}

impl<Shape> From<Shape> for NodeEntity<Shape, ()> {
    fn from(shape: Shape) -> Self { Self { shape, attrs: () } }
}

/// Entity of the edge
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EdgeEntity<Matrix, EdgeAttrs> {
    /// transform matrix of edge
    pub matrix: Matrix,
    /// extra attributes (e.g. label, material, and other properties
    pub attrs: EdgeAttrs,
}

impl<Matrix> From<Matrix> for EdgeEntity<Matrix, ()> {
    fn from(matrix: Matrix) -> Self { Self { matrix, attrs: () } }
}

/// Assembly
pub type Assembly<Shape, NodeAttrs, Matrix, EdgeAttrs> =
    dag::Dag<NodeEntity<Shape, NodeAttrs>, EdgeEntity<Matrix, EdgeAttrs>>;

impl<'a, Shape, NodeAttrs, Matrix, EdgeAttrs>
    Path<'a, NodeEntity<Shape, NodeAttrs>, EdgeEntity<Matrix, EdgeAttrs>>
{
    /// Returns transform matrix of `self`.
    /// # Examples
    /// ```
    /// use truck_assembly::assy::*;
    /// let mut assy = Assembly::<(), (), f64, ()>::new();
    /// let node = assy.create_nodes([().into(); 11]);
    /// for i in 0..10 {
    ///     assy.create_edge(node[i], node[i + 1], ((i + 1) as f64).into());
    /// }
    /// let path = assy.maximal_paths_iter(node[0]).next().unwrap();
    /// assert_eq!(path.matrix(), 3628800.0);
    /// ```
    #[inline]
    pub fn matrix(&self) -> Matrix
    where Matrix: One + Copy + std::ops::Mul<Output = Matrix> {
        self.edges()
            .iter()
            .fold(Matrix::one(), |matrix, node| matrix * node.entity().matrix)
    }
}

impl<'a, Shape, NodeAttrs, Matrix, EdgeAttrs>
    Node<'a, NodeEntity<Shape, NodeAttrs>, EdgeEntity<Matrix, EdgeAttrs>>
{
    /// Returns the shape
    #[inline]
    pub fn shape(self) -> &'a Shape { &self.entity().shape }
    /// Returns the node attributes
    #[inline]
    pub fn attrs(self) -> &'a NodeAttrs { &self.entity().attrs }
}

impl<'a, Shape, NodeAttrs, Matrix, EdgeAttrs>
    NodeMut<'a, NodeEntity<Shape, NodeAttrs>, EdgeEntity<Matrix, EdgeAttrs>>
{
    /// Returns the shape
    #[inline]
    pub fn shape(&'a mut self) -> &'a mut Shape { &mut self.entity().shape }
    /// Returns the node attributes
    #[inline]
    pub fn attrs(&'a mut self) -> &'a mut NodeAttrs { &mut self.entity().attrs }
}

impl<'a, Matrix, EdgeAttrs> Edge<'a, EdgeEntity<Matrix, EdgeAttrs>> {
    /// Returns the shape
    #[inline]
    pub fn matrix(self) -> &'a Matrix { &self.entity().matrix }
    /// Returns the node attributes
    #[inline]
    pub fn attrs(self) -> &'a EdgeAttrs { &self.entity().attrs }
}

impl<'a, Matrix, EdgeAttrs> EdgeMut<'a, EdgeEntity<Matrix, EdgeAttrs>> {
    /// Returns the shape
    #[inline]
    pub fn matrix(&'a mut self) -> &'a mut Matrix { &mut self.entity().matrix }
    /// Returns the node attributes
    #[inline]
    pub fn attrs(&'a mut self) -> &'a mut EdgeAttrs { &mut self.entity().attrs }
}

/// have contents which can be removed
/// # Example
/// ```
/// use truck_assembly::assy::Takeable;
///
/// // Takeable::take is same with Option::take.
/// let mut item = Some(3);
/// assert_eq!(Takeable::take(&mut item), Some(3));
/// assert_eq!(item, None);
///
/// // For Vec, take is the drain.
/// let mut vec = vec![1, 2, 3];
/// assert_eq!(Takeable::take(&mut vec), vec![1, 2, 3]);
/// assert!(vec.is_empty());
/// ```
pub trait Takeable {
    /// take contents
    fn take(&mut self) -> Self;
    /// Return `true` if the contents has been taken.
    /// # Examples
    /// ```
    /// use truck_assembly::assy::Takeable;
    /// assert!(!Some(()).is_taken());
    /// assert!(Option::<()>::None.is_taken());
    /// ```
    fn is_taken(&self) -> bool;
}

impl<T> Takeable for Option<T> {
    #[inline]
    fn take(&mut self) -> Self { self.take() }
    #[inline]
    fn is_taken(&self) -> bool { self.is_none() }
}

impl<T> Takeable for Vec<T> {
    #[inline]
    fn take(&mut self) -> Self { self.split_off(0) }
    #[inline]
    fn is_taken(&self) -> bool { self.is_empty() }
}

impl<Shape, NodeAttrs, Matrix, EdgeAttrs> Assembly<Shape, NodeAttrs, Matrix, EdgeAttrs> {
    /// Add nodes as needed and set all nodes except the terminal node's shape to “taken”.
    /// Assign default attributes to newly added nodes.
    /// # Examples
    /// ```
    /// use truck_assembly::assy::*;
    /// let mut assy = Assembly::<Option<usize>, usize, f64, usize>::new();
    /// let a = assy.create_nodes([
    ///     NodeEntity {
    ///         shape: None,
    ///         attrs: 1,
    ///     },
    ///     NodeEntity {
    ///         shape: Some(2),
    ///         attrs: 2,
    ///     },
    ///     NodeEntity {
    ///         shape: Some(4),
    ///         attrs: 3,
    ///     },
    ///     NodeEntity {
    ///         shape: Some(6),
    ///         attrs: 4,
    ///     },
    /// ]);
    /// assy.create_edge(a[0], a[1], EdgeEntity { matrix: 3.0, attrs: 1 });
    /// assy.create_edge(a[1], a[2], EdgeEntity { matrix: 9.0, attrs: 2 });
    /// assy.create_edge(a[0], a[3], EdgeEntity { matrix: 6.0, attrs: 3 });
    ///
    /// assy.normalize();
    ///
    /// assert_eq!(assy.len(), 5);
    ///
    /// let node = assy.all_nodes().collect::<Vec<_>>();
    /// assert!(node[1].shape().is_none());
    /// assert_eq!(*node[4].shape(), Some(2));
    /// assert_eq!(*node[4].attrs(), 0);
    ///
    /// let new_edge = node[1].edges().collect::<Vec<_>>()[1];
    /// assert_eq!(*new_edge.matrix(), 1.0);
    /// assert_eq!(*new_edge.attrs(), 0);
    /// ```
    pub fn normalize(&mut self)
    where
        Matrix: Copy + One,
        Shape: Takeable,
        NodeAttrs: Default,
        EdgeAttrs: Default, {
        self.node_indices().for_each(|index| {
            let node = self.node(index);
            if !(node.is_terminal() || node.entity().shape.is_taken()) {
                let shape = self.node_mut(index).entity().shape.take();
                let new_index = self.create_node(NodeEntity {
                    shape,
                    attrs: Default::default(),
                });
                let edge_entity = EdgeEntity {
                    matrix: Matrix::one(),
                    attrs: Default::default(),
                };
                self.create_edge(index, new_index, edge_entity);
            }
        });
    }
}
