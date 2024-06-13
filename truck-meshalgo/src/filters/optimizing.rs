use super::*;
use rustc_hash::FxHashMap as HashMap;
use std::iter::Iterator;
use std::ops::{Div, Mul};

/// Filters for optimizing data
pub trait OptimizingFilter {
    /// remove all unused position, texture coordinates, and normal vectors.
    /// # Examples
    /// ```
    /// use truck_polymesh::*;
    /// use truck_meshalgo::filters::*;
    /// let mut mesh = PolygonMesh::new(
    ///     StandardAttributes {
    ///         positions: vec![
    ///             Point3::new(0.0, 0.0, 0.0),
    ///             Point3::new(1.0, 0.0, 0.0),
    ///             Point3::new(0.0, 1.0, 0.0),
    ///             Point3::new(0.0, 0.0, 1.0),
    ///         ],
    ///         ..Default::default()
    ///     },
    ///     // 0 is not used!
    ///     Faces::from_iter(&[&[1, 2, 3]]),
    /// );
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
    /// let mut mesh = PolygonMesh::new(
    ///     StandardAttributes {
    ///         positions: vec![
    ///             Point3::new(0.0, 0.0, 0.0),
    ///             Point3::new(1.0, 0.0, 0.0),
    ///             Point3::new(0.0, 1.0, 0.0),
    ///             Point3::new(0.0, 0.0, 1.0),
    ///         ],
    ///         ..Default::default()
    ///     },
    ///     Faces::from_iter(&[
    ///         &[0, 1, 2],
    ///         &[2, 1, 2], // degenerate face!
    ///         &[2, 1, 3],
    ///     ]),
    /// );
    ///
    /// assert_eq!(mesh.faces().len(), 3);
    /// mesh.remove_degenerate_faces();
    /// assert_eq!(mesh.faces().len(), 2);
    /// ```
    fn remove_degenerate_faces(&mut self) -> &mut Self;
    /// Gives the same indices to the same positions, texture coordinate, and normal vectors, respectively.
    /// # Remarks
    /// No longer needed attributes are NOT autoremoved.
    /// One can remove such attributes by running [`remove_unused_attrs`] manually.
    ///
    /// [`remove_unused_attrs`]: ./trait.WasteEliminatingFilter.html#tymethod.remove_unused_attrs
    ///
    /// # Examples
    /// ```
    /// use truck_meshalgo::prelude::*;
    /// let mut mesh = PolygonMesh::new(
    ///     StandardAttributes {
    ///         positions: vec![
    ///             Point3::new(0.0, 0.0, 0.0),
    ///             Point3::new(1.0, 0.0, 0.0),
    ///             Point3::new(0.0, 1.0, 0.0),
    ///             Point3::new(1.0, 1.0, 0.0),
    ///             Point3::new(0.0, 1.0, 0.0),
    ///             Point3::new(1.0, 0.0, 0.0),
    ///         ],
    ///         ..Default::default()
    ///     },
    ///     Faces::from_iter(&[
    ///         &[0, 1, 2],
    ///         &[3, 4, 5],
    ///     ]),
    /// );
    ///
    /// assert_eq!(mesh.faces()[1][1], StandardVertex { pos: 4, uv: None, nor: None });
    /// mesh.put_together_same_attrs(TOLERANCE);
    /// assert_eq!(mesh.faces()[1][1], StandardVertex { pos: 2, uv: None, nor: None });
    ///
    /// // Remarks: No longer needed attributes are NOT autoremoved!
    /// assert_eq!(mesh.positions().len(), 6);
    /// mesh.remove_unused_attrs();
    /// assert_eq!(mesh.positions().len(), 4);
    /// ```
    fn put_together_same_attrs(&mut self, tol: f64) -> &mut Self;
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
        let mut mesh = self.debug_editor();
        let PolygonMeshEditor {
            attributes:
                StandardAttributes {
                    positions,
                    uv_coords,
                    normals,
                },
            faces,
            ..
        } = &mut mesh;
        let pos_iter = all_pos_mut(faces);
        let idcs = sub_remove_unused_attrs(pos_iter, positions.len());
        *positions = idcs.iter().map(|i| positions[*i]).collect();
        let uv_iter = all_uv_mut(faces);
        let idcs = sub_remove_unused_attrs(uv_iter, uv_coords.len());
        *uv_coords = idcs.iter().map(|i| uv_coords[*i]).collect();
        let nor_iter = all_nor_mut(faces);
        let idcs = sub_remove_unused_attrs(nor_iter, normals.len());
        *normals = idcs.iter().map(|i| normals[*i]).collect();
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

