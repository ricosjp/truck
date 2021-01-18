use crate::*;
use errors::Error;
use std::iter::FromIterator;

impl<V: Copy> Faces<V> {
    #[inline(always)]
    pub fn push<T: AsRef<[V]>>(&mut self, face: T) {
        let face = face.as_ref();
        match face.len() {
            3 => self.tri_faces.push([face[0], face[1], face[2]]),
            4 => self.quad_faces.push([face[0], face[1], face[2], face[3]]),
            _ => self
                .other_faces
                .push(Vec::from_iter(face.iter().map(|v| *v))),
        }
    }

    #[inline(always)]
    pub fn tri_faces(&self) -> &[[V; 3]] { &self.tri_faces }

    #[inline(always)]
    pub fn quad_faces(&self) -> &[[V; 4]] { &self.quad_faces }

    #[inline(always)]
    pub fn other_faces(&self) -> &[Vec<V>] { &self.other_faces }

    #[inline(always)]
    pub fn face_iter<'a>(&'a self) -> impl Iterator<Item = &'a [V]> {
        self.tri_faces
            .iter()
            .map(|v| v.as_ref())
            .chain(self.quad_faces.iter().map(|v| v.as_ref()))
            .chain(self.other_faces.iter().map(|v| v.as_ref()))
    }
    #[inline(always)]
    pub fn face_iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut [V]> {
        self.tri_faces
            .iter_mut()
            .map(|v| v.as_mut())
            .chain(self.quad_faces.iter_mut().map(|v| v.as_mut()))
            .chain(self.other_faces.iter_mut().map(|v| v.as_mut()))
    }

    #[inline(always)]
    fn naive_concat(&mut self, other: Self) {
        self.tri_faces.extend(other.tri_faces);
        self.quad_faces.extend(other.quad_faces);
        self.other_faces.extend(other.other_faces);
    }
}

impl<V: AsRef<[usize]>> Faces<V> {
    #[inline(always)]
    fn sub_into_positions(self) -> Faces<[usize; 1]> {
        let tri_faces = self
            .tri_faces
            .iter()
            .map(|v| [[v[0].as_ref()[0]], [v[1].as_ref()[0]], [v[2].as_ref()[0]]])
            .collect();
        let quad_faces = self
            .quad_faces
            .iter()
            .map(|v| {
                [
                    [v[0].as_ref()[0]],
                    [v[1].as_ref()[0]],
                    [v[2].as_ref()[0]],
                    [v[3].as_ref()[0]],
                ]
            })
            .collect();
        let other_faces = self
            .other_faces
            .iter()
            .map(|v| v.iter().map(|idx| [idx.as_ref()[0]]).collect())
            .collect();
        Faces {
            tri_faces,
            quad_faces,
            other_faces,
        }
    }
}

impl<V: AsRef<[usize]> + AsMut<[usize]> + Copy> Faces<V> {
    #[inline(always)]
    fn add_fair(&mut self, plus: V) {
        self.face_iter_mut().for_each(move |face| {
            face.iter_mut().for_each(move |v| {
                v.as_mut()
                    .iter_mut()
                    .zip(plus.as_ref())
                    .for_each(|(idx, p)| *idx += *p)
            })
        });
    }
}

impl Faces<[usize; 3]> {
    #[inline(always)]
    fn sub_into_two_attrs(self, idx: usize) -> Faces<[usize; 2]> {
        let tri_faces = self
            .tri_faces
            .iter()
            .map(|v| {
                [
                    [v[0][0], v[0][idx]],
                    [v[1][0], v[1][idx]],
                    [v[2][0], v[2][idx]],
                ]
            })
            .collect();
        let quad_faces = self
            .quad_faces
            .iter()
            .map(|v| {
                [
                    [v[0][0], v[0][idx]],
                    [v[1][0], v[1][idx]],
                    [v[2][0], v[2][idx]],
                    [v[3][0], v[3][idx]],
                ]
            })
            .collect();
        let other_faces = self
            .other_faces
            .iter()
            .map(|v| v.iter().map(|i| [i[0], i[idx]]).collect())
            .collect();
        Faces {
            tri_faces,
            quad_faces,
            other_faces,
        }
    }
}

