use crate::*;
use std::collections::HashMap;
use std::iter::Iterator;
use std::ops::{Div, Mul};

/// Filters for optimizing data
pub trait OptimizingFilter {
    /// remove all unused position, texture coordinates, and normal vectors.
    /// # Examples
    /// ```
    /// use truck_polymesh::prelude::*;
    /// let positions = vec![
    ///     Point3::new(0.0, 0.0, 0.0),
    ///     Point3::new(1.0, 0.0, 0.0),
    ///     Point3::new(0.0, 1.0, 0.0),
    ///     Point3::new(0.0, 0.0, 1.0),
    /// ];
    /// let faces = Faces::from_iter(&[&[1, 2, 3]]); // 0 is not used!
    /// let mut mesh = PolygonMesh::new(positions, Vec::new(), Vec::new(), faces);
    ///
    /// assert_eq!(mesh.positions().len(), 4);
    /// mesh.remove_unused_attrs();
    /// assert_eq!(mesh.positions().len(), 3);
    /// ```
    fn remove_unused_attrs(&mut self) -> &mut Self;
    /// Removes degenerate polygons.
    /// # Examples
    /// ```
    /// use truck_polymesh::prelude::*;
    /// let positions = vec![
    ///     Point3::new(0.0, 0.0, 0.0),
    ///     Point3::new(1.0, 0.0, 0.0),
    ///     Point3::new(0.0, 1.0, 0.0),
    ///     Point3::new(0.0, 0.0, 1.0),
    /// ];
    /// let faces = Faces::from_iter(&[
    ///     &[0, 1, 2],
    ///     &[2, 1, 2], // degenerate face!
    ///     &[2, 1, 3],
    /// ]);
    /// let mut mesh = PolygonMesh::new(positions, Vec::new(), Vec::new(), faces);
    /// 
    /// assert_eq!(mesh.faces().len(), 3);
    /// mesh.remove_degenerate_faces();
    /// assert_eq!(mesh.faces().len(), 2);
    /// ```
    fn remove_degenerate_faces(&mut self) -> &mut Self;
    /// Gives the same indices to the same positions, texture coordinate, and normal vectors, respectively.
    /// # Remarks
    /// No longer needed attributes are NOT autoremoved.
    /// One can remove such attributes by running [`remove_unused_attrs`] mannually.
    /// 
    /// [`remove_unused_attrs`]: ./trait.WasteEliminatingFilter.html#tymethod.remove_unused_attrs
    /// 
    /// # Examples
    /// ```
    /// use truck_polymesh::prelude::*;
    /// let positions = vec![
    ///     Point3::new(0.0, 0.0, 0.0),
    ///     Point3::new(1.0, 0.0, 0.0),
    ///     Point3::new(0.0, 1.0, 0.0),
    ///     Point3::new(1.0, 1.0, 0.0),
    ///     Point3::new(0.0, 1.0, 0.0),
    ///     Point3::new(1.0, 0.0, 0.0),
    /// ];
    /// let faces = Faces::from_iter(&[
    ///     &[0, 1, 2],
    ///     &[3, 4, 5],
    /// ]);
    /// let mut mesh = PolygonMesh::new(positions, Vec::new(), Vec::new(), faces);
    /// 
    /// assert_eq!(mesh.faces()[1][1], Vertex { pos: 4, uv: None, nor: None });
    /// mesh.put_together_same_attrs();
    /// assert_eq!(mesh.faces()[1][1], Vertex { pos: 2, uv: None, nor: None });
    /// 
    /// // Remarks: No longer needed attributes are NOT autoremoved!
    /// assert_eq!(mesh.positions().len(), 6);
    /// mesh.remove_unused_attrs();
    /// assert_eq!(mesh.positions().len(), 4);
    /// ```
    fn put_together_same_attrs(&mut self) -> &mut Self;
}

impl Faces {
    fn all_pos_mut(&mut self) -> impl Iterator<Item = &mut usize> {
        self.face_iter_mut().flatten().map(move |v| &mut v.pos)
    }

    fn all_uv_mut(&mut self) -> impl Iterator<Item = &mut usize> {
        self.face_iter_mut()
            .flatten()
            .filter_map(move |v| v.uv.as_mut())
    }

    fn all_nor_mut(&mut self) -> impl Iterator<Item = &mut usize> {
        self.face_iter_mut()
            .flatten()
            .filter_map(move |v| v.nor.as_mut())
    }
}

impl OptimizingFilter for PolygonMesh {
    fn remove_unused_attrs(&mut self) -> &mut Self {
        let mesh = self.debug_editor();
        let pos_iter = mesh.faces.all_pos_mut();
        let idcs = sub_remove_unused_attrs(pos_iter, mesh.positions.len());
        *mesh.positions = idcs.iter().map(|i| mesh.positions[*i]).collect();
        let uv_iter = mesh.faces.all_uv_mut();
        let idcs = sub_remove_unused_attrs(uv_iter, mesh.uv_coords.len());
        *mesh.uv_coords = idcs.iter().map(|i| mesh.uv_coords[*i]).collect();
        let nor_iter = mesh.faces.all_nor_mut();
        let idcs = sub_remove_unused_attrs(nor_iter, mesh.normals.len());
        *mesh.normals = idcs.iter().map(|i| mesh.normals[*i]).collect();
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
                if !positions[pair[0].pos].near(&positions[pair[1].pos]) {
                    new_face.push(pair[1]);
                }
            });
            if positions[new_face.last().unwrap().pos].near(&positions[new_face[0].pos]) {
                new_face.pop();
            }
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
        let diag = bnd_box.diagonal().map(|a| f64::max(a.abs(), 1.0));
        let normalized_positions = mesh
            .positions
            .iter()
            .map(move |position| 2.0 * (position - center).zip(diag, |a, b| a / b))
            .collect::<Vec<_>>();
        let pos_map = sub_put_together_same_attrs(&normalized_positions);
        mesh.faces
            .all_pos_mut()
            .for_each(|idx| *idx = pos_map[*idx]);
        let uv_map = sub_put_together_same_attrs(&mesh.uv_coords);
        mesh.faces.all_uv_mut().for_each(|idx| *idx = uv_map[*idx]);
        let nor_map = sub_put_together_same_attrs(&mesh.normals);
        mesh.faces
            .all_nor_mut()
            .for_each(|idx| *idx = nor_map[*idx]);
        drop(mesh);
        self
    }
}

fn sub_remove_unused_attrs<'a, I: Iterator<Item = &'a mut usize>>(
    iter: I,
    old_len: usize,
) -> Vec<usize> {
    let mut new2old = Vec::new();
    let mut old2new = vec![None; old_len];
    for idx in iter {
        *idx = match old2new[*idx] {
            Some(k) => k,
            None => {
                let k = new2old.len();
                new2old.push(*idx);
                old2new[*idx] = Some(k);
                k
            }
        };
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
