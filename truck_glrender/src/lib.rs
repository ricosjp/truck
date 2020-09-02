extern crate glium;
extern crate truck_geometry as geometry;
extern crate truck_polymesh as polymesh;
pub use geometry::*;
use glium::*;

#[derive(Debug, Clone, Copy)]
pub struct GLVertex {
    position: [f32; 3],
    uv_coord: [f32; 2],
    normal: [f32; 3],
}
glium::implement_vertex!(GLVertex, position, uv_coord, normal);

#[derive(Debug, Clone)]
pub struct GLPolygonMesh {
    vertices: Vec<GLVertex>,
    indices: Vec<u32>,
    pub matrix: Matrix4,
    pub color: [f32; 3],
    pub reflect_ratio: [f32; 3],
}

#[derive(Debug)]
pub struct RenderObject {
    vertex_buffer: glium::VertexBuffer<GLVertex>,
    indices: glium::IndexBuffer<u32>,
    matrix: [[f32; 4]; 4],
    color: [f32; 3],
    reflect_ratio: [f32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LightType {
    Point,
    Uniform,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Light {
    pub position: Vector3,
    pub strength: f64,
    pub light_type: LightType,
}

#[derive(Debug)]
pub struct Scene {
    objects: Vec<RenderObject>,
    program: glium::Program,
    clock: std::time::Instant,
    pub camera: Camera,
    pub light: Light,
}

pub use render::Render;

pub mod camera;
pub mod glpolymesh;
pub mod light;
pub mod render;
pub mod scene;
