use crate::*;
use std::collections::HashMap;
use std::iter::Iterator;
use std::ops::{Div, Mul};

/// Filters eliminating waste
pub trait WasteEliminatingFilter {
    /// remove all unused position, texture coordinates, and normal vectors.
    fn remove_unused_attrs(&mut self) -> &mut Self;
    /// remove degenerate polygons.
    fn remove_degenerate_faces(&mut self) -> &mut Self;
    /// Gives the same indices to the same positions, texture coordinate, and normal vectors, respectively.
    fn put_together_same_attrs(&mut self) -> &mut Self;
}

/// mesh healing algorithms
impl WasteEliminatingFilter for PolygonMesh {
    fn remove_unused_attrs(&mut self) -> &mut Self {
        let mesh = self.debug_editor();
        let pos_iter = mesh.faces.face_iter().flatten().map(|v| v.pos);
        let idcs = sub_remove_unused_attrs(pos_iter, mesh.positions.len());
        *mesh.positions = reindex(&mesh.positions, &idcs);
        let uv_iter = mesh.faces.face_iter().flatten().filter_map(|v| v.uv);
        let idcs = sub_remove_unused_attrs(uv_iter, mesh.uv_coords.len());
        *mesh.uv_coords = reindex(&mesh.uv_coords, &idcs);
        let nor_iter = mesh.faces.face_iter().flatten().filter_map(|v| v.nor);
        let idcs = sub_remove_unused_attrs(nor_iter, mesh.normals.len());
        *mesh.normals = reindex(&mesh.normals, &idcs);
        drop(mesh);
        self
    }

    fn remove_degenerate_faces(&mut self) -> &mut Self {
        let mesh = self.debug_editor();
        let positions = &mesh.positions;
        let mut faces = Faces::default();
        for face in mesh.faces.face_iter() {
            let mut new_face = Vec::new();
            new_face.push(face[0]);
            face.windows(2).for_each(|pair| {
                if positions[pair[0].pos].near(&positions[pair[1].pos]) {
                    new_face.push(pair[1]);
                }
            });
            faces.push(new_face);
        }
        *mesh.faces = faces;
        drop(mesh);
        self
    }

    fn put_together_same_attrs(&mut self) -> &mut Self {
        let mesh = self.debug_editor();
        let bnd_box: BoundingBox<_> = mesh.positions.iter().collect();
        let center = bnd_box.center();
        let diag = bnd_box.diagonal();
        let normalized_positions = mesh.positions.iter().map(move |position| {
            2.0 * (position - center).zip(diag, |a, b| a / b)
        }).collect::<Vec<_>>();
        let pos_map = sub_put_together_same_attrs(&normalized_positions);
        for idx in mesh.faces.face_iter_mut().flatten().map(|v| &mut v.pos) {
            *idx = pos_map[*idx];
        }
        let uv_map = sub_put_together_same_attrs(&mesh.uv_coords);
        for idx in mesh.faces.face_iter_mut().flatten().filter_map(|v| v.uv.as_mut()) {
            *idx = uv_map[*idx];
        }
        let nor_map = sub_put_together_same_attrs(&mesh.normals);
        for idx in mesh.faces.face_iter_mut().flatten().filter_map(|v| v.nor.as_mut()) {
            *idx = nor_map[*idx];
        }
        drop(mesh);
        self
    }
}

fn sub_remove_unused_attrs<I: Iterator<Item=usize>>(iter: I, old_len: usize) -> Vec<usize> {
    let mut new2old = Vec::new();
    let mut old2new = vec![None; old_len];
    for idx in iter {
        if old2new[idx].is_none() {
            let k = new2old.len();
            new2old.push(idx);
            old2new[idx] = Some(k);
        }
    }
    new2old
}

fn sub_put_together_same_attrs<T: Copy + CastIntVector>(attrs: &[T]) -> Vec<usize> {
    let mut res = Vec::new();
    let mut map = HashMap::new();
    for (i, attr) in attrs.iter().enumerate() {
        let v = (*attr / TOLERANCE).cast_int();
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

fn reindex<T: Clone>(vec: &Vec<T>, idcs: &Vec<usize>) -> Vec<T> {
    idcs.iter().map(|i| vec[*i].clone()).collect()
}

trait CastIntVector: Sized + Mul<f64, Output = Self> + Div<f64, Output = Self> {
    type IntVector: std::hash::Hash + Eq;
    fn cast_int(&self) -> Self::IntVector;
}

mod impl_cast_int {
    use cgmath::{Point3, Vector2, Vector3};
    macro_rules! impl_cast_int {
        ($typename: ident) => {
            impl super::CastIntVector for $typename<f64> {
                type IntVector = $typename<i64>;
                fn cast_int(&self) -> $typename<i64> { self.cast::<i64>().unwrap() }
            }
        };
    }
    impl_cast_int!(Vector2);
    impl_cast_int!(Vector3);
    impl_cast_int!(Point3);
}
