/***********************************/
/*** This module is a prototype. ***/
/***********************************/

#![allow(dead_code)]

use algo::curve::search_intersection_parameter;
use derive_more::*;
use itertools::Itertools;
use std::f64::consts::PI;
use truck_geometry::prelude::*;

#[cfg(test)]
use truck_meshalgo::prelude::*;

type PCurveLns = PCurve<Line<Point2>, NurbsSurface<Vector4>>;

#[derive(
    Clone,
    Debug,
    ParametricCurve,
    BoundedCurve,
    ParameterDivision1D,
    Cut,
    From,
    Invertible,
    SearchParameterD1,
)]
enum Curve {
    NurbsCurve(NurbsCurve<Vector4>),
    PCurve(PCurveLns),
    IntersectionCurve(IntersectionCurve<PCurveLns, NurbsSurface<Vector4>>),
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
    (w0, w1): (f64, f64),
) -> NurbsCurve<Vector4> {
    let origin = origin + (axis.dot(point - origin)) * axis;
    let diag = point - origin;
    let axis_trsf = Matrix4::from_cols(
        diag.extend(0.0),
        axis.cross(diag).extend(0.0),
        axis.extend(0.0),
        origin.to_homogeneous(),
    );
    let mut unit_curve = unit_circle_arc(angle, w0, w1);
    unit_curve.transform_by(axis_trsf);
    unit_curve
}

fn circle_arc_by_three_points(
    point0: Vector4,
    point1: Vector4,
    transit: Point3,
) -> NurbsCurve<Vector4> {
    let (w0, w1) = (point0.w, point1.w);
    let point0 = Point3::from_homogeneous(point0);
    let point1 = Point3::from_homogeneous(point1);
    let origin = circum_center(point0, point1, transit);
    let (vec0, vec1) = (point0 - transit, point1 - transit);
    let axis = vec1.cross(vec0).normalize();
    let angle = Rad(PI) - vec0.angle(vec1);
    circle_arc(point0, origin, axis, angle * 2.0, (w0, w1))
}

fn circum_center(pt0: Point3, pt1: Point3, pt2: Point3) -> Point3 {
    let (vec0, vec1) = (pt1 - pt0, pt2 - pt0);
    let (a2, ab, b2) = (vec0.dot(vec0), vec0.dot(vec1), vec1.dot(vec1));
    let (det, u, v) = (a2 * b2 - ab * ab, a2 * b2 - ab * b2, a2 * b2 - ab * a2);
    pt0 + u / (2.0 * det) * vec0 + v / (2.0 * det) * vec1
}

fn unit_circle_arc(angle: Rad<f64>, w0: f64, w1: f64) -> NurbsCurve<Vector4> {
    use truck_base::newton::{self, CalcOutput};
    let p0 = Vector3::new(w0, 0.0, w0);
    let p1 = Vector3::new(Rad::cos(angle), Rad::sin(angle), 1.0) * w1;
    let pt = Vector3::new(Rad::cos(angle / 2.0), Rad::sin(angle / 2.0), 1.0);
    let d = p1 - p0;
    let function = |Vector2 { x, y }| CalcOutput {
        value: Vector2::new(
            x * x + y * y - 0.5,
            d.x * x + d.y * y + d.z / f64::sqrt(2.0),
        ),
        derivation: Matrix2::new(2.0 * x, d.x, 2.0 * y, d.y),
    };
    let Vector2 { x, y } = newton::solve(function, pt.truncate(), 100).unwrap();

    let n = Vector3::new(x, y, 1.0 / f64::sqrt(2.0));
    let y_axis = Vector3::new(-n.x, -n.y, n.z);
    let x_axis = y_axis.cross(n);
    let parab_apex = n * n.dot(p0);

    let xt = (p0 + p1).dot(x_axis) / 2.0;
    let d0 = p0 - parab_apex;
    let (x0, y0) = (d0.dot(x_axis), d0.dot(y_axis));
    let yt = y0 / (x0 * x0) * xt * xt;
    let pt = parab_apex + xt * x_axis + yt * y_axis;

    let c = 2.0 * pt - (p0 + p1) / 2.0;
    let mut curve = BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            Vector4::new(p0.x, p0.y, 0.0, p0.z),
            Vector4::new(c.x, c.y, 0.0, c.z),
            Vector4::new(p1.x, p1.y, 0.0, p1.z),
        ],
    );

    curve.add_knot(0.25);
    curve.add_knot(0.5);
    curve.add_knot(0.75);
    NurbsCurve::new(curve)
}

#[inline(always)]
fn unit_circle_knot_vec() -> KnotVec { KnotVec::uniform_knot(2, 4) }

#[inline(always)]
const fn number_of_cpts_of_unit_circle() -> usize { 6 }

#[test]
fn unit_circle_info() {
    let uc = unit_circle_arc(Rad(PI), 1.0, 1.0);
    assert_eq!(uc.knot_vec(), &unit_circle_knot_vec());
    assert_eq!(uc.control_points().len(), number_of_cpts_of_unit_circle());
}

#[cfg(test)]
proptest::proptest! {
    #[test]
    fn test_unit_circle(
        angle in 0.1f64..(2.0 * PI - 0.01),
        w0 in 0.1f64..5.0,
        w1 in 0.1f64..5.0,
    ) {
        let uc = unit_circle_arc(Rad(angle), w0, w1);
        const N: usize = 10;
        for i in 0..=N {
            let t = i as f64 / N as f64;
            let p = uc.subs(t).to_vec();
            let v = uc.der(t);
            assert_near!(p.magnitude2(), 1.0, "{w0} {w1} {p:?}");
            assert!(p.z.so_small2());
            assert!(p.x * v.y - p.y * v.x > 0.0, "minus area {:?}", uc.control_point(1));
        }
        assert_near!(uc.subs(0.0), Point3::new(1.0, 0.0, 0.0));
        assert_near!(uc.subs(1.0), Point3::new(f64::cos(angle), f64::sin(angle), 0.0));
    }
}

