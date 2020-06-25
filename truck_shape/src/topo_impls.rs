use topology::*;

pub(super) trait TopoExtension {
    fn replaced(&self, search: Vertex, replace: Vertex) -> Self;
}

impl TopoExtension for Wire {
    fn replaced(&self, search: Vertex, replace: Vertex) -> Wire {
        let mut wire = Wire::new();
        for edge in self.edge_iter() {
            if search == edge.front() {
                if edge.back() != search && edge.back() != replace {
                    wire.push_back(Edge::new(replace, edge.back()));
                }
            } else if search == edge.back() {
                if edge.front() != replace {
                    wire.push_back(Edge::new(edge.front(), replace));
                }
            }
        }
        wire
    }
}
