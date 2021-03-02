use crate::*;

/// bounded plane
/// # Example
/// ```
/// use truck_geometry::*;
/// let plane: Plane = Plane::new(
///     Point3::new(0.0, 1.0, 2.0), // O
///     Point3::new(1.0, 1.0, 3.0), // A
///     Point3::new(0.0, 2.0, 3.0), // B
/// );
/// // The origin of the plane is O.
/// Point3::assert_near(&plane.origin(), &Point3::new(0.0, 1.0, 2.0));
/// // The normal is (A - O).cross(B - O)
/// Vector3::assert_near(&plane.normal(), &Vector3::new(-1.0, -1.0, 1.0).normalize());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Plane {
    o: Point3,
    p: Point3,
    q: Point3,
}

/// sphere
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sphere {
    center: Point3,
    radius: f64,
}

mod plane;
mod sphere;
