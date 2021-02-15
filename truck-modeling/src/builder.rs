use crate::*;
const PI: Rad<f64> = Rad(std::f64::consts::PI);

/// Creates and returns a vertex by a three dimensional point.
/// # Examples
/// ```
/// use truck_modeling::*;
///
/// // put a vertex
/// let vertex = builder::vertex(Point3::new(1.0, 2.0, 3.0));
/// # assert_eq!(*vertex.lock_point().unwrap(), Point3::new(1.0, 2.0, 3.0));
/// ```
#[inline(always)]
pub fn vertex(pt: Point3) -> Vertex { Vertex::new(pt) }

/// Returns a line from `vertex0` to `vertex1`.
/// # Examples
/// ```
/// use truck_modeling::*;
///
/// // draw a line
/// let vertex0 = builder::vertex(Point3::new(1.0, 2.0, 3.0));
/// let vertex1 = builder::vertex(Point3::new(6.0, 5.0, 4.0));
/// let line = builder::line(&vertex0, &vertex1);
/// # let curve = line.oriented_curve();
/// # let pt0 = Point3::new(1.0, 2.0, 3.0);
/// # let pt1 = Point3::new(6.0, 5.0, 4.0);
/// # const N: usize = 10;
/// # for i in 0..=N {
/// #       let t = i as f64 / N as f64;
/// #       assert!(curve.subs(t).near2(&(pt0 + t * (pt1 - pt0))));
/// # }
/// ```
#[inline(always)]
pub fn line(vertex0: &Vertex, vertex1: &Vertex) -> Edge {
    let pt0 = vertex0.lock_point().unwrap().to_homogeneous();
    let pt1 = vertex1.lock_point().unwrap().to_homogeneous();
    let curve = geom_impls::line(pt0, pt1);
    Edge::new(vertex0, vertex1, NURBSCurve::new(curve))
}

/// Returns a circle arc from `vertex0` to `vertex1` via `transit`.
/// # Examples
/// ```
/// use truck_modeling::*;
///
/// // draw the unit upper semicircle
/// let vertex0 = builder::vertex(Point3::new(1.0, 0.0, 0.0));
/// let vertex1 = builder::vertex(Point3::new(-1.0, 0.0, 0.0));
/// let semi_circle = builder::circle_arc(&vertex0, &vertex1, Point3::new(0.0, 1.0, 0.0));
/// # let curve = semi_circle.oriented_curve();
/// # const N: usize = 10;
/// # for i in 0..=N {
/// #       let t = curve.knot_vec()[0] + curve.knot_vec().range_length() * i as f64 / N as f64;
/// #       assert!(curve.subs(t).to_vec().magnitude().near(&1.0));
/// # }
/// ```
#[inline(always)]
pub fn circle_arc(vertex0: &Vertex, vertex1: &Vertex, transit: Point3) -> Edge {
    let pt0 = vertex0.lock_point().unwrap().to_homogeneous();
    let pt1 = vertex1.lock_point().unwrap().to_homogeneous();
    let curve = geom_impls::circle_arc_by_three_points(pt0, pt1, transit);
    Edge::new(vertex0, vertex1, NURBSCurve::new(curve))
}

