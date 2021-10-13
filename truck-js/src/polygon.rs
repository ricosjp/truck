use crate::*;
use truck_meshalgo::prelude::*;

/// Wasm wrapper by Polygonmesh
#[wasm_bindgen]
#[derive(Clone, Debug, Into, From, Deref, DerefMut)]
pub struct PolygonMesh(truck_meshalgo::prelude::PolygonMesh);

impl IntoWasm for truck_meshalgo::prelude::PolygonMesh {
	type WasmWrapper = PolygonMesh;
}

/// STL Type
#[wasm_bindgen]
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub enum STLType {
	/// Determine stl type automatically.
	///
	/// **Reading**: if the first 5 bytes are..
	/// - "solid" => ascii format
	/// - otherwise => binary format
	///
	/// **Writing**: always binary format.
	Automatic,
	/// ascii format
	ASCII,
	/// binary format
	Binary,
}

impl From<STLType> for stl::STLType {
	fn from(stl_type: STLType) -> stl::STLType {
		match stl_type {
			STLType::Automatic => stl::STLType::Automatic,
			STLType::ASCII => stl::STLType::ASCII,
			STLType::Binary => stl::STLType::Binary,
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
	#[inline(always)]
	pub fn from_obj(data: &[u8]) -> Option<PolygonMesh> {
		obj::read::<&[u8]>(data)
			.map_err(|e| eprintln!("{}", e))
			.ok()
			.map(|mesh| mesh.into_wasm())
	}
	/// input from STL format
	#[inline(always)]
	pub fn from_stl(data: &[u8], stl_type: STLType) -> Option<PolygonMesh> {
		stl::read::<&[u8]>(data, stl_type.into())
			.map_err(|e| eprintln!("{}", e))
			.ok()
			.map(|mesh| mesh.into_wasm())
	}
	/// output obj format
	#[inline(always)]
	pub fn to_obj(&self) -> Option<Vec<u8>> {
		let mut res = Vec::new();
		obj::write(&self.0, &mut res)
			.map_err(|e| eprintln!("{}", e))
			.ok()?;
		Some(res)
	}
	/// output stl format
	#[inline(always)]
	pub fn to_stl(&self, stl_type: STLType) -> Option<Vec<u8>> {
		let mut res = Vec::new();
		stl::write(&self.0, &mut res, stl_type.into())
			.map_err(|e| eprintln!("{}", e))
			.ok()?;
		Some(res)
	}
	/// Returns polygon buffer
	#[inline(always)]
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
	#[inline(always)]
	pub fn from_shell(shell: Shell, tol: f64) -> Option<PolygonMesh> { shell.to_polygon(tol) }
	/// meshing solid
	#[inline(always)]
	pub fn from_solid(solid: Solid, tol: f64) -> Option<PolygonMesh> { solid.to_polygon(tol) }
	/// Returns the bonding box
	#[inline(always)]
	pub fn bounding_box(&self) -> Vec<f64> {
		let bdd = self.0.bounding_box();
		let min = bdd.min();
		let max = bdd.max();
		vec![min[0], min[1], min[2], max[0], max[1], max[2]]
	}
}

#[wasm_bindgen]
impl PolygonBuffer {
	/// vertex buffer. One attribute contains `position: [f32; 3]`, `uv_coord: [f32; 2]` and `normal: [f32; 3]`.
	#[inline(always)]
	pub fn vertex_buffer(&self) -> Vec<f32> { self.vertices.clone() }
	#[inline(always)]
	/// the length (bytes) of vertex buffer. (Num of attributes) * 8 components * 4 bytes.
	pub fn vertex_buffer_size(&self) -> usize { self.vertices.len() * 4 }
	/// index buffer. `u32`.
	#[inline(always)]
	pub fn index_buffer(&self) -> Vec<u32> { self.indices.clone() }
	/// the length (bytes) of index buffer. (Num of triangles) * 3 vertices * 4 bytes.
	#[inline(always)]
	pub fn index_buffer_size(&self) -> usize { self.indices.len() * 4 }
}
