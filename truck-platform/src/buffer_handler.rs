use crate::*;

impl BufferHandler {
    /// Creates a buffer handler from a slice.
    #[inline(always)]
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
    
    /// Returns the reference of the buffer.
    #[inline(always)]
    pub fn buffer(&self) -> &Buffer { &self.buffer }

    /// Returns the size of the buffer.
    #[inline(always)]
    pub fn size(&self) -> u64 { self.size }

    /// Creates a binding resource from buffer slice.
    #[inline(always)]
    pub fn binding_resource<'a>(&'a self) -> BindingResource<'a> {
        BindingResource::Buffer(self.buffer.slice(..))
    }

    /// Copy the values of buffer to `dest`.
    /// # Panic
    /// Panic occurs if the size of `dest` is smaller than the one of `self`.
    #[inline(always)]
    pub fn copy_buffer(&self, encoder: &mut CommandEncoder, dest: &BufferHandler) {
        assert!(
            self.size < dest.size,
            "The destination buffer size must be shorter than the source buffer size."
        );
        encoder.copy_buffer_to_buffer(&self.buffer, 0, &dest.buffer, 0, self.size);
    }
}
