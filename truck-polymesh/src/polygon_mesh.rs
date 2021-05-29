use crate::*;
use errors::Error;
use std::iter::FromIterator;

impl From<(usize, Option<usize>, Option<usize>)> for Vertex {
    fn from(tuple: (usize, Option<usize>, Option<usize>)) -> Vertex {
        Vertex {
            pos: tuple.0,
            uv: tuple.1,
            nor: tuple.2,
        }
    }
}

impl From<&(usize, Option<usize>, Option<usize>)> for Vertex {
    fn from(tuple: &(usize, Option<usize>, Option<usize>)) -> Vertex {
        Vertex {
            pos: tuple.0,
            uv: tuple.1,
            nor: tuple.2,
        }
    }
}

impl From<[usize; 3]> for Vertex {
    fn from(arr: [usize; 3]) -> Vertex {
        Vertex {
            pos: arr[0],
            uv: Some(arr[1]),
            nor: Some(arr[2]),
        }
    }
}

impl From<&[usize; 3]> for Vertex {
    fn from(arr: &[usize; 3]) -> Vertex {
        Vertex {
            pos: arr[0],
            uv: Some(arr[1]),
            nor: Some(arr[2]),
        }
    }
}

impl From<usize> for Vertex {
    fn from(idx: usize) -> Vertex {
        Vertex {
            pos: idx,
            uv: None,
            nor: None,
        }
    }
}

impl From<&usize> for Vertex {
    fn from(idx: &usize) -> Vertex {
        Vertex {
            pos: *idx,
            uv: None,
            nor: None,
        }
    }
}

impl Faces {
    /// Creates faces of a polygon by iterator of slice.
    ///
    /// If `face.len() < 3`, the face is ignored.
    /// # Examples
    /// ```
    /// use truck_polymesh::*;
    /// let slice: &[&[[usize; 3]]] = &[
    ///     &[[0, 0, 0], [1, 1, 1], [2, 2, 2]],
    ///     &[[0, 0, 0], [2, 2, 2], [3, 3, 3]],
    ///     &[[0, 0, 0], [4, 4, 4], [5, 5, 5], [1, 1, 1]],
    /// ];
    /// let faces = Faces::from_iter(slice);
    /// ```
    #[inline(always)]
    pub fn from_iter<V: Copy + Into<Vertex>, T: AsRef<[V]>, I: IntoIterator<Item = T>>(
        iter: I,
    ) -> Faces {
        let mut faces = Faces::default();
        faces.extend(iter);
        faces
    }

    /// Extends faces by an iterator.
    #[inline(always)]
    pub fn extend<V: Copy + Into<Vertex>, T: AsRef<[V]>, I: IntoIterator<Item = T>>(
        &mut self,
        iter: I,
    ) {
        iter.into_iter().for_each(|face| self.push(face))
    }

    /// Creates faces of a polygon mesh by the vectors of triangle and quadrangle.
    /// # Examples
    /// ```
    /// // Creates faces consisis only triangles.
    /// use truck_polymesh::*;
    /// let tri_faces: Vec<[Vertex; 3]> = vec![
    ///     [[0, 0, 0].into(), [1, 1, 1].into(), [2, 2, 2].into()],
    ///     [[0, 0, 0].into(), [2, 2, 2].into(), [3, 3, 3].into()],
    /// ];
    /// let faces = Faces::from_tri_and_quad_faces(tri_faces, Vec::new());
    /// ```
    #[inline(always)]
    pub fn from_tri_and_quad_faces(
        tri_faces: Vec<[Vertex; 3]>,
        quad_faces: Vec<[Vertex; 4]>,
    ) -> Faces {
        Faces {
            tri_faces,
            quad_faces,
            other_faces: Vec::new(),
        }
    }

    /// Push a face to the faces.
    ///
    /// If `face.len() < 3`, the face is ignored with warning.
    /// # Examples
    /// ```
    /// use truck_polymesh::*;
    /// let mut faces = Faces::default(); // empty faces
    /// faces.push(&[[0, 0, 0], [1, 1, 1], [2, 2, 2]]);
    /// faces.push(&[[3, 3, 3], [0, 0, 0], [2, 2, 2]]);
    /// faces.push(&[[0, 0, 0], [4, 4, 4], [5, 5, 5], [1, 1, 1]]);
    /// faces.push(&[[100, 1000, 10]]); // Wargning: ignored one vertex "face"
    /// ```
    #[inline(always)]
    pub fn push<V: Copy + Into<Vertex>, T: AsRef<[V]>>(&mut self, face: T) {
        let face = face.as_ref();
        match face.len() {
            0 => {}
            1 => {}
            2 => {}
            3 => self
                .tri_faces
                .push([face[0].into(), face[1].into(), face[2].into()]),
            4 => self.quad_faces.push([
                face[0].into(),
                face[1].into(),
                face[2].into(),
                face[3].into(),
            ]),
            _ => self
                .other_faces
                .push(Vec::from_iter(face.iter().map(|v| (*v).into()))),
        }
    }