/// Returns a Bezier curve from `vertex0` to `vertex1` with inter control points `inter_points`.
/// # Examples
/// ```
/// use truck_modeling::*;
///
/// // draw a Bezier curve
/// let vertex0 = builder::vertex(Point3::origin());
/// let vertex1 = builder::vertex(Point3::new(3.0, 0.0, 0.0));
/// let inter_points = vec![Point3::new(1.0, 1.0, 0.0), Point3::new(2.0, -1.0, 0.0)];
/// let bezier = builder::bezier(&vertex0, &vertex1, inter_points);
/// # let curve = bezier.oriented_curve();
/// # const N: usize = 10;
/// # for i in 0..=N {
/// #       let t = i as f64 / N as f64;
/// #       let pt = Point3::new(t * 3.0, 6.0 * t * t * t - 9.0 * t * t + 3.0 * t, 0.0);
/// #       assert!(curve.subs(t).near(&pt));
/// # }
/// ```
#[inline(always)]
pub fn bezier(vertex0: &Vertex, vertex1: &Vertex, mut inter_points: Vec<Point3>) -> Edge {
    let pt0 = *vertex0.lock_point().unwrap();
    let pt1 = *vertex1.lock_point().unwrap();
    let mut pre_ctrl_pts = vec![pt0];
    pre_ctrl_pts.append(&mut inter_points);
    pre_ctrl_pts.push(pt1);
    let ctrl_pts: Vec<_> = pre_ctrl_pts
        .into_iter()
        .map(|pt| pt.to_homogeneous())
        .collect();
    let knot_vec = KnotVec::bezier_knot(ctrl_pts.len() - 1);
    let curve = BSplineCurve::new(knot_vec, ctrl_pts);
    Edge::new(vertex0, vertex1, NURBSCurve::new(curve))
}

/// Returns a homotopic face from `edge0` to `edge1`.
/// # Examples
/// ```
/// use truck_modeling::*;
///
/// // homotopy between skew lines
/// let v0 = builder::vertex(Point3::new(0.0, 0.0, 0.0));
/// let v1 = builder::vertex(Point3::new(1.0, 0.0, 0.0));
/// let v2 = builder::vertex(Point3::new(0.0, 1.0, 0.0));
/// let v3 = builder::vertex(Point3::new(0.0, 1.0, 1.0));
/// let line0 = builder::line(&v0, &v1);
/// let line1 = builder::line(&v2, &v3);
/// let homotopy = builder::homotopy(&line0, &line1);
/// # let surface = homotopy.oriented_surface();
/// # const N: usize = 10;
/// # for i in 0..=N {
/// #       for j in 0..=N {
/// #           let s = i as f64 / N as f64;
/// #           let t = j as f64 / N as f64;
/// #           let pt = Point3::new(s * (1.0 - t), t, s * t);
/// #           assert!(surface.subs(s, t).near(&pt));
/// #       }
/// # }
/// ```
#[inline(always)]
pub fn homotopy(edge0: &Edge, edge1: &Edge) -> Face {
    let wire: Wire = vec![
        edge0.clone(),
        line(edge0.back(), edge1.back()),
        edge1.inverse(),
        line(edge1.front(), edge0.front()),
    ]
    .into();
    let curve0 = edge0.oriented_curve().into_non_rationalized();
    let curve1 = edge1.oriented_curve().into_non_rationalized();
    let surface = BSplineSurface::homotopy(curve0, curve1);
    Face::new(vec![wire], NURBSSurface::new(surface))
}

