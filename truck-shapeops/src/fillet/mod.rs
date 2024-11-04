/***********************************/
/*** This module is a prototype. ***/
/***********************************/

#![allow(dead_code)]

use derive_more::*;
use itertools::Itertools;
use std::f64::consts::PI;
use truck_geometry::prelude::*;

#[cfg(test)]
use truck_meshalgo::prelude::*;

type PCurveLns = PCurve<Line<Point2>, NurbsSurface<Vector4>>;

#[derive(
    Clone, Debug, ParametricCurve, BoundedCurve, ParameterDivision1D, Cut, From, Invertible,
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
    use algo::curve::search_intersection_parameter;

    let (front_edge, back_edge) = find_adjacent_edge(face, filleted_edge_id)?;

    let new_front_edge = {
        let curve = front_edge.curve();
        let hint = take_ori(!front_edge.orientation(), curve.range_tuple());
        let (t0, t1) = search_intersection_parameter(&bezier, &curve, (0.0, hint), 100)?;
        let v0 = Vertex::new(bezier.subs(t0));
        bezier = bezier.cut(t0);
        front_edge.cut_with_parameter(&v0, t1)?.0
    };

    let new_back_edge = {
        let curve = back_edge.curve();
        let hint = take_ori(back_edge.orientation(), curve.range_tuple());
        let (t0, t1) = search_intersection_parameter(&bezier, &curve, (1.0, hint), 100)?;
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
                boundary.rotate_left(idx);
                if face.orientation() {
                    boundary[0] = new_front_edge.clone();
                    boundary[1] = fillet_edge.clone();
                    boundary[2] = new_back_edge.clone();
                } else {
                    boundary[0] = new_back_edge.inverse();
                    boundary[1] = fillet_edge.inverse();
                    boundary[2] = new_front_edge.inverse();
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
) -> Option<(Face, Face, Face)> {
    let find_filleted_edge = move |edge: &Edge| edge.id() == filleted_edge_id;
    let filleted_edge = face0.edge_iter().find(find_filleted_edge)?;

    let fillet_surface = {
        let surface0 = face0.oriented_surface();
        let surface1 = face1.oriented_surface();
        let curve = filleted_edge.oriented_curve();
        rolling_ball_fillet_surface(&surface0, &surface1, &curve, 5, radius)?
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
) -> Option<(Face, Face, Face, Option<Face>, Option<Face>)> {
    let (new_face0, new_face1, fillet) = simple_fillet(face0, face1, filleted_edge_id, radius)?;

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
    let surface = rolling_ball_fillet_surface(&surface0, &surface1, &curve, 5, |_| 0.3).unwrap();
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

    let (face0, face1, fillet) = simple_fillet(&face0, &face1, shared_edge_id, |_| 0.3).unwrap();

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
    )
    .unwrap();

    let shell: Shell = vec![face0, face1, fillet, side1.unwrap()].into();

    let poly = shell.robust_triangulation(0.001).to_polygon();
    let file = std::fs::File::create("fillet-with-edge.obj").unwrap();
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
