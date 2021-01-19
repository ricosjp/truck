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

impl From<[usize; 3]> for Vertex {
    fn from(arr: [usize; 3]) -> Vertex {
        Vertex {
            pos: arr[0],
            uv: Some(arr[1]),
            nor: Some(arr[2]),
        }
    }
}

impl Faces {
    #[inline(always)]
    pub fn push<V: Copy + Into<Vertex>, T: AsRef<[V]>>(&mut self, face: T) {
        let face = face.as_ref();
        match face.len() {
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
    #[inline(always)]
    pub fn tri_faces(&self) -> &[[Vertex; 3]] { &self.tri_faces }

    #[inline(always)]
    pub fn tri_faces_mut(&mut self) -> &mut [[Vertex; 3]] { &mut self.tri_faces }

    #[inline(always)]
    pub fn quad_faces(&self) -> &[[Vertex; 4]] { &self.quad_faces }

    #[inline(always)]
    pub fn quad_faces_mut(&mut self) -> &mut [[Vertex; 4]] { &mut self.quad_faces }

    #[inline(always)]
    pub fn other_faces(&self) -> &[Vec<Vertex>] { &self.other_faces }

    #[inline(always)]
    pub fn other_faces_mut(&mut self) -> &mut [Vec<Vertex>] { &mut self.other_faces }

    #[inline(always)]
    pub fn face_iter<'a>(&'a self) -> impl Iterator<Item = &'a [Vertex]> {
        self.tri_faces
            .iter()
            .map(|v| v.as_ref())
            .chain(self.quad_faces.iter().map(|v| v.as_ref()))
            .chain(self.other_faces.iter().map(|v| v.as_ref()))
    }

    #[inline(always)]
    pub fn face_iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut [Vertex]> {
        self.tri_faces
            .iter_mut()
            .map(|v| v.as_mut())
            .chain(self.quad_faces.iter_mut().map(|v| v.as_mut()))
            .chain(self.other_faces.iter_mut().map(|v| v.as_mut()))
    }

    #[inline(always)]
    pub fn naive_concat(&mut self, other: Self) {
        self.tri_faces.extend(other.tri_faces);
        self.quad_faces.extend(other.quad_faces);
        self.other_faces.extend(other.other_faces);
    }
}

impl PolygonMesh {
    /// complete constructor
    /// # Panics
    /// Panic occurs if there is an index is out of range.
    /// # Remarks
    /// This method does not check whether the normal is normalized or not.
    pub fn new<'a, V: Copy + Into<Vertex>, T: AsRef<[V]>, I: IntoIterator<Item = T>>(
        positions: Vec<Point3>,
        uv_coords: Vec<Vector2>,
        normals: Vec<Vector3>,
        face_iter: I,
    ) -> PolygonMesh {
        PolygonMesh::try_new(positions, uv_coords, normals, face_iter)
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
    pub fn try_new<'a, V: Copy + Into<Vertex>, T: AsRef<[V]>, I: IntoIterator<Item = T>>(
        positions: Vec<Point3>,
        uv_coords: Vec<Vector2>,
        normals: Vec<Vector3>,
        face_iter: I,
    ) -> Result<PolygonMesh> {
        let mut faces = Faces::default();
        for face in face_iter {
            for v in face.as_ref() {
                let v = (*v).into();
                let pos_out_range = v.pos >= positions.len();
                let uv_out_range = v.uv.map(|uv| uv >= uv_coords.len()).unwrap_or(false);
                let nor_out_range = v.nor.map(|nor| nor >= normals.len()).unwrap_or(false);
                if pos_out_range || uv_out_range || nor_out_range {
                    return Err(Error::OutOfRange);
                }
            }
            faces.push(face);
        }
        Ok(PolygonMesh {
            positions,
            uv_coords,
            normals,
            faces,
        })
    }

    /// Returns the slice of all positions.
    #[inline(always)]
    pub fn positions(&self) -> &[Point3] { &self.positions }

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

    /// Returns the slice of all uv coords.
    #[inline(always)]
    pub fn uv_coords(&self) -> &[Vector2] { &self.uv_coords }

    /// Returns the mutable slice of all uv coords.
    #[inline(always)]
    pub fn uv_coords_mut(&mut self) -> &mut [Vector2] { &mut self.uv_coords }

    /// Adds a uv coord.
    #[inline(always)]
    pub fn push_uv_coord(&mut self, uv_coord: Vector2) { self.uv_coords.push(uv_coord) }

    /// Extend uv coords by iterator.
    #[inline(always)]
    pub fn extend_uv_coords<I: IntoIterator<Item = Vector2>>(&mut self, iter: I) {
        self.uv_coords.extend(iter)
    }

    /// Returns the slice of all normals.
    #[inline(always)]
    pub fn normals(&self) -> &[Vector3] { &self.normals }
    /// Returns the mutable slice of all normals.
    #[inline(always)]
    pub fn normals_mut(&mut self) -> &mut [Vector3] { &mut self.normals }

    /// Extend normals by iterator
    #[inline(always)]
    pub fn extend_normals<I: IntoIterator<Item = Vector3>>(&mut self, iter: I) {
        self.normals.extend(iter)
    }

    #[inline(always)]
    pub fn tri_faces(&self) -> &[[Vertex; 3]] { &self.faces.tri_faces }

    #[inline(always)]
    pub fn tri_faces_mut(&mut self) -> &mut [[Vertex; 3]] { &mut self.faces.tri_faces }

    #[inline(always)]
    pub fn quad_faces(&self) -> &[[Vertex; 4]] { &self.faces.quad_faces }

    #[inline(always)]
    pub fn quad_faces_mut(&mut self) -> &mut [[Vertex; 4]] { &mut self.faces.quad_faces }

    #[inline(always)]
    pub fn other_faces(&self) -> &[Vec<Vertex>] { &self.faces.other_faces }

    #[inline(always)]
    pub fn other_faces_mut(&mut self) -> &mut [Vec<Vertex>] { &mut self.faces.other_faces }

    #[inline(always)]
    pub fn face_iter<'a>(&'a self) -> impl Iterator<Item = &'a [Vertex]> { self.faces.face_iter() }

    #[inline(always)]
    pub fn face_iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut [Vertex]> {
        self.faces.face_iter_mut()
    }

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

pub struct PolygonMeshEditor<'a> {
    pub positions: &'a mut Vec<Point3>,
    pub uv_coords: &'a mut Vec<Vector2>,
    pub normals: &'a mut Vec<Vector3>,
    pub faces: &'a mut Faces,
    bound_check: bool,
}

impl<'a> Drop for PolygonMeshEditor<'a> {
    #[inline(always)]
    fn drop(&mut self) {
        if self.bound_check {
            for v in self.faces.face_iter().flatten() {
                let pos_out_range = v.pos >= self.positions.len();
                let uv_out_range = v.uv.map(|uv| uv >= self.uv_coords.len()).unwrap_or(false);
                let nor_out_range = v.nor.map(|nor| nor >= self.normals.len()).unwrap_or(false);
                if pos_out_range || uv_out_range || nor_out_range {
                    panic!("{:?}", Error::OutOfRange);
                }
            }
        }
    }
}
