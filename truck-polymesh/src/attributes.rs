use crate::*;

impl<T: Copy> Attributes<usize> for Vec<T> {
    type Output = T;
    fn get(&self, idx: usize) -> Option<T> { self.as_slice().get(idx).copied() }
}

impl Attributes<StandardVertex> for StandardAttributes {
    type Output = StandardAttribute;
    fn get(&self, v: StandardVertex) -> Option<Self::Output> {
        Some(StandardAttribute {
            position: self.positions.get(v.pos)?,
            uv_coord: match v.uv {
                Some(i) => Some(self.uv_coords.get(i)?),
                None => None,
            },
            normal: match v.nor {
                Some(i) => Some(self.normals.get(i)?),
                None => None,
            },
        })
    }
}

impl StandardAttributes {
    /// Returns the vector of all positions.
    #[inline(always)]
    pub const fn positions(&self) -> &Vec<Point3> { &self.positions }

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
    pub const fn uv_coords(&self) -> &Vec<Vector2> { &self.uv_coords }

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
    pub const fn normals(&self) -> &Vec<Vector3> { &self.normals }

    /// Returns the mutable slice of all normals.
    #[inline(always)]
    pub fn normals_mut(&mut self) -> &mut [Vector3] { &mut self.normals }

    /// Extend normals by iterator
    #[inline(always)]
    pub fn extend_normals<I: IntoIterator<Item = Vector3>>(&mut self, iter: I) {
        self.normals.extend(iter)
    }
}

impl TransformedAttributes for Vec<Point3> {
    #[inline(always)]
    fn transform_by(&mut self, trans: Matrix4) {
        self.iter_mut().for_each(|p| {
            *p = trans.transform_point(*p);
        });
    }
    #[inline(always)]
    fn transformed(&self, trans: Matrix4) -> Self {
        self.iter().map(|p| trans.transform_point(*p)).collect()
    }
}

impl TransformedAttributes for Vec<StandardAttribute> {
    #[inline(always)]
    fn transform_by(&mut self, trans: Matrix4) {
        self.iter_mut().for_each(|attr| {
            attr.position = trans.transform_point(attr.position);
            if let Some(n) = &mut attr.normal {
                *n = trans.transform_vector(*n);
            }
        })
    }
    #[inline(always)]
    fn transformed(&self, trans: Matrix4) -> Self {
        self.iter()
            .map(|attr| StandardAttribute {
                position: trans.transform_point(attr.position),
                uv_coord: attr.uv_coord,
                normal: attr.normal.map(|n| trans.transform_vector(n)),
            })
            .collect()
    }
}

impl TransformedAttributes for StandardAttributes {
    #[inline(always)]
    fn transform_by(&mut self, trans: Matrix4) {
        self.positions_mut().iter_mut().for_each(|p| {
            *p = trans.transform_point(*p);
        });
        self.normals_mut().iter_mut().for_each(|n| {
            *n = trans.transform_vector(*n);
        });
    }
    #[inline(always)]
    fn transformed(&self, trans: Matrix4) -> Self {
        Self {
            positions: self
                .positions()
                .iter()
                .map(|&p| trans.transform_point(p))
                .collect(),
            uv_coords: self.uv_coords().clone(),
            normals: self
                .normals()
                .iter()
                .map(|&n| trans.transform_vector(n))
                .collect(),
        }
    }
}
