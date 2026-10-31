//! Compute pipeline utilities based on wgpu.

use crate::*;
use std::sync::Arc;

/// A compute shader pipeline and execution engine.
#[derive(Debug)]
pub struct ComputeEngine {
    handler: DeviceHandler,
    pipeline: Arc<ComputePipeline>,
    bind_group_layout: Arc<BindGroupLayout>,
}

impl ComputeEngine {
    /// Creates a new compute engine from WGSL shader source, entry point, and bind group layout entries.
    pub fn new(
        handler: &DeviceHandler,
        shader_source: &str,
        entry_point: &str,
        bind_group_layout_entries: &[BindGroupLayoutEntry],
    ) -> Self {
        let device = handler.device();
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("compute_shader_module"),
            source: ShaderSource::Wgsl(shader_source.into()),
        });
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("compute_bind_group_layout"),
            entries: bind_group_layout_entries,
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("compute_pipeline_layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("compute_pipeline"),
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: Some(entry_point),
            compilation_options: Default::default(),
            cache: None,
        });
        Self {
            handler: handler.clone(),
            pipeline: Arc::new(pipeline),
            bind_group_layout: Arc::new(bind_group_layout),
        }
    }

    /// Returns the reference of the bind group layout.
    #[inline(always)]
    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    /// Returns the reference of the compute pipeline.
    #[inline(always)]
    pub fn pipeline(&self) -> &ComputePipeline {
        &self.pipeline
    }

    /// Creates a bind group matching this engine's layout from binding resources.
    pub fn create_bind_group<'a, I: IntoIterator<Item = BindingResource<'a>>>(
        &self,
        resources: I,
    ) -> BindGroup {
        bind_group_util::create_bind_group(self.handler.device(), &self.bind_group_layout, resources)
    }

    /// Dispatches the compute pass with the given bind group and workgroup counts (x, y, z).
    pub fn dispatch(&self, bind_group: &BindGroup, workgroups: (u32, u32, u32)) {
        let device = self.handler.device();
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("compute_dispatch_encoder"),
        });
        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("compute_pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, bind_group, &[]);
            compute_pass.dispatch_workgroups(workgroups.0, workgroups.1, workgroups.2);
        }
        self.handler.queue().submit(Some(encoder.finish()));
    }
}
