use crate::*;

impl BufferHandler {
    pub fn new(buffer: Buffer, size: u64) -> Self { BufferHandler { buffer, size } }

    pub fn binding(&self, idx: u32) -> Binding {
        Binding {
            binding: idx,
            resource: BindingResource::Buffer {
                buffer: &self.buffer,
                range: 0..self.size,
            },
        }
    }
}

pub(super) fn create_bind_group<'a, T: IntoIterator<Item = &'a BufferHandler>>(
    device: &Device,
    layout: &BindGroupLayout,
    buffers: T,
) -> BindGroup
{
    let bindings: &Vec<Binding> = &buffers
        .into_iter()
        .enumerate()
        .map(|(i, buffer)| buffer.binding(i as u32))
        .collect();
    device.create_bind_group(&BindGroupDescriptor {
        layout,
        bindings,
        label: None,
    })
}
