use crate::{prelude::*, *};
use std::ops::{Deref, DerefMut, Mul};

/// revolution
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
struct Revolution {
    origin: Point3,
    axis: Vector3,
}

/// surface constructed by revoluting a curve
/// # Examples
/// Revoluted sphere
/// ```
/// use truck_geometry::prelude::*;
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
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, SelfSameGeometry)]
pub struct RevolutedCurve<C> {
    curve: C,
    revolution: Revolution,
}

/// Linearly extruded curve
///
/// # Examples
/// ```
/// use truck_geometry::prelude::*;
///
/// // entity curve
/// let cpts = vec![
///     Point3::new(0.0, 0.0, 0.0),
///     Point3::new(0.0, 1.0, 0.0),
///     Point3::new(1.0, 0.0, 0.0),
/// ];
/// let spts = vec![
///     vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, 1.0)],
///     vec![Point3::new(0.0, 1.0, 0.0), Point3::new(0.0, 1.0, 1.0)],
///     vec![Point3::new(1.0, 0.0, 0.0), Point3::new(1.0, 0.0, 1.0)],
/// ];
/// let curve = BSplineCurve::new(KnotVec::bezier_knot(2), cpts);
///
/// // create extruded curve
/// let surface0 = ExtrudedCurve::by_extrusion(curve, Vector3::unit_z());
///
/// // same curve defined by B-spline description
/// let surface1 = BSplineSurface::new((KnotVec::bezier_knot(2), KnotVec::bezier_knot(1)), spts);
///
/// assert_eq!(surface0.range_tuple(), surface1.range_tuple());
///
/// const N: usize = 10;
/// for i in 0..=N {
///     for j in 0..=N {
///         let u = i as f64 / N as f64;
///         let v = j as f64 / N as f64;
///         assert_near!(
///             surface0.subs(u, v),
///             ParametricSurface::subs(&surface1, u, v)
///         );
///         assert_near!(surface0.uder(u, v), surface1.uder(u, v));
///         assert_near!(surface0.vder(u, v), surface1.vder(u, v));
///         assert_near!(surface0.uuder(u, v), surface1.uuder(u, v));
///         assert_near!(surface0.uvder(u, v), surface1.uvder(u, v));
///         assert_near!(surface0.vvder(u, v), surface1.vvder(u, v));
///         assert_near!(surface0.normal(u, v), surface1.normal(u, v));
///     }
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, SelfSameGeometry)]
pub struct ExtrudedCurve<C, V> {
    curve: C,
    vector: V,
}

/// invertible and transformable geometric element
/// # Examples
/// Curve processing example
/// ```
/// use truck_geometry::prelude::*;
///
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
/// use truck_geometry::prelude::*;
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
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Processor<E, T> {
    entity: E,
    transform: T,
    orientation: bool,
}

/// The composited maps
///
/// # Examples
/// ```
/// use truck_geometry::prelude::*;
///
/// // parameter curve
/// let curve = BSplineCurve::new(
///     KnotVec::bezier_knot(2),
///     vec![
///         Point2::new(1.0, 1.0),
///         Point2::new(1.0, 0.0),
///         Point2::new(0.0, 0.0),
///     ],
/// );
/// // surface
/// let surface = BSplineSurface::new(
///     (KnotVec::bezier_knot(2), KnotVec::bezier_knot(1)),
///     vec![
///         vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0)],
///         vec![Point3::new(0.0, 0.0, 1.0), Point3::new(0.0, 1.0, 1.0)],
///         vec![Point3::new(1.0, 0.0, 1.0), Point3::new(1.0, 1.0, 1.0)],
///     ],
/// );
/// // the composite of parameter curve and surface
/// let pcurve = PCurve::new(curve, surface);
/// assert_eq!(pcurve.range_tuple(), (0.0, 1.0));
///
/// const N: usize = 100;
/// for i in 0..=N {
///     let t = i as f64 / N as f64;
///     assert_near!(
///         pcurve.subs(t),
///         Point3::new(
///             (1.0 - t * t) * (1.0 - t * t),
///             (1.0 - t) * (1.0 - t),
///             1.0 - t * t * t * t,
///         ),
///     );
///     assert_near!(
///         pcurve.der(t),
///         Vector3::new(4.0 * t * (t * t - 1.0), 2.0 * (t - 1.0), -4.0 * t * t * t,),
///     );
///     assert_near!(
///         pcurve.der2(t),
///         Vector3::new(4.0 * (3.0 * t * t - 1.0), 2.0, -12.0 * t * t,),
///     );
/// }
///
/// let t = 0.675;
/// let pt = pcurve.subs(t);
/// assert_near!(pcurve.search_parameter(pt, None, 100).unwrap(), t);
///
/// let pt = pt + Vector3::new(0.01, 0.06, -0.03);
/// assert!(pcurve.search_parameter(pt, None, 100).is_none());
/// let t = pcurve.search_nearest_parameter(pt, None, 100).unwrap();
/// assert!(pcurve.der(t).dot(pcurve.subs(t) - pt).so_small());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, SelfSameGeometry)]
pub struct PCurve<C, S> {
    curve: C,
    surface: S,
}

