use crate::*;
use glium::*;

impl Scene {
    #[inline(always)]
    pub fn new(display: &Display) -> Scene {
        let program = Program::from_source(
            display,
            include_str!("vshader.vert"),
            include_str!("fshader.frag"),
            None,
        )
        .unwrap();
        Scene {
            objects: Default::default(),
            program,
            clock: std::time::Instant::now(),
            camera: Default::default(),
            light: Default::default(),
        }
    }

    #[inline(always)]
    pub fn with_shader(
        display: &Display,
        vertex_shader: &str,
        fragment_shader: &str,
        geometry_shader: Option<&str>,
    ) -> Scene
    {
        let program =
            Program::from_source(display, vertex_shader, fragment_shader, geometry_shader).unwrap();
        Scene {
            objects: Default::default(),
            program,
            clock: std::time::Instant::now(),
            camera: Default::default(),
            light: Default::default(),
        }
    }

    #[inline(always)]
    pub fn add_glpolymesh(&mut self, glpolymesh: &GLPolygonMesh, display: &glium::Display) {
        let (vertex_buffer, indices) = glpolymesh.signup(&display);
        self.objects.push(RenderObject {
            vertex_buffer,
            indices,
            matrix: (&glpolymesh.matrix).into(),
            color: glpolymesh.color,
            reflect_ratio: glpolymesh.reflect_ratio,
        })
    }

    #[inline(always)]
    pub fn remove_object(&mut self, idx: usize) -> RenderObject { self.objects.remove(idx) }

    #[inline(always)]
    pub fn number_of_objects(&self) -> usize { self.objects.len() }

    pub fn objects_bounding_box(&self) -> (Vector3, Vector3) {
        let mut bdd_box = (
            vector!(std::f64::MAX, std::f64::MAX, std::f64::MAX),
            vector!(std::f64::MIN, std::f64::MIN, std::f64::MIN),
        );
        for object in self.objects.iter() {
            for vert in object.vertex_buffer.as_slice().read().unwrap() {
                let tmp = vert.position;
                let pos = vector!(tmp[0], tmp[1], tmp[2], 1) * self.camera.matrix.inverse();
                if pos[0] < bdd_box.0[0] {
                    bdd_box.0[0] = pos[0];
                }
                if pos[0] > bdd_box.1[0] {
                    bdd_box.1[0] = pos[0];
                }
                if pos[1] < bdd_box.0[1] {
                    bdd_box.0[1] = pos[1];
                }
                if pos[1] > bdd_box.1[1] {
                    bdd_box.1[1] = pos[1];
                }
                if pos[2] < bdd_box.0[2] {
                    bdd_box.0[2] = pos[2];
                }
                if pos[2] > bdd_box.1[2] {
                    bdd_box.1[2] = pos[2];
                }
            }
        }
        bdd_box
    }

    fn fit_perspective(&mut self, bdd_box: &(Vector3, Vector3)) {
        let bdd_box = (&bdd_box.0, &bdd_box.1);
        let size_vec = bdd_box.1 - bdd_box.0;
        let size = if size_vec[0] < size_vec[1] {
            size_vec[1]
        } else {
            size_vec[0]
        };
        let length = size / self.camera.screen_size;
        let mut move_vec = (bdd_box.0 + bdd_box.1) / 2.0;
        move_vec[2] = bdd_box.1[2] + length;
        let move_mat = Matrix3::identity().affine(&move_vec);

        let cammat = self.camera.matrix.clone();
        let caminv = cammat.inverse();
        self.camera *= &caminv * move_mat * &cammat;
    }

    fn fit_parallel(&mut self, bdd_box: &(Vector3, Vector3)) {
        let bdd_box = (&bdd_box.0, &bdd_box.1);
        let size_vec = bdd_box.1 - bdd_box.0;
        self.camera.screen_size = if size_vec[0] < size_vec[1] {
            size_vec[1]
        } else {
            size_vec[0]
        };
        let mut move_vec = (bdd_box.0 + bdd_box.1) / 2.0;
        move_vec[2] = bdd_box.1[2] + 1.0;
        let move_mat = Matrix3::identity().affine(&move_vec);
        let cammat = self.camera.matrix.clone();
        let caminv = cammat.inverse();
        self.camera *= &caminv * move_mat * &cammat;
    }

    pub fn fit_camera(&mut self) {
        let bdd_box = self.objects_bounding_box();
        match self.camera.projection_type {
            ProjectionType::Perspective => self.fit_perspective(&bdd_box),
            ProjectionType::Parallel => self.fit_parallel(&bdd_box),
        }
    }

    fn camera_projection(&self, target: &glium::Frame) -> [[f32; 4]; 4] {
        let dim = target.get_dimensions();
        let as_rat = (dim.1 as f64) / (dim.0 as f64);
        let mat = self.camera.projection() * Matrix4::diagonal(&vector!(as_rat, 1, 1, 1));
        mat.into()
    }

    pub fn render_scene(&mut self, target: &mut glium::Frame) {
        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            //backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            ..Default::default()
        };
        for object in &self.objects {
            let uniform = uniform!(
                model_matrix: object.matrix,
                camera_matrix: Into::<[[f32; 4]; 4]>::into(self.camera.matrix()),
                camera_projection: self.camera_projection(target),
                light_position: Into::<[f32; 3]>::into(&self.light.position),
                light_strength: self.light.strength as f32,
                light_type: self.light.light_type.type_id(),
                material: object.color,
                reflect_ratio: object.reflect_ratio,
                elapsed_time: self.clock.elapsed().as_secs_f32(),
            );
            target
                .draw(
                    &object.vertex_buffer,
                    &object.indices,
                    &self.program,
                    &uniform,
                    &params,
                )
                .unwrap();
        }
    }
}
