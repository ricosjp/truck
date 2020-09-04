use crate::*;
use lazy_static::*;

impl RenderObject {
    pub fn from_polygon<T: Into<WGPUPolygonMesh>>(mesh: T, device: &Device) -> RenderObject {
        let wgpupolymesh = mesh.into();
        let (vertex_buffer, index_buffer) = wgpupolymesh.signup(&device);
        let matrix_data: [[f32; 4]; 4] = wgpupolymesh.matrix.cast::<f32>().unwrap().into();
        let matrix_buffer = device.create_buffer_init(
            &BufferInitDescriptor {
                contents: bytemuck::cast_slice(&matrix_data),
                usage: BufferUsage::UNIFORM,
                label: None,
            }
        );
        let rr = wgpupolymesh.reflect_ratio;
        let color_data: [[f32; 4]; 4] = [
            wgpupolymesh.ambient.cast::<f32>().unwrap().into(),
            wgpupolymesh.diffuse.cast::<f32>().unwrap().into(),
            wgpupolymesh.specular.cast::<f32>().unwrap().into(),
            [rr[0] as f32, rr[1] as f32, rr[2] as f32, 0.0],
        ];
        RenderObject {
            vertex_buffer: Arc::new(vertex_buffer),
            vertex_size: wgpupolymesh.vertices.len(),
            index_buffer: Some(Arc::new(index_buffer)),
            index_size: Some(wgpupolymesh.indices.len()),
            matrix: Matrix4::identity(),
            color: Vector4::from([1.0; 4]),
            reflect_ratio: [0.2, 0.6, 0.2],
            bind_group: None,
        }
    }
    pub fn object_buffer(&self, device: &Device) -> BufferHandler {
        let material_info = ObjectInfo {
            matrix: (&self.matrix).cast().unwrap().into(),
            material: (&self.color).cast().unwrap().into(),
            reflect_ratio: self.reflect_ratio,
        };
        let buffer = device.create_buffer_with_data(bytemuck::cast_slice(&[material_info]), BufferUsage::UNIFORM);
        BufferHandler::new(buffer, std::mem::size_of::<ObjectInfo>() as u64)
    }

    pub fn update_bind_group(
        &mut self,
        camera_buffer: &BufferHandler,
        light_buffer: &BufferHandler,
        scene_status_buffer: &BufferHandler,
        bind_group_layout: &BindGroupLayout,
        device: &Device,
    )
    {
        let object_buffer = self.object_buffer(device);
        let buffers = vec![camera_buffer, light_buffer, &object_buffer, scene_status_buffer];
        let bind_group = buffer_handler::create_bind_group(device, bind_group_layout, buffers);
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
where Matrix4: std::ops::MulAssign<T>
{
    #[inline(always)]
    fn mul_assign(&mut self, mat: T) { self.matrix *= mat; }
}