/// Creates a cone by R-sweeping.
/// # Examples
/// ```
/// use truck_modeling::*;
/// use std::f64::consts::PI;
/// let v0 = builder::vertex(Point3::new(0.0, 1.0, 0.0));
/// let v1 = builder::vertex(Point3::new(0.0, 0.0, 1.0));
/// let v2 = builder::vertex(Point3::new(0.0, 0.0, 0.0));
/// let wire: Wire = vec![
///     builder::line(&v0, &v1),
///     builder::line(&v1, &v2),
/// ].into();
/// let cone = builder::cone(&wire, Vector3::unit_y(), Rad(2.0 * PI));
/// let irregular = builder::rsweep(&wire, Point3::origin(), Vector3::unit_y(), Rad(2.0 * PI));
/// 
/// // the degenerate edge of cone is removed!
/// assert_eq!(cone[0].boundaries()[0].len(), 3);
/// assert_eq!(irregular[0].boundaries()[0].len(), 4);
/// # assert_eq!(cone[1].boundaries()[0].len(), 3);
/// # assert_eq!(irregular[1].boundaries()[0].len(), 4);
/// # assert_eq!(cone[2].boundaries()[0].len(), 3);
/// # assert_eq!(irregular[2].boundaries()[0].len(), 4);
/// # assert_eq!(cone[3].boundaries()[0].len(), 3);
/// # assert_eq!(irregular[3].boundaries()[0].len(), 4);
/// 
/// // this cone is closed
/// Solid::new(vec![cone]);
/// ```
#[inline(always)]
pub fn cone<R: Into<Rad<f64>>>(wire: &Wire, axis: Vector3, angle: R) -> Shell {
    let angle = angle.into();
    let closed = angle.0.abs() >= 2.0 * PI.0;
    let mut wire = wire.clone();
    if wire.is_empty() {
        return Shell::new();
    }
    let pt0 = *wire.front_vertex().unwrap().lock_point().unwrap();
    let pt1 = *wire.back_vertex().unwrap().lock_point().unwrap();
    let pt1_on_axis = (pt1 - pt0).cross(axis).so_small();
    if wire.len() == 1 && pt1_on_axis {
        let edge = wire.pop_back().unwrap();
        let v0 = edge.front().clone();
        let v2 = edge.back().clone();
        let mut curve = edge.lock_curve().unwrap().clone();
        let t = curve.knot_vec()[0] + curve.knot_vec().range_length() * 0.5;
        let v1 = Vertex::new(curve.subs(t));
        let curve1 = curve.cut(t);
        wire.push_back(Edge::debug_new(&v0, &v1, curve));
        wire.push_back(Edge::debug_new(&v1, &v2, curve1));
    }
    let mut shell = rsweep(&wire, pt0, axis, angle);
    let mut edge = shell[0].boundaries()[0][0].clone();
    for i in 0..shell.len() / wire.len() {
        let idx = i * wire.len();
        let face = shell[idx].clone();
        let surface = face.oriented_surface();
        let old_wire = face.into_boundaries().pop().unwrap();
        let mut new_wire = Wire::new();
        new_wire.push_back(edge.clone());
        new_wire.push_back(old_wire[1].clone());
        let new_edge = if closed && i + 1 == shell.len() / new_wire.len() {
            shell[0].boundaries()[0][0].inverse()
        } else {
            let curve = old_wire[2].oriented_curve();
            Edge::debug_new(old_wire[2].front(), new_wire[0].front(), curve)
        };
        new_wire.push_back(new_edge.clone());
        shell[idx] = Face::debug_new(vec![new_wire], surface);
        edge = new_edge.inverse();
    }
    if pt1_on_axis {
        let mut edge = shell[wire.len() - 1].boundaries()[0][0].clone();
        for i in 0..shell.len() / wire.len() {
            let idx = (i + 1) * wire.len() - 1;
            let face = shell[idx].clone();
            let surface = face.oriented_surface();
            let old_wire = face.into_boundaries().pop().unwrap();
            let mut new_wire = Wire::new();
            new_wire.push_back(edge.clone());
            let new_edge = if closed && i + 1 == shell.len() / wire.len() {
                shell[wire.len() - 1].boundaries()[0][0].inverse()
            } else {
                let curve = old_wire[2].oriented_curve();
                Edge::debug_new(new_wire[0].back(), old_wire[2].back(), curve)
            };
            new_wire.push_back(new_edge.clone());
            new_wire.push_back(old_wire[3].clone());
            shell[idx] = Face::debug_new(vec![new_wire], surface);
            edge = new_edge.inverse();
        }
    }
    shell
}

