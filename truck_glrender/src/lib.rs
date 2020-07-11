extern crate glium;
extern crate truck_polymesh as polymesh;
extern crate truck_geometry as geometry;
pub type Matrix4 = geometry::Matrix;

#[derive(Clone, Copy)]
pub struct GLVertex {
    position: [f32; 3],
    normal: [f32; 3],
}
glium::implement_vertex!(GLVertex, position, normal);

#[derive(Clone, Default)]
pub struct GLPolygonMesh {
    vertices: Vec<GLVertex>,
    indices: Vec<u32>,
}

pub enum ProjectionType {
    Perspective,
    Parallel,
}

pub struct Camera {
    /// the matrix of camera
    pub matrix: Matrix4,
    /// the field of view. If `None`, this camera is parallel projection.
    pub larger_screen_size: f64,
    /// the distance to the front clipping plane
    pub front_clipping_plane: f64,
    /// the distance to the back clipping plane
    pub back_clipping_plane: f64,
    /// parallel or perspective
    pub projection_type: ProjectionType,
}

pub mod glpolymesh;
pub mod camera;
