use crate::*;
use truck_meshalgo::prelude::*;

/// Wasm wrapper by Polygonmesh
#[wasm_bindgen]
#[derive(Clone, Debug, Into, From, Deref, DerefMut)]
pub struct PolygonMesh(truck_meshalgo::prelude::PolygonMesh);

impl IntoWasm for truck_meshalgo::prelude::PolygonMesh {
    type WasmWrapper = PolygonMesh;
}

/// STL type.
#[wasm_bindgen]
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum StlType {
    /// Determine STL type automatically.
    ///
    /// # Reading
    /// If the first 5 bytes are..
    /// - "solid" => ascii format
    /// - otherwise => binary format
    ///
    /// # Writing
    /// Always binary format.
    Automatic,
    /// ASCII format.
    Ascii,
    /// Binary format.
    Binary,
}

impl From<StlType> for stl::StlType {
    fn from(stl_type: StlType) -> stl::StlType {
        match stl_type {
            StlType::Automatic => stl::StlType::Automatic,
            StlType::Ascii => stl::StlType::Ascii,
            StlType::Binary => stl::StlType::Binary,
        }
    }
}

/// Buffer for rendering polygon
#[wasm_bindgen]
#[derive(Debug, Clone, Default)]
pub struct PolygonBuffer {
    vertices: Vec<f32>,
    indices: Vec<u32>,
}

#[wasm_bindgen]
impl PolygonMesh {
    /// input from obj format
    pub fn from_obj(data: &[u8]) -> Option<PolygonMesh> {
        obj::read::<&[u8]>(data)
            .map_err(|e| gloo::console::error!(format!("{e}")))
            .ok()
            .map(|mesh| mesh.into_wasm())
    }
    /// input from STL format
    pub fn from_stl(data: &[u8], stl_type: StlType) -> Option<PolygonMesh> {
        stl::read::<&[u8]>(data, stl_type.into())
            .map_err(|e| gloo::console::error!(format!("{e}")))
            .ok()
            .map(|mesh| mesh.into_wasm())
    }
    /// output obj format
    pub fn to_obj(&self) -> Option<Vec<u8>> {
        let mut res = Vec::new();
        obj::write(&self.0, &mut res)
            .map_err(|e| gloo::console::error!(format!("{e}")))
            .ok()?;
        Some(res)
    }
    /// output stl format
    pub fn to_stl(&self, stl_type: StlType) -> Option<Vec<u8>> {
        let mut res = Vec::new();
        stl::write(&self.0, &mut res, stl_type.into())
            .map_err(|e| gloo::console::error!(format!("{e}")))
            .ok()?;
        Some(res)
    }
    /// Returns polygon buffer
    pub fn to_buffer(&self) -> PolygonBuffer {
        let exp = self.0.expands(|attr| {
            let position = attr.position;
            let uv_coord = attr.uv_coord.unwrap_or_else(Vector2::zero);
            let normal = attr.normal.unwrap_or_else(Vector3::zero);
            [
                position[0] as f32,
                position[1] as f32,
                position[2] as f32,
                uv_coord[0] as f32,
                uv_coord[1] as f32,
                normal[0] as f32,
                normal[1] as f32,
                normal[2] as f32,
            ]
        });
        PolygonBuffer {
            vertices: exp.attributes().iter().flatten().copied().collect(),
            indices: exp
                .faces()
                .triangle_iter()
                .flatten()
                .map(|x| x as u32)
                .collect(),
        }
    }
    /// meshing shell
    pub fn from_shell(shell: Shell, tol: f64) -> PolygonMesh { shell.to_polygon(tol) }
    /// meshing solid
    pub fn from_solid(solid: Solid, tol: f64) -> PolygonMesh { solid.to_polygon(tol) }
    /// Returns the bonding box
    pub fn bounding_box(&self) -> Vec<f64> {
        let bdd = self.0.bounding_box();
        let min = bdd.min();
        let max = bdd.max();
        vec![min[0], min[1], min[2], max[0], max[1], max[2]]
    }
    /// merge two polygons: `self` and `other`.
    pub fn merge(&mut self, other: PolygonMesh) { self.0.merge(other.0); }
}

#[wasm_bindgen]
impl PolygonBuffer {
    /// vertex buffer. One attribute contains `position: [f32; 3]`, `uv_coord: [f32; 2]` and `normal: [f32; 3]`.
    pub fn vertex_buffer(&self) -> Vec<f32> { self.vertices.clone() }
    /// the length (bytes) of vertex buffer. (Num of attributes) * 8 components * 4 bytes.
    pub fn vertex_buffer_size(&self) -> usize { self.vertices.len() * 4 }
    /// index buffer. `u32`.
    pub fn index_buffer(&self) -> Vec<u32> { self.indices.clone() }
    /// the length (bytes) of index buffer. (Num of triangles) * 3 vertices * 4 bytes.
    pub fn index_buffer_size(&self) -> usize { self.indices.len() * 4 }
}