fn interpole_bezier(points: &[Vector4]) -> BSplineCurve<Vector4> {
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
    let vec = (0..4)
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
        .zip(&vec[3])
        .map(|(((a, b), c), d)| (*a, *b, *c, *d).into())
        .collect::<Vec<_>>();
    BSplineCurve::new(KnotVec::bezier_knot(n), control_points)
}

fn composite_line_bezier(
    line: Line<Point2>,
    surface: &NurbsSurface<Vector4>,
) -> BSplineCurve<Vector4> {
    let curve = PCurve::new(line, surface.non_rationalized());
    let degree = surface.udegree() + surface.vdegree();
    let points = (0..=degree)
        .map(|i| curve.subs(i as f64 / degree as f64))
        .collect::<Vec<_>>();
    let res = interpole_bezier(&points);

    res
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
        circle_arc_by_three_points(
            self.contact0.0.to_homogeneous(),
            self.contact1.0.to_homogeneous(),
            self.transit,
        )
    }
}

fn relay_spheres(
    surface0: &NurbsSurface<Vector4>,
    surface1: &NurbsSurface<Vector4>,
    curve: &impl FilletCurve,
    division: usize,
    radius: impl Fn(f64) -> f64,
    extend: bool,
) -> Option<Vec<RelaySphere>> {
    let (t0, t1) = curve.range_tuple();
    let generator = move |i: isize| {
        let a = i as f64 / division as f64;
        let t = (1.0 - a) * t0 + a * t1;
        RelaySphere::generate((curve.subs(t), curve.der(t)), surface0, surface1, radius(a))
    };
    let range = match extend {
        true => -1..=(division as isize + 1),
        false => 0..=division as isize,
    };
    range.map(generator).collect()
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
        let transit_line = Line(u[0].transit, u[1].transit);
        let mut bezier0 = {
            let line0 = Line(u[0].contact0.1, u[1].contact0.1);
            composite_line_bezier(line0, surface0)
        };
        let mut bezier1 = {
            let line1 = Line(u[0].contact1.1, u[1].contact1.1);
            composite_line_bezier(line1, surface1)
        };
        BSplineCurve::syncro_degree(&mut bezier0, &mut bezier1);

        let (cpts0, cpts1) = (bezier0.control_points(), bezier1.control_points());
        let iter = cpts0.iter().zip(cpts1).enumerate();
        let fillet_wires = iter
            .map(|(i, (p0, p1))| {
                let t = i as f64 / (cpts0.len() - 1) as f64;
                let transit = transit_line.subs(t);
                circle_arc_by_three_points(*p0, *p1, transit)
            })
            .collect::<Vec<_>>();
        let homo_surface = BSplineSurface::new(
            (bezier0.knot_vec().clone(), unit_circle_knot_vec()),
            fillet_wires
                .into_iter()
                .map(|wire| wire.into_non_rationalized().destruct().1)
                .collect(),
        );

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
    extend: bool,
) -> Option<NurbsSurface<Vector4>> {
    let relay_spheres = relay_spheres(surface0, surface1, curve, division, radius, extend)?;
    Some(expand_fillet(&relay_spheres, surface0, surface1))
}

fn find_adjacent_edge(face: &Face, edge_id: EdgeID) -> Option<(Edge, Edge)> {
    face.boundary_iters()
        .into_iter()
        .flat_map(|boundary_iter| boundary_iter.circular_tuple_windows())
        .find(|(_, edge, _)| edge.id() == edge_id)
        .map(|(x, _, y)| (x, y))
}

fn take_ori<T>(ori: bool, (a, b): (T, T)) -> T {
    match ori {
        true => a,
        false => b,
    }
}

fn cut_face_by_bezier(
    face: &Face,
    mut bezier: NurbsCurve<Vector4>,
    filleted_edge_id: EdgeID,
) -> Option<(Face, Edge)> {
    let (front_edge, back_edge) = find_adjacent_edge(face, filleted_edge_id)?;

    let new_front_edge = {
        let curve = front_edge.curve();
        let hint = algo::curve::presearch_closest_point(
            &bezier,
            &curve,
            (bezier.range_tuple(), curve.range_tuple()),
            10,
        );
        let (t0, t1) = search_intersection_parameter(&bezier, &curve, hint, 100)?;
        let v0 = Vertex::new(bezier.subs(t0));
        bezier = bezier.cut(t0);
        front_edge.cut_with_parameter(&v0, t1)?.0
    };

    let new_back_edge = {
        let curve = back_edge.curve();
        let hint = algo::curve::presearch_closest_point(
            &bezier,
            &curve,
            (bezier.range_tuple(), curve.range_tuple()),
            10,
        );
        let (t0, t1) = search_intersection_parameter(&bezier, &curve, hint, 100)?;
        let v1 = Vertex::new(bezier.subs(t0));
        bezier.cut(t0);
        back_edge.cut_with_parameter(&v1, t1)?.1
    };

    let fillet_edge = Edge::new(
        &new_front_edge.back(),
        &new_back_edge.front(),
        bezier.into(),
    );
    let new_boundaries = face
        .absolute_boundaries()
        .iter()
        .cloned()
        .map(|mut boundary| {
            if let Some(idx) = boundary.iter().position(|edge0| edge0.is_same(&front_edge)) {
                let len = boundary.len();
                if face.orientation() {
                    boundary[idx] = new_front_edge.clone();
                    boundary[(idx + 1) % len] = fillet_edge.clone();
                    boundary[(idx + 2) % len] = new_back_edge.clone();
                } else {
                    boundary[(len + idx - 2) % len] = new_back_edge.inverse();
                    boundary[(len + idx - 1) % len] = fillet_edge.inverse();
                    boundary[idx] = new_front_edge.inverse();
                }
            }
            boundary
        })
        .collect::<Vec<_>>();
    let mut new_face = Face::new(new_boundaries, face.surface());
    if !face.orientation() {
        new_face.invert();
    }
    Some((new_face, fillet_edge))
}

