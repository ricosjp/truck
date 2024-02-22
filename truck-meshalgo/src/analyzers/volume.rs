use super::*;
use array_macro::array;

/// Calculate the volume and the center of gravity of mesh.
pub trait CalcVolume {
    /// Returns the volume of the mesh if the mesh is closed.
    ///
    /// # Details
    /// More strictly, it returns the integral value of `xdydz`. Because of the linearity of this operation,
    /// if multiple meshes represent a single closed geometry, the overall volume can be calculated by computing
    /// this value for each and adding them together.
    ///
    /// # Examples
    /// ```
    /// // The regular trihedron in the unit sphere.
    /// use truck_meshalgo::prelude::*;
    /// let a = 1.0 / f64::sqrt(3.0);
    /// let positions = vec![
    ///     Point3::new(-a, -a, -a),    
    ///     Point3::new(a, a, -a),    
    ///     Point3::new(a, -a, a),    
    ///     Point3::new(-a, a, a),    
    /// ];
    /// let attrs = StandardAttributes {
    ///     positions,
    ///     ..Default::default()
    /// };
    /// let faces = Faces::from_iter(&[
    ///     [0, 1, 2],
    ///     [1, 3, 2],
    ///     [1, 0, 3],
    ///     [0, 2, 3],
    /// ]);
    /// let polygon = PolygonMesh::new(attrs, faces);
    /// assert_near!(polygon.volume(), 8.0 * f64::sqrt(3.0) / 27.0);
    /// ```
    /// ```
    /// // Cube with a hollow inside, defined by two boundaries.
    /// use truck_meshalgo::prelude::*;
    /// let positions0 = vec![
    ///     Point3::new(0.0, 0.0, 0.0),
    ///     Point3::new(3.0, 0.0, 0.0),
    ///     Point3::new(0.0, 3.0, 0.0),
    ///     Point3::new(0.0, 0.0, 3.0),
    ///     Point3::new(0.0, 3.0, 3.0),
    ///     Point3::new(3.0, 0.0, 3.0),
    ///     Point3::new(3.0, 3.0, 0.0),
    ///     Point3::new(3.0, 3.0, 3.0),
    /// ];
    /// let attrs0 = StandardAttributes {
    ///     positions: positions0,
    ///     ..Default::default()
    /// };
    /// let faces0 = Faces::from_iter(&[
    ///     [0, 2, 6, 1],
    ///     [0, 1, 5, 3],
    ///     [1, 6, 7, 5],
    ///     [6, 2, 4, 7],
    ///     [2, 0, 3, 4],
    ///     [3, 5, 7, 4],
    /// ]);
    /// let polygon0 = PolygonMesh::new(attrs0, faces0);
    /// let positions1 = vec![
    ///     Point3::new(1.0, 1.0, 1.0),
    ///     Point3::new(2.0, 1.0, 1.0),
    ///     Point3::new(1.0, 2.0, 1.0),
    ///     Point3::new(1.0, 1.0, 2.0),
    ///     Point3::new(1.0, 2.0, 2.0),
    ///     Point3::new(2.0, 1.0, 2.0),
    ///     Point3::new(2.0, 2.0, 1.0),
    ///     Point3::new(2.0, 2.0, 2.0),
    /// ];
    /// let attrs1 = StandardAttributes {
    ///     positions: positions1,
    ///     ..Default::default()
    /// };
    /// let faces1 = Faces::from_iter(&[
    ///     [1, 6, 2, 0],
    ///     [3, 5, 1, 0],
    ///     [5, 7, 6, 1],
    ///     [7, 4, 2, 6],
    ///     [4, 3, 0, 2],
    ///     [4, 7, 5, 3],
    /// ]);
    /// let polygon1 = PolygonMesh::new(attrs1, faces1);
    /// // We consider the solid defined by two boundaries(polygon0 and polygon1).
    /// // The volume will be calculated by `position0.volume() + position1.volume()`.
    /// assert_near!(polygon0.volume() + polygon1.volume(), 26.0);
    /// ```
    fn volume(&self) -> f64;
    /// Returns the center of gravity of the mesh by homogeneous coordinate if the mesh is closed.
    ///
    /// # Details
    /// More strictly, it returns the integral value of `(1/2 x^2 dydz, 1/2 y^2dzdx, 1/2 z^2 dxdy, xdydz)`.
    /// Because of the linearity of this operation, if multiple meshes represent a single closed geometry,
    /// the overall volume can be calculated by computing this value for each and adding them together.
    ///
    /// # Examples
    /// ```
    /// // The triangular prism
    /// use truck_meshalgo::prelude::*;
    /// let positions = vec![
    ///     Point3::new(0.0, 0.0, 0.0),    
    ///     Point3::new(2.0, 5.0, 0.0),    
    ///     Point3::new(-5.0, 1.0, 0.0),    
    ///     Point3::new(0.0, 0.0, 2.0),    
    ///     Point3::new(2.0, 5.0, 2.0),    
    ///     Point3::new(-5.0, 1.0, 2.0),    
    /// ];
    /// let attrs = StandardAttributes {
    ///     positions,
    ///     ..Default::default()
    /// };
    /// let faces = Faces::from_iter(&[
    ///     [2, 1, 0].as_slice(),
    ///     &[0, 1, 4, 3],
    ///     &[1, 2, 5, 4],
    ///     &[2, 0, 3, 5],
    ///     &[3, 4, 5],
    /// ]);
    /// let polygon = PolygonMesh::new(attrs, faces);
    /// let homog = polygon.center_of_gravity();
    /// assert_near!(homog.to_point(), Point3::new(-1.0, 2.0, 1.0));
    /// ```
    fn center_of_gravity(&self) -> Vector4;
}