/// Try attatiching a plane whose boundary is `wire`.
/// Todo: Define the crate error and make return value `Result<Face>`!
/// # Examples
/// ```
/// use truck_modeling::*;
///
/// // make a disk by attaching a plane into circle
/// let vertex = builder::vertex(Point3::new(1.0, 0.0, 0.0));
/// let circle = builder::rsweep(&vertex, Point3::origin(), Vector3::unit_y(), Rad(7.0));
/// let disk = builder::try_attach_plane(&vec![circle]).expect("failed to attach plane.");
/// # let surface = disk.oriented_surface();
/// # let normal = surface.normal(0.5, 0.5);
/// # assert!(normal.near(&Vector3::unit_y()));
/// ```
/// # Remarks
/// If wires are not closed or not in one plane, then return `None`.
/// ```
/// use truck_modeling::*;
///
/// let v0 = builder::vertex(Point3::new(0.0, 0.0, 0.0));
/// let v1 = builder::vertex(Point3::new(1.0, 0.0, 0.0));
/// let v2 = builder::vertex(Point3::new(0.0, 1.0, 0.0));
/// let v3 = builder::vertex(Point3::new(0.0, 0.0, 1.0));
/// let wire: Wire = vec![
///     builder::line(&v0, &v1),
///     builder::line(&v1, &v2),
/// ]
/// .into();
/// let mut wires = vec![wire];
/// // failed to attach plane, because wire is not closed.
/// assert!(builder::try_attach_plane(&wires).is_none());
///
/// wires[0].push_back(builder::line(&v2, &v3));
/// wires[0].push_back(builder::line(&v3, &v0));
/// // failed to attach plane, because wire is not in the plane.
/// assert!(builder::try_attach_plane(&wires).is_none());
///
/// wires[0].pop_back();
/// wires[0].pop_back();
/// wires[0].push_back(builder::line(&v2, &v0));
/// // sucess in attaching plane!
/// assert!(builder::try_attach_plane(&wires).is_some());
/// ```
#[inline(always)]
pub fn try_attach_plane(wires: &Vec<Wire>) -> Option<Face> {
    let pts = wires
        .iter()
        .flatten()
        .flat_map(|edge| {
            edge.oriented_curve()
                .control_points()
                .clone()
                .into_iter()
                .map(|pt| pt.to_point())
        })
        .collect::<Vec<_>>();
    let surface = NURBSSurface::new(geom_impls::attach_plane(pts)?);
    Face::try_new(wires.clone(), surface).ok()
}

/// Returns another topology whose points, curves, and surfaces are cloned.
#[inline(always)]
pub fn clone<T: Mapped<Point3, NURBSCurve, NURBSSurface>>(elem: &T) -> T {
    elem.topological_clone()
}

/// Returns a transformed vertex, edge, wire, face, shell or solid.
#[inline(always)]
pub fn transformed<T: Mapped<Point3, NURBSCurve, NURBSSurface>>(elem: &T, mat: Matrix4) -> T {
    elem.mapped(
        &move |pt: &Point3| mat.transform_point(*pt),
        &move |curve: &NURBSCurve| NURBSCurve::new(mat * curve.non_rationalized()),
        &move |surface: &NURBSSurface| NURBSSurface::new(mat * surface.non_rationalized()),
    )
}

/// Returns a translated vertex, edge, wire, face, shell or solid.
#[inline(always)]
pub fn translated<T: Mapped<Point3, NURBSCurve, NURBSSurface>>(elem: &T, vector: Vector3) -> T {
    transformed(elem, Matrix4::from_translation(vector))
}

/// Returns a rotated vertex, edge, wire, face, shell or solid.
#[inline(always)]
pub fn rotated<T: Mapped<Point3, NURBSCurve, NURBSSurface>>(
    elem: &T,
    origin: Point3,
    axis: Vector3,
    angle: Rad<f64>,
) -> T {
    let mat0 = Matrix4::from_translation(-origin.to_vec());
    let mat1 = Matrix4::from_axis_angle(axis, angle);
    let mat2 = Matrix4::from_translation(origin.to_vec());
    transformed(elem, mat2 * mat1 * mat0)
}

/// Returns a scaled vertex, edge, wire, face, shell or solid.
#[inline(always)]
pub fn scaled<T: Mapped<Point3, NURBSCurve, NURBSSurface>>(
    elem: &T,
    origin: Point3,
    scalars: Vector3,
) -> T {
    let mat0 = Matrix4::from_translation(-origin.to_vec());
    let mat1 = Matrix4::from_nonuniform_scale(scalars[0], scalars[1], scalars[2]);
    let mat2 = Matrix4::from_translation(origin.to_vec());
    transformed(elem, mat2 * mat1 * mat0)
}