    /// Returns the vector of triangles.
    #[inline(always)]
    pub fn tri_faces(&self) -> &Vec<[Vertex; 3]> { &self.tri_faces }

    /// Returns the mutable slice of triangles.
    #[inline(always)]
    pub fn tri_faces_mut(&mut self) -> &mut [[Vertex; 3]] { &mut self.tri_faces }

    /// Returns the vector of quadrangles.
    #[inline(always)]
    pub fn quad_faces(&self) -> &Vec<[Vertex; 4]> { &self.quad_faces }

    /// Returns the mutable slice of quadrangles.
    #[inline(always)]
    pub fn quad_faces_mut(&mut self) -> &mut [[Vertex; 4]] { &mut self.quad_faces }

    /// Returns the vector of n-gons (n > 4).
    #[inline(always)]
    pub fn other_faces(&self) -> &Vec<Vec<Vertex>> { &self.other_faces }

    /// Returns the mutable iterator of n-gons (n > 4).
    #[inline(always)]
    pub fn other_faces_mut(&mut self) -> impl Iterator<Item = &mut [Vertex]> {
        self.other_faces.iter_mut().map(|face| face.as_mut())
    }

    /// Returns the iterator of the slice.
    ///
    /// By the internal optimization, this iterator does not runs in the simple order
    /// in which they are registered, but runs order: triangle, square, and the others.
    /// # Examples
    /// ```
    /// use truck_polymesh::*;
    /// let slice: &[&[usize]] = &[
    ///     &[0, 1, 2],
    ///     &[0, 4, 5, 1],
    ///     &[1, 2, 6, 7, 8, 9],
    ///     &[0, 2, 3],
    /// ];
    /// let faces = Faces::from_iter(slice);
    /// let mut iter = faces.face_iter();
    /// assert_eq!(iter.next(), Some([
    ///     Vertex { pos: 0, uv: None, nor: None },
    ///     Vertex { pos: 1, uv: None, nor: None },
    ///     Vertex { pos: 2, uv: None, nor: None },
    /// ].as_ref()));
    /// assert_eq!(iter.next(), Some([
    ///     Vertex { pos: 0, uv: None, nor: None },
    ///     Vertex { pos: 2, uv: None, nor: None },
    ///     Vertex { pos: 3, uv: None, nor: None },
    /// ].as_ref()));
    /// assert_eq!(iter.next(), Some([
    ///     Vertex { pos: 0, uv: None, nor: None },
    ///     Vertex { pos: 4, uv: None, nor: None },
    ///     Vertex { pos: 5, uv: None, nor: None },
    ///     Vertex { pos: 1, uv: None, nor: None },
    /// ].as_ref()));
    /// assert_eq!(iter.next(), Some([
    ///     Vertex { pos: 1, uv: None, nor: None },
    ///     Vertex { pos: 2, uv: None, nor: None },
    ///     Vertex { pos: 6, uv: None, nor: None },
    ///     Vertex { pos: 7, uv: None, nor: None },
    ///     Vertex { pos: 8, uv: None, nor: None },
    ///     Vertex { pos: 9, uv: None, nor: None },
    /// ].as_ref()));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline(always)]
    pub fn face_iter<'a>(&'a self) -> impl Iterator<Item = &'a [Vertex]> {
        self.tri_faces
            .iter()
            .map(|v| v.as_ref())
            .chain(self.quad_faces.iter().map(|v| v.as_ref()))
            .chain(self.other_faces.iter().map(|v| v.as_ref()))
    }

    /// Returns the iterator of the slice.
    ///
    /// By the internal optimization, this iterator does not runs in the simple order
    /// in which they are registered, but runs order: triangle, square, and the others.
    /// cf: [`Faces:face_iter`](./struct.Faces.html#method.face_iter)
    #[inline(always)]
    pub fn face_iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut [Vertex]> {
        self.tri_faces
            .iter_mut()
            .map(|v| v.as_mut())
            .chain(self.quad_faces.iter_mut().map(|v| v.as_mut()))
            .chain(self.other_faces.iter_mut().map(|v| v.as_mut()))
    }

    /// Returns the number of faces.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.tri_faces.len() + self.quad_faces.len() + self.other_faces.len()
    }

    /// Merges `other` into `self`.
    #[inline(always)]
    pub fn naive_concat(&mut self, other: Self) {
        self.tri_faces.extend(other.tri_faces);
        self.quad_faces.extend(other.quad_faces);
        self.other_faces.extend(other.other_faces);
    }

    #[inline(always)]
    fn is_compatible(&self, n_pos: usize, n_uv: usize, n_nor: usize) -> Result<()> {
        self.face_iter().flatten().try_for_each(|v| {
            if v.pos >= n_pos {
                Err(Error::OutOfRange("positions", n_pos, v.pos))
            } else if v.uv.map(|uv| uv >= n_uv).unwrap_or(false) {
                Err(Error::OutOfRange("uv_coords", n_uv, v.uv.unwrap()))
            } else if v.nor.map(|nor| nor >= n_nor).unwrap_or(false) {
                Err(Error::OutOfRange("normals", n_nor, v.nor.unwrap()))
            } else {
                Ok(())
            }
        })
    }
}

