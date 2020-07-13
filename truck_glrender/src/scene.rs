use crate::*;
use glium::*;

impl Scene {
    #[inline(always)]
    pub fn new(display: &glium::Display) -> Scene {
        let program = glium::Program::from_source(
            display,
            include_str!("vshader.vert"),
            include_str!("fshader.frag"),
            None,
        )
        .unwrap();
        Scene {
            objects: Default::default(),
            program,
            camera: Default::default(),
            light: Default::default(),
        }
    }

    pub fn add_glpolymesh(&mut self, glpolymesh: &GLPolygonMesh, display: &glium::Display) {
        let (vertex_buffer, indices) = glpolymesh.signup(&display);
        self.objects.push(RenderObject {
            vertex_buffer,
            indices,
            color: glpolymesh.color,
            reflect_ratio: glpolymesh.reflect_ratio,
        })
    }

    #[inline(always)]
    pub fn remove_object(&mut self, idx: usize) -> RenderObject { self.objects.remove(idx) }

    #[inline(always)]
    pub fn number_of_objects(&self) -> usize { self.objects.len() }

    pub fn objects_bounding_box(&self) -> [(f64, f64); 3] {
        let mut bdd_box = [(std::f64::MAX, std::f64::MIN); 3];
        for object in self.objects.iter() {
            for vert in object.vertex_buffer.as_slice().read().unwrap() {
                let tmp = vert.position;
                let pos = vector!(tmp[0], tmp[1], tmp[2], 1)
                    * self.camera.matrix.inverse();
                if pos[0] < bdd_box[0].0 {
                    bdd_box[0].0 = pos[0];
                }
                if pos[0] > bdd_box[0].1 {
                    bdd_box[0].1 = pos[0];
                }
                if pos[1] < bdd_box[1].0 {
                    bdd_box[1].0 = pos[1];
                }
                if pos[1] > bdd_box[1].1 {
                    bdd_box[1].1 = pos[1];
                }
                if pos[2] < bdd_box[2].0 {
                    bdd_box[2].0 = pos[2];
                }
                if pos[2] > bdd_box[2].1 {
                    bdd_box[2].1 = pos[2];
                }
            }
        }
        bdd_box
    }

    fn fit_perspective(&mut self, bdd_box: &[(f64, f64); 3]) {
        let camera = &self.camera;
        
        let size_x = bdd_box[0].1 - bdd_box[0].0;
        let size_y = bdd_box[1].1 - bdd_box[1].0;
        let size = if size_x < size_y { size_y } else { size_x };
        let length = size / self.camera.screen_size;
        
        let mut move_mat = Matrix4::identity();
        move_mat[3][0] = (bdd_box[0].0 + bdd_box[0].1) / 2.0;
        move_mat[3][1] = (bdd_box[1].0 + bdd_box[1].1) / 2.0;
        move_mat[3][2] = bdd_box[2].1 + length;

        let cammat = camera.matrix.clone();
        let caminv = cammat.inverse();
        self.camera *= &caminv * move_mat * &cammat;
    }

    fn fit_parallel(&mut self, _: &[(f64, f64); 3]) {}

    pub fn fit_camera(&mut self) {
        let bdd_box = self.objects_bounding_box();
        match self.camera.projection_type {
            ProjectionType::Perspective => self.fit_perspective(&bdd_box),
            ProjectionType::Parallel => self.fit_parallel(&bdd_box),
        }
    }

    pub fn render_scene(&mut self, target: &mut glium::Frame) {
        let dim = target.get_dimensions();
        let as_rat = (dim.1 as f64) / (dim.0 as f64);
        let mat = &self.camera.matrix;
        let proj = self.camera.projection() * Matrix4::diagonal(&vector!(as_rat, 1, 1, 1));
        let camera_matrix = [
                [mat[0][0] as f32, mat[0][1] as f32, mat[0][2] as f32, mat[0][3] as f32],
                [mat[1][0] as f32, mat[1][1] as f32, mat[1][2] as f32, mat[1][3] as f32],
                [mat[2][0] as f32, mat[2][1] as f32, mat[2][2] as f32, mat[2][3] as f32],
                [mat[3][0] as f32, mat[3][1] as f32, mat[3][2] as f32, mat[3][3] as f32],
        ];
        let camera_projection = [
                [proj[0][0] as f32, proj[0][1] as f32, proj[0][2] as f32, proj[0][3] as f32],
                [proj[1][0] as f32, proj[1][1] as f32, proj[1][2] as f32, proj[1][3] as f32],
                [proj[2][0] as f32, proj[2][1] as f32, proj[2][2] as f32, proj[2][3] as f32],
                [proj[3][0] as f32, proj[3][1] as f32, proj[3][2] as f32, proj[3][3] as f32],
        ];
        let (light_position, light_strength) = match &self.light {
            Light::Point { position: pos, strength } => {
                ([pos[0] as f32, pos[1] as f32, pos[2] as f32], *strength as f32)
            },
            Light::Uniform { .. } => {
                todo!()
            },
        };
        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            ..Default::default()
        };
        for object in &self.objects {
            let uniform = uniform!(
                camera_matrix: camera_matrix,
                camera_projection: camera_projection,
                light_position: light_position,
                light_strength: light_strength,
                material: object.color,
                reflect_ratio: object.reflect_ratio,
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

