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
	pub fn to_expanded(&self) -> PolygonBuffer { PolygonBuffer::from(&self.0) }
	/// meshing shell
	#[inline(always)]
	pub fn from_shell(shell: Shell, tol: f64) -> Option<PolygonMesh> { shell.to_polygon(tol) }
	/// meshing solid
	#[inline(always)]
	pub fn from_solid(solid: Solid, tol: f64) -> Option<PolygonMesh> { solid.to_polygon(tol) }
}

#[wasm_bindgen]
impl PolygonBuffer {
	/// vertex buffer. One attribute contains `position: [f32; 3]`, `uv_coord: [f32; 2]` and `normal: [f32; 3]`.
	#[inline(always)]
	pub fn vertex_buffer(&self) -> *const f32 { self.vertices.as_ptr() }
	#[inline(always)]
	/// the length of vertex buffer. (Num of attributes) * 32.
	pub fn vertex_buffer_size(&self) -> usize { self.vertices.len() * 4 }
	/// index buffer. `u32`.
	#[inline(always)]
	pub fn index_buffer(&self) -> *const u32 { self.indices.as_ptr() }
	/// the length of index buffer. (Num of triangle) * 12.
	#[inline(always)]
	pub fn indices_length(&self) -> usize { self.indices.len() * 4 }
}

mod to_expanded {
	use super::PolygonBuffer;
	use rustc_hash::FxHashMap as HashMap;
	use truck_meshalgo::prelude::*;
	fn signup_vertex(
		polymesh: &PolygonMesh,
		vertex: Vertex,
		glpolymesh: &mut PolygonBuffer,
		vertex_map: &mut HashMap<Vertex, u32>,
	) {
		let idx = *vertex_map.entry(vertex).or_insert_with(|| {
			let idx = glpolymesh.vertices.len() as u32;
			let position: [f32; 3] = polymesh.positions()[vertex.pos].cast().unwrap().into();
			let uv_coord = match vertex.uv {
				Some(uv) => polymesh.uv_coords()[uv].cast().unwrap().into(),
				None => [0.0, 0.0],
			};
			let normal = match vertex.nor {
				Some(nor) => polymesh.normals()[nor].cast().unwrap().into(),
				None => [0.0, 0.0, 0.0],
			};
			glpolymesh.vertices.push(position[0]);
			glpolymesh.vertices.push(position[1]);
			glpolymesh.vertices.push(position[2]);
			glpolymesh.vertices.push(uv_coord[0]);
			glpolymesh.vertices.push(uv_coord[1]);
			glpolymesh.vertices.push(normal[0]);
			glpolymesh.vertices.push(normal[1]);
			glpolymesh.vertices.push(normal[2]);
			idx
		});
		glpolymesh.indices.push(idx);
	}

	impl From<&PolygonMesh> for PolygonBuffer {
		fn from(polymesh: &PolygonMesh) -> PolygonBuffer {
			let mut glpolymesh = PolygonBuffer::default();
			let mut vertex_map = HashMap::<Vertex, u32>::default();
			for tri in polymesh.faces().tri_faces() {
				signup_vertex(polymesh, tri[0], &mut glpolymesh, &mut vertex_map);
				signup_vertex(polymesh, tri[1], &mut glpolymesh, &mut vertex_map);
				signup_vertex(polymesh, tri[2], &mut glpolymesh, &mut vertex_map);
			}
			for quad in polymesh.faces().quad_faces() {
				signup_vertex(polymesh, quad[0], &mut glpolymesh, &mut vertex_map);
				signup_vertex(polymesh, quad[1], &mut glpolymesh, &mut vertex_map);
				signup_vertex(polymesh, quad[3], &mut glpolymesh, &mut vertex_map);
				signup_vertex(polymesh, quad[1], &mut glpolymesh, &mut vertex_map);
				signup_vertex(polymesh, quad[2], &mut glpolymesh, &mut vertex_map);
				signup_vertex(polymesh, quad[3], &mut glpolymesh, &mut vertex_map);
			}
			for face in polymesh.faces().other_faces() {
				for i in 2..face.len() {
					signup_vertex(polymesh, face[0], &mut glpolymesh, &mut vertex_map);
					signup_vertex(polymesh, face[i - 1], &mut glpolymesh, &mut vertex_map);
					signup_vertex(polymesh, face[i], &mut glpolymesh, &mut vertex_map);
				}
			}
			glpolymesh
		}
	}
}
