#[macro_export]
macro_rules! impl_get_set_id {
    ($($id_member: ident).*) => {
        #[inline(always)]
        fn get_id(&self) -> RenderID { self.$($id_member).* }
        #[inline(always)]
        fn set_id(&mut self, objects_handler: &mut ObjectsHandler) {
            objects_handler.set_id(&mut self.$($id_member).*)
        }
    };
}

#[macro_export]
macro_rules! derive_get_set_id {
    ($($id_member: ident).*) => {
        #[inline(always)]
        fn get_id(&self) -> RenderID { self.$($id_member).*.get_id() }
        #[inline(always)]
        fn set_id(&mut self, objects_handler: &mut ObjectsHandler) {
            self.$($id_member)*.set_id(objects_handler);
        }
    };
}

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

#[macro_export]
macro_rules! derive_bind_group_layout {
    ($($id_member: ident).*) => {
        #[inline(always)]
        fn bind_group_layout(&self, device_handler: &DeviceHandler) -> Arc<BindGroupLayout> {
            self.$($id_member)*.bind_group_layout(device_handler)
        }
    };
}

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
