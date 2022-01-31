use crate::*;
use rustc_hash::FxHashMap as HashMap;
use std::fmt::Debug;
use std::hash::Hash;

impl<V: Copy + Hash + Debug + Eq, A: Attributes<V>> PolygonMesh<V, A> {
    /// Contract attributes and expand polygon.
    ///
    /// # Examples
    /// ```
    /// use truck_polymesh::*;
    /// let polygon = PolygonMesh::new(
    ///     StandardAttributes {
    ///         positions: vec![
    ///             Point3::new(0.0, 0.0, 0.0),
    ///             Point3::new(1.0, 0.0, 0.0),
    ///             Point3::new(0.0, 1.0, 0.0),
    ///             Point3::new(1.0, 1.0, 0.0),
    ///         ],
    ///         normals: vec![
    ///             Vector3::new(0.0, 0.0, 1.0),
    ///             Vector3::new(0.0, 0.0, -1.0),
    ///         ],
    ///         ..Default::default()
    ///     },
    ///     Faces::from_iter(&[
    ///         &[(0, None, Some(0)), (1, None, Some(0)), (2, None, Some(0))],
    ///         &[(3, None, Some(1)), (1, None, Some(1)), (2, None, Some(1))],
    ///     ])
    /// );
    /// let expands = polygon.expands(|attr| (attr.position, attr.normal.unwrap()));
    /// assert_eq!(
    ///     expands,
    ///     PolygonMesh::<usize, Vec<(Point3, Vector3)>>::new(
    ///         vec![
    ///            (Point3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
    ///            (Point3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
    ///            (Point3::new(0.0, 1.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
    ///            (Point3::new(1.0, 1.0, 0.0), Vector3::new(0.0, 0.0, -1.0)),
    ///            (Point3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, -1.0)),
    ///            (Point3::new(0.0, 1.0, 0.0), Vector3::new(0.0, 0.0, -1.0)),
    ///         ],
    ///         Faces::from_iter(&[[0, 1, 2], [3, 4, 5]]),
    ///     )
    /// );
    /// ```
    pub fn expands<T: Copy>(
        &self,
        contraction: impl Fn(A::Output) -> T,
    ) -> PolygonMesh<usize, Vec<T>> {
        let mut vec = Vec::<T>::new();
        let mut vertex_map = HashMap::<V, usize>::default();
        let faces: Faces<usize> = self
            .face_iter()
            .map(|face| {
                face.iter()
                    .cloned()
                    .map(|vertex| {
                        *vertex_map.entry(vertex).or_insert_with(|| {
                            let idx = vec.len();
                            vec.push(contraction(self.attributes.get(vertex).unwrap()));
                            idx
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        PolygonMesh::new(vec, faces)
    }
}
