use super::*;
use derive_more::{Deref, DerefMut};

#[derive(Clone, Copy, Debug)]
struct Ray {
    origin: Point3,
    direction: Vector3,
}

#[derive(Clone, Copy, Debug, Deref, DerefMut)]
struct Triangle([Point3; 3]);

impl Triangle {
    #[inline(always)]
    fn normal(self) -> Vector3 { (self[1] - self[0]).cross(self[2] - self[0]).normalize() }

    fn is_crossing(self, ray: Ray) -> bool {
        let a = self[0] - self[1];
        let b = self[0] - self[2];
        let mat = Matrix3::from_cols(a, b, ray.direction);
        if mat.determinant().so_small() {
            false
        } else {
            let inv = mat.invert().unwrap();
            let uvt = inv * (self[0] - ray.origin);
            uvt[0] > 0.0 && uvt[1] > 0.0 && uvt[0] + uvt[1] < 1.0 && uvt[2] > 0.0
        }
    }
}

/// whether a point is in a domain rounded by a closed polygon.
pub trait IncludingPointInDomain {
    /// Count signed number of faces crossing ray with origin `point` and direction `ray_direction`.
    /// Counter increase if the dot product of the ray and the normal of a face is positive,
    /// and decrease if it is negative.
    ///
    /// # Examples
    /// ```
    /// use truck_meshalgo::prelude::*;
    /// let simplex = PolygonMesh::new(
    ///     StandardAttributes {
    ///         positions: vec![
    ///             Point3::new(0.0, 0.0, 0.0),
    ///             Point3::new(1.0, 0.0, 0.0),
    ///             Point3::new(0.0, 1.0, 0.0),
    ///             Point3::new(0.0, 0.0, 1.0),
    ///         ],
    ///         ..Default::default()
    ///     },
    ///     Faces::from_iter(vec![
    ///         [0, 2, 1],
    ///         [0, 1, 3],
    ///         [0, 3, 2],
    ///         [1, 2, 3],
    ///     ]),
    /// );
    ///
    /// assert_eq!(simplex.signed_crossing_faces(Point3::new(0.1, 0.1, 0.1), Vector3::unit_x()), 1);
    /// assert_eq!(simplex.signed_crossing_faces(Point3::new(-0.1, 0.1, 0.1), Vector3::unit_x()), 0);
    /// ```
    fn signed_crossing_faces(&self, point: Point3, ray_direction: Vector3) -> isize;
    /// whether `point` is in a domain rounded by a closed polygon.
    ///
    /// # Examples
    /// ```
    /// use truck_meshalgo::prelude::*;
    /// let simplex = PolygonMesh::new(
    ///     StandardAttributes {
    ///         positions: vec![
    ///             Point3::new(0.0, 0.0, 0.0),
    ///             Point3::new(1.0, 0.0, 0.0),
    ///             Point3::new(0.0, 1.0, 0.0),
    ///             Point3::new(0.0, 0.0, 1.0),
    ///         ],
    ///         ..Default::default()
    ///     },
    ///     Faces::from_iter(vec![
    ///         [0, 2, 1],
    ///         [0, 1, 3],
    ///         [0, 3, 2],
    ///         [1, 2, 3],
    ///     ]),
    /// );
    ///
    /// assert!(simplex.inside(Point3::new(0.1, 0.1, 0.1)));
    /// assert!(!simplex.inside(Point3::new(-0.1, 0.1, 0.1)));
    /// ```
    fn inside(&self, point: Point3) -> bool;
}

impl IncludingPointInDomain for PolygonMesh {
    fn signed_crossing_faces(&self, point: Point3, ray_direction: Vector3) -> isize {
        let ray = Ray {
            origin: point,
            direction: ray_direction,
        };
        self.face_iter().fold(0, |mut counter, face| {
            for i in 2..face.len() {
                let tri = Triangle([
                    self.positions()[face[0].pos],
                    self.positions()[face[i - 1].pos],
                    self.positions()[face[i].pos],
                ]);
                if tri.is_crossing(ray) {
                    counter += f64::signum(tri.normal().dot(ray.direction)) as isize;
                }
            }
            counter
        })
    }
    fn inside(&self, point: Point3) -> bool {
        let dir = hash::take_one_unit(point);
        self.signed_crossing_faces(point, dir) >= 1
    }
}

#[test]
fn inside100() {
    let positions = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
    ];
    let faces: Faces = vec![[0, 2, 1], [0, 1, 3], [0, 3, 2], [1, 2, 3]]
        .into_iter()
        .collect();
    let simplex = PolygonMesh::new(
        StandardAttributes {
            positions,
            ..Default::default()
        },
        faces,
    );

    for _ in 0..100 {
        assert!(simplex.inside(Point3::new(0.1, 0.1, 0.1)));
        assert!(!simplex.inside(Point3::new(-0.1, 0.1, 0.1)));
    }
}
