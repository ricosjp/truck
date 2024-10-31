/***********************************/
/*** This module is a prototype. ***/
/***********************************/

#![allow(dead_code)]

use derive_more::*;
use itertools::Itertools;
use std::f64::consts::PI;
use truck_geometry::prelude::*;

#[derive(
    Clone, Debug, ParametricCurve, BoundedCurve, ParameterDivision1D, Cut, From, Invertible,
)]
enum Curve {
    NurbsCurve(NurbsCurve<Vector4>),
    PCurve(PCurve<Line<Point2>, NurbsSurface<Vector4>>),
}

truck_topology::prelude!(Point3, Curve, NurbsSurface<Vector4>);

pub trait FilletCurve: ParametricCurve3D + BoundedCurve + ParameterDivision1D {}
impl<C: ParametricCurve3D + BoundedCurve + ParameterDivision1D> FilletCurve for C {}
pub trait FilletSurface: ParametricSurface3D + SearchNearestParameter<D2> {}

fn circle_arc(
    point: Point3,
    origin: Point3,
    axis: Vector3,
    angle: Rad<f64>,
) -> NurbsCurve<Vector4> {
    let origin = origin + (axis.dot(point - origin)) * axis;
    let diag = point - origin;
    let axis_trsf = Matrix4::from_cols(
        diag.extend(0.0),
        axis.cross(diag).extend(0.0),
        axis.extend(0.0),
        origin.to_homogeneous(),
    );
    let mut unit_curve = unit_circle_arc(angle);
    unit_curve.transform_by(axis_trsf);
    unit_curve
}

fn circle_arc_by_three_points(
    point0: Point3,
    point1: Point3,
    transit: Point3,
) -> NurbsCurve<Vector4> {
    let origin = circum_center(point0, point1, transit);
    let (vec0, vec1) = (point0 - transit, point1 - transit);
    let axis = vec1.cross(vec0).normalize();
    let angle = Rad(PI) - vec0.angle(vec1);
    circle_arc(point0, origin, axis, angle * 2.0)
}

fn circum_center(pt0: Point3, pt1: Point3, pt2: Point3) -> Point3 {
    let (vec0, vec1) = (pt1 - pt0, pt2 - pt0);
    let (a2, ab, b2) = (vec0.dot(vec0), vec0.dot(vec1), vec1.dot(vec1));
    let (det, u, v) = (a2 * b2 - ab * ab, a2 * b2 - ab * b2, a2 * b2 - ab * a2);
    pt0 + u / (2.0 * det) * vec0 + v / (2.0 * det) * vec1
}

fn unit_circle_arc(angle: Rad<f64>) -> NurbsCurve<Vector4> {
    let (cos2, sin2) = (Rad::cos(angle / 2.0), Rad::sin(angle / 2.0));
    let mut curve = NurbsCurve::new(BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            Vector4::new(1.0, 0.0, 0.0, 1.0),
            Vector4::new(cos2, sin2, 0.0, cos2),
            Vector4::new(Rad::cos(angle), Rad::sin(angle), 0.0, 1.0),
        ],
    ));
    curve.add_knot(0.25);
    curve.add_knot(0.5);
    curve.add_knot(0.75);
    curve
}

#[inline(always)]
fn unit_circle_knot_vec() -> KnotVec { KnotVec::uniform_knot(2, 4) }

#[inline(always)]
const fn number_of_cpts_of_unit_circle() -> usize { 6 }

#[test]
fn unit_circle_info() {
    let uc = unit_circle_arc(Rad(PI));
    assert_eq!(uc.knot_vec(), &unit_circle_knot_vec());
    assert_eq!(uc.control_points().len(), number_of_cpts_of_unit_circle());
}