fn create_pcurve_edge(
    (v0, hint0): (&Vertex, (f64, f64)),
    (v1, hint1): (&Vertex, (f64, f64)),
    fillet_surface: &NurbsSurface<Vector4>,
) -> Option<Edge> {
    let uv0 = fillet_surface.search_parameter(v0.point(), hint0, 100)?;
    let uv1 = fillet_surface.search_parameter(v1.point(), hint1, 100)?;
    let curve = PCurve::new(Line(uv0.into(), uv1.into()), fillet_surface.clone());
    Some(Edge::new(v0, v1, curve.into()))
}

fn simple_fillet(
    face0: &Face,
    face1: &Face,
    filleted_edge_id: EdgeID,
    radius: impl Fn(f64) -> f64,
    fillet_division: usize,
) -> Option<(Face, Face, Face)> {
    let is_filleted_edge = move |edge: &Edge| edge.id() == filleted_edge_id;
    let filleted_edge = face0.edge_iter().find(is_filleted_edge)?;

    let fillet_surface = {
        let surface0 = face0.oriented_surface();
        let surface1 = face1.oriented_surface();
        let curve = filleted_edge.oriented_curve();
        rolling_ball_fillet_surface(&surface0, &surface1, &curve, fillet_division, radius, true)?
    };

    let (new_face0, fillet_edge0) = {
        let bezier = fillet_surface.column_curve(0);
        cut_face_by_bezier(&face0, bezier, filleted_edge.id())?
    };
    let (new_face1, fillet_edge1) = {
        let bezier = fillet_surface.column_curve(fillet_surface.control_points().len() - 1);
        cut_face_by_bezier(&face1, bezier.inverse(), filleted_edge.id())?
    };

    let ((v0, v1), (v2, v3)) = (fillet_edge0.ends(), fillet_edge1.ends());
    let edge0 = create_pcurve_edge((&v0, (0.0, 0.0)), (&v3, (1.0, 0.0)), &fillet_surface)?;
    let edge1 = create_pcurve_edge((&v2, (1.0, 1.0)), (&v1, (0.0, 1.0)), &fillet_surface)?;
    let fillet = {
        let fillet_boundary = [fillet_edge0.inverse(), edge0, fillet_edge1.inverse(), edge1];
        Face::new(vec![fillet_boundary.into()], fillet_surface)
    };

    Some((new_face0, new_face1, fillet))
}

fn create_new_side(
    side: &Face,
    fillet_edge: &Edge,
    corner_vertex_id: VertexID,
    left_face_front_edge: &Edge,
    right_face_back_edge: &Edge,
) -> Option<Face> {
    let (boundary_idx, edge_idx) = side.boundary_iters().into_iter().enumerate().find_map(
        |(boundary_idx, boundary_iter)| {
            boundary_iter
                .enumerate()
                .find(|(_, edge)| edge.back().id() == corner_vertex_id)
                .map(move |(edge_idx, _)| (boundary_idx, edge_idx))
        },
    )?;
    let new_boundaries = side
        .absolute_boundaries()
        .iter()
        .enumerate()
        .map(|(idx, boundary)| {
            let mut new_boundary = boundary.clone();
            if idx == boundary_idx {
                let len = new_boundary.len();
                if side.orientation() {
                    new_boundary[edge_idx] = right_face_back_edge.inverse();
                    new_boundary[(edge_idx + 1) % len] = left_face_front_edge.inverse();
                    new_boundary.insert(edge_idx + 1, fillet_edge.inverse());
                } else {
                    new_boundary[len - edge_idx - 1] = right_face_back_edge.clone();
                    new_boundary[(2 * len - edge_idx - 2) % len] = left_face_front_edge.clone();
                    new_boundary.insert(len - edge_idx, fillet_edge.clone());
                }
            }
            new_boundary
        })
        .collect();
    let side_surface = Box::new(side.oriented_surface());
    let Curve::PCurve(fillet_curve) = fillet_edge.curve() else {
        return None;
    };
    let fillet_surface = Box::new(fillet_curve.surface().clone());
    let new_curve = IntersectionCurve::new_unchecked(side_surface, fillet_surface, fillet_curve);
    fillet_edge.set_curve(new_curve.into());
    let mut new_face = Face::new(new_boundaries, side.surface());
    if !side.orientation() {
        new_face.invert();
    }
    Some(new_face)
}

