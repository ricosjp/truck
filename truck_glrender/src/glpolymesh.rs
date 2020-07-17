use crate::*;
use glium::*;
use polymesh::*;
use std::collections::HashMap;

impl GLPolygonMesh {
    pub fn signup(
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

fn signup_vertex(
    polymesh: &PolygonMesh,
    vertex: &[usize; 3],
    glpolymesh: &mut GLPolygonMesh,
    vertex_map: &mut HashMap<[usize; 2], u32>,
)
{
    let key = [vertex[0], vertex[2]];
    let idx = match vertex_map.get(&key) {
        Some(idx) => *idx,
        None => {
            let idx = glpolymesh.vertices.len() as u32;
            let glvertex = GLVertex {
                position: (&polymesh.positions[key[0]]).into(),
                normal: (&polymesh.normals[key[1]]).into(),
            };
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
            color: [1.0; 3],
            reflect_ratio: [0.2, 0.6, 0.2],
        }
    }
}

impl From<&PolygonMesh> for GLPolygonMesh {
    fn from(polymesh: &PolygonMesh) -> GLPolygonMesh {
        if polymesh.normals.is_empty() {
            panic!("There is no normal.");
        }
        let mut glpolymesh = GLPolygonMesh::default();
        let mut vertex_map = HashMap::<[usize; 2], u32>::new();
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
