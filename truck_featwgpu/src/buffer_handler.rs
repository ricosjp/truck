use crate::*;

impl BufferHandler {
    pub fn new(buffer: Buffer, size: u64) -> Self { BufferHandler { buffer, size } }

    pub fn binding(&self, idx: u32) -> BindGroupEntry {
        BindGroupEntry {
            binding: idx,
            resource: BindingResource::Buffer(self.buffer.slice(..)),
        }
    }
}

pub(super) fn create_bind_group<'a, T: IntoIterator<Item = &'a BufferHandler>>(
    device: &Device,
    layout: &BindGroupLayout,
    buffers: T,
) -> BindGroup
{
    let entries: &Vec<BindGroupEntry> = &buffers
        .into_iter()
        .enumerate()
        .map(|(i, buffer)| buffer.binding(i as u32))
        .collect();
    device.create_bind_group(&BindGroupDescriptor {
        layout,
        entries,
        label: None,
    })
}