fn fillet_with_side(
    face0: &Face,
    face1: &Face,
    filleted_edge_id: EdgeID,
    side0: Option<&Face>,
    side1: Option<&Face>,
    radius: impl Fn(f64) -> f64,
    fillet_division: usize,
) -> Option<(Face, Face, Face, Option<Face>, Option<Face>)> {
    let (new_face0, new_face1, fillet) =
        simple_fillet(face0, face1, filleted_edge_id, radius, fillet_division)?;

    let (front_edge0, back_edge0) = {
        let fillet_edge_id = fillet.absolute_boundaries()[0][0].id();
        find_adjacent_edge(&new_face0, fillet_edge_id)?
    };
    let (front_edge1, back_edge1) = {
        let fillet_edge_id = fillet.absolute_boundaries()[0][2].id();
        find_adjacent_edge(&new_face1, fillet_edge_id)?
    };

    let is_filleted_edge = |edge: &Edge| edge.id() == filleted_edge_id;
    let filleted_edge = face0.edge_iter().find(is_filleted_edge)?;
    let (v0, v1) = filleted_edge.ends();

    let new_side0 = side0.and_then(|side0| {
        let fillet_edge = &fillet.absolute_boundaries()[0][1];
        create_new_side(side0, fillet_edge, v0.id(), &front_edge0, &back_edge1)
    });
    let new_side1 = side1.and_then(|side1| {
        let fillet_edge = &fillet.absolute_boundaries()[0][3];
        create_new_side(side1, &fillet_edge, v1.id(), &front_edge1, &back_edge0)
    });
    Some((new_face0, new_face1, fillet, new_side0, new_side1))
}

#[derive(Clone, Copy, Debug)]
struct FaceBoundaryEdgeIndex {
    face_index: usize,
    boundary_index: usize,
    edge_index: usize,
}

impl From<(usize, usize, usize)> for FaceBoundaryEdgeIndex {
    fn from((face_index, boundary_index, edge_index): (usize, usize, usize)) -> Self {
        Self {
            face_index,
            boundary_index,
            edge_index,
        }
    }
}

fn find_shared_face_with_front_edge(shell: &Shell, wire: &Wire) -> Option<FaceBoundaryEdgeIndex> {
    shell.iter().enumerate().find_map(|(face_idx, face)| {
        let mut boundary_iter = face.boundary_iters().into_iter().enumerate();
        boundary_iter.find_map(|(boundary_idx, boundary_iter)| {
            let mut edge_iter = boundary_iter.circular_tuple_windows().enumerate();
            edge_iter.find_map(|(edge_idx, (edge0, edge1))| {
                match edge0.is_same(&wire[0]) && edge1.is_same(&wire[1]) {
                    true => Some((face_idx, boundary_idx, edge_idx).into()),
                    false => None,
                }
            })
        })
    })
}

fn enumerate_adjacent_faces(
    shell: &Shell,
    wire: &Wire,
    shared_face: FaceBoundaryEdgeIndex,
) -> Option<Vec<FaceBoundaryEdgeIndex>> {
    let iter = wire.edge_iter().map(|edge0| {
        shell.iter().enumerate().find_map(|(face_idx, face)| {
            if face_idx == shared_face.face_index {
                return None;
            }
            let mut boundary_iter = face.boundary_iters().into_iter().enumerate();
            boundary_iter.find_map(|(boundary_idx, boundary_iter)| {
                let mut edge_iter = boundary_iter.enumerate();
                edge_iter.find_map(|(edge_idx, edge)| match edge.is_same(&edge0) {
                    true => Some((face_idx, boundary_idx, edge_idx).into()),
                    false => None,
                })
            })
        })
    });
    iter.collect()
}

fn fillet_surfaces_along_wire(
    shell: &Shell,
    wire: &Wire,
    shared_face_index: FaceBoundaryEdgeIndex,
    adjacent_faces: &[FaceBoundaryEdgeIndex],
    radius: impl Fn(f64) -> f64,
    fillet_division: usize,
) -> Option<Vec<NurbsSurface<Vector4>>> {
    let wire_faces_iter = wire.edge_iter().zip(adjacent_faces);
    let create_fillet_surface = move |(edge, face_index): (&Edge, &FaceBoundaryEdgeIndex)| {
        let surface0 = &shell[shared_face_index.face_index].oriented_surface();
        let surface1 = &shell[face_index.face_index].oriented_surface();
        let curve = &edge.oriented_curve();
        let first_wire = edge.id() == wire.front_edge().unwrap().id();
        let last_wire = edge.id() == wire.back_edge().unwrap().id();
        let extend = first_wire || last_wire;
        let mut relay_spheres =
            relay_spheres(surface0, surface1, curve, fillet_division, &radius, extend)?;
        if first_wire {
            relay_spheres.pop();
        }
        if last_wire {
            relay_spheres.remove(0);
        }
        Some(expand_fillet(&relay_spheres, surface0, surface1))
    };
    wire_faces_iter.map(create_fillet_surface).collect()
}

fn concat_fillet_surface(surfaces: &[NurbsSurface<Vector4>]) -> NurbsSurface<Vector4> {
    let concat_beziers = |i: usize| {
        let mut collector = CurveCollector::<NurbsCurve<Vector4>>::Singleton;
        (0..surfaces.len()).for_each(|n| {
            let mut curve = surfaces[n].column_curve(i);
            curve.knot_translate(n as f64);
            collector.concat(&curve);
        });
        collector.unwrap()
    };
    let len = surfaces[0].control_points().len();
    let long_beziers = (0..len).map(concat_beziers).collect::<Vec<_>>();

    let uknot_vec = unit_circle_knot_vec();
    let vknot_vec = long_beziers[0].knot_vec().clone();
    let destruct_bezier = |bezier: NurbsCurve<Vector4>| bezier.into_non_rationalized().destruct().1;
    let control_points = long_beziers.into_iter().map(destruct_bezier).collect();
    NurbsSurface::new(BSplineSurface::new((uknot_vec, vknot_vec), control_points))
}

fn create_free_edge(curve: Curve) -> Edge {
    let v0 = Vertex::new(curve.front());
    let v1 = Vertex::new(curve.back());
    Edge::new(&v0, &v1, curve)
}