impl std::ops::Index<usize> for Faces {
    type Output = [Vertex];
    fn index(&self, idx: usize) -> &Self::Output {
        if idx < self.tri_faces.len() {
            &self.tri_faces[idx]
        } else if idx < self.tri_faces.len() + self.quad_faces.len() {
            &self.quad_faces[idx - self.tri_faces.len()]
        } else {
            &self.other_faces[idx - self.tri_faces.len() - self.quad_faces.len()]
        }
    }
}

impl std::ops::IndexMut<usize> for Faces {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        if idx < self.tri_faces.len() {
            &mut self.tri_faces[idx]
        } else if idx < self.tri_faces.len() + self.quad_faces.len() {
            &mut self.quad_faces[idx - self.tri_faces.len()]
        } else {
            &mut self.other_faces[idx - self.tri_faces.len() - self.quad_faces.len()]
        }
    }
}

impl PolygonMesh {
    /// complete constructor
    /// # Panics
    /// Panic occurs if there is an index is out of range.
    /// # Remarks
    /// This method does not check whether the normal is normalized or not.
    pub fn new(
        positions: Vec<Point3>,
        uv_coords: Vec<Vector2>,
        normals: Vec<Vector3>,
        faces: Faces,
    ) -> PolygonMesh {
        PolygonMesh::try_new(positions, uv_coords, normals, faces)
            .unwrap_or_else(|e| panic!("{:?}", e))
    }

    /// complete constructor
    /// # Errors
    /// Returns [`Error::OutOfRange`] if there is an index is out of range.
    ///
    /// [`Error::OutOfRange`]: ./errors/enum.Error.html#variant.OutOfRange
    ///
    /// # Remarks
    /// This method does not check whether the normal is normalized or not.
    pub fn try_new(
        positions: Vec<Point3>,
        uv_coords: Vec<Vector2>,
        normals: Vec<Vector3>,
        faces: Faces,
    ) -> Result<PolygonMesh> {
        faces
            .is_compatible(positions.len(), uv_coords.len(), normals.len())
            .map(|_| PolygonMesh::new_unchecked(positions, uv_coords, normals, faces))
    }

    /// constructor without boundary check
    #[inline(always)]
    pub fn new_unchecked(
        positions: Vec<Point3>,
        uv_coords: Vec<Vector2>,
        normals: Vec<Vector3>,
        faces: Faces,
    ) -> PolygonMesh {
        PolygonMesh {
            positions,
            uv_coords,
            normals,
            faces,
        }
    }

    /// constructor, boundary check is acrivated only in debug mode.
    #[inline(always)]
    pub fn debug_new(
        positions: Vec<Point3>,
        uv_coords: Vec<Vector2>,
        normals: Vec<Vector3>,
        faces: Faces,
    ) -> PolygonMesh {
        match cfg!(debug_assertions) {
            true => Self::new(positions, uv_coords, normals, faces),
            false => Self::new_unchecked(positions, uv_coords, normals, faces),
        }
    }

    /// Returns the vector of all positions.
    #[inline(always)]
    pub fn positions(&self) -> &Vec<Point3> { &self.positions }

