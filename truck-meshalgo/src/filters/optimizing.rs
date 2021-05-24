use super::*;
use std::collections::HashMap;
use std::iter::Iterator;
use std::ops::{Div, Mul};

/// Filters for optimizing data
pub trait OptimizingFilter {
    /// remove all unused position, texture coordinates, and normal vectors.
    /// # Examples
    /// ```
    /// use truck_polymesh::*;
    /// use truck_meshalgo::filters::*;
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
    /// use truck_polymesh::*;
    /// use truck_meshalgo::filters::*;
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
    /// use truck_polymesh::*;
    /// use truck_meshalgo::filters::*;
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

fn all_pos_mut(faces: &mut Faces) -> impl Iterator<Item = &mut usize> {
    faces.face_iter_mut().flatten().map(move |v| &mut v.pos)
}

fn all_uv_mut(faces: &mut Faces) -> impl Iterator<Item = &mut usize> {
    faces
        .face_iter_mut()
        .flatten()
        .filter_map(move |v| v.uv.as_mut())
}

fn all_nor_mut(faces: &mut Faces) -> impl Iterator<Item = &mut usize> {
    faces
        .face_iter_mut()
        .flatten()
        .filter_map(move |v| v.nor.as_mut())
}

impl OptimizingFilter for PolygonMesh {
    fn remove_unused_attrs(&mut self) -> &mut Self {
        let mesh = self.debug_editor();
        let pos_iter = all_pos_mut(mesh.faces);
        let idcs = sub_remove_unused_attrs(pos_iter, mesh.positions.len());
        *mesh.positions = idcs.iter().map(|i| mesh.positions[*i]).collect();
        let uv_iter = all_uv_mut(mesh.faces);
        let idcs = sub_remove_unused_attrs(uv_iter, mesh.uv_coords.len());
        *mesh.uv_coords = idcs.iter().map(|i| mesh.uv_coords[*i]).collect();
        let nor_iter = all_nor_mut(mesh.faces);
        let idcs = sub_remove_unused_attrs(nor_iter, mesh.normals.len());
        *mesh.normals = idcs.iter().map(|i| mesh.normals[*i]).collect();
        drop(mesh);
        self
    }

    fn remove_degenerate_faces(&mut self) -> &mut Self {
        let mesh = self.debug_editor();
        let mut faces = Faces::default();
        for tri in mesh.faces.tri_faces() {
            if !degenerate_triangle(*tri) {
                faces.push(tri);
            }
        }
        for quad in mesh.faces.quad_faces() {
            match degenerate_quadrangle(*quad) {
                QuadrangleType::TotallyDegenerate => {}
                QuadrangleType::Triangle(tri) => faces.push(tri),
                QuadrangleType::NonDegenerate => faces.push(quad),
            }
        }
        for face in mesh.faces.other_faces() {
            faces.extend(split_into_nondegenerate(face.clone()));
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
        all_pos_mut(mesh.faces).for_each(|idx| *idx = pos_map[*idx]);
        let uv_map = sub_put_together_same_attrs(&mesh.uv_coords);
        all_uv_mut(mesh.faces).for_each(|idx| *idx = uv_map[*idx]);
        let nor_map = sub_put_together_same_attrs(&mesh.normals);
        all_nor_mut(mesh.faces).for_each(|idx| *idx = nor_map[*idx]);
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

fn degenerate_triangle(tri: [Vertex; 3]) -> bool {
    tri[0].pos == tri[1].pos || tri[1].pos == tri[2].pos || tri[2].pos == tri[0].pos
}

enum QuadrangleType {
    NonDegenerate,
    Triangle([Vertex; 3]),
    TotallyDegenerate,
}

fn degenerate_quadrangle(quad: [Vertex; 4]) -> QuadrangleType {
    if quad[0].pos == quad[2].pos || quad[1].pos == quad[3].pos {
        QuadrangleType::TotallyDegenerate
    } else if quad[0].pos == quad[1].pos && quad[2].pos == quad[3].pos {
        QuadrangleType::TotallyDegenerate
    } else if quad[1].pos == quad[2].pos && quad[3].pos == quad[0].pos {
        QuadrangleType::TotallyDegenerate
    } else if quad[0].pos == quad[1].pos || quad[1].pos == quad[2].pos {
        QuadrangleType::Triangle([quad[0], quad[2], quad[3]])
    } else if quad[2].pos == quad[3].pos || quad[3].pos == quad[0].pos {
        QuadrangleType::Triangle([quad[0], quad[1], quad[2]])
    } else {
        QuadrangleType::NonDegenerate
    }
}

fn split_into_nondegenerate(poly: Vec<Vertex>) -> Vec<Vec<Vertex>> {
    for i in 0..poly.len() {
        for j in (i + 1)..poly.len() {
            if poly[i].pos == poly[j].pos {
                let polygon0: Vec<_> = (0..(j - i)).map(|k| poly[k + i]).collect();
                let polygon1: Vec<_> = ((j - i)..poly.len())
                    .map(|k| poly[(k + i) % poly.len()])
                    .collect();
                let mut result = split_into_nondegenerate(polygon0);
                result.extend(split_into_nondegenerate(polygon1));
                return result;
            }
        }
    }
    vec![poly]
}

trait CastIntVector: Sized + Mul<f64, Output = Self> + Div<f64, Output = Self> {
    type IntVector: std::hash::Hash + Eq;
    fn cast_int(&self) -> Self::IntVector;
}

macro_rules! impl_cast_int {
    ($typename: ident, $n: expr) => {
        impl CastIntVector for $typename {
            type IntVector = [i64; $n];
            fn cast_int(&self) -> [i64; $n] { self.cast::<i64>().unwrap().into() }
        }
    };
}
impl_cast_int!(Vector2, 2);
impl_cast_int!(Vector3, 3);
impl_cast_int!(Point3, 3);

#[cfg(test)]
mod tests {
    use super::*;

    fn into_vertices(iter: &[usize]) -> Vec<Vertex> { iter.iter().map(|i| i.into()).collect() }

    #[test]
    fn degenerate_polygon_test() {
        let poly = into_vertices(&[0, 1, 2, 0, 3, 4, 5, 6, 3, 7, 8, 9]);
        let polys = split_into_nondegenerate(poly);
        assert_eq!(polys[0], into_vertices(&[0, 1, 2]));
        assert_eq!(polys[1], into_vertices(&[3, 4, 5, 6]));
        assert_eq!(polys[2], into_vertices(&[3, 7, 8, 9, 0]));
    }
}