/// Sweeps a vertex, an edge, a wire, a face, or a shell by a vector.
/// # Examples
/// ```
/// use truck_modeling::*;
/// let vertex: Vertex = builder::vertex(Point3::new(0.0, 0.0, 0.0));
/// let line: Edge = builder::tsweep(&vertex, Vector3::unit_x());
/// let square: Face = builder::tsweep(&line, Vector3::unit_y());
/// let cube: Solid = builder::tsweep(&square, Vector3::unit_z());
/// #
/// # let b_shell = &cube.boundaries()[0];
/// # assert_eq!(b_shell.len(), 6); // This solid is a cube!
/// # assert!(cube.is_geometric_consistent());
/// #
/// # let b_loop = &b_shell[0].boundaries()[0];
/// # let mut loop_iter = b_loop.vertex_iter();
/// # assert_eq!(*loop_iter.next().unwrap().lock_point().unwrap(), Point3::new(0.0, 0.0, 0.0));
/// # assert_eq!(*loop_iter.next().unwrap().lock_point().unwrap(), Point3::new(0.0, 1.0, 0.0));
/// # assert_eq!(*loop_iter.next().unwrap().lock_point().unwrap(), Point3::new(1.0, 1.0, 0.0));
/// # assert_eq!(*loop_iter.next().unwrap().lock_point().unwrap(), Point3::new(1.0, 0.0, 0.0));
/// # assert_eq!(loop_iter.next(), None);
/// #
/// # let b_loop = &b_shell[3].boundaries()[0];
/// # let mut loop_iter = b_loop.vertex_iter();
/// # assert_eq!(*loop_iter.next().unwrap().lock_point().unwrap(), Point3::new(1.0, 1.0, 0.0));
/// # assert_eq!(*loop_iter.next().unwrap().lock_point().unwrap(), Point3::new(0.0, 1.0, 0.0));
/// # assert_eq!(*loop_iter.next().unwrap().lock_point().unwrap(), Point3::new(0.0, 1.0, 1.0));
/// # assert_eq!(*loop_iter.next().unwrap().lock_point().unwrap(), Point3::new(1.0, 1.0, 1.0));
/// # assert_eq!(loop_iter.next(), None);
/// #
/// # let b_loop = &b_shell[5].boundaries()[0];
/// # let mut loop_iter = b_loop.vertex_iter();
/// # assert_eq!(*loop_iter.next().unwrap().lock_point().unwrap(), Point3::new(0.0, 0.0, 1.0));
/// # assert_eq!(*loop_iter.next().unwrap().lock_point().unwrap(), Point3::new(1.0, 0.0, 1.0));
/// # assert_eq!(*loop_iter.next().unwrap().lock_point().unwrap(), Point3::new(1.0, 1.0, 1.0));
/// # assert_eq!(*loop_iter.next().unwrap().lock_point().unwrap(), Point3::new(0.0, 1.0, 1.0));
/// # assert_eq!(loop_iter.next(), None);
/// ```
pub fn tsweep<T: Sweep<Point3, NURBSCurve, NURBSSurface>>(elem: &T, vector: Vector3) -> T::Swept {
    let trsl = Matrix4::from_translation(vector);
    elem.sweep(
        &move |pt| trsl.transform_point(*pt),
        &move |curve| NURBSCurve::new(trsl * curve.non_rationalized()),
        &move |surface| NURBSSurface::new(trsl * surface.non_rationalized()),
        &move |pt0, pt1| {
            NURBSCurve::new(geom_impls::line(pt0.to_homogeneous(), pt1.to_homogeneous()))
        },
        &move |curve0, curve1| {
            NURBSSurface::new(BSplineSurface::homotopy(
                curve0.clone().into_non_rationalized(),
                curve1.clone().into_non_rationalized(),
            ))
        },
    )
}

