use crate::*;
use polymesh::*;
use std::collections::HashMap;

impl GLPolygonMesh {
    pub fn signup(
        &self,
        display: &glium::Display,
    ) -> (glium::VertexBuffer<GLVertex>, glium::IndexBuffer<u32>)
    {
        (
            glium::VertexBuffer::new(display, &self.vertices).unwrap(),
            glium::index::IndexBuffer::new(
                display,
                glium::index::PrimitiveType::TrianglesList,
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
                position: polymesh.positions[key[0]].clone().into(),
                normal: polymesh.normals[key[1]].clone().into(),
            };
            vertex_map.insert(key, idx);
            glpolymesh.vertices.push(glvertex);
            idx
        }
    };
    glpolymesh.indices.push(idx);
}

impl std::convert::TryFrom<&PolygonMesh> for GLPolygonMesh {
    type Error = ();
    fn try_from(polymesh: &PolygonMesh) -> Result<GLPolygonMesh, ()> {
        if polymesh.normals.is_empty() {
            return Err(());
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
        Ok(glpolymesh)
    }
}
