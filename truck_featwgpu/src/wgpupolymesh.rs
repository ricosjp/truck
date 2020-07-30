use crate::*;
use polymesh::*;
use std::collections::HashMap;

impl WGPUPolygonMesh {
    pub fn signup(&self, device: &Device) -> (Buffer, Buffer) {
        #[cfg(debug_assertions)]
        println!("Signup lender object\nvertices: {}\nindices: {}",
            self.vertices.len(),
            self.indices.len()
        );
        let vertex_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&self.vertices),
            BufferUsage::VERTEX,
        );
        let index_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&self.indices),
            BufferUsage::INDEX,
        );
        (vertex_buffer, index_buffer)
    }
}

fn signup_vertex(
    polymesh: &PolygonMesh,
    vertex: &[usize; 3],
    glpolymesh: &mut WGPUPolygonMesh,
    vertex_map: &mut HashMap<[usize; 3], u32>,
)
{
    let key = [vertex[0], vertex[1], vertex[2]];
    let idx = match vertex_map.get(&key) {
        Some(idx) => *idx,
        None => {
            let idx = glpolymesh.vertices.len() as u32;
            let position = (&polymesh.positions[key[0]]).into();
            let uv_coord = match polymesh.uv_coords.is_empty() {
                true => [0.0, 0.0],
                false => (&polymesh.uv_coords[key[1]]).into(),
            };
            let normal = match polymesh.normals.is_empty() {
                true => [0.0, 0.0, 0.0],
                false => (&polymesh.normals[key[2]]).into(),
            };
            let wgpuvertex = WGPUVertex {
                position,
                uv_coord,
                normal,
            };
            vertex_map.insert(key, idx);
            glpolymesh.vertices.push(wgpuvertex);
            idx
        }
    };
    glpolymesh.indices.push(idx);
}

impl Default for WGPUPolygonMesh {
    fn default() -> WGPUPolygonMesh {
        WGPUPolygonMesh {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}

impl From<&PolygonMesh> for WGPUPolygonMesh {
    fn from(polymesh: &PolygonMesh) -> WGPUPolygonMesh {
        let mut glpolymesh = WGPUPolygonMesh::default();
        let mut vertex_map = HashMap::<[usize; 3], u32>::new();
        for tri in &polymesh.tri_faces {
            signup_vertex(polymesh, &tri[0], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &tri[1], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &tri[2], &mut glpolymesh, &mut vertex_map);
        }
        for quad in &polymesh.quad_faces {
            signup_vertex(polymesh, &quad[0], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &quad[1], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &quad[3], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &quad[1], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &quad[2], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &quad[3], &mut glpolymesh, &mut vertex_map);
        }
        for face in &polymesh.other_faces {
            for i in 2..face.len() {
                signup_vertex(polymesh, &face[0], &mut glpolymesh, &mut vertex_map);
                signup_vertex(polymesh, &face[i - 1], &mut glpolymesh, &mut vertex_map);
                signup_vertex(polymesh, &face[i], &mut glpolymesh, &mut vertex_map);
            }
        }
        glpolymesh
    }
}

impl From<PolygonMesh> for WGPUPolygonMesh {
    #[inline(always)]
    fn from(polymesh: PolygonMesh) -> WGPUPolygonMesh { (&polymesh).into() }
}

impl From<MeshHandler> for WGPUPolygonMesh {
    #[inline(always)]
    fn from(mesh_handler: MeshHandler) -> WGPUPolygonMesh {
        Into::<PolygonMesh>::into(mesh_handler).into()
    }
}
