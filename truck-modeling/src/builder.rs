use crate::*;
const PI: Rad<f64> = Rad(std::f64::consts::PI);

/// Creates and returns a vertex by a three dimensional point.
#[inline(always)]
pub fn vertex(pt: Point3) -> Vertex {
    Vertex::new(pt)
}

/// Returns a line from `vertex0` to `vertex1`.
#[inline(always)]
pub fn line(vertex0: &Vertex, vertex1: &Vertex) -> Edge {
    let curve = geom_impls::line(
        (*vertex0.lock_point().unwrap()).to_homogeneous(),
        (*vertex1.lock_point().unwrap()).to_homogeneous(),
    );
    Edge::new(vertex0, vertex1, NURBSCurve::new(curve))
}

/// Returns a circle arc from `vertex0` to `vertex1` via `transit`.
#[inline(always)]
pub fn circle_arc(vertex0: &Vertex, vertex1: &Vertex, transit: Point3) -> Edge {
    let curve = geom_impls::circle_arc_by_three_points(
        (*vertex0.lock_point().unwrap()).to_homogeneous(),
        (*vertex1.lock_point().unwrap()).to_homogeneous(),
        transit,
    );
    Edge::new(vertex0, vertex1, NURBSCurve::new(curve))
}

/// Returns a bezier curve from `vertex0` to `vertex1` with inter control points `inter_points`.
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
#[inline(always)]
pub fn homotopy(edge0: &Edge, edge1: &Edge) -> Face {
    let wire: Wire = vec![
        edge0.clone(),
        line(edge0.back(), edge1.front()),
        edge1.inverse(),
        line(edge1.front(), edge1.back()),
    ]
    .into();
    let curve0 = edge0.oriented_curve().into_non_rationalized();
    let curve1 = edge1.oriented_curve().into_non_rationalized();
    let surface = BSplineSurface::homotopy(curve0, curve1);
    Face::new(vec![wire], NURBSSurface::new(surface))
}

/// Try attatiching a plane whose boundary is `wire`.
/// Todo: Define the crate error and make return value `Result<Face>`!
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
///
/// This method is a redefinition of `Mapped::topological_clone()`.
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
/// # Examples
/// ```
/// // Modeling a pipe.
/// use truck_modeling::*;
/// const PI: Rad<f64> = Rad(std::f64::consts::PI);
///
/// // Creates the base circle
/// let v: Vertex = builder::vertex(Point3::new(1.0, 0.0, 4.0));
/// let circle: Wire = builder::rsweep(&v, Point3::new(2.0, 0.0, 4.0), -Vector3::unit_z());
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
/// let mut bend_part: Shell = builder::partial_rsweep(&another_circle, Point3::origin(), Vector3::unit_y(), PI / 2.0);
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
pub fn partial_rsweep<T: Sweep<Point3, NURBSCurve, NURBSSurface>>(
    elem: &T,
    origin: Point3,
    axis: Vector3,
    angle: Rad<f64>,
) -> T::Swept {
    let mat0 = Matrix4::from_translation(-origin.to_vec());
    let mat1 = Matrix4::from_axis_angle(axis, angle);
    let mat2 = Matrix4::from_translation(origin.to_vec());
    let trsl = mat2 * mat1 * mat0;
    elem.sweep(
        &move |pt| trsl.transform_point(*pt),
        &move |curve| NURBSCurve::new(trsl * curve.non_rationalized()),
        &move |surface| NURBSSurface::new(trsl * surface.non_rationalized()),
        &move |pt, _| {
            NURBSCurve::new(geom_impls::circle_arc(
                pt.to_homogeneous(),
                origin,
                axis,
                angle,
            ))
        },
        &move |curve, _| {
            NURBSSurface::new(geom_impls::rsweep_surface(
                curve.non_rationalized(),
                origin,
                axis,
                angle,
            ))
        },
    )
}

/// Sweeps a vertex, an edge, a wire, a face, or a shell by the whole circle.
/// # Examples
/// ```
/// // Torus
/// use truck_modeling::*;
/// const PI: Rad<f64> = Rad(std::f64::consts::PI);
///
/// let v: Vertex = builder::vertex(Point3::new(3.0, 0.0, 0.0));
/// let circle: Wire = builder::rsweep(&v, Point3::new(2.0, 0.0, 0.0), Vector3::unit_z());
/// let torus: Shell = builder::rsweep(&circle, Point3::origin(), Vector3::unit_y());
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
pub fn rsweep<T: ClosedSweep<Point3, NURBSCurve, NURBSSurface>>(
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
