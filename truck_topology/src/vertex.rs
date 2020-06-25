use crate::Vertex;
use crate::id::IDGenerator;

lazy_static! {
    static ref ID_GENERATOR: IDGenerator = IDGenerator::new();
}

impl Vertex {
    /// constructor
    #[inline(always)]
    pub fn new() -> Vertex {
        Vertex { id: ID_GENERATOR.generate() }
    }

    /// Creates `len` distinct vertices and return them by vector.
    /// ```
    /// # use truck_topology::Vertex;
    /// let v = Vertex::news(3);
    /// assert_eq!(v.len(), 3);
    /// assert_ne!(v[0], v[2]);
    /// ```
    #[inline(always)]
    pub fn news(len: usize) -> Vec<Vertex> {
        ID_GENERATOR.multi_generate(len).into_iter().map(|id| Vertex { id: id }).collect()
    }

    /// Returns the id of vertex
    #[inline(always)]
    pub fn id(&self) -> usize { self.id }
}