fn interpole_bezier(points: &[Point3]) -> BSplineCurve<Point3> {
    use gaussian_elimination::gaussian_elimination;
    let n = points.len() - 1;
    let rows = (0..=n)
        .map(|i| {
            let t = i as f64 / n as f64;
            let mut s = 1;
            (0..=n)
                .map(|k| {
                    let b = (1.0 - t).powi((n - k) as i32) * t.powi(k as i32) * s as f64;
                    s *= n - k;
                    s /= k + 1;
                    b
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let vec = (0..3)
        .map(|i| {
            let mut rows = rows.clone();
            rows.iter_mut()
                .zip(points)
                .for_each(|(row, point)| row.push(point[i]));
            gaussian_elimination(rows.as_mut_slice())
        })
        .collect::<Vec<_>>();
    let control_points = vec[0]
        .iter()
        .zip(&vec[1])
        .zip(&vec[2])
        .map(|((a, b), c)| (*a, *b, *c).into())
        .collect::<Vec<_>>();
    BSplineCurve::new(KnotVec::bezier_knot(n), control_points)
}

fn composite_line_bezier(
    line: Line<Point2>,
    surface: &NurbsSurface<Vector4>,
) -> BSplineCurve<Point3> {
    let curve = PCurve::new(line, surface);
    let degree = usize::max(surface.udegree(), surface.vdegree());
    let points = (0..=degree)
        .map(|i| curve.subs(i as f64 / degree as f64))
        .collect::<Vec<_>>();
    interpole_bezier(&points)
}

#[derive(Clone, Copy, Debug)]
struct RelaySphere {
    // center of fillet circle
    center: Point3,
    // contact point of sphere and surface0, 3d coordinate and parameter
    contact0: (Point3, Point2),
    // contact point of sphere and surface1, 3d coordinate and parameter
    contact1: (Point3, Point2),
    // transit of fillet circle
    transit: Point3,
}

impl RelaySphere {
    /// contact point on planes and sphere with radius `radius`.
    /// Returns `(center, contact_point0, contact_point1)`.
    fn contact_point(
        // point and derivation
        point_on_curve: (Point3, Vector3),
        // origin and normal
        plane0: (Point3, Vector3),
        // origin and normal
        plane1: (Point3, Vector3),
        radius: f64,
    ) -> (Point3, Point3, Point3) {
        let ((p, der), (p0, n0), (p1, n1)) = (point_on_curve, plane0, plane1);
        let n = n0.cross(n1);
        let sign = f64::signum(n.dot(der));
        let mat = Matrix3::from_cols(n, n0, n1).transpose();
        let vec = Vector3::new(
            n.dot(p.to_vec()),
            n0.dot(p0.to_vec()) - sign * radius,
            n1.dot(p1.to_vec()) - sign * radius,
        );
        let center = Point3::from_vec(mat.invert().unwrap() * vec);
        let q0 = center + sign * radius * n0;
        let q1 = center + sign * radius * n1;
        (center, q0, q1)
    }

    fn next_point(
        surface: &(impl ParametricSurface3D + SearchParameter<D2, Point = Point3>),
        (u, v): (f64, f64),
        (p, q): (Point3, Point3),
    ) -> (Point3, (f64, f64)) {
        let uder = surface.uder(u, v);
        let vder = surface.vder(u, v);
        let d = q - p;
        let uu = uder.dot(uder);
        let uv = uder.dot(vder);
        let vv = vder.dot(vder);
        let mat = Matrix2::new(uu, uv, uv, vv);
        let vec = Vector2::new(uder.dot(d), vder.dot(d));
        let del = mat.invert().unwrap() * vec;
        let (u, v) = (u + del.x, v + del.y);
        (surface.subs(u, v), (u, v))
    }

    fn generate(
        point_on_curve: (Point3, Vector3),
        surface0: &(impl ParametricSurface3D + SearchParameter<D2, Point = Point3>),
        surface1: &(impl ParametricSurface3D + SearchParameter<D2, Point = Point3>),
        radius: f64,
    ) -> Option<Self> {
        let (p, der) = point_on_curve;
        let (mut p0, mut p1) = (p, p);
        let (mut u0, mut v0) = surface0.search_parameter(p0, None, 10)?;
        let (mut u1, mut v1) = surface1.search_parameter(p1, None, 10)?;
        let mut center = Point3::origin();
        for _ in 0..100 {
            let (n0, n1) = (surface0.normal(u0, v0), surface1.normal(u1, v1));
            let (c, q0, q1) = Self::contact_point((p, der), (p0, n0), (p1, n1), radius);
            if p0.near(&q0) && p1.near(&q1) {
                center = c;
                break;
            } else {
                (p0, (u0, v0)) = Self::next_point(surface0, (u0, v0), (p0, q0));
                (p1, (u1, v1)) = Self::next_point(surface1, (u1, v1), (p1, q1));
            }
        }
        Some(Self {
            center,
            contact0: (p0, (u0, v0).into()),
            contact1: (p1, (u1, v1).into()),
            transit: center + radius * (p - center).normalize(),
        })
    }

    fn fillet_wire(self) -> NurbsCurve<Vector4> {
        circle_arc_by_three_points(self.contact0.0, self.contact1.0, self.transit)
    }
}

fn relay_spheres(
    surface0: &NurbsSurface<Vector4>,
    surface1: &NurbsSurface<Vector4>,
    curve: &impl FilletCurve,
    division: usize,
    radius: impl Fn(f64) -> f64,
) -> Option<Vec<RelaySphere>> {
    let (t0, t1) = curve.range_tuple();
    let generator = move |i: isize| {
        let a = i as f64 / division as f64;
        let t = (1.0 - a) * t0 + a * t1;
        RelaySphere::generate((curve.subs(t), curve.der(t)), surface0, surface1, radius(a))
    };
    (-1..=(division as isize + 1)).map(generator).collect()
}

fn almost_fillet_patch(rs0: RelaySphere, rs1: RelaySphere) -> BSplineSurface<Vector4> {
    let nurbs0 = rs0.fillet_wire();
    let nurbs1 = rs1.fillet_wire();
    let knot_vecs = (KnotVec::bezier_knot(1), nurbs0.knot_vec().clone());
    let control_points = vec![
        nurbs0.control_points().clone(),
        nurbs1.control_points().clone(),
    ];
    BSplineSurface::new(knot_vecs, control_points)
}

fn expand_fillet(
    relay_spheres: &[RelaySphere],
    surface0: &NurbsSurface<Vector4>,
    surface1: &NurbsSurface<Vector4>,
) -> NurbsSurface<Vector4> {
    const LEN: usize = number_of_cpts_of_unit_circle();
    let mut collectors = vec![CurveCollector::<BSplineCurve<_>>::Singleton; LEN];
    relay_spheres.windows(2).enumerate().for_each(|(n, u)| {
        let mut homo_surface = almost_fillet_patch(u[0], u[1]);
        let mut bezier0 = {
            let line0 = Line(u[0].contact0.1, u[1].contact0.1);
            composite_line_bezier(line0, surface0)
        };
        let mut bezier1 = {
            let line1 = Line(u[0].contact1.1, u[1].contact1.1);
            composite_line_bezier(line1, surface1)
        };
        BSplineCurve::syncro_degree(&mut bezier0, &mut bezier1);
        for _ in 1..bezier0.degree() {
            homo_surface.elevate_udegree();
        }

        let (cpts0, cpts1) = (bezier0.control_points(), bezier1.control_points());
        let iter = cpts0.iter().zip(cpts1).enumerate();
        iter.for_each(|(i, (p0, p1))| {
            *homo_surface.control_point_mut(i, 0) = p0.to_homogeneous();
            let n = homo_surface.control_points()[i].len();
            *homo_surface.control_point_mut(i, n - 1) = p1.to_homogeneous();
        });

        let connect = move |(i, collector): (usize, &mut CurveCollector<_>)| {
            let mut curve = homo_surface.row_curve(i);
            curve.knot_translate(n as f64);
            collector.concat(&curve);
        };
        collectors.iter_mut().enumerate().for_each(connect)
    });
    let curves = collectors
        .into_iter()
        .map(CurveCollector::unwrap)
        .collect::<Vec<_>>();
    let vknot_vec = curves[0].knot_vec().clone();
    let control_points = curves
        .into_iter()
        .map(|curve| curve.destruct().1)
        .collect::<Vec<_>>();

    let knot_vecs = (unit_circle_knot_vec(), vknot_vec);
    let mut bsp_surface = BSplineSurface::new(knot_vecs, control_points);
    bsp_surface.knot_normalize();
    NurbsSurface::new(bsp_surface)
}

// Orientation of `curve` should be compatible with orientation of `surface0`.
fn rolling_ball_fillet_surface(
    surface0: &NurbsSurface<Vector4>,
    surface1: &NurbsSurface<Vector4>,
    curve: &impl FilletCurve,
    division: usize,
    radius: impl Fn(f64) -> f64,
) -> Option<NurbsSurface<Vector4>> {
    let relay_spheres = relay_spheres(surface0, surface1, curve, division, radius)?;
    Some(expand_fillet(&relay_spheres, surface0, surface1))
}

fn simple_fillet(
    face0: &Face,
    face1: &Face,
    radius: impl Fn(f64) -> f64,
) -> Option<(Face, Face, Face)> {
    let surface0 = face0.oriented_surface();
    let surface1 = face1.oriented_surface();

    let boundaries0 = face0.boundaries();
    let boundaries1 = face1.boundaries();

    let (edge, boundary0, boundary1) = boundaries0.iter().find_map(|boundary0| {
        boundaries1.iter().find_map(|boundary1| {
            let edge = boundary0
                .iter()
                .find(|edge0| boundary1.edge_iter().any(|edge1| edge0.is_same(&edge1)))?;
            Some((edge, boundary0, boundary1))
        })
    })?;

    let curve = edge.oriented_curve();
    let fillet_surface = rolling_ball_fillet_surface(&surface0, &surface1, &curve, 5, radius)?;

    let mut bezier0 = fillet_surface.column_curve(0);
    let (front_edge, back_edge) =
        boundary0
            .iter()
            .circular_tuple_windows()
            .find_map(|(x, edge0, y)| match edge.is_same(&edge0) {
                true => Some((x, y)),
                false => None,
            })?;
    let front_curve = front_edge.curve();
    let hint = match front_edge.orientation() {
        true => front_curve.range_tuple().1,
        false => front_curve.range_tuple().0,
    };
    let (t0, t1) =
        algo::curve::search_intersection_parameter(&bezier0, &front_curve, (0.0, hint), 100)?;
    let v0 = Vertex::new(bezier0.subs(t0));
    bezier0 = bezier0.cut(t0);
    let (front_edge0, front_edge1) = front_edge.cut_with_parameter(&v0, t1)?;
    let new_front_edge = match edge.orientation() {
        true => front_edge0,
        false => front_edge1,
    };
    let back_curve = back_edge.curve();
    let hint = match back_edge.orientation() {
        true => back_curve.range_tuple().0,
        false => back_curve.range_tuple().1,
    };
    let (t0, t1) =
        algo::curve::search_intersection_parameter(&bezier0, &back_curve, (1.0, hint), 100)?;
    let v1 = Vertex::new(bezier0.subs(t0));
    bezier0.cut(t0);
    let (back_edge0, back_edge1) = back_edge.cut_with_parameter(&v1, t1)?;
    let new_back_edge = match edge.orientation() {
        true => back_edge1,
        false => back_edge0,
    };
    let fillet_edge0 = Edge::new(&v0, &v1, bezier0.into());

    let new_boundaries0 = face0
        .absolute_boundaries()
        .iter()
        .cloned()
        .map(|mut boundary| {
            let Some(idx) = boundary.iter().position(|edge0| edge0.is_same(front_edge)) else {
                return boundary;
            };
            boundary.rotate_left(idx);
            if face0.orientation() {
                boundary[0] = new_front_edge.clone();
                boundary[1] = fillet_edge0.clone();
                boundary[2] = new_back_edge.clone();
            } else {
                boundary[0] = new_back_edge.inverse();
                boundary[1] = fillet_edge0.inverse();
                boundary[2] = new_front_edge.inverse();
            }
            boundary
        })
        .collect::<Vec<_>>();
    let mut new_face0 = Face::new(new_boundaries0, face0.surface());
    if !face0.orientation() {
        new_face0.invert();
    }

    let mut bezier1 = fillet_surface.column_curve(fillet_surface.control_points().len() - 1);
    bezier1.invert();
    let (front_edge, back_edge) =
        boundary1
            .iter()
            .circular_tuple_windows()
            .find_map(|(x, edge0, y)| match edge.is_same(&edge0) {
                true => Some((x, y)),
                false => None,
            })?;
    let front_curve = front_edge.curve();
    let hint = match front_edge.orientation() {
        true => front_curve.range_tuple().1,
        false => front_curve.range_tuple().0,
    };
    let (t0, t1) =
        algo::curve::search_intersection_parameter(&bezier1, &front_curve, (0.0, hint), 100)?;
    let v0 = Vertex::new(bezier1.subs(t0));
    bezier1 = bezier1.cut(t0);
    let (front_edge0, front_edge1) = front_edge.cut_with_parameter(&v0, t1)?;
    let new_front_edge = match edge.orientation() {
        true => front_edge0,
        false => front_edge1,
    };
    let back_curve = back_edge.curve();
    let hint = match back_edge.orientation() {
        true => back_curve.range_tuple().0,
        false => back_curve.range_tuple().1,
    };
    let (t0, t1) =
        algo::curve::search_intersection_parameter(&bezier1, &back_curve, (1.0, hint), 100)?;
    let v1 = Vertex::new(bezier1.subs(t0));
    bezier1.cut(t0);
    let (back_edge0, back_edge1) = back_edge.cut_with_parameter(&v1, t1)?;
    let new_back_edge = match edge.orientation() {
        true => back_edge1,
        false => back_edge0,
    };
    let fillet_edge1 = Edge::new(&v0, &v1, bezier1.into());

    let new_boundaries1 = face1
        .absolute_boundaries()
        .iter()
        .cloned()
        .map(|mut boundary| {
            let Some(idx) = boundary.iter().position(|edge0| edge0.is_same(front_edge)) else {
                return boundary;
            };
            boundary.rotate_left(idx);
            if face1.orientation() {
                boundary[0] = new_front_edge.clone();
                boundary[1] = fillet_edge1.clone();
                boundary[2] = new_back_edge.clone();
            } else {
                boundary[0] = new_back_edge.inverse();
                boundary[1] = fillet_edge1.inverse();
                boundary[2] = new_front_edge.inverse();
            }
            boundary
        })
        .collect::<Vec<_>>();
    let mut new_face1 = Face::new(new_boundaries1, face1.surface());
    if !face1.orientation() {
        new_face1.invert();
    }

    let (v0, v1) = fillet_edge0.ends();
    let (v2, v3) = fillet_edge1.ends();

    let uv0 = fillet_surface.search_parameter(v0.point(), (0.0, 0.0), 100)?;
    let uv1 = fillet_surface.search_parameter(v1.point(), (0.0, 1.0), 100)?;
    let uv2 = fillet_surface.search_parameter(v2.point(), (1.0, 1.0), 100)?;
    let uv3 = fillet_surface.search_parameter(v3.point(), (1.0, 1.0), 100)?;

    let edge0 = Edge::new(v0, v3, PCurve::new(Line(uv0.into(), uv3.into()), fillet_surface.clone()).into());
    let edge1 = Edge::new(v2, v1, PCurve::new(Line(uv2.into(), uv1.into()), fillet_surface.clone()).into());

    let fillet_boundary = Wire::from_iter([
        fillet_edge0.inverse(),
        edge0,
        fillet_edge1.inverse(),
        edge1,
    ]);
    let fillet = Face::new(vec![fillet_boundary], fillet_surface);

    Some((new_face0, new_face1, fillet))
}

#[test]
fn create_fillet_surface() {
    use truck_meshalgo::prelude::*;
    let surface0 = BSplineSurface::new(
        (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)),
        vec![
            vec![
                Point3::new(0.2, 0.0, 0.0),
                Point3::new(0.0, 0.5, 0.0),
                Point3::new(-0.2, 1.0, 0.0),
            ],
            vec![
                Point3::new(0.5, 0.0, 0.1),
                Point3::new(0.5, 0.5, 0.0),
                Point3::new(0.5, 1.0, 0.2),
            ],
            vec![
                Point3::new(1.0, 0.0, 0.3),
                Point3::new(1.0, 0.5, 0.3),
                Point3::new(1.0, 1.0, 0.1),
            ],
        ],
    )
    .into();
    let surface1 = BSplineSurface::new(
        (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)),
        vec![
            vec![
                Point3::new(0.2, 0.0, 0.0),
                Point3::new(0.0, 0.0, -0.5),
                Point3::new(-0.2, 0.0, -1.0),
            ],
            vec![
                Point3::new(0.0, 0.5, 0.0),
                Point3::new(0.0, 0.5, -0.5),
                Point3::new(0.0, 0.5, -1.0),
            ],
            vec![
                Point3::new(-0.2, 1.0, 0.0),
                Point3::new(0.2, 1.0, -0.5),
                Point3::new(0.0, 1.0, -1.0),
            ],
        ],
    )
    .into();

    let mut poly0 =
        StructuredMesh::from_surface(&surface0, ((0.0, 1.0), (0.0, 1.0)), 0.001).destruct();
    let poly1 = StructuredMesh::from_surface(&surface1, ((0.0, 1.0), (0.0, 1.0)), 0.001).destruct();
    poly0.merge(poly1);

    let file0 = std::fs::File::create("edged.obj").unwrap();
    obj::write(&poly0, file0).unwrap();

    let curve = BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            Point3::new(-0.2, 1.0, 0.0),
            Point3::new(0.0, 0.5, 0.0),
            Point3::new(0.2, 0.0, 0.0),
        ],
    );
    let surface = rolling_ball_fillet_surface(&surface0, &surface1, &curve, 5, |_| 0.3).unwrap();
    let poly = StructuredMesh::from_surface(&surface, ((0.0, 1.0), (0.0, 1.0)), 0.01).destruct();
    let file1 = std::fs::File::create("fillet.obj").unwrap();
    obj::write(&poly, file1).unwrap();
}