fn cut_face_by_last_bezier(
    shell: &mut Shell,
    face_index: FaceBoundaryEdgeIndex,
    fillet_surface: &NurbsSurface<Vector4>,
) -> Option<Edge> {
    let len = fillet_surface.control_points().len();
    let last_long_bezier = fillet_surface.column_curve(len - 1);
    let face = &shell[face_index.face_index];
    let filleted_edge = &face.boundaries()[face_index.boundary_index][face_index.edge_index];
    let (trimmed_face, edge1) = cut_face_by_bezier(face, last_long_bezier.inverse(), filleted_edge.id())?;
    shell[face_index.face_index] = trimmed_face;
    Some(edge1)
}

fn fillet_along_wire(
    shell: &mut Shell,
    wire: &Wire,
    radius: impl Fn(f64) -> f64,
    fillet_division: usize,
) -> Option<()> {
    if !radius(0.0).near2(&radius(1.0)) {
        return None;
    }
    if !wire.is_continuous() {
        eprintln!("fillet_along_wire failure: Wire must be continuous.");
        return None;
    }
    if wire.is_closed() {
        eprintln!("fillet_along_wire failure: Closed wire case is not implemented.");
        return None;
    }

    let shared_face_index = find_shared_face_with_front_edge(shell, wire)?;
    let adjacent_faces = enumerate_adjacent_faces(shell, wire, shared_face_index)?;

    let fillet_surfaces = fillet_surfaces_along_wire(
        shell,
        wire,
        shared_face_index,
        &adjacent_faces,
        radius,
        fillet_division,
    )?;

    type CffTuple<'a> = (&'a [NurbsSurface<Vector4>], &'a FaceBoundaryEdgeIndex);
    let create_fillet_face = |(surfaces, face_index): CffTuple<'_>| {
        let fillet_surface = concat_fillet_surface(surfaces);
        let edge0 = create_free_edge(surfaces[1].column_curve(0).into());

        let edge1 = cut_face_by_last_bezier(shell, *face_index, &fillet_surface)?;

        let edge2 = {
            let (v0, v1) = (edge0.front(), edge1.back());
            let (u, v) = fillet_surface.search_parameter(v1.point(), (1.0, 1.0), 100)?;
            let param_line = Line((0.0, 1.0).into(), (u, v).into());
            let pcurve = PCurveLns::new(param_line, fillet_surface.clone());
            Edge::new(v0, v1, pcurve.into())
        };

        let edge3 = {
            let (v0, v1) = (edge0.back(), edge1.front());
            let (u, v) = fillet_surface.search_parameter(v1.point(), (1.0, 2.0), 100)?;
            let param_line = Line((0.0, 2.0).into(), (u, v).into());
            let pcurve = PCurveLns::new(param_line, fillet_surface.clone());
            Edge::new(v0, v1, pcurve.into())
        };

        let boundary = [edge0.inverse(), edge2, edge1.inverse(), edge3.inverse()].into();
        Some(Face::new(vec![boundary], fillet_surface))
    };

    let mut fillet_faces = fillet_surfaces
        .windows(3)
        .zip(adjacent_faces.iter().skip(1))
        .map(create_fillet_face)
        .collect::<Option<Shell>>()?;

    let first_fillet = {
        let fillet_surface = concat_fillet_surface(&fillet_surfaces[0..=1]);

        let (front_edge, _) =
            find_adjacent_edge(&shell[shared_face_index.face_index], wire[0].id())?;

        let edge0 = {
            let mut bezier = fillet_surfaces[0].column_curve(0);
            let curve = front_edge.oriented_curve();
            let (t0, _) = search_intersection_parameter(&bezier, &curve, (0.0, 1.0), 100)?;
            bezier = bezier.cut(t0);
            let v0 = Vertex::new(bezier.front());
            let v1 = Vertex::new(bezier.back());
            Edge::new(&v0, &v1, bezier.into())
        };

        let edge1 = cut_face_by_last_bezier(shell, adjacent_faces[0], &fillet_surface)?;

        let edge2 = {
            let (v0, v1) = (edge0.front(), edge1.back());
            let t0 = edge0.curve().range_tuple().0;
            let (u, v) = fillet_surface.search_parameter(v1.point(), (1.0, 0.0), 100)?;
            let param_line = Line((0.0, t0).into(), (u, v).into());
            let pcurve = PCurveLns::new(param_line, fillet_surface.clone());
            Edge::new(v0, v1, pcurve.into())
        };

        let edge3 = {
            let (v0, v1) = (edge0.back(), edge1.front());
            let (u, v) = fillet_surface.search_parameter(v1.point(), (1.0, 1.0), 100)?;
            let param_line = Line((0.0, 1.0).into(), (u, v).into());
            let pcurve = PCurveLns::new(param_line, fillet_surface.clone());
            Edge::new(v0, v1, pcurve.into())
        };

        let wire = [edge0.inverse(), edge2, edge1.inverse(), edge3.inverse()].into();
        Face::new(vec![wire], fillet_surface)
    };
    fillet_faces.insert(0, first_fillet);

    let last_fillet = {
        let len = wire.len();

        let (_, last_edge) =
            find_adjacent_edge(&shell[shared_face_index.face_index], wire[len - 1].id())?;

        let edge0 = {
            let mut bezier = fillet_surfaces[len - 1].column_curve(0);
            let curve = last_edge.oriented_curve();
            let (t0, _) = search_intersection_parameter(&bezier, &curve, (1.0, 2.0), 100)?;
            bezier.cut(t0);
            let v0 = Vertex::new(bezier.front());
            let v1 = Vertex::new(bezier.back());
            Edge::new(&v0, &v1, bezier.into())
        };

        let fillet_surface = concat_fillet_surface(&fillet_surfaces[len - 2..len]);

        let edge1 = cut_face_by_last_bezier(shell, adjacent_faces[len - 1], &fillet_surface)?;

        let edge2 = {
            let (v0, v1) = (edge0.front(), edge1.back());
            let (u, v) = fillet_surface.search_parameter(v1.point(), (1.0, 1.0), 100)?;
            let param_line = Line((0.0, 1.0).into(), (u, v).into());
            let pcurve = PCurveLns::new(param_line, fillet_surface.clone());
            Edge::new(v0, v1, pcurve.into())
        };

        let edge3 = {
            let (v0, v1) = (edge0.back(), edge1.front());
            let t0 = edge0.curve().range_tuple().1;
            let (u, v) = fillet_surface.search_parameter(v1.point(), (1.0, 2.0), 100)?;
            let param_line = Line((0.0, t0 + 1.0).into(), (u, v).into());
            let pcurve = PCurveLns::new(param_line, fillet_surface.clone());
            Edge::new(v0, v1, pcurve.into())
        };

        let wire = [edge0.inverse(), edge2, edge1.inverse(), edge3.inverse()].into();
        Face::new(vec![wire], fillet_surface)
    };
    fillet_faces.push(last_fillet);

    {
        let mut previous_vertex = None;
        let mut new_wire = fillet_faces
            .face_iter()
            .map(|face| {
                let edge = &face.boundaries()[0][0];
                let v0 = Vertex::new(edge.front().point());
                let v1 = match &previous_vertex {
                    Some(v) => Vertex::clone(v),
                    None => Vertex::new(edge.back().point()),
                };
                let new_edge = Edge::new(&v0, &v1, edge.oriented_curve());
                previous_vertex = Some(v0);
                new_edge.inverse()
            })
            .collect::<Wire>();

        let shared_face = &mut shell[shared_face_index.face_index];
        let (front_edge, _) = find_adjacent_edge(&shared_face, wire[0].id())?;
        let (_, back_edge) = find_adjacent_edge(&shared_face, wire[wire.len() - 1].id())?;

        let mut boundaries = shared_face.boundaries();

        if front_edge == back_edge {
            let pre_new_edge = front_edge.cut(new_wire.front_vertex().unwrap())?.0;
            let new_edge = pre_new_edge.cut(new_wire.back_vertex().unwrap())?.1;
            new_wire.push_front(new_edge);
        } else {
            let new_front_edge = front_edge.cut(new_wire.front_vertex().unwrap())?.0;
            let new_back_edge = back_edge.cut(new_wire.back_vertex().unwrap())?.1;
            new_wire.push_front(new_front_edge);
            new_wire.push_back(new_back_edge);

            let boundary = &boundaries[shared_face_index.boundary_index];
            let len = boundary.len() - new_wire.len();
            let top_index = shared_face_index.edge_index + new_wire.len() - 1;
            (0..len).for_each(|i| {
                new_wire.push_back(boundary[(top_index + i) % boundary.len()].clone());
            });
        }
        boundaries[shared_face_index.boundary_index] = new_wire;
        *shared_face = Face::new(boundaries, shared_face.oriented_surface())
    }

    shell.extend(fillet_faces);

    Some(())
}