impl PolygonMesh {
    /// Constructs by positions
    /// # Panics
    /// Panic occurs if there is an index is out of range.
    #[inline(always)]
    pub fn from_positions<'a, T: AsRef<[[usize; 1]]>, I: IntoIterator<Item = T>>(
        positions: Vec<Point3>,
        face_iter: I,
    ) -> PolygonMesh {
        PolygonMesh::try_from_positions(positions, face_iter).unwrap_or_else(|e| panic!("{:?}", e))
    }

    /// Constructs by positions
    /// # Errors
    /// Returns [`Error::OutOfRange`] if there is an index is out of range.
    ///
    /// [`Error::OutOfRange`]: ./errors/enum.Error.html#variant.OutOfRange
    pub fn try_from_positions<'a, T: AsRef<[[usize; 1]]>, I: IntoIterator<Item = T>>(
        positions: Vec<Point3>,
        face_iter: I,
    ) -> Result<PolygonMesh> {
        let mut faces = Faces::default();
        for face in face_iter {
            for idx in face.as_ref() {
                if idx[0] >= positions.len() {
                    return Err(Error::OutOfRange);
                }
            }
            faces.push(face);
        }
        Ok(PolygonMesh::Positions { positions, faces })
    }

    /// Constructs by positions and uv coordinates.
    /// # Panics
    /// Panic occurs if there is an index is out of range.
    #[inline(always)]
    pub fn from_positions_and_uvs<'a, T: AsRef<[[usize; 2]]>, I: IntoIterator<Item = T>>(
        positions: Vec<Point3>,
        uv_coords: Vec<Vector2>,
        face_iter: I,
    ) -> PolygonMesh {
        PolygonMesh::try_from_positions_and_uvs(positions, uv_coords, face_iter)
            .unwrap_or_else(|e| panic!("{:?}", e))
    }

    /// Constructs by positions and uv coordinates.
    /// # Errors
    /// Returns [`Error::OutOfRange`] if there is an index is out of range.
    ///
    /// [`Error::OutOfRange`]: ./errors/enum.Error.html#variant.OutOfRange
    pub fn try_from_positions_and_uvs<'a, T: AsRef<[[usize; 2]]>, I: IntoIterator<Item = T>>(
        positions: Vec<Point3>,
        uv_coords: Vec<Vector2>,
        face_iter: I,
    ) -> Result<PolygonMesh> {
        let mut faces = Faces::default();
        for face in face_iter {
            for v in face.as_ref() {
                if v[0] >= positions.len() || v[1] >= uv_coords.len() {
                    return Err(Error::OutOfRange);
                }
            }
            faces.push(face);
        }
        Ok(PolygonMesh::Textured {
            positions,
            uv_coords,
            faces,
        })
    }
    /// Constructs by positions and normals.
    /// # Panics
    /// Panic occurs if there is an index is out of range.
    /// # Remarks
    /// This method does not check whether the normal is normalized or not.
    #[inline(always)]
    pub fn from_positions_and_normals<'a, T: AsRef<[[usize; 2]]>, I: IntoIterator<Item = T>>(
        positions: Vec<Point3>,
        normals: Vec<Vector3>,
        face_iter: I,
    ) -> PolygonMesh {
        PolygonMesh::try_from_positions_and_normals(positions, normals, face_iter)
            .unwrap_or_else(|e| panic!("{:?}", e))
    }

    /// Constructs by positions and normals.
    /// # Errors
    /// Returns [`Error::OutOfRange`] if there is an index is out of range.
    ///
    /// [`Error::OutOfRange`]: ./errors/enum.Error.html#variant.OutOfRange
    ///
    /// # Remarks
    /// This method does not check whether the normal is normalized or not.
    pub fn try_from_positions_and_normals<'a, T: AsRef<[[usize; 2]]>, I: IntoIterator<Item = T>>(
        positions: Vec<Point3>,
        normals: Vec<Vector3>,
        face_iter: I,
    ) -> Result<PolygonMesh> {
        let mut faces = Faces::default();
        for face in face_iter {
            for v in face.as_ref() {
                if v[0] >= positions.len() || v[1] >= normals.len() {
                    return Err(Error::OutOfRange);
                }
            }
            faces.push(face);
        }
        Ok(PolygonMesh::WithNormals {
            positions,
            normals,
            faces,
        })
    }

    /// complete constructor
    /// # Panics
    /// Panic occurs if there is an index is out of range.
    /// # Remarks
    /// This method does not check whether the normal is normalized or not.
    pub fn new<'a, T: AsRef<[[usize; 3]]>, I: IntoIterator<Item = T>>(
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
    pub fn try_new<'a, T: AsRef<[[usize; 3]]>, I: IntoIterator<Item = T>>(
        positions: Vec<Point3>,
        uv_coords: Vec<Vector2>,
        normals: Vec<Vector3>,
        face_iter: I,
    ) -> Result<PolygonMesh> {
        let mut faces = Faces::default();
        for face in face_iter {
            for v in face.as_ref() {
                if v[0] >= positions.len() || v[1] >= uv_coords.len() || v[2] >= normals.len() {
                    return Err(Error::OutOfRange);
                }
            }
            faces.push(face);
        }
        Ok(PolygonMesh::Complete {
            positions,
            uv_coords,
            normals,
            faces,
        })
    }

    /// Returns the slice of all positions.
    #[inline(always)]
    pub fn positions(&self) -> &[Point3] {
        match self {
            PolygonMesh::Positions { positions, .. } => positions,
            PolygonMesh::Textured { positions, .. } => positions,
            PolygonMesh::WithNormals { positions, .. } => positions,
            PolygonMesh::Complete { positions, .. } => positions,
        }
    }
    /// Returns the mutable slice of all positions.
    #[inline(always)]
    pub fn positions_mut(&mut self) -> &mut [Point3] {
        match self {
            PolygonMesh::Positions { positions, .. } => positions,
            PolygonMesh::Textured { positions, .. } => positions,
            PolygonMesh::WithNormals { positions, .. } => positions,
            PolygonMesh::Complete { positions, .. } => positions,
        }
    }

    /// Returns the slice of all uv coords.
    #[inline(always)]
    pub fn uv_coords(&self) -> &[Vector2] {
        match self {
            PolygonMesh::Textured { uv_coords, .. } => uv_coords,
            PolygonMesh::Complete { uv_coords, .. } => uv_coords,
            _ => &[],
        }
    }
    /// Returns the mutable slice of all uv coords.
    #[inline(always)]
    pub fn uv_coords_mut(&mut self) -> &mut [Vector2] {
        match self {
            PolygonMesh::Textured { uv_coords, .. } => uv_coords,
            PolygonMesh::Complete { uv_coords, .. } => uv_coords,
            _ => &mut [],
        }
    }

    /// Returns the slice of all normals.
    #[inline(always)]
    pub fn normals(&self) -> &[Vector3] {
        match self {
            PolygonMesh::WithNormals { normals, .. } => normals,
            PolygonMesh::Complete { normals, .. } => normals,
            _ => &[],
        }
    }
    /// Returns the mutable slice of all normals.
    #[inline(always)]
    pub fn normals_mut(&mut self) -> &mut [Vector3] {
        match self {
            PolygonMesh::WithNormals { normals, .. } => normals,
            PolygonMesh::Complete { normals, .. } => normals,
            _ => &mut [],
        }
    }

    /// Returns the reference of faces
    #[inline(always)]
    pub fn faces(&self) -> FacesRef {
        match self {
            PolygonMesh::Positions { faces, .. } => FacesRef::Positions(faces),
            PolygonMesh::Textured { faces, .. } => FacesRef::Textured(faces),
            PolygonMesh::WithNormals { faces, .. } => FacesRef::WithNormals(faces),
            PolygonMesh::Complete { faces, .. } => FacesRef::Complete(faces),
        }
    }

    /// Into `PolygonMesh::Positions`.
    #[inline(always)]
    pub fn into_positions(self) -> PolygonMesh {
        match self {
            PolygonMesh::Positions { .. } => self,
            PolygonMesh::Textured {
                positions, faces, ..
            } => {
                let faces = faces.sub_into_positions();
                PolygonMesh::Positions { positions, faces }
            }
            PolygonMesh::WithNormals {
                positions, faces, ..
            } => {
                let faces = faces.sub_into_positions();
                PolygonMesh::Positions { positions, faces }
            }
            PolygonMesh::Complete {
                positions, faces, ..
            } => {
                let faces = faces.sub_into_positions();
                PolygonMesh::Positions { positions, faces }
            }
        }
    }

    /// Into `PolygonMesh::Textured`.
    /// # Errors
    /// Returns [`Error::NotEnoughAttrs`] if `self` matches `PolygonMesh::Positions` or `PolygonMesh::WithNormals`.
    /// 
    /// [`Error::NotEnoughAttrs`]: ./errors/enum.Error.html#variant.NotEnoughAttrs
    #[inline(always)]
    pub fn try_into_textured(self) -> Result<PolygonMesh> {
        match self {
            PolygonMesh::Positions { .. } => Err(Error::NotEnoughAttrs),
            PolygonMesh::Textured { .. } => Ok(self),
            PolygonMesh::WithNormals { .. } => Err(Error::NotEnoughAttrs),
            PolygonMesh::Complete {
                positions,
                uv_coords,
                faces,
                ..
            } => {
                let faces = faces.sub_into_two_attrs(1);
                Ok(PolygonMesh::Textured {
                    positions,
                    uv_coords,
                    faces,
                })
            }
        }
    }

    /// Into `PolygonMesh::Textured`.
    /// # Panics
    /// Panic occurs if `self` matches `PolygonMesh::Positions` or `PolygonMesh::WithNormals`.
    #[inline(always)]
    pub fn into_textured(self) -> PolygonMesh {
        self.try_into_textured()
            .unwrap_or_else(|e| panic!("{:?}", e))
    }
    
    /// Into `PolygonMesh::Normals`.
    /// # Errors
    /// Returns [`Error::NotEnoughAttrs`] if `self` matches `PolygonMesh::Positions` or `PolygonMesh::Textured`.
    /// 
    /// [`Error::NotEnoughAttrs`]: ./errors/enum.Error.html#variant.NotEnoughAttrs
    #[inline(always)]
    pub fn try_into_with_normals(self) -> Result<PolygonMesh> {
        match self {
            PolygonMesh::Positions { .. } => Err(Error::NotEnoughAttrs),
            PolygonMesh::Textured { .. } => Err(Error::NotEnoughAttrs),
            PolygonMesh::WithNormals { .. } => Ok(self),
            PolygonMesh::Complete {
                positions,
                normals,
                faces,
                ..
            } => {
                let faces = faces.sub_into_two_attrs(1);
                Ok(PolygonMesh::WithNormals {
                    positions,
                    normals,
                    faces,
                })
            }
        }
    }

    /// Into `PolygonMesh::Textured`.
    /// # Panics
    /// Panic occurs if `self` matches `PolygonMesh::Positions` or `PolygonMesh::Textured`.
    #[inline(always)]
    pub fn into_with_normals(self) -> PolygonMesh {
        self.try_into_with_normals()
            .unwrap_or_else(|e| panic!("{:?}", e))
    }

    /// Returns polygonmesh merged `self` and `mesh`.
    /// 
    /// # Remarks
    /// If there is a difference in the amount of information between the two meshes,
    /// the excess information will be discarded.
    /// More specifically, the mesh types correspond as follows:
    /// - `PolygonMesh::Positions` + `_` => `PolygonMesh::Positions`
    /// - `PolygonMesh::Textured` + `PolygonMesh::Textured` => `PolygonMesh::Textured`
    /// - `PolygonMesh::Textured` + `PolygonMesh::WithNormals` => `PolygonMesh::Positions`
    /// - `PolygonMesh::Textured` + `PolygonMesh::Complete` => `PolygonMesh::Textured`
    /// - `PolygonMesh::WithNormals` + `PolygonMesh::WithNormals` => `PolygonMesh::WithNormals`
    /// - `PolygonMesh::WithNormals` + `PolygonMesh::Complete` => `PolygonMesh::WithNormals`
    /// - `PolygonMesh::Complete` + `PolygonMesh::Complete` => `PolygonMesh::WithNormals`
    /// - _ => interchangeable
    pub fn merge(self, mesh: PolygonMesh) -> PolygonMesh {
        let tuple = (self, mesh);
        match tuple {
            (
                PolygonMesh::Positions {
                    mut positions,
                    mut faces,
                },
                PolygonMesh::Positions {
                    positions: another_positions,
                    faces: mut another_faces,
                },
            ) => {
                positions.extend(another_positions);
                let n = [positions.len()];
                another_faces.add_fair(n);
                faces.naive_concat(another_faces);
                PolygonMesh::Positions { positions, faces }
            }
            (
                PolygonMesh::Textured {
                    mut positions,
                    mut uv_coords,
                    mut faces,
                },
                PolygonMesh::Textured {
                    positions: another_positions,
                    uv_coords: another_uv_coords,
                    faces: mut another_faces,
                },
            ) => {
                positions.extend(another_positions);
                uv_coords.extend(another_uv_coords);
                let n = [positions.len(), uv_coords.len()];
                another_faces.add_fair(n);
                faces.naive_concat(another_faces);
                PolygonMesh::Textured {
                    positions,
                    uv_coords,
                    faces,
                }
            }
            (
                PolygonMesh::WithNormals {
                    mut positions,
                    mut normals,
                    mut faces,
                },
                PolygonMesh::WithNormals {
                    positions: another_positions,
                    normals: another_normals,
                    faces: mut another_faces,
                },
            ) => {
                positions.extend(another_positions);
                normals.extend(another_normals);
                let n = [positions.len(), normals.len()];
                another_faces.add_fair(n);
                faces.naive_concat(another_faces);
                PolygonMesh::WithNormals {
                    positions,
                    normals,
                    faces,
                }
            }
            (
                PolygonMesh::Complete {
                    mut positions,
                    mut uv_coords,
                    mut normals,
                    mut faces,
                },
                PolygonMesh::Complete {
                    positions: another_positions,
                    uv_coords: another_uv_coords,
                    normals: another_normals,
                    faces: mut another_faces,
                },
            ) => {
                positions.extend(another_positions);
                uv_coords.extend(another_uv_coords);
                normals.extend(another_normals);
                let n = [positions.len(), uv_coords.len(), normals.len()];
                another_faces.add_fair(n);
                faces.naive_concat(another_faces);
                PolygonMesh::Complete {
                    positions,
                    uv_coords,
                    normals,
                    faces,
                }
            }
            (PolygonMesh::Positions { .. }, PolygonMesh::Textured { .. }) => {
                tuple.0.merge(tuple.1.into_positions())
            }
            (PolygonMesh::Positions { .. }, PolygonMesh::WithNormals { .. }) => {
                tuple.0.merge(tuple.1.into_positions())
            }
            (PolygonMesh::Positions { .. }, PolygonMesh::Complete { .. }) => {
                tuple.0.merge(tuple.1.into_positions())
            }
            (PolygonMesh::Textured { .. }, PolygonMesh::Positions { .. }) => {
                tuple.0.into_positions().merge(tuple.1)
            }
            (PolygonMesh::Textured { .. }, PolygonMesh::WithNormals { .. }) => {
                tuple.0.into_positions().merge(tuple.1.into_positions())
            }
            (PolygonMesh::Textured { .. }, PolygonMesh::Complete { .. }) => {
                tuple.0.merge(tuple.1.into_textured())
            }
            (PolygonMesh::WithNormals { .. }, PolygonMesh::Positions { .. }) => {
                tuple.0.into_positions().merge(tuple.1)
            }
            (PolygonMesh::WithNormals { .. }, PolygonMesh::Textured { .. }) => {
                tuple.0.into_positions().merge(tuple.1.into_positions())
            }
            (PolygonMesh::WithNormals { .. }, PolygonMesh::Complete { .. }) => {
                tuple.0.merge(tuple.1.into_with_normals())
            }
            (PolygonMesh::Complete { .. }, PolygonMesh::Positions { .. }) => {
                tuple.0.into_positions().merge(tuple.1)
            }
            (PolygonMesh::Complete { .. }, PolygonMesh::Textured { .. }) => {
                tuple.0.into_textured().merge(tuple.1)
            }
            (PolygonMesh::Complete { .. }, PolygonMesh::WithNormals { .. }) => {
                tuple.0.into_with_normals().merge(tuple.1)
            }
        }
    }
    /// Creates the bounding box of the polygon mesh.
    #[inline(always)]
    pub fn bounding_box(&self) -> BoundingBox<Point3> {
        self.positions().iter().collect()
    }
}
