use crate::*;

impl BufferHandler {
    pub fn new(buffer: Buffer, size: u64) -> Self { BufferHandler { buffer, size } }

    pub fn from_slice<T: Sized + Pod + Zeroable, A: AsRef<[T]>>(
        vec: &A,
        device: &Device,
        usage: BufferUsage,
    ) -> Self
    {
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            contents: bytemuck::cast_slice(vec.as_ref()),
            usage,
            label: None,
        });
        let size = (vec.as_ref().len() * std::mem::size_of::<T>()) as u64;
        BufferHandler { buffer, size }
    }

    pub fn binding_resource<'a>(&'a self) -> BindingResource<'a> {
        BindingResource::Buffer(self.buffer.slice(..))
    }
}
