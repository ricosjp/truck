use crate::*;
use errors::Error;
use std::iter::FromIterator;

impl<V: Copy> Faces<V> {
    #[inline(always)]
    pub fn push(&mut self, face: &[V]) {
        match face.len() {
            3 => self.tri_faces.push([face[0], face[1], face[2]]),
            4 => self.quad_faces.push([face[0], face[1], face[2], face[3]]),
            _ => self
                .other_faces
                .push(Vec::from_iter(face.iter().map(|v| *v))),
        }
    }

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
}

impl<V: AsRef<[usize]>> Faces<V> {
    #[inline(always)]
    fn sub_into_positions(self) -> Faces<usize> {
        let tri_faces = self
            .tri_faces
            .iter()
            .map(|v| [v[0].as_ref()[0], v[1].as_ref()[0], v[2].as_ref()[0]])
            .collect();
        let quad_faces = self
            .quad_faces
            .iter()
            .map(|v| {
                [
                    v[0].as_ref()[0],
                    v[1].as_ref()[0],
                    v[2].as_ref()[0],
                    v[3].as_ref()[0],
                ]
            })
            .collect();
        let other_faces = self
            .other_faces
            .iter()
            .map(|v| v.iter().map(|idx| idx.as_ref()[0]).collect())
            .collect();
        Faces {
            tri_faces,
            quad_faces,
            other_faces,
        }
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
    pub fn from_positions<'a, I: Iterator<Item = &'a [usize]>>(
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
    pub fn try_from_positions<'a, I: Iterator<Item = &'a [usize]>>(
        positions: Vec<Point3>,
        face_iter: I,
    ) -> Result<PolygonMesh> {
        let mut faces = Faces::default();
        for face in face_iter {
            for idx in face.iter() {
                if *idx >= positions.len() {
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
    pub fn from_positions_and_uvs<'a, I: Iterator<Item = &'a [[usize; 2]]>>(
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
    pub fn try_from_positions_and_uvs<'a, I: Iterator<Item = &'a [[usize; 2]]>>(
        positions: Vec<Point3>,
        uv_coords: Vec<Vector2>,
        face_iter: I,
    ) -> Result<PolygonMesh> {
        let mut faces = Faces::default();
        for face in face_iter {
            for v in face.iter() {
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
    pub fn from_positions_and_normals<'a, I: Iterator<Item = &'a [[usize; 2]]>>(
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
    pub fn try_from_positions_and_normals<'a, I: Iterator<Item = &'a [[usize; 2]]>>(
        positions: Vec<Point3>,
        normals: Vec<Vector3>,
        face_iter: I,
    ) -> Result<PolygonMesh> {
        let mut faces = Faces::default();
        for face in face_iter {
            for v in face.iter() {
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
    pub fn new<'a, I: Iterator<Item = &'a [[usize; 3]]>>(
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
    pub fn try_new<'a, I: Iterator<Item = &'a [[usize; 3]]>>(
        positions: Vec<Point3>,
        uv_coords: Vec<Vector2>,
        normals: Vec<Vector3>,
        face_iter: I,
    ) -> Result<PolygonMesh> {
        let mut faces = Faces::default();
        for face in face_iter {
            for v in face.iter() {
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

    #[inline(always)]
    pub fn normals(&self) -> &[Vector3] {
        match self {
            PolygonMesh::WithNormals { normals, .. } => normals,
            PolygonMesh::Complete { normals, .. } => normals,
            _ => &[],
        }
    }
    #[inline(always)]
    pub fn normals_mut(&mut self) -> &mut [Vector3] {
        match self {
            PolygonMesh::WithNormals { normals, .. } => normals,
            PolygonMesh::Complete { normals, .. } => normals,
            _ => &mut [],
        }
    }

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

    #[inline(always)]
    pub fn into_textured(self) -> PolygonMesh {
        self.try_into_textured()
            .unwrap_or_else(|e| panic!("{:?}", e))
    }
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

    #[inline(always)]
    pub fn into_with_normals(self) -> PolygonMesh {
        self.try_into_with_normals()
            .unwrap_or_else(|e| panic!("{:?}", e))
    }

    /// Creates the bounding box of the polygon mesh.
    #[inline(always)]
    pub fn bounding_box(&self) -> BoundingBox<Point3> {
        self.positions().iter().collect()
    }

    /// Returns polygonmesh merged `self` and `mesh`.
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
                let n_pos = positions.len();
                another_faces
                    .face_iter_mut()
                    .for_each(move |face| face.iter_mut().for_each(move |idx| *idx += n_pos));
                faces.tri_faces.extend(another_faces.tri_faces);
                faces.quad_faces.extend(another_faces.quad_faces);
                faces.other_faces.extend(another_faces.other_faces);
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
                let n_pos = positions.len();
                let n_uv = uv_coords.len();
                another_faces.face_iter_mut().for_each(move |face| {
                    face.iter_mut().for_each(move |idx| {
                        idx[0] += n_pos;
                        idx[1] += n_uv;
                    })
                });
                faces.tri_faces.extend(another_faces.tri_faces);
                faces.quad_faces.extend(another_faces.quad_faces);
                faces.other_faces.extend(another_faces.other_faces);
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
                let n_pos = positions.len();
                let n_norm = normals.len();
                another_faces.face_iter_mut().for_each(move |face| {
                    face.iter_mut().for_each(move |idx| {
                        idx[0] += n_pos;
                        idx[1] += n_norm;
                    })
                });
                faces.tri_faces.extend(another_faces.tri_faces);
                faces.quad_faces.extend(another_faces.quad_faces);
                faces.other_faces.extend(another_faces.other_faces);
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
                let n_pos = positions.len();
                let n_uv = uv_coords.len();
                let n_norm = normals.len();
                another_faces.face_iter_mut().for_each(move |face| {
                    face.iter_mut().for_each(move |idx| {
                        idx[0] += n_pos;
                        idx[1] += n_uv;
                        idx[2] += n_norm;
                    })
                });
                faces.tri_faces.extend(another_faces.tri_faces);
                faces.quad_faces.extend(another_faces.quad_faces);
                faces.other_faces.extend(another_faces.other_faces);
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
}
