use crate::*;

impl RenderObject {
    pub fn new<T: Into<WGPUPolygonMesh>>(mesh: T, display: &Device) -> RenderObject {
        let wgpupolymesh = mesh.into();
        let (vertex_buffer, index_buffer) = wgpupolymesh.signup(&display);
        RenderObject {
            vertex_buffer: Arc::new(vertex_buffer),
            vertex_size: wgpupolymesh.vertices.len(),
            index_buffer: Arc::new(index_buffer),
            index_size: wgpupolymesh.indices.len(),
            matrix: Matrix4::identity(),
            color: Vector4::from([1.0; 4]),
            reflect_ratio: [0.2, 0.6, 0.2],
            bind_group: None,
        }
    }
    
    pub(super) fn object_buffer(&self, device: &Device) -> Buffer {
        let material_info = ObjectInfo {
            matrix: (&self.matrix).into(),
            material: (&self.color).into(),
            reflect_ratio: self.reflect_ratio,
        };
        device.create_buffer_with_data(
            bytemuck::cast_slice(&[material_info]),
            BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        )
    }

    pub(super) fn create_bind_group(
        &mut self,
        camera_buffer: &Buffer,
        light_buffer: &Buffer,
        scene_status_buffer: &Buffer,
        bind_group_layout: &BindGroupLayout,
        device: &Device,
    )
    {
        let object_buffer = self.object_buffer(device);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            bindings: &[
                // Camera
                Binding {
                    binding: 0,
                    resource: BindingResource::Buffer {
                        buffer: &camera_buffer,
                        range: 0..std::mem::size_of::<CameraInfo>() as u64,
                    },
                },
                // Light
                Binding {
                    binding: 1,
                    resource: BindingResource::Buffer {
                        buffer: &light_buffer,
                        range: 0..std::mem::size_of::<LightInfo>() as u64,
                    },
                },
                // Material
                Binding {
                    binding: 2,
                    resource: BindingResource::Buffer {
                        buffer: &object_buffer,
                        range: 0..std::mem::size_of::<ObjectInfo>() as u64,
                    },
                },
                // Scene Status
                Binding {
                    binding: 3,
                    resource: BindingResource::Buffer {
                        buffer: &scene_status_buffer,
                        range: 0..std::mem::size_of::<f32>() as u64,
                    },
                },
            ],
            label: None,
        });
        self.bind_group = Some(bind_group);
    }
}

impl Clone for RenderObject {
    fn clone(&self) -> RenderObject {
        RenderObject {
            vertex_buffer: Arc::clone(&self.vertex_buffer),
            vertex_size: self.vertex_size,
            index_buffer: Arc::clone(&self.index_buffer),
            index_size: self.index_size,
            bind_group: None,
            matrix: self.matrix.clone(),
            color: self.color.clone(),
            reflect_ratio: self.reflect_ratio.clone(),
        }
    }
}

impl<T> std::ops::MulAssign<T> for RenderObject
where Matrix4: std::ops::MulAssign<T> {
    #[inline(always)]
    fn mul_assign(&mut self, mat: T) {
        self.matrix *= mat;
    }
}