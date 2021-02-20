use crate::*;

impl<V: Sized + Zeroable + Pod> ExpandedPolygon<V> {
    pub fn buffers(
        &self,
        vertex_usage: BufferUsage,
        index_usage: BufferUsage,
        device: &Device,
    ) -> (BufferHandler, BufferHandler) {
        let vertex_buffer = BufferHandler::from_slice(&self.vertices, device, vertex_usage);
        let index_buffer = BufferHandler::from_slice(&self.indices, device, index_usage);
        (vertex_buffer, index_buffer)
    }
}

impl<V> Default for ExpandedPolygon<V> {
    fn default() -> ExpandedPolygon<V> {
        ExpandedPolygon {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}