/// Sweeps a vertex, an edge, a wire, a face, or a shell by the rotation.
/// # Details
/// If the absolute value of `angle` is more than 2π rad, then the result is closed shape.
/// For example, the result of sweeping a disk is a bent cylinder if `angle` is less than 2π rad
/// and a solid torus if `angle` is more than 2π rad.
/// # Examples
/// ```
/// // Torus
/// use truck_modeling::*;
/// const PI: Rad<f64> = Rad(std::f64::consts::PI);
///
/// let v: Vertex = builder::vertex(Point3::new(3.0, 0.0, 0.0));
/// let circle: Wire = builder::rsweep(&v, Point3::new(2.0, 0.0, 0.0), Vector3::unit_z(), PI * 2.0);
/// let torus: Shell = builder::rsweep(&circle, Point3::origin(), Vector3::unit_y(), PI * 2.0);
/// let solid: Solid = Solid::new(vec![torus]);
/// #
/// # assert!(solid.is_geometric_consistent());
/// # const N: usize = 100;
/// # let shell = &solid.boundaries()[0];
/// # for face in shell.iter() {
/// #   let surface = face.lock_surface().unwrap().clone();
/// #   for i in 0..=N {
/// #       for j in 0..=N {
/// #           let u = i as f64 / N as f64;
/// #           let v = j as f64 / N as f64;
/// #           let pt = surface.subs(u, v);
/// #
/// #           // this surface is a part of torus.
/// #           let tmp = f64::sqrt(pt[0] * pt[0] + pt[2] * pt[2]) - 2.0;
/// #           let res = tmp * tmp + pt[1] * pt[1];
/// #           assert!(Tolerance::near(&res, &1.0));
/// #       }
/// #    }
/// # }
/// ```
/// ```
/// // Modeling a pipe.
/// use truck_modeling::*;
/// const PI: Rad<f64> = Rad(std::f64::consts::PI);
///
/// // Creates the base circle
/// let v: Vertex = builder::vertex(Point3::new(1.0, 0.0, 4.0));
/// let circle: Wire = builder::rsweep(&v, Point3::new(2.0, 0.0, 4.0), -Vector3::unit_z(), PI * 2.0);
///
/// // the result shell of the pipe.
/// let mut pipe: Shell = Shell::new();
///
/// // Draw the first line pipe
/// let mut first_line_part: Shell = builder::tsweep(&circle, Vector3::new(0.0, 0.0, -4.0));
/// pipe.append(&mut first_line_part);
///
/// // Get the new wire
/// let boundaries: Vec<Wire> = pipe.extract_boundaries();
/// let another_circle: Wire = boundaries.into_iter().find(|wire| wire != &circle).unwrap().inverse();
///
/// // Draw the bent part
/// let mut bend_part: Shell = builder::rsweep(
///     &another_circle,
///     Point3::origin(),
///     Vector3::unit_y(),
///     PI / 2.0,
/// );
/// # let surface = bend_part[0].lock_surface().unwrap().clone();
/// pipe.append(&mut bend_part);
///
/// // Get the new wire
/// let boundaries: Vec<Wire> = pipe.extract_boundaries();
/// let another_circle: Wire = boundaries.into_iter().find(|wire| wire != &circle).unwrap().inverse();
///
/// // Draw the second line pipe
/// let mut second_line_part: Shell = builder::tsweep(&another_circle, Vector3::new(-4.0, 0.0, 0.0));
/// pipe.append(&mut second_line_part);
///
/// assert_eq!(pipe.shell_condition(), ShellCondition::Oriented);
/// # assert!(pipe.is_geometric_consistent());
/// # const N: usize = 100;
/// # for i in 0..=N {
/// #    for j in 0..=N {
/// #        let u = i as f64 / N as f64;
/// #        let v = j as f64 / N as f64;
/// #        let pt = surface.subs(u, v);
/// #
/// #        // the y coordinate is positive.
/// #        //assert!(pt[1] >= 0.0);
/// #
/// #        // this surface is a part of torus.
/// #        let tmp = f64::sqrt(pt[0] * pt[0] + pt[2] * pt[2]) - 2.0;
/// #        let res = tmp * tmp + pt[1] * pt[1];
/// #        assert!(Tolerance::near(&res, &1.0));
/// #    }
/// # }
/// ```
#[inline(always)]
pub fn rsweep<T: ClosedSweep<Point3, NURBSCurve, NURBSSurface>, R: Into<Rad<f64>>>(
    elem: &T,
    origin: Point3,
    axis: Vector3,
    angle: R,
) -> T::Swept {
    let angle = angle.into();
    if angle.0.abs() < 2.0 * PI.0 {
        partial_rsweep(elem, origin, axis, angle)
    } else if angle.0 > 0.0 {
        whole_rsweep(elem, origin, axis)
    } else {
        whole_rsweep(elem, origin, -axis)
    }
}

