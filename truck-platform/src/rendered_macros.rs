/// Implements [`Rendered::render_id()`](./trait.Rendered.html#tymethod.render_id).
/// # Arguments
/// `id_member`: the member variant of render id.
#[macro_export]
macro_rules! impl_render_id {
    ($($id_member: ident).*) => {
        #[inline(always)]
        fn render_id(&self) -> RenderID { self.$($id_member).* }
    };
}

/// Derives [`Rendred::render_id()`](./trait.Rendered.html#tymethod.render_id)
/// # Arguments
/// `id_member`: the member variant of the super `Rendered` struct.
#[macro_export]
macro_rules! derive_render_id {
    ($($id_member: ident).*) => {
        #[inline(always)]
        fn render_id(&self) -> RenderID { self.$($id_member).*.render_id() }
    };
}

/// Derives [`Rendred::vertex_buffer()`](./trait.Rendered.html#tymethod.vertex_buffer)
/// # Arguments
/// `id_member`: the member variant of the super `Rendered` struct.
#[macro_export]
macro_rules! derive_vertex_buffer {
    ($($id_member: ident).*) => {
        #[inline(always)]
        fn vertex_buffer(
            &self,
            device_handler: &DeviceHandler,
        ) -> (Arc<BufferHandler>, Option<Arc<BufferHandler>>)
        {
            self.$($id_member)*.vertex_buffer(device_handler)
        }
    };
}

/// Derives [`Rendred::bind_group_layout()`](./trait.Rendered.html#tymethod.bind_group_layout)
/// # Arguments
/// `id_member`: the member variant of the super `Rendered` struct.
#[macro_export]
macro_rules! derive_bind_group_layout {
    ($($id_member: ident).*) => {
        #[inline(always)]
        fn bind_group_layout(&self, device_handler: &DeviceHandler) -> Arc<BindGroupLayout> {
            self.$($id_member)*.bind_group_layout(device_handler)
        }
    };
}

/// Derives [`Rendred::bind_group()`](./trait.Rendered.html#tymethod.bind_group)
/// # Arguments
/// `id_member`: the member variant of the super `Rendered` struct.
#[macro_export]
macro_rules! derive_bind_group {
    ($($id_member: ident).*) => {
        #[inline(always)]
        fn bind_group(
            &self,
            device_handler: &DeviceHandler,
            layout: &BindGroupLayout,
        ) -> Arc<BindGroup>
        {
            self.$($id_member)*.bind_group(device_handler, layout)
        }
    };
}

/// Derives [`Rendred::pipeline()`](./trait.Rendered.html#tymethod.pipeline)
/// # Arguments
/// `id_member`: the member variant of the super `Rendered` struct.
#[macro_export]
macro_rules! derive_pipeline {
    ($($id_member: ident).*) => {
        fn pipeline(
            &self,
            device_handler: &DeviceHandler,
            layout: &PipelineLayout,
        ) -> Arc<RenderPipeline> {
            self.$($id_member)*.pipeline(device_handler, layout)
        }
    };
}
