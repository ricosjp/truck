use crate::{get_tri, MeshHandler};
use geometry::{Tolerance, Vector3};
use std::collections::HashMap;
use std::iter::Iterator;

impl MeshHandler {
    pub fn remove_unused_attrs(&mut self) -> &mut Self {
        let mesh = &mut self.mesh;
        if !mesh.positions.is_empty() {
            let idcs = mesh.sub_remove_unused_attrs(0, mesh.positions.len());
            mesh.positions = reindex(&mesh.positions, &idcs);
        }
        if !mesh.uv_coords.is_empty() {
            let idcs = mesh.sub_remove_unused_attrs(1, mesh.uv_coords.len());
            mesh.uv_coords = reindex(&mesh.uv_coords, &idcs);
        }
        if !mesh.normals.is_empty() {
            let idcs = mesh.sub_remove_unused_attrs(2, mesh.normals.len());
            mesh.normals = reindex(&mesh.normals, &idcs);
        }
        self
    }

    pub fn remove_degenerate_faces(&mut self) -> &mut Self {
        let mesh = &mut self.mesh;
        let positions = &mesh.positions;

        let mut new_tri_faces = Vec::new();
        let mut new_quad_faces = Vec::new();
        for face in &mesh.quad_faces {
            let is_deg0 = is_degenerate_tri(positions, face[0][0], face[1][0], face[3][0]);
            let is_deg1 = is_degenerate_tri(positions, face[2][0], face[3][0], face[1][0]);
            match (is_deg0, is_deg1) {
                (true, true) => {}
                (true, false) => new_tri_faces.push(get_tri(face, 2, 3, 1)),
                (false, true) => new_tri_faces.push(get_tri(face, 0, 1, 3)),
                (false, false) => new_quad_faces.push(face.clone()),
            }
        }
        for face in &mesh.tri_faces {
            if !is_degenerate_tri(positions, face[0][0], face[1][0], face[2][0]) {
                new_tri_faces.push(face.clone());
            }
        }
        mesh.tri_faces = new_tri_faces;
        mesh.quad_faces = new_quad_faces;

        self
    }

    pub fn put_together_same_attrs(&mut self) -> &mut Self {
        let mesh = &mut self.mesh;
        let vert_map = sub_put_together_same_attrs(&mesh.positions);
        reflect_matching(&mut mesh.tri_faces, 0, &vert_map);
        reflect_matching(&mut mesh.quad_faces, 0, &vert_map);
        reflect_matching(&mut mesh.other_faces, 0, &vert_map);
        if !mesh.uv_coords.is_empty() {
            let uv_map = sub_put_together_same_attrs(&mesh.uv_coords);
            reflect_matching(&mut mesh.tri_faces, 1, &uv_map);
            reflect_matching(&mut mesh.quad_faces, 1, &uv_map);
            reflect_matching(&mut mesh.other_faces, 1, &uv_map);
        }
        if !mesh.normals.is_empty() {
            let norm_map = sub_put_together_same_attrs(&mesh.normals);
            reflect_matching(&mut mesh.tri_faces, 2, &norm_map);
            reflect_matching(&mut mesh.quad_faces, 2, &norm_map);
            reflect_matching(&mut mesh.other_faces, 2, &norm_map);
        }
        self
    }
}

impl crate::PolygonMesh {
    fn sub_remove_unused_attrs(&mut self, idx: usize, old_len: usize) -> Vec<usize> {
        let vec0 = self.tri_faces.iter_mut().flat_map(|arr| arr.iter_mut());
        let vec1 = self.quad_faces.iter_mut().flat_map(|arr| arr.iter_mut());
        let vec2 = self.other_faces.iter_mut().flat_map(|arr| arr.iter_mut());
        sub_remove_unused_attrs(vec0.chain(vec1).chain(vec2), idx, old_len)
    }
}

fn is_degenerate_tri(positions: &Vec<Vector3>, i0: usize, i1: usize, i2: usize) -> bool {
    positions[i0].near(&positions[i1])
        || positions[i0].near(&positions[i2])
        || positions[i1].near(&positions[i2])
}

fn sub_remove_unused_attrs<'a, I: Iterator<Item = &'a mut [usize; 3]>>(
    iter: I,
    idx: usize,
    old_len: usize,
) -> Vec<usize>
{
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

fn sub_put_together_same_attrs<T: AsRef<[f64]>>(attrs: &[T]) -> Vec<usize> {
    let mut res = Vec::new();
    let mut map = HashMap::new();
    for (i, attr) in attrs.iter().enumerate() {
        let v: Vec<_> = attr
            .as_ref()
            .iter()
            .map(|x| (x / f64::TOLERANCE) as i64)
            .collect();
        match map.get(&v) {
            Some(j) => res.push(*j),
            None => {
                map.insert(v, i);
                res.push(i);
            }
        }
    }
    res
}

fn reflect_matching<T: AsMut<[[usize; 3]]>>(faces: &mut [T], i: usize, map: &Vec<usize>) {
    for face in faces.iter_mut() {
        for vert in face.as_mut() {
            vert[i] = map[vert[i]];
        }
    }
}

fn reindex<T: Clone>(vec: &Vec<T>, idcs: &Vec<usize>) -> Vec<T> {
    idcs.iter().map(|i| vec[*i].clone()).collect()
}
