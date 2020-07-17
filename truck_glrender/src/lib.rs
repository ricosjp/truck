extern crate glium;
extern crate truck_geometry as geometry;
extern crate truck_polymesh as polymesh;
pub use geometry::{matrix, vector, Matrix4, Vector3};

#[derive(Debug, Clone, Copy)]
pub struct GLVertex {
    position: [f32; 3],
    normal: [f32; 3],
}
glium::implement_vertex!(GLVertex, position, normal);

#[derive(Debug, Clone)]
pub struct GLPolygonMesh {
    vertices: Vec<GLVertex>,
    indices: Vec<u32>,
    pub color: [f32; 3],
    pub reflect_ratio: [f32; 3],
}

#[derive(Debug)]
pub struct RenderObject {
    vertex_buffer: glium::VertexBuffer<GLVertex>,
    indices: glium::IndexBuffer<u32>,
    color: [f32; 3],
    reflect_ratio: [f32; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectionType {
    Perspective,
    Parallel,
}

#[derive(Debug, Clone)]
pub struct Camera {
    matrix: Matrix4,
    screen_size: f64,
    front_clipping_plane: f64,
    back_clipping_plane: f64,
    projection_type: ProjectionType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Light {
    Point {
        position: Vector3,
        strength: f64,
    },
    Uniform {
        direction: Vector3,
        strength: f64,
    },
}

impl Default for Light {
    #[inline(always)]
    fn default() -> Light {
        Light::Point {
            position: Vector3::zero(),
            strength: 1.0,
        }
    }
}

#[derive(Debug)]
pub struct Scene {
    objects: Vec<RenderObject>,
    program: glium::Program,
    pub camera: Camera,
    pub light: Light,
}

pub use renderer::Render;

pub mod camera;
pub mod glpolymesh;
pub mod renderer;
pub mod scene;