#[test]
fn create_fillet_surface() {
    use truck_meshalgo::prelude::*;
    #[rustfmt::skip]
    let surface0 = BSplineSurface::new(
        (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)),
        vec![
            vec![Point3::new(0.2, 0.0, 0.0), Point3::new(0.0, 0.5, 0.0), Point3::new(-0.2, 1.0, 0.0)],
            vec![Point3::new(0.5, 0.0, 0.1), Point3::new(0.5, 0.5, 0.0), Point3::new(0.5, 1.0, 0.2)],
            vec![Point3::new(1.0, 0.0, 0.3), Point3::new(1.0, 0.5, 0.3), Point3::new(1.0, 1.0, 0.1)],
        ],
    )
    .into();
    #[rustfmt::skip]
    let surface1 = BSplineSurface::new(
        (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)),
        vec![
            vec![Point3::new(0.2, 0.0, 0.0),  Point3::new(0.0, 0.0, -0.5), Point3::new(-0.2, 0.0, -1.0)],
            vec![Point3::new(0.0, 0.5, 0.0),  Point3::new(0.0, 0.5, -0.5), Point3::new(0.0, 0.5, -1.0)],
            vec![Point3::new(-0.2, 1.0, 0.0), Point3::new(0.2, 1.0, -0.5), Point3::new(0.0, 1.0, -1.0)],
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
    let surface =
        rolling_ball_fillet_surface(&surface0, &surface1, &curve, 5, |_| 0.3, true).unwrap();
    let poly = StructuredMesh::from_surface(&surface, ((0.0, 1.0), (0.0, 1.0)), 0.01).destruct();
    let file1 = std::fs::File::create("fillet.obj").unwrap();
    obj::write(&poly, file1).unwrap();
}

