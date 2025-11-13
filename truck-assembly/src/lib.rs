use rustc_hash::FxHashMap as HashMap;

pub type NodeIdx = usize;

#[derive(Clone, Debug)]
pub struct Node<Shape, Matrix, Meta> {
    parents: Vec<NodeIdx>,
    children: Vec<NodeIdx>,
    matrix: Matrix,
    shapes: Vec<Shape>,
    meta: Meta,
}

#[derive(Clone, Debug)]
pub struct Assembly<Shape, Matrix, Meta> {
    nodes: HashMap<NodeIdx, Node<Shape, Matrix, Meta>>,
    current_index: NodeIdx,
}

#[derive(Clone, Debug)]
pub struct Path<'a, Shape, Matrix, Meta>(&'a Node<Shape, Matrix, Meta>);

impl<Shape, Matrix, Meta> Assembly<Shape, Matrix, Meta> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            nodes: Vec::new(),
            current_index: 0,
        }
    }
}