    fn put_together_same_attrs(&mut self, tol: f64) -> &mut Self {
        let mut mesh = self.debug_editor();
        let PolygonMeshEditor {
            attributes:
                StandardAttributes {
                    positions,
                    uv_coords,
                    normals,
                },
            faces,
            ..
        } = &mut mesh;
        let bnd_box: BoundingBox<_> = positions.iter().collect();
        let center = bnd_box.center();
        let diag = bnd_box.diagonal().map(|a| f64::max(a.abs(), 1.0));
        let normalized_positions = positions
            .iter()
            .map(move |position| 2.0 * (position - center).zip(diag, |a, b| a / b))
            .collect::<Vec<_>>();
        let pos_map = sub_put_together_same_attrs(&normalized_positions, tol);
        all_pos_mut(faces).for_each(|idx| *idx = pos_map[*idx]);
        let uv_map = sub_put_together_same_attrs(uv_coords, tol);
        all_uv_mut(faces).for_each(|idx| *idx = uv_map[*idx]);
        let nor_map = sub_put_together_same_attrs(normals, tol);
        all_nor_mut(faces).for_each(|idx| *idx = nor_map[*idx]);
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

trait SameAttr: Copy + CastIntVector + MetricSpace<Metric = f64> {}
impl<T> SameAttr for T where T: Copy + CastIntVector + MetricSpace<Metric = f64> {}

fn sub_put_together_same_attrs<T: SameAttr>(attrs: &[T], tol: f64) -> Vec<usize> {
    let map = create_blocks(attrs, tol);
    let adj = create_adjacency(map, attrs, tol);
    let components = create_components(adj, attrs);
    centralize(components, attrs)
}

type BlockMap<'a, T> = HashMap<<T as CastIntVector>::IntVector, Vec<usize>>;
fn create_blocks<T: SameAttr>(attrs: &[T], tol: f64) -> BlockMap<'_, T> {
    let mut map = HashMap::default();
    attrs.iter().enumerate().for_each(|(i, attr)| {
        attr.round(tol).neighborhood().into_iter().for_each(|idx| {
            map.entry(idx).or_insert_with(Vec::new).push(i);
        });
    });
    map
}

fn create_adjacency<T: SameAttr>(map: BlockMap<'_, T>, attrs: &[T], tol: f64) -> Vec<Vec<usize>> {
    let mut adj = vec![Vec::new(); attrs.len()];
    map.into_iter().for_each(|(_, vec)| {
        if vec.len() > 1 {
            vec.iter().copied().enumerate().for_each(|(k, i)| {
                vec[(k + 1)..].iter().copied().for_each(|j| {
                    if attrs[i].distance2(attrs[j]) < tol * tol {
                        adj[i].push(j);
                        adj[j].push(i);
                    }
                });
            });
        }
    });
    adj
}

fn create_components<T: SameAttr>(
    mut adj: Vec<Vec<usize>>,
    attrs: &[T],
) -> impl Iterator<Item = Vec<usize>> {
    let mut already = vec![false; attrs.len()];
    (0..adj.len()).filter_map(move |i| {
        if already[i] {
            return None;
        }
        let mut stack = vec![i];
        let mut component = Vec::new();
        while let Some(i) = stack.pop() {
            if !already[i] {
                component.push(i);
                already[i] = true;
                stack.append(&mut adj[i]);
            }
        }
        Some(component)
    })
}

fn centralize<T: SameAttr>(
    components: impl Iterator<Item = Vec<usize>>,
    attrs: &[T],
) -> Vec<usize> {
    let mut res = vec![0; attrs.len()];
    components.for_each(|component| {
        let center = component
            .iter()
            .fold(T::zero(), |sum, j| sum.add_element_wise(attrs[*j]))
            / component.len() as f64;
        let (mut mindist, mut mindx) = (f64::INFINITY, 0);
        component.iter().copied().for_each(|i| {
            let dist = attrs[i].distance2(center);
            if dist < mindist {
                mindist = dist;
                mindx = i;
            }
        });
        component.into_iter().for_each(|i| {
            res[i] = mindx;
        });
    });
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
    if (quad[0].pos == quad[2].pos || quad[1].pos == quad[3].pos)
        || (quad[0].pos == quad[1].pos && quad[2].pos == quad[3].pos)
        || quad[1].pos == quad[2].pos && quad[3].pos == quad[0].pos
    {
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

trait CastIntVector:
    Sized + ElementWise<Self> + Mul<f64, Output = Self> + Div<f64, Output = Self> {
    type IntVector: IntVector;
    fn cast_int(self) -> Self::IntVector;
    fn round(self, tol: f64) -> Self::IntVector;
    fn zero() -> Self;
}

macro_rules! impl_cast_int {
    ($typename: ident, $n: expr) => {
        impl CastIntVector for $typename {
            type IntVector = [i64; $n];
            fn cast_int(self) -> [i64; $n] {
                self.cast::<i64>()
                    .unwrap_or_else(|| panic!("failed to cast: {self:?}"))
                    .into()
            }
            fn round(self, tol: f64) -> Self::IntVector { (self / tol).cast_int() }
            fn zero() -> Self { Self::from([0.0; $n]) }
        }
    };
}
impl_cast_int!(Vector2, 2);
impl_cast_int!(Vector3, 3);
impl_cast_int!(Point3, 3);

trait IntVector: Copy + std::hash::Hash + Eq {
    type Neighborhood: IntoIterator<Item = Self>;
    fn neighborhood(self) -> Self::Neighborhood;
}

impl IntVector for [i64; 2] {
    type Neighborhood = [[i64; 2]; 9];
    fn neighborhood(self) -> Self::Neighborhood {
        let [a, b] = self;
        [
            [a - 1, b - 1],
            [a, b - 1],
            [a + 1, b - 1],
            [a - 1, b],
            [a, b],
            [a + 1, b],
            [a - 1, b + 1],
            [a, b + 1],
            [a + 1, b + 1],
        ]
    }
}

impl IntVector for [i64; 3] {
    type Neighborhood = [[i64; 3]; 27];
    fn neighborhood(self) -> Self::Neighborhood {
        let [a, b, c] = self;
        [
            [a - 1, b - 1, c - 1],
            [a, b - 1, c - 1],
            [a + 1, b - 1, c - 1],
            [a - 1, b, c - 1],
            [a, b, c - 1],
            [a + 1, b, c - 1],
            [a - 1, b + 1, c - 1],
            [a, b + 1, c - 1],
            [a + 1, b + 1, c - 1],
            [a - 1, b - 1, c],
            [a, b - 1, c],
            [a + 1, b - 1, c],
            [a - 1, b, c],
            [a, b, c],
            [a + 1, b, c],
            [a - 1, b + 1, c],
            [a, b + 1, c],
            [a + 1, b + 1, c],
            [a - 1, b - 1, c + 1],
            [a, b - 1, c + 1],
            [a + 1, b - 1, c + 1],
            [a - 1, b, c + 1],
            [a, b, c + 1],
            [a + 1, b, c + 1],
            [a - 1, b + 1, c + 1],
            [a, b + 1, c + 1],
            [a + 1, b + 1, c + 1],
        ]
    }
}
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