#[test]
fn create_simple_fillet() {
    #[rustfmt::skip]
    let surface0: NurbsSurface<_> = BSplineSurface::new(
        (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)),
        vec![
            vec![Point3::new(-1.0, 0.0, 0.0), Point3::new(-1.0, 0.5, 0.0), Point3::new(-1.0, 1.0, 1.0)],
            vec![Point3::new(0.0, 0.0, 0.0),  Point3::new(0.0, 0.5, 0.0),  Point3::new(0.0, 1.0, 1.0)],
            vec![Point3::new(1.0, 0.0, 0.0),  Point3::new(1.0, 0.5, 0.0),  Point3::new(1.0, 1.0, 1.0)],
        ],
    )
    .into();
    #[rustfmt::skip]
    let surface1: NurbsSurface<_> = BSplineSurface::new(
        (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)),
        vec![
            vec![Point3::new(1.0, 0.0, 0.0),  Point3::new(1.0, 0.0, -0.5),  Point3::new(1.0, 1.0, -1.0)],
            vec![Point3::new(0.0, 0.0, 0.0),  Point3::new(0.0, 0.5, -0.5),  Point3::new(0.0, 1.0, -1.0)],
            vec![Point3::new(-1.0, 0.0, 0.0), Point3::new(-1.0, 0.0, -0.5), Point3::new(-1.0, 1.0, -1.0)],
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

    let shared_edge_id = wire0[0].id();
    let face0 = Face::new(vec![wire0], surface0);
    let face1 = Face::new(vec![wire1], surface1);

    let shell: Shell = [face0.clone(), face1.clone()].into();
    let poly = shell.robust_triangulation(0.001).to_polygon();
    let file = std::fs::File::create("edged-shell.obj").unwrap();
    obj::write(&poly, file).unwrap();

    let (face0, face1, fillet) = simple_fillet(&face0, &face1, shared_edge_id, |_| 0.3, 5).unwrap();

    let shell: Shell = [face0, face1, fillet].into();
    let poly = shell.robust_triangulation(0.001).to_polygon();
    let file = std::fs::File::create("fillet-shell.obj").unwrap();
    obj::write(&poly, file).unwrap();
}