impl CalcVolume for PolygonMesh {
    fn volume(&self) -> f64 {
        point_triangles(self).fold(0.0, |sum, [p, q, r]| {
            sum + (p.x + q.x + r.x) * ((q.y - p.y) * (r.z - p.z) - (r.y - p.y) * (q.z - p.z))
        }) / 6.0
    }
    fn center_of_gravity(&self) -> Vector4 {
        let arr = point_triangles(self).fold([0.0; 4], |sum, [p, q, r]| {
            let det = array![i => {
                let (j, k) = ((i + 1) % 3, (i + 2) % 3);
                (q[j] - p[j]) * (r[k] - p[k]) - (r[j] - p[j]) * (q[k] - p[k])
            }; 3];
            let vals = array![i => {
                let s = p[i] + q[i] + r[i];
                s * s - p[i] * q[i] - q[i] * r[i] - r[i] * p[i]
            }; 3];
            let res = array![i => sum[i] + vals[i] * det[i]; 3];
            let res3 = sum[3] + (p.x + q.x + r.x) * det[0];
            [res[0], res[1], res[2], res3]
        });
        Vector4::new(arr[0] / 24.0, arr[1] / 24.0, arr[2] / 24.0, arr[3] / 6.0)
    }
}

fn point_triangles(poly: &PolygonMesh) -> impl Iterator<Item = [Point3; 3]> + '_ {
    poly.faces()
        .triangle_iter()
        .map(|faces| array![i => poly.positions()[faces[i].pos]; 3])
}

impl CalcVolume for truck_topology::Solid<Point3, PolylineCurve<Point3>, PolygonMesh> {
    fn volume(&self) -> f64 {
        self.face_iter()
            .map(|face| match face.orientation() {
                true => face.surface().volume(),
                false => -face.surface().volume(),
            })
            .sum::<f64>()
    }
    fn center_of_gravity(&self) -> Vector4 {
        self.face_iter()
            .map(|face| match face.orientation() {
                true => face.surface().center_of_gravity(),
                false => -face.surface().center_of_gravity(),
            })
            .sum::<Vector4>()
    }
}
