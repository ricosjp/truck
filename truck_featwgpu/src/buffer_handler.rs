use crate::*;

impl BufferHandler {
    pub fn new(buffer: Buffer, size: u64) -> Self { BufferHandler { buffer, size } }

    pub fn binding_resource<'a>(&'a self) -> BindingResource<'a> {
        BindingResource::Buffer(self.buffer.slice(..))
    }
}