#[test]
fn create_simple_fillet() {
    use truck_meshalgo::prelude::*;
    let surface0: NurbsSurface<_> = BSplineSurface::new(
        (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)),
        vec![
            vec![
                Point3::new(-1.0, 0.0, 0.0),
                Point3::new(-1.0, 0.5, 0.0),
                Point3::new(-1.0, 1.0, 1.0),
            ],
            vec![
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(0.0, 0.5, 0.0),
                Point3::new(0.0, 1.0, 1.0),
            ],
            vec![
                Point3::new(1.0, 0.0, 0.0),
                Point3::new(1.0, 0.5, 0.0),
                Point3::new(1.0, 1.0, 1.0),
            ],
        ],
    )
    .into();
    let surface1: NurbsSurface<_> = BSplineSurface::new(
        (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)),
        vec![
            vec![
                Point3::new(1.0, 0.0, 0.0),
                Point3::new(1.0, 0.0, -0.5),
                Point3::new(1.0, 1.0, -1.0),
            ],
            vec![
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(0.0, 0.5, -0.5),
                Point3::new(0.0, 1.0, -1.0),
            ],
            vec![
                Point3::new(-1.0, 0.0, 0.0),
                Point3::new(-1.0, 0.0, -0.5),
                Point3::new(-1.0, 1.0, -1.0),
            ],
        ],
    )
    .into();

    let v = Vertex::news(&[
        Point3::new(-1.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(-1.0, 1.0, 1.0),
        Point3::new(-1.0, 1.0, -1.0),
        Point3::new(1.0, 1.0, -1.0),
    ]);

    let boundary0 = surface0.splitted_boundary();
    let boundary1 = surface1.splitted_boundary();

    let wire0: Wire = [
        Edge::new(&v[0], &v[1], boundary0[0].clone().into()),
        Edge::new(&v[1], &v[2], boundary0[1].clone().into()),
        Edge::new(&v[2], &v[3], boundary0[2].clone().into()),
        Edge::new(&v[3], &v[0], boundary0[3].clone().into()),
    ]
    .into();

    let wire1: Wire = [
        wire0[0].inverse(),
        Edge::new(&v[0], &v[4], boundary1[1].clone().into()),
        Edge::new(&v[4], &v[5], boundary1[2].clone().into()),
        Edge::new(&v[5], &v[1], boundary1[3].clone().into()),
    ]
    .into();

    let face0 = Face::new(vec![wire0], surface0);
    let face1 = Face::new(vec![wire1], surface1);
    
    let shell: Shell = [face0.clone(), face1.clone()].into();
    let poly = shell.robust_triangulation(0.001).to_polygon();
    let file = std::fs::File::create("edged-shell.obj").unwrap();
    obj::write(&poly, file).unwrap();

    let (face0, face1, fillet) = simple_fillet(&face0, &face1, |_| 0.3).unwrap();

    let shell: Shell = [face0, face1, fillet].into();
    let poly = shell.robust_triangulation(0.001).to_polygon();
    let file = std::fs::File::create("fillet-shell.obj").unwrap();
    obj::write(&poly, file).unwrap();
}

