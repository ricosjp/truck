use crate::Vertex;
use crate::id::IDGenerator;

lazy_static! {
    static ref ID_GENERATOR: IDGenerator = IDGenerator::new();
}

impl Vertex {
    #[inline(always)]
    pub fn new() -> Vertex {
        Vertex { id: ID_GENERATOR.generate() }
    }

    #[inline(always)]
    pub fn news(len: usize) -> Vec<Vertex> {
        ID_GENERATOR.multi_generate(len).into_iter().map(|id| Vertex { id: id }).collect()
    }

    #[inline(always)]
    pub fn id(&self) -> usize { self.id }
}
