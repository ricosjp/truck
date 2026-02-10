use std::f64::consts::PI;
use truck_geometry::prelude::*;

use super::types::*;

pub(super) fn circle_arc(
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

pub(super) fn circle_arc_by_three_points(
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
pub(super) fn unit_circle_knot_vec() -> KnotVec { KnotVec::uniform_knot(2, 4) }

#[inline(always)]
pub(super) const fn number_of_cpts_of_unit_circle() -> usize { 6 }

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
        angle in 0.1f64..(1.5 * PI),
        w0 in 0.1f64..5.0,
        w1 in 0.1f64..5.0,
    ) {
        use proptest::prelude::*;
        let uc = unit_circle_arc(Rad(angle), w0, w1);
        const N: usize = 10;
        for i in 0..=N {
            let t = i as f64 / N as f64;
            let p = uc.subs(t).to_vec();
            let v = uc.der(t);
            prop_assert_near!(p.magnitude2(), 1.0, "{w0} {w1} {p:?} {angle}");
            prop_assert!(p.z.so_small2());
            prop_assert!(p.x * v.y - p.y * v.x > 0.0, "minus area {:?}", uc.control_point(1));
        }
        prop_assert_near!(uc.subs(0.0), Point3::new(1.0, 0.0, 0.0));
        prop_assert_near!(uc.subs(1.0), Point3::new(f64::cos(angle), f64::sin(angle), 0.0));
    }
}

fn interpolate_bezier(points: &[Vector4]) -> BSplineCurve<Vector4> {
    let n = points.len() - 1;
    let parameter_points = points
        .iter()
        .enumerate()
        .map(|(i, p)| (i as f64 / n as f64, *p))
        .collect::<Vec<_>>();
    BSplineCurve::interpolate(KnotVec::bezier_knot(n), parameter_points)
}

pub(super) fn composite_line_bezier(
    line: Line<Point2>,
    surface: &NurbsSurface<Vector4>,
) -> BSplineCurve<Vector4> {
    let curve = PCurve::new(line, surface.non_rationalized());
    let degree = surface.udegree() + surface.vdegree();
    let points = (0..=degree)
        .map(|i| curve.subs(i as f64 / degree as f64))
        .collect::<Vec<_>>();
    interpolate_bezier(&points)
}

#[derive(Clone, Copy, Debug)]
pub(super) struct RelaySphere {
    /// Center of fillet circle.
    #[allow(dead_code)]
    pub(super) center: Point3,
    /// Contact point of sphere and surface0, 3d coordinate and parameter.
    pub(super) contact0: (Point3, Point2),
    /// Contact point of sphere and surface1, 3d coordinate and parameter.
    pub(super) contact1: (Point3, Point2),
    /// Transit of fillet circle.
    pub(super) transit: Point3,
}

impl RelaySphere {
    /// Contact point on planes and sphere with radius `radius`.
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

    pub(super) fn generate(
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

    #[allow(dead_code)]
    pub(super) fn fillet_wire(self) -> NurbsCurve<Vector4> {
        circle_arc_by_three_points(
            self.contact0.0.to_homogeneous(),
            self.contact1.0.to_homogeneous(),
            self.transit,
        )
    }
}

pub(super) fn relay_spheres(
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

#[allow(dead_code)]
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

pub(super) fn expand_fillet(
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

pub(super) fn expand_chamfer(
    relay_spheres: &[RelaySphere],
    surface0: &NurbsSurface<Vector4>,
    surface1: &NurbsSurface<Vector4>,
) -> NurbsSurface<Vector4> {
    const CHAMFER_CPTS: usize = 2; // degree-1 cross-section: 2 control points
    let mut collectors = vec![CurveCollector::<BSplineCurve<_>>::Singleton; CHAMFER_CPTS];
    relay_spheres.windows(2).enumerate().for_each(|(n, u)| {
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
        // Degree-1 cross-section: straight line from contact0 to contact1
        let chamfer_wires: Vec<BSplineCurve<Vector4>> = cpts0
            .iter()
            .zip(cpts1)
            .map(|(p0, p1)| BSplineCurve::new(KnotVec::bezier_knot(1), vec![*p0, *p1]))
            .collect();
        let homo_surface = BSplineSurface::new(
            (bezier0.knot_vec().clone(), KnotVec::bezier_knot(1)),
            chamfer_wires
                .into_iter()
                .map(|wire| wire.destruct().1)
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

    let knot_vecs = (KnotVec::bezier_knot(1), vknot_vec);
    let mut bsp_surface = BSplineSurface::new(knot_vecs, control_points);
    bsp_surface.knot_normalize();
    NurbsSurface::new(bsp_surface)
}

pub(super) fn chamfer_fillet_surface(
    surface0: &NurbsSurface<Vector4>,
    surface1: &NurbsSurface<Vector4>,
    curve: &impl FilletCurve,
    division: usize,
    radius: impl Fn(f64) -> f64,
    extend: bool,
) -> Option<NurbsSurface<Vector4>> {
    let relay_spheres = relay_spheres(surface0, surface1, curve, division, radius, extend)?;
    Some(expand_chamfer(&relay_spheres, surface0, surface1))
}

/// Orientation of `curve` should be compatible with orientation of `surface0`.
pub(super) fn rolling_ball_fillet_surface(
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