// https://the-algorithms.com/algorithm/gaussian-elimination?lang=rust
mod gaussian_elimination {

    // Gaussian Elimination of Quadratic Matrices
    // Takes an augmented matrix as input, returns vector of results
    // Wikipedia reference: augmented matrix: https://en.wikipedia.org/wiki/Augmented_matrix
    // Wikipedia reference: algorithm: https://en.wikipedia.org/wiki/Gaussian_elimination

    pub fn gaussian_elimination(matrix: &mut [Vec<f64>]) -> Vec<f64> {
        let size = matrix.len();
        assert_eq!(size, matrix[0].len() - 1);

        for i in 0..size - 1 {
            for j in i..size - 1 {
                echelon(matrix, i, j);
            }
        }

        for i in (1..size).rev() {
            eliminate(matrix, i);
        }

        // Disable cargo clippy warnings about needless range loops.
        // Checking the diagonal like this is simpler than any alternative.
        #[allow(clippy::needless_range_loop)]
        for i in 0..size {
            if matrix[i][i] == 0f64 {
                println!("Infinitely many solutions");
            }
        }

        let mut result: Vec<f64> = vec![0f64; size];
        for i in 0..size {
            result[i] = matrix[i][size] / matrix[i][i];
        }
        result
    }

    fn echelon(matrix: &mut [Vec<f64>], i: usize, j: usize) {
        let size = matrix.len();
        if matrix[i][i] == 0f64 {
        } else {
            let factor = matrix[j + 1][i] / matrix[i][i];
            (i..size + 1).for_each(|k| {
                matrix[j + 1][k] -= factor * matrix[i][k];
            });
        }
    }

    fn eliminate(matrix: &mut [Vec<f64>], i: usize) {
        let size = matrix.len();
        if matrix[i][i] == 0f64 {
        } else {
            for j in (1..i + 1).rev() {
                let factor = matrix[j - 1][i] / matrix[i][i];
                for k in (0..size + 1).rev() {
                    matrix[j - 1][k] -= factor * matrix[i][k];
                }
            }
        }
    }
}