fn partial_rsweep<T: MultiSweep<Point3, NURBSCurve, NURBSSurface>>(
    elem: &T,
    origin: Point3,
    axis: Vector3,
    angle: Rad<f64>,
) -> T::Swept {
    let division = if angle.0.abs() < PI.0 { 1 } else { 2 };
    let mat0 = Matrix4::from_translation(-origin.to_vec());
    let mat1 = Matrix4::from_axis_angle(axis, angle / division as f64);
    let mat2 = Matrix4::from_translation(origin.to_vec());
    let trsl = mat2 * mat1 * mat0;
    elem.multi_sweep(
        &move |pt| trsl.transform_point(*pt),
        &move |curve| NURBSCurve::new(trsl * curve.non_rationalized()),
        &move |surface| NURBSSurface::new(trsl * surface.non_rationalized()),
        &move |pt, _| {
            NURBSCurve::new(geom_impls::circle_arc(
                pt.to_homogeneous(),
                origin,
                axis,
                angle / division as f64,
            ))
        },
        &move |curve, _| {
            NURBSSurface::new(geom_impls::rsweep_surface(
                curve.non_rationalized(),
                origin,
                axis,
                angle / division as f64,
            ))
        },
        division,
    )
}

fn whole_rsweep<T: ClosedSweep<Point3, NURBSCurve, NURBSSurface>>(
    elem: &T,
    origin: Point3,
    axis: Vector3,
) -> T::Swept {
    let mat0 = Matrix4::from_translation(-origin.to_vec());
    let mat1 = Matrix4::from_axis_angle(axis, PI);
    let mat2 = Matrix4::from_translation(origin.to_vec());
    let trsl = mat2 * mat1 * mat0;
    elem.closed_sweep(
        &move |pt| trsl.transform_point(*pt),
        &move |curve| NURBSCurve::new(trsl * curve.non_rationalized()),
        &move |surface| NURBSSurface::new(trsl * surface.non_rationalized()),
        &move |pt, _| {
            NURBSCurve::new(geom_impls::circle_arc(
                pt.to_homogeneous(),
                origin,
                axis,
                PI,
            ))
        },
        &move |curve, _| {
            NURBSSurface::new(geom_impls::rsweep_surface(
                curve.non_rationalized(),
                origin,
                axis,
                PI,
            ))
        },
        2,
    )
}

#[test]
fn partial_torus() {
    let v = vertex(Point3::new(0.5, 0.0, 0.0));
    let w = rsweep(&v, Point3::new(0.75, 0.0, 0.0), Vector3::unit_y(), Rad(7.0));
    let face = try_attach_plane(&vec![w]).unwrap();
    let torus = rsweep(&face, Point3::origin(), Vector3::unit_z(), Rad(2.0));
    assert!(torus.is_geometric_consistent());
    let torus = rsweep(&face, Point3::origin(), Vector3::unit_z(), Rad(5.0));
    assert!(torus.is_geometric_consistent());
    let torus = rsweep(&face, Point3::origin(), Vector3::unit_z(), Rad(-2.0));
    assert!(torus.is_geometric_consistent());
    let torus = rsweep(&face, Point3::origin(), Vector3::unit_z(), Rad(-5.0));
    assert!(torus.is_geometric_consistent());
}
