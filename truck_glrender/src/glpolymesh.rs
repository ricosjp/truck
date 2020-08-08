use crate::*;
use polymesh::*;
use std::collections::HashMap;

impl GLPolygonMesh {
    fn signup(
        &self,
        display: &Display,
    ) -> (VertexBuffer<GLVertex>, IndexBuffer<u32>)
    {
        (
            VertexBuffer::new(display, &self.vertices).unwrap(),
            IndexBuffer::new(
                display,
                index::PrimitiveType::TrianglesList,
                &self.indices,
            )
            .unwrap(),
        )
    }
}

impl RenderObject {
    pub fn new(glpolymesh: &GLPolygonMesh, display: &Display) -> RenderObject {
        let (vertex_buffer, indices) = glpolymesh.signup(&display);
        RenderObject {
            vertex_buffer,
            indices,
            matrix: (&glpolymesh.matrix).cast().unwrap().into(),
            color: glpolymesh.color,
            reflect_ratio: glpolymesh.reflect_ratio,
        }
    }
}

fn signup_vertex(
    polymesh: &PolygonMesh,
    vertex: &[usize; 3],
    glpolymesh: &mut GLPolygonMesh,
    vertex_map: &mut HashMap<[usize; 3], u32>,
)
{
    let key = [vertex[0], vertex[1], vertex[2]];
    let idx = match vertex_map.get(&key) {
        Some(idx) => *idx,
        None => {
            let idx = glpolymesh.vertices.len() as u32;
            let position = (&polymesh.positions[key[0]]).cast().unwrap().into();
            let uv_coord = match polymesh.uv_coords.is_empty() {
                true => [0.0, 0.0],
                false => (&polymesh.uv_coords[key[1]]).cast().unwrap().into(),
            };
            let normal = match polymesh.normals.is_empty() {
                true => [0.0, 0.0, 0.0],
                false => (&polymesh.normals[key[2]]).cast().unwrap().into(),
            };
            let glvertex = GLVertex { position, uv_coord, normal };
            vertex_map.insert(key, idx);
            glpolymesh.vertices.push(glvertex);
            idx
        }
    };
    glpolymesh.indices.push(idx);
}

impl Default for GLPolygonMesh {
    fn default() -> GLPolygonMesh {
        GLPolygonMesh {
            vertices: Vec::new(),
            indices: Vec::new(),
            matrix: Matrix4::identity(),
            color: [1.0; 3],
            reflect_ratio: [0.2, 0.6, 0.2],
        }
    }
}

impl From<&PolygonMesh> for GLPolygonMesh {
    fn from(polymesh: &PolygonMesh) -> GLPolygonMesh {
        let mut glpolymesh = GLPolygonMesh::default();
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

impl From<PolygonMesh> for GLPolygonMesh {
    #[inline(always)]
    fn from(polymesh: PolygonMesh) -> GLPolygonMesh { (&polymesh).into() }
}

impl From<MeshHandler> for GLPolygonMesh {
    #[inline(always)]
    fn from(mesh_handler: MeshHandler) -> GLPolygonMesh {
        Into::<PolygonMesh>::into(mesh_handler).into()
    }
}
