use crate::MeshHandler;
use geometry::Tolerance;
use std::iter::Iterator;

impl MeshHandler {
    pub fn remove_unused_attrs(&mut self) -> &mut Self {
        let mesh = &mut self.mesh;
        if !mesh.vertices.is_empty() {
            let vec0 = mesh.tri_faces.iter_mut().flat_map(|arr| arr.iter_mut());
            let vec1 = mesh.quad_faces.iter_mut().flat_map(|arr| arr.iter_mut());
            let vec = vec0.chain(vec1);
            let idcs = sub_remove_unused_attrs(vec, 0, mesh.vertices.len());
            mesh.vertices = idcs.iter().map(|idx| mesh.vertices[*idx].clone()).collect();
        }
        if !mesh.uv_coords.is_empty() {
            let vec0 = mesh.tri_faces.iter_mut().flat_map(|arr| arr.iter_mut());
            let vec1 = mesh.quad_faces.iter_mut().flat_map(|arr| arr.iter_mut());
            let vec = vec0.chain(vec1);
            let idcs = sub_remove_unused_attrs(vec, 1, mesh.uv_coords.len());
            mesh.uv_coords = idcs
                .iter()
                .map(|idx| mesh.uv_coords[*idx].clone())
                .collect();
        }
        if !mesh.normals.is_empty() {
            let vec0 = mesh.tri_faces.iter_mut().flat_map(|arr| arr.iter_mut());
            let vec1 = mesh.quad_faces.iter_mut().flat_map(|arr| arr.iter_mut());
            let vec = vec0.chain(vec1);
            let idcs = sub_remove_unused_attrs(vec, 2, mesh.normals.len());
            mesh.normals = idcs.iter().map(|idx| mesh.normals[*idx].clone()).collect();
        }
        self
    }

    pub fn remove_degenerate_faces(&mut self) -> &mut Self {
        let mesh = &mut self.mesh;
        let mut new_quad_faces = Vec::new();
        for face in &mesh.quad_faces {
            let is_deg0 = is_degenerate_tri(&mesh.vertices, face[0][0], face[1][0], face[3][0]);
            if is_deg0 {
                mesh.tri_faces
                    .push([face[2].clone(), face[3].clone(), face[1].clone()]);
            }
            let is_deg1 = is_degenerate_tri(&mesh.vertices, face[2][0], face[3][0], face[1][0]);
            if is_deg1 {
                mesh.tri_faces
                    .push([face[0].clone(), face[1].clone(), face[3].clone()]);
            }
            if !is_deg0 && !is_deg1 {
                new_quad_faces.push(face.clone());
            }
        }
        let mut new_tri_faces = Vec::new();
        for face in &mesh.tri_faces {
            let is_deg = is_degenerate_tri(&mesh.vertices, face[0][0], face[1][0], face[2][0]);
            if !is_deg {
                new_tri_faces.push(face.clone());
            }
        }
        mesh.tri_faces = new_tri_faces;
        mesh.quad_faces = new_quad_faces;

        self
    }
}

fn is_degenerate_tri(vertices: &Vec<[f64; 3]>, i0: usize, i1: usize, i2: usize) -> bool {
    let v0 = vertices[i0];
    let v1 = vertices[i1];
    let v2 = vertices[i2];
    let v0v1 = (&v0[..]).near(&&v1[..]);
    let v0v2 = (&v0[..]).near(&&v2[..]);
    let v1v2 = (&v1[..]).near(&&v2[..]);
    v0v1 || v0v2 || v1v2
}

fn sub_remove_unused_attrs<'a, I: Iterator<Item = &'a mut [usize; 3]>>(
    iter: I,
    idx: usize,
    old_len: usize,
) -> Vec<usize> {
    let mut new2old = Vec::new();
    let mut old2new = vec![None; old_len];
    for arr in iter {
        arr[idx] = match old2new[arr[idx]] {
            Some(k) => k,
            None => {
                let k = new2old.len();
                new2old.push(arr[idx]);
                old2new[arr[idx]] = Some(k);
                k
            }
        }
    }
    new2old
}