    /// Returns the mutable slice of all positions.
    #[inline(always)]
    pub fn positions_mut(&mut self) -> &mut [Point3] { &mut self.positions }

    /// Adds a position.
    #[inline(always)]
    pub fn push_position(&mut self, position: Point3) { self.positions.push(position) }

    /// Extend positions by iterator.
    #[inline(always)]
    pub fn extend_positions<I: IntoIterator<Item = Point3>>(&mut self, iter: I) {
        self.positions.extend(iter)
    }

    /// Returns the vector of all uv (texture) coordinates.
    #[inline(always)]
    pub fn uv_coords(&self) -> &Vec<Vector2> { &self.uv_coords }

    /// Returns the mutable slice of all uv (texture) coordinates.
    #[inline(always)]
    pub fn uv_coords_mut(&mut self) -> &mut [Vector2] { &mut self.uv_coords }

    /// Adds a uv (texture) coordinate.
    #[inline(always)]
    pub fn push_uv_coord(&mut self, uv_coord: Vector2) { self.uv_coords.push(uv_coord) }

    /// Extend uv (texture) coordinates by iterator.
    #[inline(always)]
    pub fn extend_uv_coords<I: IntoIterator<Item = Vector2>>(&mut self, iter: I) {
        self.uv_coords.extend(iter)
    }

    /// Returns the vector of all normals.
    #[inline(always)]
    pub fn normals(&self) -> &Vec<Vector3> { &self.normals }

    /// Returns the mutable slice of all normals.
    #[inline(always)]
    pub fn normals_mut(&mut self) -> &mut [Vector3] { &mut self.normals }

    /// Extend normals by iterator
    #[inline(always)]
    pub fn extend_normals<I: IntoIterator<Item = Vector3>>(&mut self, iter: I) {
        self.normals.extend(iter)
    }

    /// Returns the faces of the polygon.
    #[inline(always)]
    pub fn faces(&self) -> &Faces { &self.faces }

    /// Returns the vector of all triangles of the polygon.
    #[inline(always)]
    pub fn tri_faces(&self) -> &Vec<[Vertex; 3]> { &self.faces.tri_faces }

    /// Returns the mutable slice of all triangles.
    #[inline(always)]
    pub fn tri_faces_mut(&mut self) -> &mut [[Vertex; 3]] { &mut self.faces.tri_faces }

    /// Returns the vector of all quadrangles.
    #[inline(always)]
    pub fn quad_faces(&self) -> &Vec<[Vertex; 4]> { &self.faces.quad_faces }

    /// Returns the mutable slice of all quadrangles.
    #[inline(always)]
    pub fn quad_faces_mut(&mut self) -> &mut [[Vertex; 4]] { &mut self.faces.quad_faces }

    /// Returns the vector of n-gons (n > 4).
    #[inline(always)]
    pub fn other_faces(&self) -> &[Vec<Vertex>] { &self.faces.other_faces }

    /// Returns the mutable iterator of n-gons (n > 4).
    #[inline(always)]
    pub fn other_faces_mut(&mut self) -> &mut [Vec<Vertex>] { &mut self.faces.other_faces }

