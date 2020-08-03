use super::*;

impl IndexCreator {
    fn bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                // the length of division
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::UniformBuffer { dynamic: false },
                },
                // index buffer
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: false,
                    },
                },
            ],
            label: None,
        })
    }
    #[inline(always)]
    pub(super) fn new(device: &Device) -> Self {
        let bind_group_layout = Self::bind_group_layout(device);
        let compute_handler =
            ComputeHandler::new(device, bind_group_layout, include_str!("create_index.comp"));
        Self(compute_handler)
    }

    #[inline(always)]
    pub(super) fn index_buffer(
        &self,
        device: &Device,
        queue: &Queue,
        div_lens: &[usize; 2],
    ) -> Buffer
    {
        let [udiv_length, vdiv_length] = [div_lens[0] - 1, div_lens[1] - 1];
        let div_length_buffer = BufferHandler::new(
            device.create_buffer_with_data(
                bytemuck::cast_slice(&[udiv_length as u32 + 1, vdiv_length as u32 + 1]),
                BufferUsage::UNIFORM,
            ),
            U32_SIZE as u64 * 2,
        );
        let index_buffer_size = (udiv_length * vdiv_length * U32_SIZE * 6) as u64;
        let index_storage = BufferHandler::new(
            device.create_buffer(&BufferDescriptor {
                usage: BufferUsage::STORAGE | BufferUsage::COPY_SRC,
                size: index_buffer_size,
                label: None,
            }),
            index_buffer_size,
        );
        let buffers = vec![div_length_buffer, index_storage];
        let bind_group =
            buffer_handler::create_bind_group(device, &self.0.bind_group_layout, &buffers);
        let index_buffer = device.create_buffer(&BufferDescriptor {
            usage: BufferUsage::INDEX | BufferUsage::COPY_DST,
            size: index_buffer_size,
            label: None,
        });
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        {
            let mut cpass = encoder.begin_compute_pass();
            cpass.set_pipeline(&self.0.pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch(udiv_length as u32, vdiv_length as u32, 1);
        }
        encoder.copy_buffer_to_buffer(&buffers[1].buffer, 0, &index_buffer, 0, index_buffer_size);
        queue.submit(&[encoder.finish()]);
        index_buffer
    }
}
