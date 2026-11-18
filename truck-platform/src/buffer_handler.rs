use crate::*;

impl BufferHandler {
    /// Creates a buffer handler from a slice.
    #[inline(always)]
    pub fn from_slice<T: Sized + Pod + Zeroable, A: AsRef<[T]>>(
        vec: &A,
        device: &Device,
        usage: BufferUsages,
    ) -> Self {
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            contents: bytemuck::cast_slice(vec.as_ref()),
            usage,
            label: None,
        });
        let stride = size_of::<T>() as u64;
        let size = vec.as_ref().len() as u64 * stride;
        BufferHandler {
            buffer,
            size,
            stride,
        }
    }
    /// Creates a buffer handler directly wrapping an existing `wgpu::Buffer`.
    #[inline(always)]
    pub fn from_buffer(buffer: Buffer, size: u64, stride: u64) -> Self {
        BufferHandler {
            buffer,
            size,
            stride,
        }
    }
    /// Returns the reference of the buffer.
    #[inline(always)]
    pub const fn buffer(&self) -> &Buffer { &self.buffer }

    /// Returns the size of the buffer.
    #[inline(always)]
    pub const fn size(&self) -> u64 { self.size }

    /// Creates a binding resource from buffer slice.
    #[inline(always)]
    pub const fn binding_resource(&self) -> BindingResource<'_> {
        BindingResource::Buffer(BufferBinding {
            buffer: &self.buffer,
            offset: 0,
            size: None,
        })
    }

    /// Copy the values of buffer to `dest`.
    /// # Panic
    /// Panic occurs if the size of `dest` is smaller than the one of `self`.
    #[inline(always)]
    pub fn copy_buffer(&self, encoder: &mut CommandEncoder, dest: &BufferHandler) {
        assert!(
            self.size <= dest.size,
            "The destination buffer size must be no less than the source buffer size."
        );
        encoder.copy_buffer_to_buffer(&self.buffer, 0, &dest.buffer, 0, self.size);
    }

    /// Creates a storage buffer handler from a slice.
    #[inline(always)]
    pub fn storage_from_slice<T: Sized + Pod + Zeroable, A: AsRef<[T]>>(
        vec: &A,
        device: &Device,
    ) -> Self {
        Self::from_slice(
            vec,
            device,
            BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
        )
    }

    /// Creates an empty storage buffer with the given size in bytes.
    #[inline(always)]
    pub fn storage_empty(device: &Device, size: u64) -> Self {
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("storage_empty_buffer"),
            size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        BufferHandler {
            buffer,
            size,
            stride: 1,
        }
    }

    /// Reads back the contents of the buffer into a `Vec<T>`.
    pub async fn read_to_vec<T: Pod + Zeroable>(&self, handler: &DeviceHandler) -> Vec<T> {
        handler.read_buffer(&self.buffer, self.size).await
    }

    /// Synchronously reads back the contents of the buffer into a `Vec<T>`.
    pub fn read_to_vec_block<T: Pod + Zeroable>(&self, handler: &DeviceHandler) -> Vec<T> {
        handler.read_buffer_block(&self.buffer, self.size)
    }
}