/// Intersection curve between two surfaces.
///
/// # Examples
/// ```
/// use std::f64::consts::PI;
/// use truck_geometry::prelude::*;
///
/// // The intersection curve of the two spheres is the unit circle.
/// let sphere0 = Sphere::new(Point3::new(0.0, 0.0, 1.0), f64::sqrt(2.0));
/// let sphere1 = Sphere::new(Point3::new(0.0, 0.0, -1.0), f64::sqrt(2.0));
///
/// // Approximating a semicircle with a parabola
/// let bspcurve = BSplineCurve::new(
///     KnotVec::bezier_knot(2),
///     vec![
///         Point3::new(1.0, 0.0, 0.0),
///         Point3::new(0.0, 2.0, 0.0),
///         Point3::new(-1.0, 0.0, 0.0)
///     ],
/// );
///
/// // Declare an intersection curve
/// let intersection_curve = IntersectionCurve::new(sphere0, sphere1, bspcurve);
///
/// // All points of curve is on the upper half unit circle.
/// for i in 0..=100 {
///     let t = i as f64 / 100.0;
///     let p = intersection_curve.subs(t);
///     assert_near!(p.distance2(Point3::origin()), 1.0);
/// }
///
/// // Get the length of the half unit circle by Simpson's rule.
/// let coef = |i: usize| if matches!(i, 0 | 100) { 1.0 } else { 2.0 };
/// let sum = (0..=100).fold(0.0, |sum, i| {
///     let t = i as f64 / 100.0;
///     sum + intersection_curve.der(t).magnitude() * coef(i)
/// });
/// let length = sum / 100.0 / 2.0;
/// assert!(f64::abs(length - PI) < 1.0e-4 * PI);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SelfSameGeometry)]
pub struct IntersectionCurve<C, S0, S1> {
    surface0: S0,
    surface1: S1,
    leader: C,
}

/// trimmed curve for parametric curve
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, SelfSameGeometry)]
pub struct TrimmedCurve<C> {
    curve: C,
    range: (f64, f64),
}

/// homotopy surface connecting two curves.
///
/// # Examples
/// ```
/// use truck_geometry::prelude::*;
///
/// // create homotopy between two lines
/// let line0 = Line(Point3::new(-1.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
/// let line1 = Line(Point3::new(0.0, -1.0, 1.0), Point3::new(0.0, 1.0, 1.0));
/// let homotopy = HomotopySurface::new(line0, line1);
///
/// // explicit definition
/// let surface = |u: f64, v: f64| {
///     Point3::new((2.0 * u - 1.0) * (1.0 - v), (2.0 * u - 1.0) * v, v)
/// };
/// let uder = |v: f64| Vector3::new(2.0 * (1.0 - v), 2.0 * v, 0.0);
/// let vder = |u: f64| Vector3::new(1.0 - 2.0 * u, 2.0 * u - 1.0, 1.0);
/// let uvder = Vector3::new(-2.0, 2.0, 0.0);
///
/// // test
/// for i in 0..=10 {
///     for j in 0..=10 {
///         let (u, v) = (i as f64 / 10.0, j as f64 / 10.0);
///         assert_near!(homotopy.subs(u, v), surface(u, v));
///         assert_near!(homotopy.uder(u, v), uder(v));
///         assert_near!(homotopy.vder(u, v), vder(u));
///         assert!(homotopy.uuder(u, v).so_small());
///         assert_near!(homotopy.uvder(u, v), uvder);
///         assert!(homotopy.vvder(u, v).so_small());
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, SelfSameGeometry)]
pub struct HomotopySurface<C0, C1> {
    curve0: C0,
    curve1: C1,
}

mod extruded_curve;
mod homotopy;
mod intersection_curve;
mod pcurve;
mod processor;
mod revolved_curve;
mod trimmied_curve;