    /// Returns the iterator of the slice.
    ///
    /// By the internal optimization, this iterator does not runs in the simple order
    /// in which they are registered, but runs order: triangle, square, and the others.
    /// cf: [`Faces::face_iter`](./struct.Faces.html#method.face_iter)
    #[inline(always)]
    pub fn face_iter<'a>(&'a self) -> impl Iterator<Item = &'a [Vertex]> { self.faces.face_iter() }

    /// Returns the iterator of the slice.
    ///
    /// By the internal optimization, this iterator does not runs in the simple order
    /// in which they are registered, but runs order: triangle, square, and the others.
    /// cf: [`Faces::face_iter`](./struct.Faces.html#method.face_iter)
    #[inline(always)]
    pub fn face_iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut [Vertex]> {
        self.faces.face_iter_mut()
    }

    /// Creates an editor that performs boundary checking on dropped.
    #[inline(always)]
    pub fn editor(&mut self) -> PolygonMeshEditor {
        PolygonMeshEditor {
            positions: &mut self.positions,
            uv_coords: &mut self.uv_coords,
            normals: &mut self.normals,
            faces: &mut self.faces,
            bound_check: true,
        }
    }
    /// Creates an editor that does NOT perform boundary checking on dropped.
    #[inline(always)]
    pub fn uncheck_editor(&mut self) -> PolygonMeshEditor {
        PolygonMeshEditor {
            positions: &mut self.positions,
            uv_coords: &mut self.uv_coords,
            normals: &mut self.normals,
            faces: &mut self.faces,
            bound_check: false,
        }
    }
    /// Creates an editor that performs boundary checking on dropped ONLY in debug build.
    #[inline(always)]
    pub fn debug_editor(&mut self) -> PolygonMeshEditor {
        PolygonMeshEditor {
            positions: &mut self.positions,
            uv_coords: &mut self.uv_coords,
            normals: &mut self.normals,
            faces: &mut self.faces,
            bound_check: cfg!(debug_assertions),
        }
    }

    /// Returns polygonmesh merged `self` and `mesh`.
    pub fn merge(&mut self, mut mesh: PolygonMesh) {
        let n_pos = self.positions.len();
        let n_uv = self.uv_coords.len();
        let n_nor = self.normals.len();
        mesh.faces.face_iter_mut().for_each(move |face| {
            face.iter_mut().for_each(|v| {
                v.pos += n_pos;
                v.uv.as_mut().map(|uv| *uv += n_uv);
                v.nor.as_mut().map(|nor| *nor += n_nor);
            })
        });
        self.positions.extend(mesh.positions);
        self.uv_coords.extend(mesh.uv_coords);
        self.normals.extend(mesh.normals);
        self.faces.naive_concat(mesh.faces);
    }
    /// Creates the bounding box of the polygon mesh.
    #[inline(always)]
    pub fn bounding_box(&self) -> BoundingBox<Point3> { self.positions().iter().collect() }
}

/// Editor of polygon mesh
///
/// It has mutable references to all member variables of the polygon mesh as public variables,
/// allowing for any destructive changes for optimization.
/// At drop time, the indices of each vertex are judged to be within the range of
/// the array of attributes, and a panic occurs if there is one outside the range (boundary check).
///
/// # Examples
/// ```
/// use truck_polymesh::*;
///
/// let positions = vec![
///     Point3::new(1.0, 0.0, 0.0),
///     Point3::new(0.0, 1.0, 0.0),
///     Point3::new(0.0, 0.0, 1.0),
/// ];
/// let faces = Faces::from_iter(&[[0, 1, 2]]);
/// let mut mesh = PolygonMesh::new(positions, Vec::new(), Vec::new(), faces);
///
/// // create editor
/// let editor = mesh.editor();
///
/// // destructive changes
/// editor.uv_coords.push(Vector2::new(0.0, 0.0));
/// editor.faces.tri_faces_mut()[0][0].uv = Some(0);
/// ```
/// ```should_panic
/// use truck_polymesh::*;
///
/// let positions = vec![
///     Point3::new(1.0, 0.0, 0.0),
///     Point3::new(0.0, 1.0, 0.0),
///     Point3::new(0.0, 0.0, 1.0),
/// ];
/// let faces = Faces::from_iter(&[[0, 1, 2]]);
/// let mut mesh = PolygonMesh::new(positions, Vec::new(), Vec::new(), faces);
///
/// // create editor
/// let editor = mesh.editor();
///
/// // destructive changes
/// editor.faces.tri_faces_mut()[0][0].uv = Some(0);
///
/// // Panic occurs since no uv coord is added.
/// ```
#[derive(Debug)]
pub struct PolygonMeshEditor<'a> {
    /// mutable reference to the vector of positions
    pub positions: &'a mut Vec<Point3>,
    /// mutable reference to the vector of uv coordinates
    pub uv_coords: &'a mut Vec<Vector2>,
    /// mutable reference to the vector of normals
    pub normals: &'a mut Vec<Vector3>,
    /// mutable reference to the faces of the polygon mesh
    pub faces: &'a mut Faces,
    bound_check: bool,
}

impl<'a> PolygonMeshEditor<'a> {
    #[inline(always)]
    fn is_compatible(&self) -> Result<()> {
        self.faces.is_compatible(
            self.positions.len(),
            self.uv_coords.len(),
            self.normals.len(),
        )
    }

    /// Drops with boundary check and returns `Result`.
    #[inline(always)]
    pub fn try_drop(mut self) -> Result<()> {
        self.bound_check = false;
        self.is_compatible()
    }
}

impl<'a> Drop for PolygonMeshEditor<'a> {
    #[inline(always)]
    fn drop(&mut self) {
        if self.bound_check {
            self.is_compatible().unwrap_or_else(|e| panic!("{:?}", e));
        }
    }
}
