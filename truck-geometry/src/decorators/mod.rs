use crate::*;
use std::ops::{Deref, DerefMut, Mul};

/// surface constructed by revoluting a curve
/// # Examples
/// Revoluted sphere
/// ```
/// use truck_geometry::*;
/// use std::f64::consts::PI;
/// let knot_vec = KnotVec::bezier_knot(2);
/// let control_points = vec![
///     Vector4::new(1.0, 0.0, 0.0, 1.0),
///     Vector4::new(0.0, 1.0, 0.0, 0.0),
///     Vector4::new(-1.0, 0.0, 0.0, 1.0),
/// ];
/// // upper half circle on xy-plane
/// let uhcircle = NurbsCurve::new(BSplineCurve::new(knot_vec, control_points));
/// // sphere constructed by revolute circle
/// let sphere = RevolutedCurve::by_revolution(
///     uhcircle, Point3::origin(), Vector3::unit_x(),
/// );
/// const N: usize = 30;
/// for i in 0..=N {
///     for j in 0..=N {
///         let u = i as f64 / N as f64;
///         let v = 2.0 * PI * j as f64 / N as f64;
///         let pt: Vector3 = sphere.subs(u, v).to_vec();
///         assert_near2!(pt.magnitude2(), 1.0);
///         assert_near!(pt, sphere.normal(u, v));
///     }
/// }
/// ```
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RevolutedCurve<C> {
    curve: C,
    origin: Point3,
    axis: Vector3,
}

/// Linearly extruded curve
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ExtrudedCurve<C, V> {
    curve: C,
    vector: V,
}

/// invertible and transformable geometric element
/// # Examples
/// Curve processing example
/// ```
/// use truck_geometry::*;
/// let curve: BSplineCurve<Point3> = BSplineCurve::new(
///     KnotVec::bezier_knot(2),
///     vec![
///         Point3::new(0.0, 0.0, 0.0),
///         Point3::new(0.0, 0.0, 1.0),
///         Point3::new(1.0, 0.0, 0.0),
///     ],
/// );
/// let mut processed = Processor::<_, Matrix4>::new(curve.clone());
///
/// // both curves are the same curve
/// const N: usize = 100;
/// for i in 0..=N {
///     let t = i as f64 / N as f64;
///     assert_eq!(curve.subs(t), processed.subs(t));
/// }
///
/// // Processed curve can inverted!
/// processed.invert();
/// for i in 0..=N {
///     let t = i as f64 / N as f64;
///     assert_eq!(curve.subs(1.0 - t), processed.subs(t));
/// }
/// ```
/// Surface processing example
/// ```
/// use truck_geometry::*;
/// use std::f64::consts::PI;
///
/// let sphere = Sphere::new(Point3::new(1.0, 2.0, 3.0), 2.45);
/// let mut processed = Processor::<_, Matrix4>::new(sphere);
///
/// // both surfaces are the same surface
/// const N: usize = 100;
/// for i in 0..=N {
///     for j in 0..=N {
///         let u = PI * i as f64 / N as f64;
///         let v = 2.0 * PI * j as f64 / N as f64;
///         assert_eq!(sphere.subs(u, v), processed.subs(u, v));
///     }
/// }
///
/// // Processed surface can be inverted!
/// // Here, "invert surface" means swap (u, v)-axes.
/// processed.invert();
/// for i in 0..=N {
///     for j in 0..=N {
///         let u = PI * i as f64 / N as f64;
///         let v = 2.0 * PI * j as f64 / N as f64;
///         assert_eq!(sphere.subs(u, v), processed.subs(v, u));
///     }
/// }
/// ```
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Processor<E, T> {
    entity: E,
    transform: T,
    orientation: bool,
}

/// The composited maps
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct PCurve<C, S> {
    curve: C,
    surface: S,
}

/// Intersection curve between two surfaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntersectionCurve<C, S> {
    // Considering rotational surfaces, we can consider the case
    // where the class `S` holds the curve `C` as a variable.
    surface0: Box<S>,
    surface1: Box<S>,
    leader: C,
    tol: f64,
}

/// trimmed curve for parametric curve
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TrimmedCurve<C> {
    curve: C,
    range: (f64, f64),
}

mod curve_on_surface;
mod extruded_curve;
mod intersection_curve;
mod processor;
mod revolved_curve;
mod trimmied_curve;
pub use intersection_curve::double_projection;