#[test]
fn create_fillet_with_side() {
    let p = [
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.3, 1.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let v = Vertex::news(&p);

    let line = |i: usize, j: usize| {
        let bsp = BSplineCurve::new(KnotVec::bezier_knot(1), vec![p[i], p[j]]);
        Edge::new(&v[i], &v[j], NurbsCurve::from(bsp).into())
    };

    let edge = [
        line(0, 1),
        line(1, 2),
        line(2, 3),
        line(3, 0),
        line(0, 4),
        line(1, 5),
        line(2, 6),
        line(3, 7),
        line(4, 5),
        line(5, 6),
        line(6, 7),
        line(7, 4),
    ];

    let plane = |i: usize, j: usize, k: usize, l: usize| {
        let control_points = vec![vec![p[i], p[l]], vec![p[j], p[k]]];
        let knot_vec = KnotVec::bezier_knot(1);
        let knot_vecs = (knot_vec.clone(), knot_vec);
        let bsp = BSplineSurface::new(knot_vecs, control_points);

        let wire: Wire = [i, j, k, l]
            .into_iter()
            .circular_tuple_windows()
            .map(|(i, j)| {
                edge.iter()
                    .find_map(|edge| {
                        if edge.front() == &v[i] && edge.back() == &v[j] {
                            Some(edge.clone())
                        } else if edge.back() == &v[i] && edge.front() == &v[j] {
                            Some(edge.inverse())
                        } else {
                            None
                        }
                    })
                    .unwrap()
            })
            .collect();
        Face::new(vec![wire], bsp.into())
    };

    let face = [plane(0, 1, 2, 3), plane(0, 3, 7, 4), plane(0, 4, 5, 1)];

    let (face0, face1, fillet, _, side1) = fillet_with_side(
        &face[0],
        &face[1],
        edge[3].id(),
        None,
        Some(&face[2]),
        |t| 0.3 + 0.3 * t,
        5,
    )
    .unwrap();

    let shell: Shell = vec![face0, face1, fillet, side1.unwrap()].into();

    let poly = shell.robust_triangulation(0.001).to_polygon();
    let file = std::fs::File::create("fillet-with-edge.obj").unwrap();
    obj::write(&poly, file).unwrap();
}

#[test]
fn fillet_to_nurbs() {
    let p = [
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let v = Vertex::news(&p);

    let line = |i: usize, j: usize| {
        let bsp = BSplineCurve::new(KnotVec::bezier_knot(1), vec![p[i], p[j]]);
        Edge::new(&v[i], &v[j], NurbsCurve::from(bsp).into())
    };
    let edge = [
        line(0, 1),
        Edge::new(
            &v[1],
            &v[2],
            circle_arc_by_three_points(
                p[1].to_homogeneous(),
                p[2].to_homogeneous(),
                Point3::new(1.0 / f64::sqrt(2.0), 1.0 / f64::sqrt(2.0), 1.0),
            )
            .into(),
        ),
        line(2, 0),
        line(1, 4),
        line(2, 5),
        Edge::new(
            &v[4],
            &v[5],
            circle_arc_by_three_points(
                p[4].to_homogeneous(),
                p[5].to_homogeneous(),
                Point3::new(1.0 / f64::sqrt(2.0), 1.0 / f64::sqrt(2.0), 0.0),
            )
            .into(),
        ),
    ];
    let bsp0 = NurbsSurface::new(BSplineSurface::new(
        (KnotVec::bezier_knot(1), KnotVec::bezier_knot(1)),
        vec![
            vec![
                Vector4::new(0.0, 0.0, 1.0, 1.0),
                Vector4::new(0.0, 1.0, 1.0, 1.0),
            ],
            vec![
                Vector4::new(1.0, 0.0, 1.0, 1.0),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
            ],
        ],
    ));
    let bsp1 = NurbsSurface::new(BSplineSurface::new(
        (KnotVec::bezier_knot(1), unit_circle_knot_vec()),
        vec![
            circle_arc_by_three_points(
                p[1].to_homogeneous(),
                p[2].to_homogeneous(),
                Point3::new(1.0 / f64::sqrt(2.0), 1.0 / f64::sqrt(2.0), 1.0),
            )
            .control_points()
            .clone(),
            circle_arc_by_three_points(
                p[4].to_homogeneous(),
                p[5].to_homogeneous(),
                Point3::new(1.0 / f64::sqrt(2.0), 1.0 / f64::sqrt(2.0), 0.0),
            )
            .control_points()
            .clone(),
        ],
    ));
    let shell: Shell = [
        Face::new(
            vec![[edge[0].clone(), edge[1].clone(), edge[2].clone()].into()],
            bsp0,
        ),
        Face::new(
            vec![[
                edge[3].clone(),
                edge[5].clone(),
                edge[4].inverse(),
                edge[1].inverse(),
            ]
            .into()],
            bsp1,
        ),
    ]
    .into();

    let poly = shell.triangulation(0.001).to_polygon();
    let file = std::fs::File::create("cylinder.obj").unwrap();
    obj::write(&poly, file).unwrap();

    let (face0, face1, fillet) =
        simple_fillet(&shell[0], &shell[1], edge[1].id(), |_| 0.3, 5).unwrap();
    let shell: Shell = [face0, face1, fillet].into();

    let poly = shell.triangulation(0.001).to_polygon();
    let file = std::fs::File::create("fillet-cylinder.obj").unwrap();
    obj::write(&poly, file).unwrap();
}

#[test]
fn fillet_semi_cube() {
    let p = [
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let v = Vertex::news(&p);

    let line = |i: usize, j: usize| {
        let bsp = BSplineCurve::new(KnotVec::bezier_knot(1), vec![p[i], p[j]]);
        Edge::new(&v[i], &v[j], NurbsCurve::from(bsp).into())
    };
    let edge = [
        line(0, 1),
        line(1, 2),
        line(2, 3),
        line(3, 0),
        line(0, 4),
        line(1, 5),
        line(2, 6),
        line(3, 7),
        line(4, 5),
        line(5, 6),
        line(6, 7),
        line(7, 4),
    ];

    let plane = |i: usize, j: usize, k: usize, l: usize| {
        let control_points = vec![vec![p[i], p[l]], vec![p[j], p[k]]];
        let knot_vec = KnotVec::bezier_knot(1);
        let knot_vecs = (knot_vec.clone(), knot_vec);
        let bsp = BSplineSurface::new(knot_vecs, control_points);

        let wire: Wire = [i, j, k, l]
            .into_iter()
            .circular_tuple_windows()
            .map(|(i, j)| {
                edge.iter()
                    .find_map(|edge| {
                        if edge.front() == &v[i] && edge.back() == &v[j] {
                            Some(edge.clone())
                        } else if edge.back() == &v[i] && edge.front() == &v[j] {
                            Some(edge.inverse())
                        } else {
                            None
                        }
                    })
                    .unwrap()
            })
            .collect();
        Face::new(vec![wire], bsp.into())
    };
    let mut shell: Shell = [
        plane(0, 1, 2, 3),
        plane(1, 0, 4, 5),
        plane(2, 1, 5, 6),
        plane(3, 2, 6, 7),
    ]
    .into();

    let poly = shell.robust_triangulation(0.001).to_polygon();
    let file = std::fs::File::create("semi-cube.obj").unwrap();
    obj::write(&poly, file).unwrap();

    let (face0, face1, face2, _, side1) = fillet_with_side(
        &shell[1],
        &shell[2],
        edge[5].id(),
        None,
        Some(&shell[0]),
        |_| 0.4,
        5,
    )
    .unwrap();
    (shell[1], shell[2], shell[0]) = (face0, face1, side1.unwrap());
    shell.push(face2);

    let (face0, face1, face2, _, side1) = fillet_with_side(
        &shell[2],
        &shell[3],
        edge[6].id(),
        None,
        Some(&shell[0]),
        |_| 0.4,
        5,
    )
    .unwrap();
    (shell[2], shell[3], shell[0]) = (face0, face1, side1.unwrap());
    shell.push(face2);

    let mut boundary = shell[0].boundaries().pop().unwrap();
    boundary.pop_back();
    assert_eq!(boundary.front_vertex().unwrap(), &v[0]);

    let poly = shell.robust_triangulation(0.001).to_polygon();
    let file = std::fs::File::create("pre-fillet-cube.obj").unwrap();
    obj::write(&poly, file).unwrap();

    fillet_along_wire(&mut shell, &boundary, |_| 0.2, 5).unwrap();

    let poly = shell.robust_triangulation(0.001).to_polygon();
    let file = std::fs::File::create("fillet-cube.obj").unwrap();
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

        (0..size).map(|i| matrix[i][size] / matrix[i][i]).collect()
    }

    fn echelon(matrix: &mut [Vec<f64>], i: usize, j: usize) {
        let size = matrix.len();
        if matrix[i][i] != 0f64 {
            let factor = matrix[j + 1][i] / matrix[i][i];
            (i..size + 1).for_each(|k| {
                matrix[j + 1][k] -= factor * matrix[i][k];
            });
        }
    }

    fn eliminate(matrix: &mut [Vec<f64>], i: usize) {
        let size = matrix.len();
        if matrix[i][i] != 0f64 {
            for j in (1..i + 1).rev() {
                let factor = matrix[j - 1][i] / matrix[i][i];
                for k in (0..size + 1).rev() {
                    matrix[j - 1][k] -= factor * matrix[i][k];
                }
            }
        }
    }
}
