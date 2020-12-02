mod common;
use std::sync::{Arc, Mutex};
use truck_platform::*;
use truck_rendimpl::*;
use wgpu::*;

pub struct BGCheckPolygonInstance<'a> {
    polygon: PolygonInstance,
    fragment_shader: &'a [u8],
}

impl<'a> Rendered for BGCheckPolygonInstance<'a> {
    #[inline(always)]
    fn get_id(&self) -> RenderID { self.polygon.get_id() }
    #[inline(always)]
    fn set_id(&mut self, object_handler: &mut ObjectsHandler) {
        self.polygon.set_id(object_handler)
    }
    #[inline(always)]
    fn vertex_buffer(
        &self,
        device_handler: &DeviceHandler,
    ) -> (Arc<BufferHandler>, Option<Arc<BufferHandler>>)
    {
        self.polygon.vertex_buffer(device_handler)
    }
    #[inline(always)]
    fn bind_group_layout(&self, device_handler: &DeviceHandler) -> Arc<BindGroupLayout> {
        self.polygon.bind_group_layout(device_handler)
    }
    #[inline(always)]
    fn bind_group(
        &self,
        device_handler: &DeviceHandler,
        layout: &BindGroupLayout,
    ) -> Arc<BindGroup>
    {
        self.polygon.bind_group(device_handler, layout)
    }
    #[inline(always)]
    fn pipeline(
        &self,
        device_handler: &DeviceHandler,
        layout: &PipelineLayout,
    ) -> Arc<RenderPipeline>
    {
        let vertex_shader = include_spirv!("shaders/mesh-bindgroup.vert");
        let fragment_shader = wgpu::util::make_spirv(self.fragment_shader);
        self.polygon
            .pipeline_with_shader(vertex_shader, fragment_shader, device_handler, layout)
    }
}

fn test_polygons() -> [PolygonMesh; 3] {
    let positions = vec![
        Point3::new(-1.0, 2.0, -1.0),
        Point3::new(1.0, 2.0, -1.0),
        Point3::new(-1.0, 2.0, 1.0),
        Point3::new(1.0, 2.0, 1.0),
    ];
    let uv_coords = vec![
        Vector2::new(-1.0, -1.0),
        Vector2::new(1.0, -1.0),
        Vector2::new(-1.0, 1.0),
        Vector2::new(1.0, 1.0),
    ];
    let normals = vec![
        Vector3::new(-1.0, 0.2, -1.0),
        Vector3::new(1.0, 0.2, -1.0),
        Vector3::new(-1.0, 0.2, 1.0),
        Vector3::new(1.0, 0.2, 1.0),
    ];
    let tri_faces = vec![
        [[0, 0, 0], [1, 1, 1], [2, 2, 2]],
        [[2, 2, 2], [1, 1, 1], [3, 3, 3]],
    ];
    let quad_faces = vec![[[0, 0, 0], [1, 1, 1], [3, 3, 3], [2, 2, 2]]];
    let other_faces = vec![vec![[0, 0, 0], [1, 1, 1], [3, 3, 3], [2, 2, 2]]];

    [
        PolygonMesh {
            positions: positions.clone(),
            uv_coords: uv_coords.clone(),
            normals: normals.clone(),
            tri_faces,
            ..Default::default()
        },
        PolygonMesh {
            positions: positions.clone(),
            uv_coords: uv_coords.clone(),
            normals: normals.clone(),
            quad_faces,
            ..Default::default()
        },
        PolygonMesh {
            positions,
            uv_coords,
            normals,
            other_faces,
            ..Default::default()
        },
    ]
}

fn nontex_inst_desc() -> InstanceDescriptor {
    InstanceDescriptor {
        matrix: Matrix4::from_cols(
            [1.0, 2.0, 3.0, 4.0].into(),
            [5.0, 6.0, 7.0, 8.0].into(),
            [9.0, 10.0, 11.0, 12.0].into(),
            [13.0, 14.0, 15.0, 16.0].into(),
        ),
        material: Material {
            albedo: Vector4::new(0.2, 0.4, 0.6, 1.0),
            roughness: 0.31415,
            reflectance: 0.29613,
        },
        texture: None,
        backface_culling: true,
    }
}

#[test]
fn polymesh_bind_group_test() {
    let instance = Instance::new(BackendBit::PRIMARY);
    let (device, queue) = common::init_device(&instance);
    let sc_desc = Arc::new(Mutex::new(common::swap_chain_descriptor()));
    let mut scene = Scene::new(&device, &queue, &sc_desc, &Default::default());

    let plane = new_plane!("shaders/plane.vert", "shaders/unicolor.frag");
    
    let [tris, quads, others] = test_polygons();
    let inst_desc = nontex_inst_desc();
    let tris_instance = scene.create_instance(&tris, &inst_desc);
    let quads_instance = scene.create_instance(&quads, &inst_desc);
    let others_instance = scene.create_instance(&others, &inst_desc);
}
