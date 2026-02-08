use super::*;
use crate::errors::Error;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::panic::{catch_unwind, AssertUnwindSafe};

/// Evaluates the cubic B-spline basis function at parameter `u` for knot vector `a` (length 5).
///
/// The basis function is a piecewise cubic polynomial with 4 segments over `[a[0], a[4])`.
/// Returns 0 outside this range.
fn basis_function(u: f64, a: &[f64]) -> f64 {
    if u < a[0] || a[4] <= u {
        return 0.0;
    }

    if u < a[1] {
        let d = u - a[0];
        d * d * d / ((a[3] - a[0]) * (a[2] - a[0]) * (a[1] - a[0]))
    } else if u < a[2] {
        let scalar = 1.0 / (a[2] - a[1]);
        let terms: [(usize, usize, usize, usize, usize, usize, usize); 3] = [
            (0, 0, 2, 3, 0, 2, 0),
            (0, 1, 3, 3, 0, 3, 1),
            (1, 1, 4, 4, 1, 3, 1),
        ];
        scalar
            * terms
                .iter()
                .fold(0.0, |sum, &(n0, n1, n2, d0, d1, d2, d3)| {
                    sum + ((u - a[n0]) * (u - a[n1]) * (a[n2] - u))
                        / ((a[d0] - a[d1]) * (a[d2] - a[d3]))
                })
    } else if u < a[3] {
        let scalar = 1.0 / (a[3] - a[2]);
        let terms: [(usize, usize, usize, usize, usize, usize, usize); 3] = [
            (0, 3, 3, 3, 0, 3, 1),
            (1, 4, 3, 4, 1, 3, 1),
            (2, 4, 4, 4, 1, 4, 2),
        ];
        scalar
            * terms
                .iter()
                .fold(0.0, |sum, &(n0, n1, n2, d0, d1, d2, d3)| {
                    sum + ((u - a[n0]) * (a[n1] - u) * (a[n2] - u))
                        / ((a[d0] - a[d1]) * (a[d2] - a[d3]))
                })
    } else {
        let d = a[4] - u;
        d * d * d / ((a[4] - a[1]) * (a[4] - a[2]) * (a[4] - a[3]))
    }
}

/// Evaluates the 1st derivative of the cubic B-spline basis function at parameter `u`.
fn basis_function_d1(u: f64, a: &[f64]) -> f64 {
    if u < a[0] || a[4] <= u {
        return 0.0;
    }

    if u < a[1] {
        // N = (u-a0)^3 / D => N' = 3(u-a0)^2 / D.
        let d = u - a[0];
        3.0 * d * d / ((a[3] - a[0]) * (a[2] - a[0]) * (a[1] - a[0]))
    } else if u < a[2] {
        // Segment 2: sum of terms (u-ai)(u-aj)(ak-u) / denom, times scalar.
        // Each term is a product of 3 linear factors; derivative via product rule.
        let scalar = 1.0 / (a[2] - a[1]);
        let terms: [(usize, usize, usize, usize, usize, usize, usize); 3] = [
            (0, 0, 2, 3, 0, 2, 0),
            (0, 1, 3, 3, 0, 3, 1),
            (1, 1, 4, 4, 1, 3, 1),
        ];
        scalar
            * terms
                .iter()
                .fold(0.0, |sum, &(n0, n1, n2, d0, d1, d2, d3)| {
                    // f = (u-a[n0])(u-a[n1])(a[n2]-u), f' = product rule.
                    let f0 = u - a[n0];
                    let f1 = u - a[n1];
                    let f2 = a[n2] - u;
                    let denom = (a[d0] - a[d1]) * (a[d2] - a[d3]);
                    sum + (f1 * f2 + f0 * f2 - f0 * f1) / denom
                })
    } else if u < a[3] {
        let scalar = 1.0 / (a[3] - a[2]);
        let terms: [(usize, usize, usize, usize, usize, usize, usize); 3] = [
            (0, 3, 3, 3, 0, 3, 1),
            (1, 4, 3, 4, 1, 3, 1),
            (2, 4, 4, 4, 1, 4, 2),
        ];
        scalar
            * terms
                .iter()
                .fold(0.0, |sum, &(n0, n1, n2, d0, d1, d2, d3)| {
                    // f = (u-a[n0])(a[n1]-u)(a[n2]-u), f' = product rule.
                    let f0 = u - a[n0];
                    let f1 = a[n1] - u;
                    let f2 = a[n2] - u;
                    let denom = (a[d0] - a[d1]) * (a[d2] - a[d3]);
                    sum + (f1 * f2 - f0 * f2 - f0 * f1) / denom
                })
    } else {
        // N = (a4-u)^3 / D => N' = -3(a4-u)^2 / D.
        let d = a[4] - u;
        -3.0 * d * d / ((a[4] - a[1]) * (a[4] - a[2]) * (a[4] - a[3]))
    }
}

/// Evaluates the 2nd derivative of the cubic B-spline basis function at parameter `u`.
fn basis_function_d2(u: f64, a: &[f64]) -> f64 {
    if u < a[0] || a[4] <= u {
        return 0.0;
    }

    if u < a[1] {
        // N'' = 6(u-a0) / D.
        6.0 * (u - a[0]) / ((a[3] - a[0]) * (a[2] - a[0]) * (a[1] - a[0]))
    } else if u < a[2] {
        let scalar = 1.0 / (a[2] - a[1]);
        let terms: [(usize, usize, usize, usize, usize, usize, usize); 3] = [
            (0, 0, 2, 3, 0, 2, 0),
            (0, 1, 3, 3, 0, 3, 1),
            (1, 1, 4, 4, 1, 3, 1),
        ];
        scalar
            * terms
                .iter()
                .fold(0.0, |sum, &(n0, n1, n2, d0, d1, d2, d3)| {
                    // f = (u-a[n0])(u-a[n1])(a[n2]-u).
                    // f'' = 2[(a[n2]-u) - (u-a[n0]) - (u-a[n1])].
                    // Expanding: f'' = 2(a[n2] + a[n0] + a[n1] - 3u).
                    let denom = (a[d0] - a[d1]) * (a[d2] - a[d3]);
                    sum + 2.0 * (a[n2] + a[n0] + a[n1] - 3.0 * u) / denom
                })
    } else if u < a[3] {
        let scalar = 1.0 / (a[3] - a[2]);
        let terms: [(usize, usize, usize, usize, usize, usize, usize); 3] = [
            (0, 3, 3, 3, 0, 3, 1),
            (1, 4, 3, 4, 1, 3, 1),
            (2, 4, 4, 4, 1, 4, 2),
        ];
        scalar
            * terms
                .iter()
                .fold(0.0, |sum, &(n0, n1, n2, d0, d1, d2, d3)| {
                    // f = (u-a[n0])(a[n1]-u)(a[n2]-u).
                    // f' = f1*f2 - f0*f2 - f0*f1.
                    // f'' = -2f2 - 2f1 + 2f0 = 2(f0 - f1 - f2).
                    // f'' = 2((u-a[n0]) - (a[n1]-u) - (a[n2]-u)) = 2(3u - a[n0] - a[n1] - a[n2]).
                    let denom = (a[d0] - a[d1]) * (a[d2] - a[d3]);
                    sum + 2.0 * (3.0 * u - a[n0] - a[n1] - a[n2]) / denom
                })
    } else {
        // N'' = 6(a4-u) / D.
        6.0 * (a[4] - u) / ((a[4] - a[1]) * (a[4] - a[2]) * (a[4] - a[3]))
    }
}

impl<P> Tmesh<P> {
    /// Constructs a new rectangular T-mesh from four points in space and a value for
    /// outward-facing knot intervals. The result is the following mesh, where the
    /// numbers are the indices of the array `points`. The knot interval between
    /// each point is 1.0.
    /// ```text
    ///  3|   |2
    /// --+---+--
    ///   |   |
    /// --+---+--
    ///  0|   |1
    /// ```
    pub fn new(points: [P; 4], edge_knot_interval: f64) -> Tmesh<P> {
        // Convert points into control points
        let control_points: Vec<Arc<RwLock<TmeshControlPoint<P>>>> = Vec::from(points)
            .into_iter()
            .map(|p| {
                let cont_point = TmeshControlPoint::new(p, edge_knot_interval);
                Arc::new(RwLock::new(cont_point))
            })
            .collect();

        // Set the first point as the "knot origin". This may result in some negative components in the
        // knot vectors of the points near the left and bottom edge condition, but this should not matter (test?)
        control_points[0]
            .write()
            .set_knot_coordinates(0.0, 0.0)
            .expect("No connections have been created for the current mesh");

        // Connect control points according to the diagram in the docs
        let mut dir = TmeshDirection::Right;
        for i in 0..4 {
            control_points[i]
                .write()
                .remove_edge_condition(dir)
                .expect("Point edge conditions are known at compile time");

            control_points[(i + 1) % 4]
                .write()
                .remove_edge_condition(dir.flip())
                .expect("Point edge conditions are known at compile time");

            // Connect the point i to the point i plus one
            TmeshControlPoint::connect(
                Arc::clone(&control_points[i % 4]),
                Arc::clone(&control_points[(i + 1) % 4]),
                dir,
                1.0,
            )
            .expect("T-mesh connections are known valid at compile time");

            dir = dir.anti_clockwise();
        }

        Tmesh {
            control_points,
            knot_vectors: RwLock::new(None),
        }
    }

    /// Returns an immutable reference to the control points vector
    pub fn control_points(&self) -> &Vec<Arc<RwLock<TmeshControlPoint<P>>>> { &self.control_points }

    /// Inserts a control point with real space coordinates `p` on the side `connection_side`
    /// of `con`. The knot interval of the connection between con and the new control point
    /// is the current weight of the connection multiplied by the ratio. Thus if ratio is
    /// 0.0, the connection between con and the new control point will have an interval of
    /// 0.0. `con` must be a control point in `self` and the new control point `p` must be
    /// inserted between two existing points, that is, `con`'s connection on the side
    /// `connection_side` must not be an edge condition or a T-junction.
    ///
    /// >NOTE!
    /// > This will change the shape of the resulting surface.
    /// > Use Local Knot Insertion in order to add a control point
    /// > without changing the shape of the surface.
    ///
    /// # Returns
    /// - `TmeshInvalidKnotRatio` if `knot_ratio` is not in \[0.0, 1.0\].
    ///
    /// - `TmeshConnectionNotFound` if `con` has no connection on `connection_side`.
    ///
    /// - `TmeshControlPointNotFound` if `con` is an edge condition on `connction_side`.
    ///
    /// - `TmeshForeignControlPoint` if `con` is not a control point in the T-mesh.
    ///
    /// - `TmeshConnectionInvalidKnotInterval` if the connection between `con`
    ///   and the point in the direction `connection_side`, `con_side`, does not have the same
    ///   knot interval in both directions (`con` -> `con_side` != `con` <- `con_side`).
    ///   This should never happen.
    ///
    /// - `Ok(Arc<RwLock<TmeshControlPoint<P>>>)` if the control point was successfully added, which itself is returned.
    ///
    /// # Borrows
    /// Mutably borrows `con` and the point located in the direction `connection_side`, and potentially borrows all
    /// points that are a part of the faces on either side of the edge that connects `p` and the point located in
    /// the direction `connection_side`.
    ///
    /// # Panics
    /// Panics if any borrow does not succeed.
    pub fn add_control_point(
        &mut self,
        p: P,
        con: Arc<RwLock<TmeshControlPoint<P>>>,
        connection_side: TmeshDirection,
        knot_ratio: f64,
    ) -> Result<Arc<RwLock<TmeshControlPoint<P>>>> {
        // Check that the knot ratio is valid
        if !(0.0..=1.0).contains(&knot_ratio) {
            return Err(Error::TmeshInvalidKnotRatio);
        }

        // If con is not found in the mesh, return the corresponding error.
        if self
            .control_points
            .iter()
            .position(|x| Arc::ptr_eq(x, &con))
            .is_none()
        {
            return Err(Error::TmeshForeignControlPoint);
        }

        // Get the point currently connected to the connection point. Returns the
        // requisit errors in the case that the connection is not of type Point.
        let other_point = {
            let borrow = con.read();
            Arc::clone(&borrow.try_connected_point(connection_side)?)
        };

        // Edge weights for p are set to 0.0, however, the final step will overwrite this
        // if a different edge weight was specified in the T-mesh constructor
        let p = Arc::new(RwLock::new(TmeshControlPoint::new(p, 0.0)));

        let knot_interval = con
            .read()
            .connection_knot(connection_side)
            .ok_or(Error::TmeshConnectionNotFound)?;

        let other_knot_interval = other_point
            .read()
            .connection_knot(connection_side.flip())
            .ok_or(Error::TmeshConnectionNotFound)?;

        // Confirm that the knot intervals are the same in both directions.
        if !(knot_interval - other_knot_interval).so_small() {
            return Err(Error::TmeshConnectionInvalidKnotInterval);
        }

        // Break connections between con_point and other_point
        con.write()
            .remove_connection(connection_side)
            .expect("Guaranteed by previous checks");

        // Remove edge conditions for p
        p.write()
            .remove_edge_condition(connection_side)
            .expect("New control point has known edge conditions");
        p.write()
            .remove_edge_condition(connection_side.flip())
            .expect("New control point has known edge conditions");

        // Insert p with the proper knot intervals.
        // con <-> other becomes con <-> p <-> other
        // con <-> p
        TmeshControlPoint::connect(
            Arc::clone(&con),
            Arc::clone(&p),
            connection_side,
            knot_interval * knot_ratio,
        )
        .map_err(|_| Error::TmeshUnknownError)?;

        // p <-> other
        TmeshControlPoint::connect(
            Arc::clone(&p),
            Arc::clone(&other_point),
            connection_side,
            knot_interval * (1.0 - knot_ratio),
        )
        .map_err(|_| Error::TmeshUnknownError)?;

        // When a new point is added, there can only possibly be edge conditions on
        // the two sides perpendicular to the connection. If there is no edge condition,
        // Rule 2 for T-meshes [Sederberg et al. 2003] should be checked to find any
        // inferred connections (ic), and if it does not apply, the connection is removed.

        // TODO: Currently this code does not allow for knot intervals of 0, and needs to be
        // updated once a solution to figure 9 in [Sederberg et al. 2003] is found.
        if con.read().con_type(connection_side.clockwise()) == TmeshConnectionType::Edge {
            let edge_weight = con
                .read()
                .connection_knot(connection_side.clockwise())
                .expect("Edges must have a weight");

            p.write()
                .set_edge_con_weight(connection_side.clockwise(), edge_weight)
                .expect("New points have edge conditions as default connection type.");
        } else {
            // Remove the edge condition created by the constructor.
            let _ = p.write().remove_edge_condition(connection_side.clockwise());

            // If a point that satisfies Rule 2 from [Sederberg et al. 2003] is found, connect it.
            // Should also never return an error.
            self.find_inferred_connection(Arc::clone(&p), connection_side.clockwise())
                .map_err(|_| Error::TmeshUnknownError)?;
        }

        if con.read().con_type(connection_side.anti_clockwise()) == TmeshConnectionType::Edge {
            let edge_weight = con
                .read()
                .connection_knot(connection_side.anti_clockwise())
                .expect("Edges must have a weight");

            p.write()
                .set_edge_con_weight(connection_side.anti_clockwise(), edge_weight)
                .expect("New points have edge conditions as default connection type.");
        } else {
            // Remove the edge condition created by the constructor.
            let _ = p
                .write()
                .remove_edge_condition(connection_side.anti_clockwise());

            // If a point that satisfies Rule 2 from [Sederberg et al. 2003] is found, connect it.
            // Should also never return an error.
            self.find_inferred_connection(Arc::clone(&p), connection_side.anti_clockwise())
                .map_err(|_| Error::TmeshUnknownError)?;
        }

        // Add control point
        self.control_points.push(Arc::clone(&p));
        *self.knot_vectors.write() = None;
        Ok(p)
    }

    /// Attemps to add a control point to the mesh given the cartesian point `p` and the absolute knot coordinates `knot_coords`
    /// in the form `(s, t)`. In order for insertion to succeed, there must either be an S or T edge located at the parametric
    /// point `knot_coords` in the mesh `self`. Note that zero knot insertions will return an error, as the side on which to
    /// insert the zero knot is ambiguous.
    ///
    /// # Returns
    /// - `TmeshOutOfBoundsInsertion` if a control point is being inserted with either knot coordinate out of the range `[0.0, 1.0]`.
    ///
    /// - `TmeshExistingControlPoint` if a control point already exists at parametric coordinates `knot_coords`.
    ///
    /// - `TmeshMalformedMesh` if multiple edges are found which intersect the location of the new point.
    ///
    /// - `TmeshConnectionNotFound` if no edges are found which intersect the location of the new point.
    ///
    /// - `Ok(Arc<RwLock<TmeshControlPoint<P>>>)` if the control point was successfully added, which itself is returned.
    ///
    /// # Borrows
    /// Immutably borrows every point in the mesh `self`.
    pub fn try_add_absolute_point(
        &mut self,
        p: P,
        knot_coords: (f64, f64),
    ) -> Result<Arc<RwLock<TmeshControlPoint<P>>>> {
        // Make sure desred knot coordinates are within mesh bounds
        if knot_coords.0 < 0.0 || knot_coords.0 > 1.0 || knot_coords.1 < 0.0 || knot_coords.1 > 1.0
        {
            return Err(Error::TmeshOutOfBoundsInsertion);
        }

        // If a point already exists at the desired knot coordinates, return an error. Zero knot intervals can be put
        // on any side of a point and still have the same knot coordinates, but the structure of the mesh will not be
        // different. Thus, zero knot insertion must be done manually.
        if self
            .control_points
            .iter()
            .find(|c| {
                let c_coords = c.read().knot_coordinates();
                let comparison = (c_coords.0 - knot_coords.0, c_coords.1 - knot_coords.1);
                comparison.0.so_small() && comparison.1.so_small()
            })
            .is_some()
        {
            return Err(Error::TmeshExistingControlPoint);
        }

        // The function checks for any T or S edges that intersect the point in paramtric space where the
        // point is to be insertet, then computes the knot ratio needed such that the point is inserted
        // at the correct place and inserts it using add_control_point.

        // Check for any T edges which intersect the parametric location of the new point.
        let mut point_t_coord = 0.0;
        let mut con_knot = 0.0;
        let s_axis_straddle_points = self
            .control_points
            .iter()
            // Filter all points along the S axis of inserton
            .filter(|point| (point.read().knot_coordinates().0 - knot_coords.0).so_small())
            // Filter those points to only include the point that straddles the T axis of insertion
            .filter(|point| {
                if let Some(con) = point.read().get(TmeshDirection::Up) {
                    let temp_t_coord = point.read().knot_coordinates().1;
                    let temp_inter = con.1;

                    // Knot of the new point is located on the connection being investigated?
                    if temp_t_coord < knot_coords.1 && temp_t_coord + temp_inter > knot_coords.1 {
                        point_t_coord = temp_t_coord; // T coordinate of the current point
                        con_knot = temp_inter; // Edge knot interval

                        return true;
                    }
                }
                false
            })
            .map(Arc::clone)
            .collect::<Vec<Arc<RwLock<TmeshControlPoint<P>>>>>();

        // Depending on the number of points whose connections intersect the location of the new point,
        // different errors or actions are taken
        match s_axis_straddle_points.len() {
            // No T-edge instersects the point where the point needs to be inserted,
            // try to find an S edge which intersects the location of the point
            0 => {}
            1 => {
                // A T-edge is found where the point intersects
                return self
                    .add_control_point(
                        p,
                        Arc::clone(&s_axis_straddle_points[0]),
                        TmeshDirection::Up,
                        (knot_coords.1 - point_t_coord) / con_knot,
                    )
                    .map_err(|_| Error::TmeshUnknownError);
            }
            _ => {
                // Multiple T-edges are found where the point intersects (Should never happen)
                return Err(Error::TmeshMalformedMesh);
            }
        };

        let mut point_s_coord = 0.0;
        let mut con_knot = 0.0;
        let t_axis_straddle_points = self
            .control_points
            .iter()
            // Filter all points along the T axis of inserton
            .filter(|point| (point.read().knot_coordinates().1 - knot_coords.1).so_small())
            // Filter those points to only include the point that straddles the S axis of insertion
            .filter(|point| {
                if let Some(con) = point.read().get(TmeshDirection::Right) {
                    let temp_s_coord = point.read().knot_coordinates().0;
                    let temp_inter = con.1;

                    // Knot of the new point is located on the connection being investigated?
                    if temp_s_coord < knot_coords.0 && temp_s_coord + temp_inter > knot_coords.0 {
                        point_s_coord = temp_s_coord; // S coordinate of the current point
                        con_knot = temp_inter; // Edge knot interval

                        return true;
                    }
                }
                false
            })
            .map(Arc::clone)
            .collect::<Vec<Arc<RwLock<TmeshControlPoint<P>>>>>();

        // Depending on the number of points whose connections intersect the location of the new point,
        // different errors or actions are taken
        match t_axis_straddle_points.len() {
            0 => {
                // No S-edge instersects the point where the point needs to be inserted, return an error
                Err(Error::TmeshConnectionNotFound)
            }
            1 => {
                // An S-edge is found where the point intersects
                self.add_control_point(
                    p,
                    Arc::clone(&t_axis_straddle_points[0]),
                    TmeshDirection::Right,
                    (knot_coords.0 - point_s_coord) / con_knot,
                )
                .map_err(|_| Error::TmeshUnknownError)
            }
            _ => {
                // Multiple S-edges are found where the point intersects (Should never happen)
                Err(Error::TmeshMalformedMesh)
            }
        }
    }

    /// Generates the S and T knot vectors for a particular point. The returned tuple is of the form `(S_vector, T_vector)`,
    /// where `S_vector` is the horizontal knot vector and `T_vector` is the vertical knot vector. Both knot vectors shall
    /// be of length 5
    ///
    /// # Returns
    /// - `TmeshConnectionNotFound` if a T-junction is unexpectedly found (non-rectangular face)
    ///
    /// - `TmeshControlPointNotFound` if an edge condition is unexpectedly found (internal edge condition)
    ///
    /// - `Ok((KnotVec, KnotVec))` if knot vectors are successfully generated
    ///
    /// # Borrows
    /// Immutably borrows `p` and all points connected to `p` in all directions for a distance of two knot intervals.
    /// In the case that `p` is not connected to a point in a direction, but instead a T-junction, any points
    /// that are a part of the face which `p` is a part of and the next face in that direction may be borrowed,
    /// with no guarantees as to which or how many.
    fn point_knot_vectors(p: Arc<RwLock<TmeshControlPoint<P>>>) -> Result<(KnotVec, KnotVec)> {
        let mut s_vec: Vec<f64> = vec![0.0; 5];
        let mut t_vec: Vec<f64> = vec![0.0; 5];

        // Center of the knot vec is the knot coordinate of the current point
        s_vec[2] = p.read().knot_coordinates().0;
        t_vec[2] = p.read().knot_coordinates().1;

        // Cast rays in all directions
        for dir in TmeshDirection::iter() {
            let cur_point = Arc::clone(&p);
            // Knot intervals for intersections (These are deltas, not absolutes)
            let knot_intervals = Tmesh::cast_ray(cur_point, dir, 2)?;

            for i in 0..2 {
                let inter = knot_intervals[i];

                // Knot vectors for a point go left to right and lower to upper as the index increases.
                // Knot interval will be the knot interval from the center point to the i'th point in the direction dir.
                // (The mesh will most likely look different, with T junctions and edge conditions)
                //           [T]    Initial cur_point
                //            + 4  /
                //            |   /
                //            + 3/
                //            | /
                //  +----+----+----+----+  [S]
                //  0    1    |    3    4
                //            + 1
                //            |
                //            + 0
                match dir {
                    TmeshDirection::Up => {
                        t_vec[3 + i] = t_vec[2 + i] + inter;
                    }
                    TmeshDirection::Right => {
                        s_vec[3 + i] = s_vec[2 + i] + inter;
                    }
                    TmeshDirection::Down => {
                        t_vec[1 - i] = t_vec[2 - i] - inter;
                    }
                    TmeshDirection::Left => {
                        s_vec[1 - i] = s_vec[2 - i] - inter;
                    }
                }
            }
        }
        Ok((KnotVec::from(s_vec), KnotVec::from(t_vec)))
    }

    /// Generates the knot vectors for each control point using the method in \[Sederberg et al. 2003\].
    /// The knot vector for a control point is located at the same index as the control point is in `self.control_points`.
    /// Each pair of knot vectors is arranged as `(s, t)` where `s` is the horizontal and `t` is the vertical.
    ///
    /// # Returns
    /// All errors returned from the function result from a malformed T-mesh and should not
    /// - `TmeshConnectionNotFound` if a non-rectangular face is encountered.
    ///
    /// - `TmeshControlPointNotFound` if an unexpected edge condition is found.
    ///
    /// - `Ok(())` if knot vectors are successfully generated.
    ///
    /// # Borrows
    /// Immutably borrows every point in `self.control_points`.
    fn generate_knot_vectors(&self) -> Result<()> {
        let mut knot_vecs: Vec<(KnotVec, KnotVec)> = Vec::new();

        for control_point in self.control_points.iter() {
            knot_vecs.push(Tmesh::point_knot_vectors(Arc::clone(control_point))?);
        }

        *self.knot_vectors.write() = Some(knot_vecs);
        Ok(())
    }

    /// Finds and creates an inferred connection on the point `p` for the anti-clockwise
    /// face which `face_dir` points into the face and which `p` is a part of. `p` must be part of
    /// a valid face and must not be a corner (a connection cannot already exist in the `face_dir`
    /// direction.)
    ///
    /// > **Warning**\
    /// > Does not check if the face is valid.
    ///
    ///
    /// Example mesh for reference:
    /// ```text
    /// +----+-+-|+|-----{+}
    /// |    |    ^       |
    /// +----+---[+]-+---<+>
    /// |            |    |
    /// +-+----+-----+----+
    /// ```
    /// - `face_dir` points up
    /// - `[+]` is `p`
    /// - `<+>` and `{+}` are used in the internal comments
    ///
    /// `p`, labeled `[+]`, will be connected to `|+|` after calling `find_inferred_connection`
    ///
    /// # Returns
    /// - `TmeshConnectionNotFound` if any connection that is expected to
    ///   exist does not. This should only happen on a malformed T-mesh.
    ///
    /// - `TmeshControlPointNotFound` if any control point that is expected
    ///   to exist does not. This usually happens because the current face does not exist
    ///   (`p` is an edge condition).
    ///
    /// - `TmeshExistingConnection` if a connection exists in the `face_dir` direction
    ///   (`p` is a corner).
    ///
    /// - `Ok(true)` if an inferred connection was found and connected.
    ///
    /// - `Ok(false)` if an inferred connection was not found.
    ///
    /// # Borrows
    /// Immutably borrows all points along the anti-clockwise face path between
    /// `p` and `|+|`.
    ///
    /// Mutably borrows `p` and `|+|`.
    ///
    /// # Panics
    /// If `p` or the potential point to which the inferred connection will go to
    /// cannot be borrowed mutably, `find_inferred_connection` will panic.
    ///
    /// # Zero Knot Intervals
    /// While this function is capable of inserting points with zero knot intervals in every (legaal)
    /// case, there are no guarantees as to how points will be connected with a zero knot interval
    /// regarding implicit connections (Cross-face connections).
    fn find_inferred_connection(
        &mut self,
        p: Arc<RwLock<TmeshControlPoint<P>>>,
        face_dir: TmeshDirection,
    ) -> Result<bool> {
        let mut cur_point = Arc::clone(&p);
        let mut cur_dir = face_dir.clockwise();
        let mut ic_knot_measurement = 0.0; // The distance traversed from p to <+>
        let mut ic_knot_interval = 0.0; // The interval of the ic

        // Check that p is not a corner
        if p.read().con_type(face_dir) == TmeshConnectionType::Point {
            return Err(Error::TmeshExistingConnection);
        }

        // Traverse in the direction cur_dir until an anti-clockwise connection is found.
        // Repeat once to get to the point {+}
        for i in 0..2 {
            let accumulation: f64;

            (cur_point, accumulation) = cur_point
                .read()
                .navigate_until_con(cur_dir, cur_dir.anti_clockwise())?;

            cur_dir = cur_dir.anti_clockwise();

            if i == 0 {
                // Accumulate knot intevals for comparison later. Only accumulate knots that are
                // related to the current face
                ic_knot_measurement = accumulation;
            } else if i == 1 {
                // Accumulate knot intervals for the potential IC knot weight. Only accumulate
                // knots that are related to the current face
                ic_knot_interval = accumulation;
            }
        }

        // After the above loop, cur_point is located at {+} and cur_dir points opposite
        // connection_side. Start accumulating knot intervals until the edge of the face
        // is reached, the accumulation is greater than the measurement, or the two are equal.
        let mut ic_knot_accumulation = 0.0;
        loop {
            ic_knot_accumulation += cur_point
                .read()
                .connection_knot(cur_dir)
                .ok_or(Error::TmeshConnectionNotFound)?;

            cur_point = {
                let borrow = cur_point.read();
                Arc::clone(&borrow.try_connected_point(cur_dir)?)
            };

            // Ic found
            if (ic_knot_measurement - ic_knot_accumulation).so_small() {
                let connection_res = TmeshControlPoint::connect(
                    Arc::clone(&p),
                    Arc::clone(&cur_point),
                    cur_dir.clockwise(),
                    ic_knot_interval,
                );

                // If an existing connection is found, it is possible that the next point over
                // will be a zero knot interval, in which case the connection should go to that point.
                // Any other error should be sent up and if the connection is successful the same thing should happen.
                match connection_res {
                    Ok(()) => return Ok(true),
                    Err(Error::TmeshExistingConnection) => {}
                    Err(e) => return Err(e),
                };

            // Ic not possible, knot accumulation surpassed measurment or reached face corner.
            // Shouldn't need corner detection due to rule 1 in [Sederberg et al. 2003].
            // (needs testing)
            } else if ic_knot_accumulation > ic_knot_measurement
                || cur_point.read().con_type(cur_dir.anti_clockwise()) == TmeshConnectionType::Point
            {
                return Ok(false);
            }
        }
    }

    /// Casts a ray from `p` in the direction `dir` for `num` intersections, returning a vector containing the knot
    /// intervals of each intersection. When an edge condition is encountered before `num` intersections have been
    /// crossed, the returned vector contains the edge knot interval once, after which it is padded with `0.0`.
    /// All vectors returned from this function will have a length `num`.
    ///
    /// # Returns
    /// - `TmeshConnectionNotFound` if a T-mesh is found on the edge of a face, making it non-rectangular (malformed mesh).
    ///
    /// - `TmeshControlPointNotFound` if an edge condition is found inside the mesh or
    ///   if edge condition points are not connected to each other (malformed mesh).
    ///
    /// - `Ok(vec<f64>)` if the ray was successfully cast, returns the knot intervals traversed.
    ///  
    /// # Borrows
    /// Immutably borrows `p` and any points connected to `p` in the direction `dir`, including points which go around any
    /// faces created by T-juctions in the direction `dir`, for `num` perpendicular intersections.
    pub fn cast_ray(
        p: Arc<RwLock<TmeshControlPoint<P>>>,
        dir: TmeshDirection,
        num: usize,
    ) -> Result<Vec<f64>> {
        let mut knot_intervals = Vec::with_capacity(num);
        let mut cur_point = Arc::clone(&p);

        // Some flags for special cases.
        //
        // If an edge condition is found, only the first "intersection" at the edge contion is recorded,
        // and all further deltas are 0, though according to [Sederberg et al. 2003] they do not matter.
        let mut edge_condition_found = false;

        // 'intersection_loop:
        while knot_intervals.len() < num {
            let con_type = cur_point.read().con_type(dir);
            let i = knot_intervals.len();
            knot_intervals.push(0.0);

            match con_type {
                // If dir is a T-junction, navigate around the face to the other side,
                // counting the knot intervals in the direction dir
                TmeshConnectionType::Tjunction => {
                    // Stores the distance traversed away from the ray
                    let mut ray_distance: f64;
                    (cur_point, ray_distance) = {
                        let borrow = cur_point.read();

                        // The possibility that TmeshControlPointNotFound is returned from navigate_until_con would normaly be no
                        // cuase for error, since the other direction may be tried. However, because cur_point is a T junction in
                        // the direction dir, it must be a point connection in dir.anti_clockwise(), otherwise the mesh is malformed.
                        borrow.navigate_until_con(dir.anti_clockwise(), dir)?
                    };

                    // Travrese with counting until a connection in the clockwise connection is found.
                    // Because all faces must be rectangular, this is guaranteed to be the first "ray intersection".
                    let traversal_result =
                        cur_point.read().navigate_until_con(dir, dir.clockwise())?;
                    cur_point = traversal_result.0;
                    // Set the latest pushed value to the intersection length
                    knot_intervals[i] += traversal_result.1;

                    // If a T-junction is encountered, it is (Figure 9 cases aside) guaranteed that on the other side of the face there
                    // is no point which perfectly aligns with the initial point. In this case, a special algorithm must be used to
                    // traverse across the mesh until such a point is found or the requisite number of intersections are reached.
                    // Example below (All distances are in parametric space and represented by physical space between "+", which are points):
                    // <+>---\+/----------------------------------+
                    //  |     |                                   |
                    //  |    [+]-----<+>--+---+-----+--<+>---<+>--+
                    //  |     |       |   |   |     |   |     |   |
                    //  |     |       |   |   |    <+>-<+>    |   |
                    //  |     |       |   |   |     |   |     |   |
                    //  |     |      [+]-(+)-<+>    |  /+\    |   |
                    //  |     |       |       |     |   |    <+>-<+>
                    //  |     |       |      <+>---<+>  |     |   |
                    // {+}~~~~|~~~~~~/+\~~~~~~|~~~~~|~~~|~~~~~|~~|+|
                    //  |     +-------+       |     |   |     |   |
                    //  |     |       |       +----/+\-/+\----+---+
                    //  |     |       |       |                   |
                    //  +-----+-------+-------+-------------------+
                    //  0     1       2       3     4   5     6   7     <-- Intersection numbers, used in comments
                    //
                    // {+} is point from which the ray is "cast"
                    // <+> are points that need to be visited by the algorithm
                    // [+] are the points where if normal ray casting is resumed,
                    //      an incorrect knot vector will be produced.
                    // (+) is a point whose knot interval will be accumulated but not recorded for
                    // |+| is the point at which "normal" ray casting continues (may or may not exist, and
                    //      must not have a T-junction to the right).
                    // /+\ are points which, while closer to the ray in a paramtric sense,
                    //      are not directly accessed for the reasons described in the next paragraph
                    // \+/ is the locatioin of cur_point
                    //  ~  is the "ray"
                    //
                    // In any case, the path taken shall not cross the ray. It can be guaranteed that any edge
                    // the ray pierces will be accessable by this algorithm due to the rectangular nature of the T-mesh.
                    // Lets say that there exists a vertical edge which the ray pierces. That edge must be connected on
                    // either edge to horizontal edges. At the corners, there will be control points. Thus, two of
                    // the control points must be above the ray. Furthermore, to preserve the rectangular nature of
                    // each face, those control points must be connected to two other edges, meaning that at least
                    // one edge from that control point will be pointing up or left, connecting to another edge.
                    // This means that as long as the algorithm used to traverse the mesh stays as close to the ray as possible,
                    // without crossing it, (that is, always stays on a face which is intersected by the ray), there is no
                    // danger of missing an intersection and producing an incorrect knot vector.
                    //
                    // The above code is not included in the loop below because of certain guarantees that can be made about the
                    // geometry of the mesh which cannot be made for the rest of the mesh.
                    'face_traversal: loop {
                        // It is possible that we are traversing along the edge of the mesh, in this case, the below navigate_until_con is
                        // going to navigate until the corner of the mesh, and return an error that it encountered an unexpected
                        // edge condition. This is not actually an error, so it needs to be checked before traversal. In the event that this occurs,
                        // normal ray casting is resumed, since all edge conditions in a mesh have the same weight. Do not push another knot interval,
                        // because the edge arm of the parent match statement will take care of it
                        if cur_point.read().con_type(dir) == TmeshConnectionType::Edge {
                            break 'face_traversal;
                        }

                        if knot_intervals.len() == num {
                            break 'face_traversal;
                        }

                        // Traverse down to the lowest point on this edge which is not a T-junction and has not yet crossed the ray.
                        'ray_approaching: loop {
                            let traversal_result =
                                cur_point.read().navigate_until_con(dir.clockwise(), dir)?;

                            // Subtract distance as we approach the ray (temp var because the result might be
                            // over the ray, in which case we discard it).
                            let new_ray_distance = ray_distance - traversal_result.1;

                            // Found a point where normal ray traversal will continue
                            if new_ray_distance.so_small() {
                                break 'face_traversal;

                            // The detected point crosses the ray, so cur_point is the closest point to the ray with a
                            // connection in the dir direction.
                            } else if new_ray_distance < 0.0 {
                                break 'ray_approaching;
                            }

                            // Move cur_point
                            cur_point = traversal_result.0;
                            // Synchronize distance
                            ray_distance = new_ray_distance;
                        }

                        // It is possble that the above loop exited without modifying cur_point, as is the case for the face marked by
                        // the fourth and fifth intersections above. In this case, cur_point must be navigated up to the corner of the face.
                        if cur_point.read().con_type(dir) == TmeshConnectionType::Tjunction {
                            let traversal_result = cur_point
                                .read()
                                .navigate_until_con(dir.anti_clockwise(), dir)?;

                            // Move cur_point.
                            cur_point = traversal_result.0;
                            // Add distance, since we are traversing away from the ray.
                            ray_distance += traversal_result.1;
                        }

                        // Traverse accross the "top" of the face, to the other corner
                        let traversal_result =
                            cur_point.read().navigate_until_con(dir, dir.clockwise())?;

                        // Record the traversal distance as a knot interval (guaranteed to be correct because all faces are rectangular)
                        knot_intervals.push(traversal_result.1);
                        // Move cur_point
                        cur_point = traversal_result.0;
                    }
                }

                TmeshConnectionType::Point => {
                    // Store knot interval
                    knot_intervals[i] += cur_point.read().connection_knot(dir).expect(
                        "All point connections and edge conditions must have a knot interval",
                    );

                    // Traverse to the next point
                    cur_point = {
                        let borrow = cur_point.read();
                        Arc::clone(&borrow.connected_point(dir))
                    };
                }

                TmeshConnectionType::Edge => {
                    // Edge contition already found, and pushing a zero happens before the match statement, so just continue.
                    if edge_condition_found {
                        continue;
                    }

                    // Store knot interval
                    knot_intervals[i] += cur_point.read().connection_knot(dir).expect(
                        "All point connections and edge conditions must have a knot interval",
                    );

                    // Flag to store zeros for remaining deltas
                    edge_condition_found = true;
                }
            };
        }
        Ok(knot_intervals)
    }
}

impl<P> Tmesh<P>
where P: PartialEq
{
    /// Finds the first point that was added to a T-mesh with a specific cartesian coordinate
    ///
    /// # Returns
    /// - `TmeshControlPointNotFound` if `p` is not found.
    ///
    /// - `Ok(Arc<RwLock<TmeshControlPoint<P>>>)` if the corresponding control point is found.
    ///
    /// # Borrows
    /// Immutably borrows every control point in the `self.control_points`.
    pub fn find(&self, p: P) -> Result<Arc<RwLock<TmeshControlPoint<P>>>> {
        Ok(Arc::clone(
            self.control_points()
                .iter()
                .find(|x| *x.read().point() == p)
                .ok_or(Error::TmeshControlPointNotFound)?,
        ))
    }

    /// Finds a control point with cartesian coordinates `point` and changes them to `new`.
    ///
    /// # Returns
    /// - `TmeshControlPointNotFound` if `point` is not found.
    ///
    /// - `Ok(Arc<RwLock<TmeshControlPoint<P>>>)` if the corresponding control point is found.
    ///
    /// # Borrows
    /// Immutably borrows every point in `self.control_points` and mutably borrows the
    /// control point corresponding to `point` if it is found.
    pub fn map_point(&mut self, point: P, new: P) -> Result<Arc<RwLock<TmeshControlPoint<P>>>> {
        let point = self.find(point)?;
        point.write().set_point(new);
        Ok(point)
    }
}

impl<P> Tmesh<P>
where P: ControlPoint<f64>
{
    /// Attempts to insert a new control point between two existing control points using the technique from \[Sederberg et al. 2003\]
    /// called local knot insertion (LKI), returning the added control point if successful. In order to do so, the knot vectors perpendicular
    /// to the connection for two control points in both directions (including the control points which define the edge) must be equal.
    /// See the figure below for an example.
    ///
    /// ```text
    ///     t1   t2        t3   t4
    ///     +-----+----(+)----+-----+
    ///     |     |           |     |
    ///     +-(+)-+-----------+-----+
    ///     |     |           |     |
    ///  --<+>---{+}---[+]---<+>---<+>--
    ///     |     |           |     |
    ///     +-----+-(+)-------+-----+
    ///     |     |           |     |
    ///     +-----+-----------+-(+)-+
    /// ```
    ///
    /// - `{+}` is `p`, which must exist
    /// - `<+>` are the other points which must exist. Any other points (other than `p`) may or may not exist,
    ///   and LKI will succeed so long as the perpendicular knot vectors are equal for all points `<+>` and `{+}`.
    /// - `[+]` is the point to be inserted.
    /// - `t1 - t5` are the knot vectors perpendicular to the axis of insertion
    /// - `(+)` are points which will not affect or be affected by LKI
    ///
    /// In the above example, the vertical knot vectors t1, t2, t3, and t4 must be equal
    /// (tollerance is used, so exact floating point equality is not nescessary).
    ///
    /// Other points may exist on any of the horizontal connections, so long as they are not on the primary axis
    /// (that would change which points `<+>` or `{+}` would be). Some examples are shown in the diagram as `(+)`.
    /// There can be edges between them, and even induce a connection with the newly inserted point,
    /// which will be automatically added.
    ///
    /// # Returns
    /// - `TmeshControlPointNotFound` if an edge condition is encountered instead of a control point
    ///   along the axis of insertion (Rule 3 \[Sederberg et al. 2003\]).
    ///
    /// - `TmeshConnectionNotFound` if a T-junction is encountered instead of a control point
    ///   along the axis of insertion (Rule 3 \[Sederberg et al. 2003\]).
    ///
    /// - `TmeshInvalidKnotRatio` if `knot_ratio` is not in [0.0, 1.0].
    ///
    /// - `TmeshMalformedMesh` if a knot vector was unable to be constructed for any point.
    ///
    /// - `TmeshKnotVectorsNotEqual` if the knot vectors perpendicular to `dir` are not all equal (Rule 3 \[Sederberg et al. 2003\]).
    ///
    /// - `TmeshConnectionInvalidKnotInterval` if the connection between `p` and the point in the direction `dir` does
    ///   not have the same knot interval in both directions.
    ///
    /// - `Ok(Arc<RwLock<TmeshControlPoint<P>>>)` if the control point was successfully added, where the
    ///   returned control point is the newly added control point
    ///
    /// # Borrows
    /// Immutably borrows two points in the direction `dir` of `p` and one in the direction `dir.flip()`, as well as two points in
    /// either direction perpendicular to `dir` for those points.  
    ///
    /// Mutably borrows `p` and the point connecteed to `p` in the direction `dir`, as well as the newly created control point,
    /// which lies between the two.
    ///
    /// # Notes on Rule 3
    /// Though \[Sederberg et al. 2003\] is not explicitly clear about edge condition (T-junctions imply rule 3 is broken), testing has
    /// revealed that local knot insertion cannot be done on edges connected to a point with one or more edge conditionds.
    pub fn try_local_knot_insertion(
        &mut self,
        p: Arc<RwLock<TmeshControlPoint<P>>>,
        dir: TmeshDirection,
        knot_ratio: f64,
    ) -> Result<Arc<RwLock<TmeshControlPoint<P>>>> {
        match p.read().con_type(dir) {
            TmeshConnectionType::Edge => return Err(Error::TmeshControlPointNotFound),
            TmeshConnectionType::Tjunction => return Err(Error::TmeshConnectionNotFound),
            _ => {}
        };

        if !(0.0..=1.0).contains(&knot_ratio) {
            return Err(Error::TmeshInvalidKnotRatio);
        }

        // Rule 3 of T-splines, [Sederberg et al. 2003], states that all (The paper does not specify existing or otherwise,
        // I am assuming that they may or may not exist, however, the connection from the inner two points must not be
        // a T-junction) perpendicular and in-line knot vectors of length 5 centered on the axis
        // of insertion and a distance of at most two knots from the point to be inserted must be equal. See Figure 10 in
        // [Sederberg et al. 2003] for details.
        let mut center_points: Vec<Arc<RwLock<TmeshControlPoint<P>>>> = Vec::with_capacity(4);

        // An example insertion for reference
        //
        //   --<+>--{+}--[+]---+--<+>--
        //      0    1    ~    2   3   <- center_points and knot_vectors indices
        // {+} is p
        // [+] is the new control point to be inserted
        // <+> may or may not exist (can only insert if they are replaced with edge conditions)
        center_points.push({
            match p.read().con_type(dir.flip()) {
                // Retrieve connected point
                TmeshConnectionType::Point => Arc::clone(&p.read().connected_point(dir.flip())),
                TmeshConnectionType::Edge => return Err(Error::TmeshControlPointNotFound),
                TmeshConnectionType::Tjunction => {
                    return Err(Error::TmeshConnectionNotFound);
                }
            }
        });
        center_points.push(Arc::clone(&p));
        center_points.push({
            let borrow = p.read();
            // Checked in the beginning of the function with match
            Arc::clone(&borrow.connected_point(dir))
        });
        center_points.push({
            let borrow = center_points[2].read();

            match borrow.con_type(dir.flip()) {
                // Retrieve connected point
                TmeshConnectionType::Point => Arc::clone(&borrow.connected_point(dir)),
                TmeshConnectionType::Edge => return Err(Error::TmeshControlPointNotFound),
                TmeshConnectionType::Tjunction => {
                    return Err(Error::TmeshConnectionNotFound);
                }
            }
        });

        // Store the first knot vector to compare it to the rest. If any do not match, return an error
        let knot_vec_compare: KnotVec = {
            let point_knots = Tmesh::point_knot_vectors(Arc::clone(&center_points[1]))?;

            // Depending on the direction of insertion, the S or T knot vectors are needed.
            if dir.horizontal() {
                point_knots.1
            } else {
                point_knots.0
            }
        };
        // Compare knot vectors
        for point in center_points[1..].iter() {
            // Get knot vectors in both directions for the point
            let point_knots = Tmesh::point_knot_vectors(Arc::clone(point))
                .map_err(|_| Error::TmeshMalformedMesh)?;

            // Depending on the direction of insertion, the S or T knot vectors are needed.
            let cur_kv = if dir.horizontal() {
                point_knots.1
            } else {
                point_knots.0
            };

            // Compare knot vectors using so_small because knot vector construction uses
            // knot intervals which are prone to small errors.
            if !cur_kv
                .iter()
                .zip(knot_vec_compare.iter())
                .all(|t| (t.0 - t.1).so_small())
            {
                return Err(Error::TmeshKnotVectorsNotEqual);
            }
        }

        // Get d1 - d6. See Figure 10 in [Sederberg et al. 2003].
        let mut d: Vec<f64> = Vec::with_capacity(6);
        // d1 and d2
        for point in center_points[0..2].iter() {
            d.push(
                Tmesh::cast_ray(Arc::clone(point), dir.flip(), 1)
                    .map_err(|_| Error::TmeshMalformedMesh)?[0],
            );
        }
        // d3
        d.push(
            center_points[1]
                .read()
                .connection_knot(dir)
                .ok_or(Error::TmeshConnectionNotFound)?
                * knot_ratio,
        );
        // d4
        d.push(d.last().expect("Vector should not be empty") * ((1.0 / knot_ratio) - 1.0));
        // d5 and d6
        for point in center_points[2..4].iter() {
            d.push(
                Tmesh::cast_ray(Arc::clone(point), dir, 1)
                    .map_err(|_| Error::TmeshMalformedMesh)?[0],
            );
        }

        let cartesian_points: Vec<P> = center_points.iter().map(|p| *p.read().point()).collect();

        // Equations 5, 6, and 7 from [Sederberg et al. 2003]. Remember that P3 is not a point in either
        // cartesian_points or center_points, and arrays in rust are 0 indexed,
        let p2_prime = ((cartesian_points[0] * d[3])
            + (cartesian_points[1].to_vec() * (d[0] + d[1] + d[2])))
            / (d[0] + d[1] + d[2] + d[3]);

        let p4_prime = ((cartesian_points[3] * d[2])
            + (cartesian_points[2].to_vec() * (d[3] + d[4] + d[5])))
            / (d[2] + d[3] + d[4] + d[5]);

        let p3_prime = ((cartesian_points[1] * (d[3] + d[4]))
            + (cartesian_points[2].to_vec() * (d[1] + d[2])))
            / (d[1] + d[2] + d[3] + d[4]);

        center_points[1].write().set_point(p2_prime);

        center_points[2].write().set_point(p4_prime);

        self.add_control_point(p3_prime, Arc::clone(&p), dir, knot_ratio)
    }

    /// Absolute knot coordinate interface for local knot insertion (LKI). Tries to insert a control point
    /// at the specified absolute knot coordinates `knot_coords` without changing the shape of the resulting surface.
    /// For details on LKI, see [`Tmesh::try_local_knot_insertion()`]. In order for the function to succeed, an edge must
    /// exist which passes through the knot coordinates `knot_coords`, that is, either two vertical points or horizontal
    /// points straddle the parametric coordinates where the new point is to be inserted.
    ///
    /// # Returns
    /// - `TmeshOutOfBoundsInsertion` if either component of `knot_coords` is not in the range `(0.0, 1.0)`.
    ///
    /// - `TmeshExistingControlPoint` if a control point already exists at the parametric coordinates `knot_coords`.
    ///
    /// - `TmeshMalformedMesh` if intersecting edges are found.
    ///
    /// - `TmeshConnectionNotFound` if no edges are found intersecting the knot coordinates `knot_coords`.
    ///
    /// # Borrows
    /// Immutably borrows every control point in `self`, immutably borrows two points in the direction `dir` of `p`
    /// and one in the direction `dir.flip()`, as well as two points in either direction perpendicular to `dir` for those points.  
    ///
    /// Mutably borrows the two control points which straddle the knot coordinates `knot_coords`, as well as the newly created control point,
    /// which lies at those knot coordinates.
    pub fn try_absolute_local_knot_insertion(
        &mut self,
        knot_coords: (f64, f64),
    ) -> Result<Arc<RwLock<TmeshControlPoint<P>>>> {
        // Make sure desred knot coordinates are within msh bounds
        if knot_coords.0 < 0.0 || knot_coords.0 > 1.0 || knot_coords.1 < 0.0 || knot_coords.1 > 1.0
        {
            return Err(Error::TmeshOutOfBoundsInsertion);
        }

        // If a point already exists at the desired knot coordinates, return an error. Zero knot intervals can be put
        // on any side of a point and still have the same knot coordinates, but the structure of the mesh will not be
        // different. Thus, zero knot insertion must be done manually.
        if self
            .control_points
            .iter()
            .find(|c| {
                let c_coords = c.read().knot_coordinates();
                let comparison = (c_coords.0 - knot_coords.0, c_coords.1 - knot_coords.1);
                comparison.0.so_small() && comparison.1.so_small()
            })
            .is_some()
        {
            return Err(Error::TmeshExistingControlPoint);
        }

        // The function checks for any T or S edges that intersect the point in paramtric space where the
        // point is to be insertet, then computes the knot ratio needed such that the point is inserted
        // at the correct place and inserts it using add_control_point.

        // Check for any T edges which intersect the parametric location of the new point.
        let mut point_t_coord = 0.0;
        let mut con_knot = 0.0;
        let s_axis_straddle_points = self
            .control_points
            .iter()
            // Filter all points along the S axis of inserton
            .filter(|point| (point.read().knot_coordinates().0 - knot_coords.0).so_small())
            // Filter those points to only include the point that straddles the T axis of insertion
            .filter(|point| {
                if let Some(con) = point.read().get(TmeshDirection::Up) {
                    let temp_t_coord = point.read().knot_coordinates().1;
                    let temp_inter = con.1;

                    // Knot of the new point is located on the connection being investigated?
                    if temp_t_coord < knot_coords.1 && temp_t_coord + temp_inter > knot_coords.1 {
                        point_t_coord = temp_t_coord; // T coordinate of the current point
                        con_knot = temp_inter; // Edge knot interval

                        return true;
                    }
                }
                false
            })
            .map(Arc::clone)
            .collect::<Vec<Arc<RwLock<TmeshControlPoint<P>>>>>();

        // Depending on the number of points whose connections intersect the location of the new point,
        // different errors or actions are taken
        match s_axis_straddle_points.len() {
            // No T-edge instersects the point where the point needs to be inserted,
            // try to find an S edge which intersects the location of the point
            0 => {}
            1 => {
                // A T-edge is found where the point intersects
                return self.try_local_knot_insertion(
                    Arc::clone(&s_axis_straddle_points[0]),
                    TmeshDirection::Up,
                    (knot_coords.1 - point_t_coord) / con_knot,
                );
            }
            _ => {
                // Multiple T-edges are found where the point intersects (Should never happen)
                return Err(Error::TmeshMalformedMesh);
            }
        };

        let mut point_s_coord = 0.0;
        let mut con_knot = 0.0;
        let t_axis_straddle_points = self
            .control_points
            .iter()
            // Filter all points along the T axis of inserton
            .filter(|point| (point.read().knot_coordinates().1 - knot_coords.1).so_small())
            // Filter those points to only include the point that straddles the S axis of insertion
            .filter(|point| {
                if let Some(con) = point.read().get(TmeshDirection::Right) {
                    let temp_s_coord = point.read().knot_coordinates().0;
                    let temp_inter = con.1;

                    // Knot of the new point is located on the connection being investigated?
                    if temp_s_coord < knot_coords.0 && temp_s_coord + temp_inter > knot_coords.0 {
                        point_s_coord = temp_s_coord; // S coordinate of the current point
                        con_knot = temp_inter; // Edge knot interval

                        return true;
                    }
                }
                false
            })
            .map(Arc::clone)
            .collect::<Vec<Arc<RwLock<TmeshControlPoint<P>>>>>();

        // Depending on the number of points whose connections intersect the location of the new point,
        // different errors or actions are taken
        match t_axis_straddle_points.len() {
            0 => {
                // No S-edge instersects the point where the point needs to be inserted, return an error
                Err(Error::TmeshConnectionNotFound)
            }
            1 => {
                // An S-edge is found where the point intersects
                self.try_local_knot_insertion(
                    Arc::clone(&t_axis_straddle_points[0]),
                    TmeshDirection::Right,
                    (knot_coords.0 - point_s_coord) / con_knot,
                )
            }
            _ => {
                // Multiple S-edges are found where the point intersects (Should never happen)
                Err(Error::TmeshMalformedMesh)
            }
        }
    }

    /// Convenience wrapper for local knot insertion that automatically inserts intermediate edges
    /// when `try_absolute_local_knot_insertion` fails due to no straddling edge existing at `(s, t)`.
    ///
    /// The method first attempts direct insertion. If no edge straddles the target coordinates, it
    /// scans the mesh for the nearest horizontal or vertical edge that could be extended through
    /// the target point, inserts intermediate control points along that edge using LKI, and retries.
    ///
    /// This is shape-preserving: the surface is unchanged after refinement.
    ///
    /// # Returns
    /// - `TmeshOutOfBoundsInsertion` if coordinates are outside `[0.0, 1.0]`.
    /// - `TmeshExistingControlPoint` if a point already exists at the target.
    /// - `TmeshConnectionNotFound` if no suitable edges can be found even after intermediate insertions.
    /// - `Ok(Arc<RwLock<TmeshControlPoint<P>>>)` on success.
    ///
    /// # Borrows
    /// See [`Tmesh::try_absolute_local_knot_insertion`].
    pub fn refine_at(&mut self, s: f64, t: f64) -> Result<Arc<RwLock<TmeshControlPoint<P>>>> {
        // Try direct insertion first.
        match self.try_absolute_local_knot_insertion((s, t)) {
            Ok(cp) => return Ok(cp),
            Err(Error::TmeshConnectionNotFound) => {}
            Err(e) => return Err(e),
        }

        // No straddling edge found. Create one by inserting a full column or row of
        // intermediate points using LKI. Inferred connections (Rule 2) require matching
        // points on opposite face edges, so we must insert at ALL t-levels (or s-levels)
        // to build a connected column (or row). Insertions are done bottom-to-top (or
        // left-to-right) so each successive point finds its predecessor via Rule 2.

        // Strategy A: Insert a vertical column at s by finding all horizontal edges
        // that straddle s and inserting LKI points at (s, t_level) for each.
        let mut h_t_levels: Vec<f64> = Vec::new();
        for cp in self.control_points.iter() {
            let r = cp.read();
            if let Some(con) = r.get(TmeshDirection::Right) {
                let cp_s = r.knot_coordinates().0;
                let cp_t = r.knot_coordinates().1;
                let ki = con.1;
                if cp_s < s && cp_s + ki > s {
                    h_t_levels.push(cp_t);
                }
            }
        }
        // Sort bottom-to-top so inferred connections chain upwards.
        h_t_levels.sort_by(|a, b| a.partial_cmp(b).unwrap());
        h_t_levels.dedup_by(|a, b| (*a - *b).so_small());

        if h_t_levels.len() >= 2 {
            for &t_level in &h_t_levels {
                self.try_absolute_local_knot_insertion((s, t_level))?;
            }
            // Retry — a vertical column now exists at s with edges straddling t.
            return self.try_absolute_local_knot_insertion((s, t));
        }

        // Strategy B: Insert a horizontal row at t.
        let mut v_s_levels: Vec<f64> = Vec::new();
        for cp in self.control_points.iter() {
            let r = cp.read();
            if let Some(con) = r.get(TmeshDirection::Up) {
                let cp_s = r.knot_coordinates().0;
                let cp_t = r.knot_coordinates().1;
                let ki = con.1;
                if cp_t < t && cp_t + ki > t {
                    v_s_levels.push(cp_s);
                }
            }
        }
        v_s_levels.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v_s_levels.dedup_by(|a, b| (*a - *b).so_small());

        if v_s_levels.len() >= 2 {
            for &s_level in &v_s_levels {
                self.try_absolute_local_knot_insertion((s_level, t))?;
            }
            return self.try_absolute_local_knot_insertion((s, t));
        }

        Err(Error::TmeshConnectionNotFound)
    }

    /// Returns the cartesian point corresponding to the parametric coordinates for `self`. Usually the
    /// parametric coordinates are constrained from 0 to 1 for both `s` and `t` as this is the domain of
    /// the T-mesh in parametric space. However, parameters are not checked or forcefully constrained,
    /// as there is a domain of continuity outside the usual parameter range. This domain, however, is not
    /// guaranteed, and should be accessed at your own risk.
    ///
    /// # Returns
    /// - `TmeshConnectionNotFound` if `self` contains a non-rectangular grid, in which case generating knot vectors will fail.
    ///
    /// - `TmeshControlPointNotFound` if `self` contains an edge condition inside of its mesh.
    ///
    /// - `Ok(P)` if the calculation succeeded. A `P` will be returned which is the T-mesh transformation
    ///   of `(s, t)` into cartesian space.
    ///
    /// # Borrows
    /// Immutably borrows every control point in `self`.
    pub fn subs(&self, s: f64, t: f64) -> Result<P> {
        // Generate knot vectors if stale.
        if self.knot_vectors.read().is_none() {
            self.generate_knot_vectors()?;
        }

        let borrow = self.knot_vectors.read();
        let all_kvs = borrow
            .as_ref()
            .expect("Knot vectors should have successfully generated or an error returned");

        let num = self.control_points.len();
        let basis_evaluations: Vec<f64> = all_kvs
            .iter()
            .take(num)
            .map(|kvs| basis_function(s, kvs.0.as_slice()) * basis_function(t, kvs.1.as_slice()))
            .collect();

        let numerator = basis_evaluations
            .iter()
            .zip(self.control_points().iter().map(|c| *c.read().point()))
            .fold(P::origin(), |sum, (b, p)| sum + p.to_vec() * *b);

        let denominator: f64 = basis_evaluations.iter().sum();
        Ok(numerator / denominator)
    }
}

impl<P> fmt::Display for Tmesh<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // If only Hash Maps could use f64....
        #[allow(clippy::type_complexity)]
        let mut s_levels: Vec<(f64, Vec<Arc<RwLock<TmeshControlPoint<P>>>>)> = Vec::new();
        #[allow(clippy::type_complexity)]
        let mut t_levels: Vec<(f64, Vec<Arc<RwLock<TmeshControlPoint<P>>>>)> = Vec::new();

        let sort_f64 = |a: &f64, b: &f64| -> std::cmp::Ordering {
            if (a - b).so_small() {
                return std::cmp::Ordering::Equal;
            } else if a > b {
                return std::cmp::Ordering::Greater;
            }
            std::cmp::Ordering::Less
        };

        for point in self.control_points.iter() {
            let coords = point.read().knot_coordinates();

            if let Some(s_level) = s_levels
                .iter_mut()
                .find(|c| sort_f64(&c.0, &coords.0) == std::cmp::Ordering::Equal)
            {
                let point_vec: &mut Vec<Arc<RwLock<TmeshControlPoint<P>>>> = s_level.1.as_mut();
                point_vec.push(Arc::clone(point));
            } else {
                s_levels.push((coords.0, Vec::new()));
                s_levels
                    .last_mut()
                    .expect("Pushed element on previous line.")
                    .1
                    .push(Arc::clone(point));
            }

            if let Some(t_level) = t_levels
                .iter_mut()
                .find(|c| sort_f64(&c.0, &coords.1) == std::cmp::Ordering::Equal)
            {
                let point_vec: &mut Vec<Arc<RwLock<TmeshControlPoint<P>>>> = t_level.1.as_mut();
                point_vec.push(Arc::clone(point));
            } else {
                t_levels.push((coords.1, Vec::new()));
                t_levels
                    .last_mut()
                    .expect("Pushed element on previous line.")
                    .1
                    .push(Arc::clone(point));
            }
        }

        s_levels.sort_unstable_by(|a, b| sort_f64(&a.0, &b.0));
        t_levels.sort_unstable_by(|a, b| sort_f64(&a.0, &b.0));

        t_levels = t_levels.into_iter().rev().collect();

        let mut vertical_cons: Vec<bool> = vec![false; s_levels.len()];
        for (i, (s_level, _)) in s_levels.iter().enumerate() {
            if let Some(point) = t_levels[0]
                .1
                .iter()
                .find(|p| p.read().knot_coordinates().0 == *s_level)
            {
                vertical_cons[i] =
                    point.read().con_type(TmeshDirection::Up) != TmeshConnectionType::Tjunction;
            }
        }
        write!(f, "       ")?;
        let mut line = String::new();
        for con in vertical_cons.iter() {
            if *con {
                line.push_str("|   ");
            } else {
                line.push_str("    ");
            }
        }
        writeln!(f, "{}", line)?;

        // let line_len = 2 * s_levels.len();
        for t_level in t_levels {
            let mut line = String::new();
            let mut has_left_edge = false;
            let mut has_right_edge = false;

            for (i, (s_level, _)) in s_levels.iter().enumerate() {
                if let Some(point) = t_level
                    .1
                    .iter()
                    .find(|p| p.read().knot_coordinates().0 == *s_level)
                {
                    if point.read().con_type(TmeshDirection::Left) == TmeshConnectionType::Edge {
                        line.push_str("--");
                        has_left_edge = true;
                    }

                    line.push('+');
                    vertical_cons[i] = point.read().con_type(TmeshDirection::Down)
                        != TmeshConnectionType::Tjunction;
                    line.push_str(match point.read().con_type(TmeshDirection::Right) {
                        TmeshConnectionType::Edge => "--",
                        TmeshConnectionType::Point => {
                            has_right_edge = true;
                            "---"
                        }
                        TmeshConnectionType::Tjunction => {
                            has_right_edge = false;
                            "   "
                        }
                    });
                } else if vertical_cons[i] {
                    line.push_str("|   ");
                } else if has_right_edge {
                    line.push_str("----");
                } else {
                    line.push_str("    ");
                }
            }

            write!(f, "{:.2} ", t_level.0)?;
            if !has_left_edge {
                write!(f, "  ")?;
            }
            writeln!(f, "{}", line)?;

            write!(f, "       ")?;
            let mut line = String::new();
            for con in vertical_cons.iter() {
                if *con {
                    line.push_str("|   ");
                } else {
                    line.push_str("    ");
                }
            }
            writeln!(f, "{}", line)?;
        }

        let mut s_demarcations = (
            format!("{:.2}", s_levels[0].0),
            format!("{:.2}", s_levels[1].0),
        );
        for (i, s_level) in s_levels[2..].iter().enumerate() {
            if i % 2 == 0 {
                s_demarcations
                    .0
                    .push(if vertical_cons[i + 1] { '|' } else { ' ' });
                s_demarcations
                    .0
                    .push_str(format!("   {:.2}", s_level.0).as_str());
            } else {
                s_demarcations
                    .1
                    .push_str(format!("    {:.2}", s_level.0).as_str());
            }
        }

        if *vertical_cons
            .last()
            .expect("All T-meshes have at least 2 S-levels")
            && s_levels.len().is_multiple_of(2)
        {
            s_demarcations.0.push('|');
        }

        write!(f, "       ")?;
        writeln!(f, "{}", s_demarcations.0)?;
        write!(f, "           ")?;
        writeln!(f, "{}", s_demarcations.1)?;
        Ok(())
    }
}

impl<P> Tmesh<P>
where P: Clone
{
    /// Subdivides a mesh by inserting a new control point parametrically halfway between every pair of connected control points
    /// already present in the mesh. This includes any implicit edges created during the subdivision of the mesh. Thus, a 2x2
    /// mesh created with the `new` function will become a 3x3 mesh with a point in the center of the mesh. The cartesian coordinates
    /// of the new control points is determined with a caller-specified closure, `f`, which will be given the two control points
    /// which will be on either side of the new control point. The first point parameter passed to `f` will always be either the
    /// left or bottom control point in a pair, depending on the edge being subdivided.
    ///
    /// # Returns
    /// - `TmeshConnectionInvalidKnotInterval` if a connection is found which has mismatched knot intervals
    ///   depending on which point in the connection is referenced.
    ///
    /// - `Ok()` if the mesh was successfully subdivided.
    ///
    /// # Borrows
    /// Mutably borrows every control point in `self.control_points`.
    pub fn subdivide<F>(&mut self, f: F) -> Result<()>
    where F: Fn(P, P) -> P {
        // Get all (pairs of) control points with horizontal point to point connections
        let righties: Vec<_> = self
            .control_points()
            .iter()
            .filter(|p| p.read().con_type(TmeshDirection::Right) == TmeshConnectionType::Point)
            .map(Arc::clone)
            .collect();

        // Split all the connections in two
        for cont_p in righties {
            // Get the new control point using the caller supplied closure
            let p = f(
                cont_p.read().point().clone(),
                cont_p
                    .read()
                    .connected_point(TmeshDirection::Right)
                    .read()
                    .point()
                    .clone(),
            );

            self.add_control_point(p, Arc::clone(&cont_p), TmeshDirection::Right, 0.5)?;
        }

        // The above for loop will create new connections in the DOWN direction through implicit connections.
        // Thus, the filtering of the downies must happen after addiing the righties.
        let uppies: Vec<_> = self
            .control_points()
            .iter()
            .filter(|p| p.read().con_type(TmeshDirection::Up) == TmeshConnectionType::Point)
            .map(Arc::clone)
            .collect();

        for cont_p in uppies {
            let p = f(
                cont_p.read().point().clone(),
                cont_p
                    .read()
                    .connected_point(TmeshDirection::Up)
                    .read()
                    .point()
                    .clone(),
            );

            self.add_control_point(p, Arc::clone(&cont_p), TmeshDirection::Up, 0.5)?;
        }

        Ok(())
    }
}

impl<P> Clone for Tmesh<P>
where P: Clone
{
    fn clone(&self) -> Tmesh<P> {
        // Vector containing new point objects which have the same positions as the points in the original mesh
        let mut points_copy = Vec::new();
        // Vector containing the connections for each point with the corresponding index in points_copy.
        // Each sub-vector will be 4 elements long, and each element of the sub-vector will be None if the
        // connection is a T-junction, Some((None, f64)) for an Edge condition, and Some((Some(index), f64))
        // for a Point connection, where index is the index of the connected point in self.control_points,
        // and thus points_copy by extension.
        #[allow(clippy::type_complexity)]
        let mut point_connections: Vec<Vec<Option<(Option<usize>, f64)>>> = Vec::new();

        // Copy all the points into points_copy and all connections into point_connections
        for point in self.control_points.iter() {
            // Clone the cartesian point
            let cart_point = {
                let borrow = point.read();
                borrow.point().clone()
            };
            // Push a new control point corresponding to the control point in self.control_points to points_copy
            // The edge interval is 1.0, however, this can be any value, since establishing connections will
            // overwrite this with the correct value.
            points_copy.push(Arc::new(RwLock::new(TmeshControlPoint::new(
                cart_point, 1.0,
            ))));

            // Push a new set of connections
            point_connections.push(Vec::new());
            // Retrieve the previously pushed set of connections for ease of use.
            let last = point_connections
                .last_mut()
                .expect("Previously pushed item");

            // TmeshDirection::iter() produces the same order of directions every time, so all connection
            // sub-vectors in point_connections will be ordered in the same way, and will be read the same
            // way during connection establishment.
            for dir in TmeshDirection::iter() {
                match point.read().con_type(dir) {
                    // Some((None, f64))
                    TmeshConnectionType::Edge => last.push(Some((
                        None,
                        point
                            .read()
                            .connection_knot(dir)
                            .expect("Edge connection types must have a knot interval."),
                    ))),
                    // Some((Some(Index), f64))
                    TmeshConnectionType::Point => {
                        let connected_point = point.read().connected_point(dir);

                        last.push(Some(
                        (Some(
                            self.control_points
                                .iter()
                                .position(|p| std::ptr::eq(p.as_ref(), connected_point.as_ref())).expect("All connected points must be stored in tmesh control_points vector"),
                        ), point.read().connection_knot(dir).expect("Point connection types must have a knot interval.")),
                    ))
                    }
                    // None
                    TmeshConnectionType::Tjunction => {
                        last.push(None);
                    }
                };
            }
        }

        // Establish connections
        // 'points_loop:
        for (point_index, connections) in point_connections.iter().enumerate() {
            // Zip direction with corresponding connections to index the direction for modification
            'connections_loop: for (connection, dir) in
                connections.iter().zip(TmeshDirection::iter())
            {
                if let Some(con) = connection {
                    // Point connection
                    if let Some(con_index) = con.0 {
                        // Connections has already been established. Connect will also add the connection to points_copy[con_index],
                        // so when points_copy[con_index] is reached by 'points_loop, the connection will already exist, so we skip it.
                        if points_copy[point_index].read().con_type(dir)
                            == TmeshConnectionType::Point
                        {
                            continue 'connections_loop;
                        }

                        // Remove existing edge conditions from both points to be connected.
                        {
                            points_copy[point_index]
                                .write()
                                .remove_connection(dir)
                                .expect("Connections are only modified once.");
                            points_copy[con_index]
                                .write()
                                .remove_connection(dir.flip())
                                .expect("Connections are only modified once.");
                        }

                        // Connect points to each other
                        TmeshControlPoint::connect(
                            Arc::clone(&points_copy[point_index]),
                            Arc::clone(&points_copy[con_index]),
                            dir,
                            con.1,
                        )
                        .expect("Control points have no connections between each other.")
                    // Edge condition
                    } else {
                        points_copy[point_index]
                            .write()
                            .set_edge_con_weight(dir, con.1)
                            .expect(
                                "Unmodified control points have edge conditions in all directions.",
                            );
                    }
                // T-junction
                } else {
                    points_copy[point_index]
                        .write()
                        .remove_connection(dir)
                        .expect(
                            "Unmodified control points have edge conditions in all directions.",
                        );
                }
            }
        }

        // Set absolute knot coordinates
        for (i, p) in self.control_points().iter().enumerate() {
            points_copy[i].write().knot_coordinates = p.read().knot_coordinates();
        }

        Tmesh {
            control_points: points_copy,
            knot_vectors: RwLock::new(None),
        }
    }
}

impl<T> Drop for Tmesh<T> {
    fn drop(&mut self) {
        // Destroy all connections in the mesh so that the only remaining reference to all the points is in
        // self.control_points to prevent leaks
        for p in self.control_points.iter() {
            for dir in TmeshDirection::iter() {
                let _ = p.write().remove_connection(dir);
            }
        }
    }
}

impl<T> Tmesh<T>
where T: Debug + Clone
{
    /// Prints the knot vectors for every point in the mesh.
    ///
    /// # Borrows
    /// Immutably borrows every point in `self.control_points`
    pub fn print_knot_vectors(&self) {
        for point in self.control_points() {
            let cart = {
                let borrow = point.read();
                (*borrow.point()).clone()
            };
            let knot_vectors =
                Tmesh::point_knot_vectors(Arc::clone(point)).expect("Mesh should not be malformed");
            println!("{:?}", cart);
            println!("\tS: {:?}", knot_vectors.0);
            println!("\tT: {:?}", knot_vectors.1);
            println!();
        }
    }
}

/// Step size for central finite-difference derivative approximation (fallback for orders > 2).
const DIFF_EPS: f64 = 1.0e-6;

/// Selects the appropriate basis function evaluator for the given derivative order.
fn basis_function_der(u: f64, a: &[f64], der_order: usize) -> f64 {
    match der_order {
        0 => basis_function(u, a),
        1 => basis_function_d1(u, a),
        2 => basis_function_d2(u, a),
        _ => {
            // Fall back to finite differences for orders > 2.
            let h = DIFF_EPS;
            (basis_function_der(u + h, a, der_order - 1)
                - basis_function_der(u - h, a, der_order - 1))
                / (2.0 * h)
        }
    }
}

impl<P> Tmesh<P>
where P: ControlPoint<f64> + Debug + Clone
{
    /// Creates a T-mesh from a quad mesh by converting to a T-NURCC, applying
    /// CC subdivision, and extracting a parametric surface patch.
    ///
    /// # Arguments
    /// * `positions` - Vertex positions of the quad mesh.
    /// * `quad_faces` - Quad face indices (each face is 4 vertex indices, CCW winding).
    /// * `subdivision_levels` - Number of CC subdivision iterations.
    pub fn from_quad_mesh(
        positions: Vec<P>,
        quad_faces: &[[usize; 4]],
        subdivision_levels: usize,
    ) -> Result<Self> {
        let tnurcc = Tnurcc::from_quad_mesh(positions, quad_faces)?;
        tnurcc.to_tmesh(subdivision_levels)
    }

    /// Converts a cubic `BSplineSurface` into a T-mesh with a regular rectangular grid.
    ///
    /// Any cubic B-spline surface is trivially a T-spline with no T-junctions.
    /// This enables converting existing NURBS/B-spline geometry into T-splines
    /// for further refinement or editing.
    ///
    /// # Errors
    /// Returns `TmeshNonCubicDegree` if the surface is not degree 3 in both directions.
    pub fn from_bspline_surface(surface: &BSplineSurface<P>) -> Result<Self> {
        let (udeg, vdeg) = surface.degrees();
        if udeg != 3 || vdeg != 3 {
            return Err(Error::TmeshNonCubicDegree(udeg, vdeg));
        }

        let u_kv = surface.uknot_vec();
        let v_kv = surface.vknot_vec();
        let cps = surface.control_points();
        let nv = cps[0].len();

        // Normalize knot values to [0,1].
        let u_min = u_kv[0];
        let u_range = u_kv[u_kv.len() - 1] - u_min;
        let v_min = v_kv[0];
        let v_range = v_kv[v_kv.len() - 1] - v_min;

        let norm_u = |idx: usize| -> f64 {
            if u_range.so_small() {
                0.5
            } else {
                (u_kv[idx + 2] - u_min) / u_range
            }
        };
        let norm_v = |idx: usize| -> f64 {
            if v_range.so_small() {
                0.5
            } else {
                (v_kv[idx + 2] - v_min) / v_range
            }
        };

        // Create the grid of T-mesh control points.
        let grid: Vec<Vec<Arc<RwLock<TmeshControlPoint<P>>>>> = cps
            .iter()
            .enumerate()
            .map(|(i, row_cps)| {
                row_cps
                    .iter()
                    .enumerate()
                    .map(|(j, cp)| {
                        Arc::new(RwLock::new(TmeshControlPoint {
                            point: *cp,
                            connections: [
                                Some((None, 0.0)),
                                Some((None, 0.0)),
                                Some((None, 0.0)),
                                Some((None, 0.0)),
                            ],
                            knot_coordinates: (norm_u(i), norm_v(j)),
                        }))
                    })
                    .collect()
            })
            .collect();
        let all_points: Vec<Arc<RwLock<TmeshControlPoint<P>>>> = grid
            .iter()
            .flat_map(|row| row.iter().map(Arc::clone))
            .collect();

        // Connect adjacent points horizontally (Right/Left).
        for (i, pair) in grid.windows(2).enumerate() {
            let ki = norm_u(i + 1) - norm_u(i);
            for (left, right) in pair[0].iter().zip(pair[1].iter()) {
                {
                    let mut w = left.write();
                    w.connections[TmeshDirection::Right as usize] =
                        Some((Some(Arc::clone(right)), ki));
                }
                {
                    let mut w = right.write();
                    w.connections[TmeshDirection::Left as usize] =
                        Some((Some(Arc::clone(left)), ki));
                }
            }
        }

        // Connect adjacent points vertically (Up/Down).
        for j in 0..nv - 1 {
            let ki = norm_v(j + 1) - norm_v(j);
            for row in &grid {
                {
                    let mut w = row[j].write();
                    w.connections[TmeshDirection::Up as usize] =
                        Some((Some(Arc::clone(&row[j + 1])), ki));
                }
                {
                    let mut w = row[j + 1].write();
                    w.connections[TmeshDirection::Down as usize] =
                        Some((Some(Arc::clone(&row[j])), ki));
                }
            }
        }

        // Set edge condition weights on boundary points.
        for row in &grid {
            for cell in row {
                let mut w = cell.write();
                for dir in TmeshDirection::iter() {
                    let di = dir as usize;
                    let is_zero_edge = w.connections[di]
                        .as_ref()
                        .is_some_and(|c| c.0.is_none() && c.1 == 0.0);
                    if !is_zero_edge {
                        continue;
                    }
                    // Use the nearest interior connection's knot interval.
                    let weight = [dir.flip(), dir.clockwise(), dir.anti_clockwise()]
                        .iter()
                        .filter_map(|&d| {
                            w.connections[d as usize]
                                .as_ref()
                                .and_then(|c| c.0.is_some().then_some(c.1))
                        })
                        .next()
                        .unwrap_or(0.1);
                    w.connections[di] = Some((None, weight));
                }
            }
        }

        Ok(Tmesh {
            control_points: all_points,
            knot_vectors: RwLock::new(None),
        })
    }
}

impl Tmesh<Point3> {
    /// Evaluates the analytical derivative d^(m+n)S / du^m dv^n at `(u, v)` using the quotient
    /// rule on the rational surface `S = N / W` where `N = sum(B_i * P_i)` and `W = sum(B_i)`.
    ///
    /// Supports analytical derivatives up to 2nd order in each parameter direction.
    /// Falls back to finite differences for higher orders.
    fn analytical_der_mn(&self, m: usize, n: usize, u: f64, v: f64) -> Vector3 {
        // Generate knot vectors if stale.
        if self.knot_vectors.read().is_none() {
            // SAFETY: `analytical_der_mn` is only called from trait impls and internal methods
            // that assume a well-formed mesh. Generation failure is unrecoverable here.
            self.generate_knot_vectors()
                .expect("T-mesh evaluation failed");
        }

        let borrow = self.knot_vectors.read();
        // SAFETY: The `is_none` check above guarantees generation ran; if it succeeded the
        // value is `Some`.
        let all_kvs = borrow.as_ref().expect("Knot vectors should be generated");

        let num_points = self.control_points.len();
        let n_cols = n + 1;

        // Precompute all needed partial basis derivatives: B^(p,q)_i for p in 0..=m, q in 0..=n.
        // Flat layout with strided indexing [p][q][i] populated in [i][p][q] order — the
        // transposed iteration prevents a clean iterator chain, so imperative indexing is used.
        let bd_stride = n_cols * num_points;
        let mut basis_derivs = vec![0.0f64; (m + 1) * bd_stride];
        for (i, (s_kv, t_kv)) in all_kvs.iter().enumerate().take(num_points) {
            let s_slice = s_kv.as_slice();
            let t_slice = t_kv.as_slice();
            for p in 0..=m {
                let s_val = basis_function_der(u, s_slice, p);
                for q in 0..=n {
                    let t_val = basis_function_der(v, t_slice, q);
                    basis_derivs[p * bd_stride + q * num_points + i] = s_val * t_val;
                }
            }
        }

        // Compute partial derivatives of the numerator N and denominator W.
        // Flat layout: index [p * n_cols + q].
        let pq_size = (m + 1) * n_cols;
        let mut n_derivs = vec![Vector3::new(0.0, 0.0, 0.0); pq_size];
        let mut w_derivs = vec![0.0f64; pq_size];

        for p in 0..=m {
            for q in 0..=n {
                let bd_base = p * bd_stride + q * num_points;
                let mut nx = 0.0;
                let mut ny = 0.0;
                let mut nz = 0.0;
                let mut w = 0.0;
                for (i, cp) in self.control_points.iter().enumerate() {
                    let b = basis_derivs[bd_base + i];
                    let pt = *cp.read().point();
                    nx += b * pt.x;
                    ny += b * pt.y;
                    nz += b * pt.z;
                    w += b;
                }
                let idx = p * n_cols + q;
                n_derivs[idx] = Vector3::new(nx, ny, nz);
                w_derivs[idx] = w;
            }
        }

        // Precompute binomial coefficients (orders are at most 2, so max index is 2).
        const BINOM: [[f64; 3]; 3] = [[1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [1.0, 2.0, 1.0]];

        // Apply the general Leibniz rule for the derivative of a quotient S = N/W:
        // S^(m,n) = (1/W) * (N^(m,n) - sum_{(j,k) != (0,0)} C(m,j)*C(n,k) * W^(j,k) * S^(m-j,n-k)).
        let mut s_derivs = vec![Vector3::new(0.0, 0.0, 0.0); pq_size];
        let w0 = w_derivs[0];

        #[allow(clippy::needless_range_loop)]
        for p in 0..=m {
            for q in 0..=n {
                let idx = p * n_cols + q;
                let mut val = n_derivs[idx];
                for j in 0..=p {
                    for k in 0..=q {
                        if j == 0 && k == 0 {
                            continue;
                        }
                        let s_idx = (p - j) * n_cols + (q - k);
                        val -= s_derivs[s_idx]
                            * (BINOM[p][j] * BINOM[q][k] * w_derivs[j * n_cols + k]);
                    }
                }
                s_derivs[idx] = val / w0;
            }
        }

        s_derivs[m * n_cols + n]
    }

    /// Computes the Gaussian curvature K at `(u, v)` from the first and second fundamental forms.
    fn gaussian_curvature(&self, u: f64, v: f64) -> f64 {
        let su = self.analytical_der_mn(1, 0, u, v);
        let sv = self.analytical_der_mn(0, 1, u, v);
        let suu = self.analytical_der_mn(2, 0, u, v);
        let suv = self.analytical_der_mn(1, 1, u, v);
        let svv = self.analytical_der_mn(0, 2, u, v);

        let normal = su.cross(sv);
        let normal_len = normal.magnitude();
        if normal_len.so_small() {
            0.0
        } else {
            let n = normal / normal_len;

            // First fundamental form coefficients.
            let cap_e = su.dot(su);
            let cap_f = su.dot(sv);
            let cap_g = sv.dot(sv);

            // Second fundamental form coefficients.
            let e = suu.dot(n);
            let f = suv.dot(n);
            let g = svv.dot(n);

            let denom = cap_e * cap_g - cap_f * cap_f;
            if denom.abs().so_small() {
                0.0
            } else {
                (e * g - f * f) / denom
            }
        }
    }

    /// Adaptively refines the T-mesh by inserting knots where Gaussian curvature exceeds the threshold.
    ///
    /// Knot insertion uses `try_absolute_local_knot_insertion`, which requires the target
    /// coordinate to lie on an existing edge. For each high-curvature cell, the method
    /// inserts a midpoint on the nearest straddling edge in both the u and v directions.
    ///
    /// # Arguments
    /// * `curvature_threshold` - Minimum absolute Gaussian curvature to trigger refinement.
    /// * `max_iterations` - Maximum number of refinement passes.
    /// * `initial_samples` - Grid density in each direction for the first pass (doubles each iteration).
    ///
    /// # Returns
    /// Total number of control points inserted, or an error if refinement fails.
    pub fn adaptive_refine(
        &mut self,
        curvature_threshold: f64,
        max_iterations: usize,
        initial_samples: usize,
    ) -> Result<usize> {
        let mut total_insertions = 0usize;
        let mut samples = initial_samples;

        for _ in 0..max_iterations {
            // Collect all unique knot lines (s and t) from existing control points.
            let mut s_lines: Vec<f64> = self
                .control_points
                .iter()
                .map(|cp| cp.read().knot_coordinates().0)
                .collect();
            let mut t_lines: Vec<f64> = self
                .control_points
                .iter()
                .map(|cp| cp.read().knot_coordinates().1)
                .collect();
            s_lines.sort_by(f64::total_cmp);
            s_lines.dedup_by(|a, b| (*a - *b).so_small());
            t_lines.sort_by(f64::total_cmp);
            t_lines.dedup_by(|a, b| (*a - *b).so_small());

            // Sample curvature on a grid and collect cells that exceed the threshold.
            let step = 1.0 / samples as f64;
            let high_curvature_cells: Vec<(f64, f64)> = (0..samples)
                .flat_map(|i| {
                    let u = (i as f64 + 0.5) * step;
                    (0..samples).map(move |j| (u, (j as f64 + 0.5) * step))
                })
                .filter(|&(u, v)| self.gaussian_curvature(u, v).abs() > curvature_threshold)
                .collect();

            if high_curvature_cells.is_empty() {
                break;
            }

            // For each high-curvature cell, find the straddling edge intervals and
            // insert midpoints on the existing knot lines.
            let mut targets: Vec<(f64, f64)> = high_curvature_cells
                .iter()
                .flat_map(|&(u, v)| {
                    // Nearest existing t-line → insert at (u, t_val).
                    let on_t = t_lines
                        .iter()
                        .copied()
                        .min_by(|a, b| (a - v).abs().total_cmp(&(b - v).abs()))
                        .map(|t_val| (u, t_val));
                    // Nearest existing s-line → insert at (s_val, v).
                    let on_s = s_lines
                        .iter()
                        .copied()
                        .min_by(|a, b| (a - u).abs().total_cmp(&(b - u).abs()))
                        .map(|s_val| (s_val, v));
                    on_t.into_iter().chain(on_s)
                })
                .collect();
            targets.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
            targets.dedup_by(|a, b| (a.0 - b.0).so_small() && (a.1 - b.1).so_small());

            let mut insertions = 0usize;
            for (u, v) in targets {
                // Clone before attempting insertion so a panic in LKI
                // doesn't corrupt the mesh.
                let backup = self.clone();
                let result = catch_unwind(AssertUnwindSafe(|| {
                    self.try_absolute_local_knot_insertion((u, v))
                }));
                match result {
                    Ok(Ok(_)) => insertions += 1,
                    Ok(Err(Error::TmeshExistingControlPoint))
                    | Ok(Err(Error::TmeshConnectionNotFound))
                    | Ok(Err(Error::TmeshControlPointNotFound))
                    | Ok(Err(Error::TmeshKnotVectorsNotEqual))
                    | Err(_) => {
                        // Restore from backup on any structural error or panic.
                        *self = backup;
                    }
                    Ok(Err(e)) => return Err(e),
                }
            }

            if insertions == 0 {
                break;
            }

            total_insertions += insertions;
            samples *= 2;
        }

        Ok(total_insertions)
    }
}

impl ParametricSurface for Tmesh<Point3> {
    type Point = Point3;
    type Vector = Vector3;

    fn subs(&self, u: f64, v: f64) -> Point3 {
        Tmesh::subs(self, u, v).expect("T-mesh evaluation failed")
    }

    fn uder(&self, u: f64, v: f64) -> Vector3 { self.der_mn(1, 0, u, v) }
    fn vder(&self, u: f64, v: f64) -> Vector3 { self.der_mn(0, 1, u, v) }
    fn uuder(&self, u: f64, v: f64) -> Vector3 { self.der_mn(2, 0, u, v) }
    fn uvder(&self, u: f64, v: f64) -> Vector3 { self.der_mn(1, 1, u, v) }
    fn vvder(&self, u: f64, v: f64) -> Vector3 { self.der_mn(0, 2, u, v) }

    fn der_mn(&self, m: usize, n: usize, u: f64, v: f64) -> Vector3 {
        if m == 0 && n == 0 {
            let p = <Self as ParametricSurface>::subs(self, u, v);
            return Vector3::new(p.x, p.y, p.z);
        }
        // Use analytical derivatives for orders up to 2.
        if m <= 2 && n <= 2 {
            return self.analytical_der_mn(m, n, u, v);
        }
        // Fall back to finite differences for higher orders.
        let h = DIFF_EPS;
        if m > 0 {
            let forward = self.der_mn(m - 1, n, u + h, v);
            let backward = self.der_mn(m - 1, n, u - h, v);
            (forward - backward) / (2.0 * h)
        } else {
            let forward = self.der_mn(m, n - 1, u, v + h);
            let backward = self.der_mn(m, n - 1, u, v - h);
            (forward - backward) / (2.0 * h)
        }
    }

    fn parameter_range(&self) -> (ParameterRange, ParameterRange) {
        use std::ops::Bound::Included;
        (
            (Included(0.0), Included(1.0)),
            (Included(0.0), Included(1.0)),
        )
    }
}

impl ParametricSurface3D for Tmesh<Point3> {}

impl BoundedSurface for Tmesh<Point3> {}

impl ParameterDivision2D for Tmesh<Point3> {
    fn parameter_division(
        &self,
        range: ((f64, f64), (f64, f64)),
        tol: f64,
    ) -> (Vec<f64>, Vec<f64>) {
        algo::surface::parameter_division(self, range, tol)
    }
}

impl Invertible for Tmesh<Point3> {
    fn invert(&mut self) {
        // Swap u and v by swapping Right<->Up and Left<->Down connections for every control point,
        // and swapping the (s, t) knot coordinates.
        for cp in &self.control_points {
            let mut w = cp.write();
            w.connections
                .swap(TmeshDirection::Up as usize, TmeshDirection::Right as usize);
            w.connections
                .swap(TmeshDirection::Down as usize, TmeshDirection::Left as usize);
            w.knot_coordinates = (w.knot_coordinates.1, w.knot_coordinates.0);
        }
        // Invalidate cached knot vectors.
        *self.knot_vectors.write() = None;
    }
}

impl Transformed<Matrix4> for Tmesh<Point3> {
    fn transform_by(&mut self, trans: Matrix4) {
        use truck_base::cgmath64::*;
        for cp in &self.control_points {
            let mut w = cp.write();
            let p = *w.point();
            w.set_point(trans.transform_point(p));
        }
        // Invalidate cached knot vectors.
        *self.knot_vectors.write() = None;
    }
}

impl SearchParameter<D2> for Tmesh<Point3> {
    type Point = Point3;
    fn search_parameter<H: Into<SPHint2D>>(
        &self,
        point: Point3,
        hint: H,
        trials: usize,
    ) -> Option<(f64, f64)> {
        let hint = match hint.into() {
            SPHint2D::Parameter(u, v) => (u, v),
            SPHint2D::Range(x, y) => algo::surface::presearch(self, point, (x, y), 100),
            SPHint2D::None => algo::surface::presearch(self, point, self.range_tuple(), 100),
        };
        algo::surface::search_parameter(self, point, hint, trials)
    }
}

impl SearchNearestParameter<D2> for Tmesh<Point3> {
    type Point = Point3;
    fn search_nearest_parameter<H: Into<SPHint2D>>(
        &self,
        point: Point3,
        hint: H,
        trials: usize,
    ) -> Option<(f64, f64)> {
        let hint = match hint.into() {
            SPHint2D::Parameter(u, v) => (u, v),
            SPHint2D::Range(x, y) => algo::surface::presearch(self, point, (x, y), 100),
            SPHint2D::None => algo::surface::presearch(self, point, self.range_tuple(), 100),
        };
        algo::surface::search_nearest_parameter(self, point, hint, trials)
    }
}

/// Serializable representation of a single T-mesh connection.
type TmeshSerdeConnection = Option<(Option<usize>, f64)>;

/// Flat serialization helper for `Tmesh<P>`.
#[derive(Serialize, Deserialize)]
struct TmeshSerde<P> {
    /// Control point positions and knot coordinates.
    points: Vec<(P, (f64, f64))>,
    /// For each point, 4 connections (Up, Right, Down, Left).
    /// `None` = T-junction, `Some((None, ki))` = edge, `Some((Some(idx), ki))` = point connection.
    connections: Vec<[TmeshSerdeConnection; 4]>,
}

impl Serialize for Tmesh<Point3> {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        let mut points = Vec::with_capacity(self.control_points.len());
        let mut connections = Vec::with_capacity(self.control_points.len());

        for cp in &self.control_points {
            let r = cp.read();
            points.push((*r.point(), r.knot_coordinates()));

            let mut cons = [None; 4];
            for dir in TmeshDirection::iter() {
                cons[dir as usize] = match r.con_type(dir) {
                    TmeshConnectionType::Tjunction => None,
                    TmeshConnectionType::Edge => Some((None, r.connection_knot(dir).unwrap())),
                    TmeshConnectionType::Point => {
                        let connected = r.connected_point(dir);
                        let idx = self
                            .control_points
                            .iter()
                            .position(|p| std::ptr::eq(p.as_ref(), connected.as_ref()))
                            .unwrap();
                        Some((Some(idx), r.connection_knot(dir).unwrap()))
                    }
                };
            }
            connections.push(cons);
        }

        TmeshSerde {
            points,
            connections,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Tmesh<Point3> {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let data = TmeshSerde::<Point3>::deserialize(deserializer)?;

        // Create control points with dummy edge conditions.
        let points: Vec<Arc<RwLock<TmeshControlPoint<Point3>>>> = data
            .points
            .iter()
            .map(|(p, _kc)| Arc::new(RwLock::new(TmeshControlPoint::new(*p, 1.0))))
            .collect();

        // Establish connections (same logic as Clone impl).
        for (point_index, cons) in data.connections.iter().enumerate() {
            for dir in TmeshDirection::iter() {
                let con = &cons[dir as usize];
                if let Some((maybe_idx, ki)) = con {
                    if let Some(con_index) = maybe_idx {
                        // Point connection — skip if already established from the other side.
                        if points[point_index].read().con_type(dir) == TmeshConnectionType::Point {
                            continue;
                        }
                        points[point_index].write().remove_connection(dir).ok();
                        points[*con_index]
                            .write()
                            .remove_connection(dir.flip())
                            .ok();
                        TmeshControlPoint::connect(
                            Arc::clone(&points[point_index]),
                            Arc::clone(&points[*con_index]),
                            dir,
                            *ki,
                        )
                        .map_err(serde::de::Error::custom)?;
                    } else {
                        // Edge condition.
                        points[point_index]
                            .write()
                            .set_edge_con_weight(dir, *ki)
                            .ok();
                    }
                } else {
                    // T-junction.
                    points[point_index].write().remove_connection(dir).ok();
                }
            }
        }

        // Set knot coordinates.
        for (i, (_, kc)) in data.points.iter().enumerate() {
            points[i].write().knot_coordinates = *kc;
        }

        Ok(Tmesh {
            control_points: points,
            knot_vectors: RwLock::new(None),
        })
    }
}

/// Computes the Greville abscissae for a knot vector of given degree.
/// These are the optimal parameter values for B-spline interpolation.
fn greville_abscissae(knots: &KnotVec, degree: usize) -> Vec<f64> {
    let n = knots.len() - degree - 1;
    (0..n)
        .map(|i| (1..=degree).map(|j| knots[i + j]).sum::<f64>() / degree as f64)
        .collect()
}

impl Tmesh<Point3> {
    /// Converts this T-spline surface to an approximate `BSplineSurface`.
    ///
    /// STEP (ISO 10303) has no T-spline entity, so T-spline surfaces must be
    /// decomposed into B-spline patches for export. This method evaluates the
    /// T-spline at Greville abscissae and uses tensor-product interpolation
    /// to find the B-spline control points.
    ///
    /// `division` controls the number of spans in each parametric direction.
    /// Higher values give better approximation at the cost of more control
    /// points: `division + 3` control points per direction.
    pub fn to_bspline_surface(&self, division: usize) -> BSplineSurface<Point3> {
        let u_knots = KnotVec::uniform_knot(3, division);
        let v_knots = KnotVec::uniform_knot(3, division);
        let n = division + 3;

        let u_grev = greville_abscissae(&u_knots, 3);
        let v_grev = greville_abscissae(&v_knots, 3);

        // Evaluate T-spline at the grid of Greville abscissae.
        let surface_points: Vec<Vec<Point3>> = u_grev
            .iter()
            .map(|&u| {
                v_grev
                    .iter()
                    .map(|&v| ParametricSurface::subs(self, u, v))
                    .collect()
            })
            .collect();

        // Tensor-product interpolation: first interpolate each row (v-direction).
        let row_curves: Vec<BSplineCurve<Point3>> = surface_points
            .iter()
            .map(|row| {
                let params: Vec<(f64, Point3)> =
                    v_grev.iter().copied().zip(row.iter().copied()).collect();
                BSplineCurve::try_interpolate(v_knots.clone(), params)
                    .expect("V-direction interpolation failed")
            })
            .collect();

        // Collect intermediate control points (one row per u-sample).
        let intermediate: Vec<Vec<Point3>> = row_curves
            .iter()
            .map(|c| c.control_points().to_vec())
            .collect();

        // Interpolate each column (u-direction) through the intermediate control points.
        // col_cps[j] contains the U-direction control points for V-index j.
        let col_cps: Vec<Vec<Point3>> = (0..n)
            .map(|j| {
                let params: Vec<(f64, Point3)> = u_grev
                    .iter()
                    .copied()
                    .zip(intermediate.iter().map(|row| row[j]))
                    .collect();
                let col_curve = BSplineCurve::try_interpolate(u_knots.clone(), params)
                    .expect("U-direction interpolation failed");
                col_curve.control_points().to_vec()
            })
            .collect();

        // Transpose from [V][U] to [U][V] for BSplineSurface.
        let control_points: Vec<Vec<Point3>> = (0..n)
            .map(|i| (0..n).map(|j| col_cps[j][i]).collect())
            .collect();

        BSplineSurface::new((u_knots, v_knots), control_points)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns a result which provides information regarding the connection of two points on
    /// `point`'s connection in the direction `dir`.
    ///
    /// # Returns.
    /// - `(0, ERROR)` when `point`'s connection is invalid.
    /// - `(1, ERROR)` when `other`'s connection is invalid.
    ///
    /// - `(#, TmeshConnectionNotFound)` when the connection is a T-mesh.
    /// - `(#, TmeshControlPointNotFound)` when the connection is an edge condition.
    /// - `(#, TmeshExistingConnection)` when the connection does not lead to the correct point.
    ///
    /// - `Ok(())` if the connection is correctly configured.
    fn test_points_are_connected<P: PartialEq>(
        point: Arc<RwLock<TmeshControlPoint<P>>>,
        other: Arc<RwLock<TmeshControlPoint<P>>>,
        dir: TmeshDirection,
    ) -> std::result::Result<(), (i32, Error)> {
        // Check that point is connected to other
        let point_borrow = point.read();
        let point_con = &point_borrow.try_connected_point(dir).map_err(|e| (0, e))?;
        let point_equal = Arc::ptr_eq(point_con, &other);
        point_equal
            .then_some(0)
            .ok_or((0, Error::TmeshExistingConnection))?;

        // Check that other is connected to point
        let other_borrow = other.read();
        let other_con = &other_borrow
            .try_connected_point(dir.flip())
            .map_err(|e| (1, e))?;
        let other_equal = Arc::ptr_eq(other_con, &point);
        other_equal
            .then_some(0)
            .ok_or((1, Error::TmeshExistingConnection))?;
        Ok(())
    }

    /// Tests the construction of a new T-mesh, verifying that all the points are correctly connected and exist.
    #[test]
    fn test_t_mesh_new() {
        let points = [
            Point3::from((0.0, 0.0, 0.0)),
            Point3::from((1.0, 0.0, 0.0)),
            Point3::from((1.0, 1.0, 0.0)),
            Point3::from((0.0, 1.0, 0.0)),
        ];

        let mesh = Tmesh::new(points, 1.0);

        // Test that there are four control points in the mesh after creation.
        assert!(
            mesh.control_points().len() == 4,
            "T-mesh retained {} of 4 points during creation",
            mesh.control_points.len(),
        );

        // Test that the origin and the right are correctly connected
        let con_test = test_points_are_connected(
            mesh.find(Point3::from((0.0, 0.0, 0.0))).unwrap(),
            mesh.find(Point3::from((1.0, 0.0, 0.0))).unwrap(),
            TmeshDirection::Right,
        );
        assert!(
            con_test.is_ok(),
            "The origin is not correctly connected to (1, 0, 0)"
        );

        // Test that the origin and the up are correctly connected
        let con_test: std::result::Result<(), (i32, Error)> = test_points_are_connected(
            mesh.find(Point3::from((0.0, 0.0, 0.0))).unwrap(),
            mesh.find(Point3::from((0.0, 1.0, 0.0))).unwrap(),
            TmeshDirection::Up,
        );
        assert!(
            con_test.is_ok(),
            "The origin is not correctly connected to (0, 1, 0)"
        );

        // Test that (1,1,0) and the up are correctly connected
        let con_test: std::result::Result<(), (i32, Error)> = test_points_are_connected(
            mesh.find(Point3::from((1.0, 1.0, 0.0))).unwrap(),
            mesh.find(Point3::from((0.0, 1.0, 0.0))).unwrap(),
            TmeshDirection::Left,
        );
        assert!(
            con_test.is_ok(),
            "(1, 1, 0) is not correctly connected to (0, 1, 0)"
        );

        // Test that (1,1,0) and the right are correctly connected
        let con_test: std::result::Result<(), (i32, Error)> = test_points_are_connected(
            mesh.find(Point3::from((1.0, 1.0, 0.0))).unwrap(),
            mesh.find(Point3::from((1.0, 0.0, 0.0))).unwrap(),
            TmeshDirection::Down,
        );
        assert!(
            con_test.is_ok(),
            "(1, 1, 0) is not correctly connected to (1, 0, 0)"
        );
    }

    /// Constructs a T-mesh, testing that inserting a new control point with no inferred connections
    /// produces the correct result.
    ///
    /// ```
    ///    |  |  |
    ///  --+-[+]-+--
    ///    |     |
    ///  --+-----+--
    ///    |     |
    /// ```
    /// `[+]` is the inserted control point, which is tested. Testing includes verifying conenctions to other points,
    /// making sure the T-junction in the `DOWN` direction is correct, and verifying the edge condition.
    #[test]
    fn test_t_mesh_insert_control_point() {
        let points = [
            Point3::from((0.0, 0.0, 0.0)),
            Point3::from((1.0, 0.0, 0.0)),
            Point3::from((1.0, 1.0, 0.0)),
            Point3::from((0.0, 1.0, 0.0)),
        ];

        let mut mesh = Tmesh::new(points, 1.0);

        mesh.add_control_point(
            Point3::from((0.5, 1.0, 0.0)),
            mesh.find(Point3::from((0.0, 1.0, 0.0)))
                .expect("Point (0, 1, 0) is a valid point in the T-mesh"),
            TmeshDirection::Right,
            0.5,
        )
        .expect("Arguments provided to add_control_point are valid and insertion is allowed");

        let top_left = mesh.find(Point3::from((0.0, 1.0, 0.0))).unwrap();
        let top_mid = mesh.find(Point3::from((0.5, 1.0, 0.0))).unwrap();
        let top_right = mesh.find(Point3::from((1.0, 1.0, 0.0))).unwrap();

        // Test that there are five control points in the mesh after insertion.
        assert!(
            mesh.control_points().len() == 5,
            "Inserted control point was not added to control_points vector"
        );

        // Test that the left and the middle are correctly connected
        let left_mid_con = test_points_are_connected(
            Arc::clone(&top_left),
            Arc::clone(&top_mid),
            TmeshDirection::Right,
        );
        assert!(
            left_mid_con.is_ok(),
            "Top left and top middle points are not correctly connected"
        );

        // Test that the right and the middle are correctly connected
        let right_mid_con = test_points_are_connected(
            Arc::clone(&top_right),
            Arc::clone(&top_mid),
            TmeshDirection::Left,
        );
        assert!(
            right_mid_con.is_ok(),
            "Top left and top middle points are not correctly connected"
        );

        // Check edge condition for the middle
        assert!(
            top_mid
                .read()
                .get(TmeshDirection::Up)
                .as_ref()
                .is_some_and(|c| c.0.is_none() && (c.1 - 1.0).so_small()),
            "Top middle edge condition (direction UP) is incorrectly configured"
        );

        // Check T-junction for the middle
        assert!(
            top_mid.read().get(TmeshDirection::Down).is_none(),
            "Top middle T-junction (direction DOWN) is incorrectly configured"
        );
    }

    /// Constructs a T-mesh, testing that inserting a new control point with one inferred connection
    /// produces the correct result.
    ///
    /// ```
    ///    |  |  |
    ///  --+--+--+--
    ///    |  :  |
    ///  --+-[+]-+--
    ///    |  |  |
    /// ```
    /// `[+]` is the inserted control point, which is tested. The control point is inserted on the `RIGHT`
    /// connection of the bottom left point, and the connection marked `:` is the inferred connection which
    /// should exist after `[+]` is inserted.
    #[test]
    fn test_t_mesh_insert_control_point_one_inferred_connection() {
        let points = [
            Point3::from((0.0, 0.0, 0.0)),
            Point3::from((1.0, 0.0, 0.0)),
            Point3::from((1.0, 1.0, 0.0)),
            Point3::from((0.0, 1.0, 0.0)),
        ];

        let mut mesh = Tmesh::new(points, 1.0);

        // Add the first control points
        mesh.add_control_point(
            Point3::from((0.5, 1.0, 0.0)),
            mesh.find(Point3::from((0.0, 1.0, 0.0)))
                .expect("Point (0, 1, 0) is a valid point in the T-mesh"),
            TmeshDirection::Right,
            0.5,
        )
        .expect("Arguments provided to add_control_point are valid and insertion is allowed");

        // Add second control point with inferred connection
        mesh.add_control_point(
            Point3::from((0.5, 0.0, 0.0)),
            mesh.find(Point3::from((0.0, 0.0, 0.0)))
                .expect("Point (0, 0, 0) is a valid point in the T-mesh"),
            TmeshDirection::Right,
            0.5,
        )
        .expect("Arguments provided to add_control_point are valid and insertion is allowed");

        let top_mid = mesh.find(Point3::from((0.5, 1.0, 0.0))).unwrap();
        let bottom_mid = mesh.find(Point3::from((0.5, 0.0, 0.0))).unwrap();

        // Test that the inferrect connection exists
        let inferred_con_exist = test_points_are_connected(
            Arc::clone(&bottom_mid),
            Arc::clone(&top_mid),
            TmeshDirection::Up,
        );
        assert!(
            inferred_con_exist.is_ok(),
            "Inferred connection is not correctly configured"
        );

        // Test that inferred connection knot interval is correctly configured
        let inferred_con_interval = {
            let borrow = top_mid.read();

            (borrow
                .connection_knot(TmeshDirection::Down)
                .expect("Connection should exist")
                - 1.0)
                .so_small()
        };
        assert!(
            inferred_con_interval,
            "Inferred connection knot interval is incorrect"
        );
    }

    /// Tests to make sure that a mesh with the following shape is correctly formed and connected. Knot intervals may be arbitrary,
    /// however, cartesian points must be located on a 0.5 spaced grid with a 0 z-coordinate. Thus, the center point is
    /// located at `(0.5, 0.5, 0)` and so on.
    /// ```
    ///    |  |  |
    ///  --+--+--+--
    ///    |  |  |
    ///  --+--+--+--
    ///    |  |  |
    ///  --+--+--+--
    ///    |  |  |
    /// ```
    fn test_t_mesh_plus_mesh(mesh: &Tmesh<Point3>) {
        let middle = mesh.find(Point3::from((0.5, 0.5, 0.0))).unwrap();

        // Test connections in the four directions
        let up_con = test_points_are_connected(
            Arc::clone(&middle),
            Arc::clone(&mesh.find(Point3::from((0.5, 1.0, 0.0))).unwrap()),
            TmeshDirection::Up,
        );
        assert!(up_con.is_ok(), "Middle is not correctly connected to UP");

        let right_con = test_points_are_connected(
            Arc::clone(&middle),
            Arc::clone(&mesh.find(Point3::from((1.0, 0.5, 0.0))).unwrap()),
            TmeshDirection::Right,
        );
        assert!(
            right_con.is_ok(),
            "Middle is not correctly connected to RIGHT"
        );

        let down_con = test_points_are_connected(
            Arc::clone(&middle),
            Arc::clone(&mesh.find(Point3::from((0.5, 0.0, 0.0))).unwrap()),
            TmeshDirection::Down,
        );
        assert!(
            down_con.is_ok(),
            "Middle is not correctly connected to DOWN"
        );

        let left_con = test_points_are_connected(
            Arc::clone(&middle),
            Arc::clone(&mesh.find(Point3::from((0.0, 0.5, 0.0))).unwrap()),
            TmeshDirection::Left,
        );
        assert!(
            left_con.is_ok(),
            "Middle is not correctly connected to LEFT"
        );
    }

    /// Constructs a T-mesh, testing that inserting a new control point with two inferred connections
    /// produces the correct result. Utilizes the `add_control_point` function for point insertion.
    ///
    /// ```
    ///    |  |  |
    ///  --+-<+>-+--
    ///    |  |  |
    ///  --+~[+]~+--
    ///    |  |  |
    ///  --+--+--+--
    ///    |  |  |
    /// ```
    /// `[+]` is the inserted control point, which is tested. The control point is inserted on the `DOWN` connection of
    /// `<+>`, and the connections marked `~` are inferred connections which should exist after `[+]` is inserted.
    #[test]
    fn test_t_mesh_insert_control_point_two_inferred_connections() {
        let points = [
            Point3::from((0.0, 0.0, 0.0)),
            Point3::from((1.0, 0.0, 0.0)),
            Point3::from((1.0, 1.0, 0.0)),
            Point3::from((0.0, 1.0, 0.0)),
        ];

        let mut mesh = Tmesh::new(points, 1.0);

        // Add the four control points
        let points = [
            ((0.5, 0.0, 0.0), (0.0, 0.0, 0.0)), // bottom mid, connects to (0, 0, 0) from its right
            ((1.0, 0.5, 0.0), (1.0, 0.0, 0.0)), // right mid,  connects to (1, 0, 0) from its up
            ((0.5, 1.0, 0.0), (1.0, 1.0, 0.0)), // top mid,    connects to (1, 1, 0) from its left
            ((0.0, 0.5, 0.0), (0.0, 1.0, 0.0)), // right mid,  connects to (0, 1, 0) from its down
        ];
        let mut dir = TmeshDirection::Right;

        for point_pair in points {
            mesh.add_control_point(
                Point3::from(point_pair.0),
                mesh.find(Point3::from(point_pair.1)).unwrap_or_else(|_| {
                    panic!(
                        "Point {:?} is a valid point in the T-mesh",
                        point_pair.1
                    )
                }),
                dir,
                0.5,
            )
            .expect("Arguments provided to add_control_point are valid and insertion is allowed");
            dir = dir.anti_clockwise();
        }

        // Add center control point with inferred connections
        mesh.add_control_point(
            Point3::from((0.5, 0.5, 0.0)),
            mesh.find(Point3::from((0.5, 0.0, 0.0)))
                .expect("Point (0.5, 0, 0) is a valid point in the T-mesh"),
            TmeshDirection::Up,
            0.5,
        )
        .expect("Arguments provided to add_control_point are valid and insertion is allowed");

        test_t_mesh_plus_mesh(&mesh);
    }

    /// Constructs a T-mesh, testing that inserting a new control point with two inferred connections
    /// produces the correct result. Utilizes the `try_add_absolute_point` function for point insertion.
    ///
    /// ```
    ///    |  |  |
    ///  --+-<+>-+--
    ///    |  |  |
    ///  --+~[+]~+--
    ///    |  |  |
    ///  --+--+--+--
    ///    |  |  |
    /// ```
    /// `[+]` is the inserted control point, which is tested. The control point is inserted on the `DOWN` connection of
    /// `<+>`, and the connections marked `~` are inferred connections which should exist after `[+]` is inserted.
    #[test]
    fn test_t_mesh_try_add_absolute_point_mesh_construction() {
        let points = [
            Point3::from((0.0, 0.0, 0.0)),
            Point3::from((1.0, 0.0, 0.0)),
            Point3::from((1.0, 1.0, 0.0)),
            Point3::from((0.0, 1.0, 0.0)),
        ];

        let mut mesh = Tmesh::new(points, 1.0);

        // Insert vertical aspect of the plus
        mesh.try_add_absolute_point(Point3::from((0.0, 0.5, 0.0)), (0.0, 0.5))
            .expect("Legal point insertion");
        mesh.try_add_absolute_point(Point3::from((1.0, 0.5, 0.0)), (1.0, 0.5))
            .expect("Legal point insertion");

        // Insert horizontal aspect of the plus
        mesh.try_add_absolute_point(Point3::from((0.5, 0.0, 0.0)), (0.5, 0.0))
            .expect("Legal point insertion");
        mesh.try_add_absolute_point(Point3::from((0.5, 1.0, 0.0)), (0.5, 1.0))
            .expect("Legal point insertion");

        // Insert center point of the plus
        mesh.try_add_absolute_point(Point3::from((0.5, 0.5, 0.0)), (0.5, 0.5))
            .expect("Legal point insertion");

        test_t_mesh_plus_mesh(&mesh);
    }

    /// Constructs the following T-mesh, testing that inserting a new control point using
    /// `try_add_absolute_point` function produces a point with the correct knot intervals.
    ///
    /// ```
    ///    |       |
    ///  --+-------+--
    ///    |       |
    ///  --+-+-----+--
    ///    | |     |
    /// ```
    #[test]
    fn test_t_mesh_try_add_absolute_point_knot_intervals() {
        let points = [
            Point3::from((0.0, 0.0, 0.0)),
            Point3::from((1.0, 0.0, 0.0)),
            Point3::from((1.0, 1.0, 0.0)),
            Point3::from((0.0, 1.0, 0.0)),
        ];

        let mut mesh = Tmesh::new(points, 1.0);
        mesh.try_add_absolute_point(Point3::from((0.2, 0.0, 0.0)), (0.2, 0.0))
            .expect("Legal point insertion");

        // Insert a point asymetrically into a mesh to test if knot interval calculations work
        let knot_interval_check = mesh
            .find(Point3::from((0.2, 0.0, 0.0)))
            .expect("Control point previously inserted into mesh");

        // Left connection should be connected to (0, 0, 0), with interval 0.2
        assert_eq!(
            knot_interval_check
                .read()
                .connection_knot(TmeshDirection::Left)
                .expect("Known existing connection"),
            0.2,
            "Knot interval on LEFT does not match expectation"
        );

        // Right connection should be connected to (1, 0, 0), with interval 0.8
        assert_eq!(
            knot_interval_check
                .read()
                .connection_knot(TmeshDirection::Right)
                .expect("Known existing connection"),
            0.8,
            "Knot interval on RIGHT does not match expectation"
        );
    }

    /// Constructs a T-mesh, testing that inserting a new control point using
    /// `try_add_absolute_point` function produces errors when attempting to insert an unconnected point,
    /// an existing point, and an out-of-bound point.
    ///
    /// ```            
    ///             {+}
    ///    |   |     
    ///  --+---+--
    ///    |[+]|
    ///  -<+>--+--
    ///    |   |
    /// ```
    /// <+> is the duplicate point
    /// [+] is the unconnected pont
    /// {+} is the out-of-bounds point
    #[test]
    fn test_t_mesh_try_add_absolute_point_invalid_insertion() {
        let points = [
            Point3::from((0.0, 0.0, 0.0)),
            Point3::from((1.0, 0.0, 0.0)),
            Point3::from((1.0, 1.0, 0.0)),
            Point3::from((0.0, 1.0, 0.0)),
        ];

        let mut mesh = Tmesh::new(points, 1.0);

        // Test errors on inserting a point into the center of a face (unconnected point)
        assert!(mesh
            .try_add_absolute_point(Point3::from((0.5, 0.5, 0.0)), (0.5, 0.5))
            .is_err_and(|e| { e == Error::TmeshConnectionNotFound }),
            "Expected Error TmeshConnectionNotFound when attempting to insert a point in a location with no intersecting mesh edges.");

        // Test errors on zero intervals (duplicate point)
        assert!(mesh
            .try_add_absolute_point(Point3::from((0.0, 0.0, 0.0)), (0.0, 0.0))
            .is_err_and(|e| { e == Error::TmeshExistingControlPoint }),
            "Expected Error TmeshExistingControlPoint when attempting to insert a point in a location where a control point already exists.");

        // Test errrors on out-of-bounds insertions.
        assert!(mesh
            .try_add_absolute_point(Point3::from((2.0, 2.0, 0.0)), (2.0, 2.0))
            .is_err_and(|e| { e == Error::TmeshOutOfBoundsInsertion }),
            "Expected Error TmeshOutOfBoundsInsertion when attempting to insert a point outside the parametric domain of the mesh.");
    }

    /// Constructs the following T-mesh, testing that navigating from the origin to a connection in the
    /// right direction functions as expected.
    ///
    /// ```
    ///    |      |
    ///  --+------+--
    ///    |      |
    ///  --+      |
    ///    |      |
    ///  --+------+--
    ///    |      |
    /// ```
    /// <+> is the duplicate point
    /// [+] is the unconnected pont
    /// {+} is the out-of-bounds point
    #[test]
    fn test_t_mesh_navigate_until_con_existing_con() {
        let points = [
            Point3::from((0.0, 0.0, 0.0)),
            Point3::from((1.0, 0.0, 0.0)),
            Point3::from((1.0, 1.0, 0.0)),
            Point3::from((0.0, 1.0, 0.0)),
        ];

        let mut mesh = Tmesh::new(points, 1.0);
        let origin = mesh
            .find(Point3::from((0.0, 0.0, 0.0)))
            .expect("Point exists in T-mesh");

        // Add control point for navigation
        mesh.add_control_point(
            Point3::from((0.0, 0.5, 0.0)),
            Arc::clone(&origin),
            TmeshDirection::Up,
            0.5,
        )
        .expect("Valid addition of control point.");

        // Navigates to the top left point
        let navigation_result = origin
            .read()
            .navigate_until_con(TmeshDirection::Up, TmeshDirection::Right);

        assert!(
            navigation_result.is_ok(),
            "Error navigating until existing connecton"
        );
        assert_eq!(
            navigation_result.as_ref().unwrap().0.read().point,
            Point3::from((0.0, 1.0, 0.0)),
            "Navigation returned incorrect point"
        );
        assert_eq!(
            navigation_result.as_ref().unwrap().1,
            1.0,
            "Navigation knot interval incorrect"
        );
    }

    /// Constructs the following T-mesh, testing that navigating from the origin to a connection in the
    /// left direction returns an error.
    ///
    /// ```
    ///    |      |
    ///  --+------+--
    ///    |      |
    ///  --+      |
    ///    |      |
    ///  --+------+--
    ///    |      |
    /// ```
    /// <+> is the duplicate point
    /// [+] is the unconnected pont
    /// {+} is the out-of-bounds point
    #[test]
    fn test_t_mesh_navigate_until_con_no_existing_con() {
        let points = [
            Point3::from((0.0, 0.0, 0.0)),
            Point3::from((1.0, 0.0, 0.0)),
            Point3::from((1.0, 1.0, 0.0)),
            Point3::from((0.0, 1.0, 0.0)),
        ];

        let mut mesh = Tmesh::new(points, 1.0);
        let origin = mesh
            .find(Point3::from((0.0, 0.0, 0.0)))
            .expect("Point exists in T-mesh");

        // Add control point for navigation
        mesh.add_control_point(
            Point3::from((0.0, 0.5, 0.0)),
            Arc::clone(&origin),
            TmeshDirection::Up,
            0.5,
        )
        .expect("Valid addition of control point.");

        // Navigate until error
        let navigation_result = origin
            .read()
            .navigate_until_con(TmeshDirection::Up, TmeshDirection::Left);

        assert!(
            navigation_result.is_err(),
            "Navigation to non-existant connection succeeded (Should have failed)"
        );
        assert_eq!(
            navigation_result.as_ref().err(),
            Some(&Error::TmeshControlPointNotFound),
            "Expected TmeshControlPointNotFound, got {:?}",
            navigation_result.as_ref().err()
        );
    }

    /// Constructs the following (unsolvable) T-mesh, with the knot coordinates specified on the left and bottom. All edge condition
    ///  intervals have a knot interval of 2.5.
    ///
    /// ```
    ///  1.0   +-----+-----------------------------------+
    ///        |     |                                   |
    ///  0.9   |     +-------+---+---+-----+---+-----+---+
    ///        |     |       |   |   |     |   |     |   |
    ///  0.8   |     |       |   |   |     +---+     |   |
    ///        |     |       |   |   |     |   |     |   |
    ///  0.7   |     |       +---+---+     |   +     |   |
    ///  0.6   |     |       |       |     |   |     +---+
    ///  0.5   |     |       |       +-----+   |     |   |
    ///  0.4   +     |       +       |     |   |     |   +
    ///  0.3   |     +-------+       |     |   |     |   |
    ///  0.2   |     |       |       +-----+---+-----+---+
    ///        |     |       |       |                   |
    ///  0.0   +-----+-------+-------+-------------------+
    ///       0.0   0.2     0.3 0.4 0.5  0.6  0.7   0.9 1.0
    /// ```
    fn construct_ray_casting_example_mesh() -> Tmesh<Point3> {
        let points = [
            Point3::from((0.0, 0.0, 0.0)),
            Point3::from((1.0, 0.0, 0.0)),
            Point3::from((1.0, 1.0, 0.0)),
            Point3::from((0.0, 1.0, 0.0)),
        ];

        let mut mesh = Tmesh::new(points, 2.5);

        // Absolute knot coordinatess of the points from the mesh above. They are ordered such that the
        // edges in the above image will be constructed without conflict, and so that points are only
        // inserted on existing edges.
        let knot_pairs = Vec::from([
            (0.0, 0.4),
            (0.2, 1.0),
            (1.0, 0.9),
            (1.0, 0.6),
            (1.0, 0.2),
            (0.5, 0.0),
            (0.3, 0.0),
            (0.2, 0.0),
            (0.2, 0.3),
            (0.2, 0.9),
            (0.3, 0.9),
            (0.4, 0.9),
            (0.5, 0.9),
            (0.6, 0.9),
            (0.7, 0.9),
            (0.9, 0.9),
            (0.3, 0.7),
            (0.3, 0.4),
            (0.3, 0.3),
            (0.5, 0.7),
            (0.5, 0.5),
            (0.5, 0.2),
            (0.4, 0.7),
            (0.6, 0.2),
            (0.7, 0.2),
            (0.9, 0.2),
            (0.6, 0.5),
            (0.6, 0.8),
            (0.7, 0.7),
            (0.7, 0.8),
            (0.9, 0.6),
            (1.0, 0.4),
        ]);

        // Construct mesh
        for knot_pair in knot_pairs {
            mesh.try_add_absolute_point(Point3::from((knot_pair.0, knot_pair.1, 0.0)), knot_pair)
                .unwrap_or_else(|_| {
                    panic!(
                        "Valid addition of control point ({}, {}).",
                        knot_pair.0, knot_pair.1
                    )
                });
        }

        mesh
    }

    /// Tests if the face intersection algorithm in cast_ray functions as expected, including testing if the
    /// point-edge detection, but not connection traversal, aspect of the algorithm function as expected. Uses the mesh
    /// constructed by construct_ray_casting_example_mesh to test cast_ray, by casting a ray from the point
    /// located at (0.0, 0.4) in parametric space in the direction RIGHT.
    #[test]
    fn test_t_mesh_ray_casting_face_intersection() {
        // Construct mesh
        let mesh = construct_ray_casting_example_mesh();

        // Select the initial point
        let start = mesh
            .find(Point3::from((0.0, 0.4, 0.0)))
            .expect("Known existing point in mesh");

        // Cast ray
        let intersections = Tmesh::cast_ray(Arc::clone(&start), TmeshDirection::Right, 9);

        assert!(
            intersections.is_ok(),
            "Ray casting produces unexpectd error"
        );
        let intersections = intersections.unwrap();

        // Because 9 intersections are requested in the cast_ray function call, the returned vector must be of length 9
        assert_eq!(
            intersections.len(),
            9,
            "The incorrect number of intervals was returned"
        );

        // Check values in the returned vector.
        assert!(
            intersections
                .iter()
                .zip(Vec::from([0.2, 0.1, 0.2, 0.1, 0.1, 0.2, 0.1, 2.5, 0.0]))
                .all(|p| (p.0 - p.1).so_small()),
            "Recorded knot intervals differ form expectation"
        );
    }

    /// Tests if the face intersection algorithm in cast_ray functions as expected. Does not test if the
    /// edge detection or connection traversal aspects of the algorithm function as expected, however,
    /// it does test if the T-junction traversal algorithm terminates when expected. Uses the mesh
    /// constructed by construct_ray_casting_example_mesh to test cast_ray, by casting a ray from the point
    /// located at (0.0, 0.4) in parametric space in the direction RIGHT.
    #[test]
    fn test_t_mesh_ray_casting_face_intersection_incomplete_cast() {
        // Construct mesh
        let mesh = construct_ray_casting_example_mesh();

        // Select the initial point
        let start = mesh
            .find(Point3::from((0.0, 0.4, 0.0)))
            .expect("Known existing point in mesh");

        // Cast ray
        let intersections = Tmesh::cast_ray(Arc::clone(&start), TmeshDirection::Right, 5);

        assert!(
            intersections.is_ok(),
            "Ray casting produces unexpectd error"
        );
        let intersections = intersections.unwrap();

        // Because 9 intersections are requested in the cast_ray function call, the returned vector must be of length 9
        assert_eq!(
            intersections.len(),
            5,
            "The incorrect number of intervals was returned"
        );

        // Check values in the returned vector.
        assert!(
            intersections
                .iter()
                .zip(Vec::from([0.2, 0.1, 0.2, 0.1, 0.1]))
                .all(|p| (p.0 - p.1).so_small()),
            "Recorded knot intervals differ form expectation"
        );
    }

    /// Tests if the face intersection algorithm in cast_ray functions as expected, including testing if the
    /// T-junction edge detection and connection traversal aspects of the algorithm function as expected. Uses the mesh
    /// constructed by construct_ray_casting_example_mesh to test cast_ray, by casting a ray from the point
    /// located at (0.0, 0.4) in parametric space in the direction RIGHT.
    #[test]
    fn test_t_mesh_ray_casting_non_point_edge_condition() {
        // Construct mesh
        let mesh = construct_ray_casting_example_mesh();

        // Select the initial point
        let start = mesh
            .find(Point3::from((0.3, 0.7, 0.0)))
            .expect("Known existing point in mesh");

        // Cast ray
        let intersections = Tmesh::cast_ray(Arc::clone(&start), TmeshDirection::Right, 8);

        assert!(
            intersections.is_ok(),
            "Ray casting produces unexpectd error"
        );
        let intersections = intersections.unwrap();
        println!("{:?}", intersections);
        // Because 9 intersections are requested in the cast_ray function call, the returned vector must be of length 9
        assert_eq!(
            intersections.len(),
            8,
            "The incorrect number of intervals was returned"
        );

        // Check values in the returned vector.
        assert!(
            intersections
                .iter()
                .zip(Vec::from([0.1, 0.1, 0.1, 0.1, 0.2, 0.1, 2.5, 0.0]))
                .all(|p| (p.0 - p.1).so_small()),
            "Recorded knot intervals differ form expectation"
        );
    }

    /// Clones the mesh produced by `construct_ray_casting_example_mesh` and then compares it to a second,
    /// uncloned mesh from `construct_ray_casting_example_mesh`.
    #[test]
    fn test_t_mesh_clone() {
        let tmesh_test = construct_ray_casting_example_mesh().clone();
        let tmesh_comp = construct_ray_casting_example_mesh();

        // Test number of control points
        assert!(
            tmesh_test.control_points().len() == tmesh_comp.control_points().len(),
            "Number of control points in mesh is not the same as original mesh"
        );

        // Test cartesian points
        assert!(
            tmesh_test
                .control_points()
                .iter()
                .zip(tmesh_comp.control_points().iter())
                .all(|p| { p.0.read().point() == p.1.read().point() }),
            "Control points of cloned mesh are not the same as original mesh"
        );

        // Test parametric points
        assert!(
            tmesh_test
                .control_points()
                .iter()
                .zip(tmesh_comp.control_points().iter())
                .all(|p| { p.0.read().knot_coordinates() == p.1.read().knot_coordinates() }),
            "Parametric coordinates of cloned mesh are not the same as original mesh"
        );

        // Test connections
        assert!(tmesh_test
            .control_points()
            .iter()
            .zip(tmesh_comp.control_points().iter())
            .all(|p| {
                // Test all directions of every point in the meshes
                for dir in TmeshDirection::iter() {
                    // Compare connection types
                    if p.0.read().con_type(dir) != p.1.read().con_type(dir) {
                        return false;
                    }

                    // Based on the conenction type, compare connected objects
                    match p.0.read().con_type(dir) {
                        TmeshConnectionType::Edge => {
                            // Compare knot intervals
                            if p.0.read().connection_knot(dir) != p.1.read().connection_knot(dir) {
                                return false;
                            }
                        }
                        TmeshConnectionType::Point => {
                            // Compare knot intervals
                            if p.0.read().connection_knot(dir) != p.1.read().connection_knot(dir) {
                                return false;
                            }

                            // Get connection object from both meshes
                            let test_borrow = p.0.read();
                            let test_con = test_borrow
                                .get(dir)
                                .as_ref()
                                .expect("Point con type must have a connection");
                            let comp_borrow = p.1.read();
                            let comp_con = comp_borrow
                                .get(dir)
                                .as_ref()
                                .expect("Point con type must have a connection");

                            // Compare connected points
                            if test_con
                                .0
                                .as_ref()
                                .expect("Point con type must have a point connected")
                                .read()
                                .point()
                                != comp_con
                                    .0
                                    .as_ref()
                                    .expect("Point con type must have a point connected")
                                    .read()
                                    .point()
                            {
                                return false;
                            }
                        }
                        TmeshConnectionType::Tjunction => {}
                    }
                }

                true
            }))
    }

    /// Creates a plane of the form `x + y = z` and solves it using `subs`.
    #[test]
    fn test_t_mesh_subs() {
        const C: usize = 100;
        let points = [
            Point3::from((0.0, 0.0, 0.0)),
            Point3::from((1.0, 0.0, 1.0)),
            Point3::from((1.0, 1.0, 2.0)),
            Point3::from((0.0, 1.0, 1.0)),
        ];

        // Tmesh is now the surface x + y = z
        let mesh = Tmesh::new(points, 1.0);

        for s in 0..C {
            let s = s as f64 / C as f64;
            for t in 0..C {
                let t = t as f64 / C as f64;
                let p = mesh
                    .subs(s, t)
                    .expect("Solvable T-mesh with s and t within bounds");

                assert!(
                    ((p.x + p.y) - p.z).so_small(),
                    "Returned subs value does not match expectation."
                );
            }
        }
    }

    /// Returns a point half-way between `a` and `b`.
    fn average_points(a: Point3, b: Point3) -> Point3 { 0.5 * (a + ControlPoint::to_vec(b)) }

    /// Subdivides a T-mesh from a two by two into a three by three, checking that the connections and knot vectors
    /// are correct. Does not check if control points are correctly spaced in cartesian space, since that is calculated
    /// with a caller provided function.
    #[test]
    fn test_t_mesh_subdivide() {
        let points = [
            Point3::from((0.0, 0.0, 0.0)),
            Point3::from((1.0, 0.0, 1.0)),
            Point3::from((1.0, 1.0, 2.0)),
            Point3::from((0.0, 1.0, 1.0)),
        ];

        // Tmesh is now a surface where all point on the surface are of the form (f(x), g(y), f(x) + g(y))
        // approximates x + y = z with medial x and y values
        let mut mesh = Tmesh::new(points, 1.0);

        // Subdivision should be successful
        let sub_res = mesh.subdivide(average_points);
        assert!(
            sub_res.is_ok(),
            "Error while subdividing mesh {}.",
            sub_res.err().unwrap()
        );

        // Mesh becomes a 3x3 grid, 9 control points
        assert_eq!(
            mesh.control_points().len(),
            9,
            "Incorrect number of control points found in the subdivided mesh."
        );

        // Test middle point for inifered connection shenanegins
        let middle_point = mesh
            .find(Point3::from((0.5, 0.5, 1.0)))
            .expect("Control point should be located in subdivided mesh");
        for dir in TmeshDirection::iter() {
            assert_eq!(
                middle_point.read().con_type(dir),
                TmeshConnectionType::Point,
                "Expected a point connection in the direction {}.",
                dir
            );
            assert!(
                (middle_point.read().connection_knot(dir).unwrap() - 0.5).so_small(),
                "Expected knot interval of 0.5."
            );
        }

        // Make sure each point still follows the x + y = z scheme (averaging will have no effect on this)
        for point in mesh.control_points.iter() {
            let p = *point.read().point();
            assert!(
                (p.x + p.y - p.z).so_small(),
                "Point does not follow expected cartesian scheme."
            );
        }
    }

    /// Checks if two `Point3` instances are eqaul using tollerance.
    fn points_eq(a: Point3, b: Point3) -> bool { (a.x + a.y + a.z - (b.x + b.y + b.z)).so_small() }

    /// Test legal local knot insertion by creating two identical surfaces, then performing LKI on one of
    /// them and checking with `subs` if the surfaces differ. In order to maximize any differences between the
    /// surfaces, the control points which are affected by the LKI are moved such that they have no lienear
    /// realtion between them in any axis. The cartesian space of the coordinates is cross-referenced with manually
    /// performed mathematics, which can be seen in desmos links in some of the inline comments on top of the manual
    /// confirmation through the use of `subs`.
    #[test]
    fn test_t_mesh_local_knot_insertion_no_edge_conditions() {
        const N: usize = 25;
        let points = [
            Point3::from((0.0, 0.0, 0.0)),
            Point3::from((1.0, 0.0, 1.0)),
            Point3::from((1.0, 1.0, 0.0)),
            Point3::from((0.0, 1.0, 1.0)),
        ];

        let mut mesh = Tmesh::new(points, 1.0);

        // Make mesh a 5x5
        mesh.subdivide(average_points)
            .expect("Mesh is not malformed.");
        mesh.subdivide(average_points)
            .expect("Mesh is not malformed.");

        // Modify mesh so that the form is highly dependant on all elements of a point. Nescessary because if the control points
        // are on a (flat) plane, then the elements which change due to LKI (x and y) can be almost anything and the limit surface
        // will be the same. If the points are more scattered, then deviation in elements which get canceled out by the "averging"
        // nature of the LKI algorithm will become more evident in the elements which are not "averaged out".
        mesh.map_point(
            Point3::from((0.25, 0.25, 0.375)),
            Point3::from((0.25, 0.10, 0.375)),
        )
        .expect("Control point is in mesh");
        mesh.map_point(
            Point3::from((0.50, 0.25, 0.500)),
            Point3::from((0.50, 0.30, 0.300)),
        )
        .expect("Control point is in mesh");
        mesh.map_point(
            Point3::from((0.75, 0.25, 0.625)),
            Point3::from((0.75, 0.15, 0.625)),
        )
        .expect("Control point is in mesh");
        mesh.map_point(
            Point3::from((1.00, 0.25, 0.750)),
            Point3::from((1.00, 0.25, 0.200)),
        )
        .expect("Control point is in mesh");

        let mut test = mesh.clone();

        let ins_point = test
            .try_local_knot_insertion(
                test.find(Point3::from((0.50, 0.30, 0.300)))
                    .expect("Point is a valid point in mesh"),
                TmeshDirection::Right,
                0.1,
            )
            .expect("Local knot insertion should succeed");

        let p3_prime = *ins_point.read().point();

        let p4_prime = *ins_point
            .read()
            .connected_point(TmeshDirection::Right)
            .read()
            .point();

        let p2_prime = *ins_point
            .read()
            .connected_point(TmeshDirection::Left)
            .read()
            .point();

        // Values verified via https://www.desmos.com/3d/pitkyckhfn
        assert!(
            points_eq(p3_prime, Point3::from((0.5916666, 0.245, 0.41916666))),
            "Inserted point does not match expected cartesian coordinates"
        );
        assert!(
            points_eq(p4_prime, Point3::from((0.75416666, 0.1516666, 0.617916666))),
            "Point right of inserted point does not match expected cartesian coordinates"
        );
        assert!(
            points_eq(p2_prime, Point3::from((0.425, 0.24, 0.3225))),
            "Point left of inserted point does not match expected cartesian coordinates"
        );

        for s in 0..N {
            let s = s as f64 / N as f64;
            for t in 0..N {
                let t = t as f64 / N as f64;
                let mesh_sub = mesh.subs(s, t).expect("Parametric point is within bounds");
                let test_sub = test.subs(s, t).expect("Parametric point is within bounds");
                assert!(
                    (mesh_sub - test_sub).so_small(),
                    "Surfaces do not match at ({}, {}).",
                    s,
                    t
                );
            }
        }
    }

    /// Test illegal local knot insertion by creating two identical surfaces, then performing LKI on one of
    /// them and checking if an error is returned. Initially, this test ould have succeeded (at leats, LKI would have),
    /// however, it was discovered that the surface would change shape if done with one of the control points missing
    /// (substituted with an edge condition). Thus, that change was reverted.
    #[test]
    fn test_t_mesh_local_knot_insertion_edge_conditions() {
        let points = [
            Point3::from((0.0, 0.0, 0.0)),
            Point3::from((1.0, 0.0, 1.0)),
            Point3::from((1.0, 1.0, 0.0)),
            Point3::from((0.0, 1.0, 1.0)),
        ];

        let mut mesh = Tmesh::new(points, 1.0);

        // Make mesh a 5x5
        let _ = mesh.subdivide(average_points);
        let _ = mesh.subdivide(average_points);

        println!("{}", mesh);

        let mut test = mesh.clone();
        let ins_point = test.try_local_knot_insertion(
            test.find(Point3::from((1.0, 1.0, 0.0)))
                .expect("Point is a valid point in mesh"),
            TmeshDirection::Down,
            0.1,
        );

        assert!(
            ins_point.is_err(),
            "Local knot insertion should not have succeedd"
        );
    }

    #[test]
    fn test_t_mesh_absolute_local_knot_insertion_mesh_construction() {
        let points = [
            Point3::from((0.0, 0.0, 0.0)),
            Point3::from((1.0, 0.0, 0.0)),
            Point3::from((1.0, 1.0, 0.0)),
            Point3::from((0.0, 1.0, 0.0)),
        ];

        // 5x5
        let mut mesh = Tmesh::new(points, 1.0);
        mesh.subdivide(average_points)
            .expect("Mesh is not malformed.");
        mesh.subdivide(average_points)
            .expect("Mesh is not malformed.");

        // Insert vertical aspect of the plus
        mesh.try_absolute_local_knot_insertion((0.52, 0.00))
            .expect("Legal point insertion");
        mesh.try_absolute_local_knot_insertion((0.52, 0.25))
            .expect("Legal point insertion");
        mesh.try_absolute_local_knot_insertion((0.52, 0.50))
            .expect("Legal point insertion");
        mesh.try_absolute_local_knot_insertion((0.52, 0.75))
            .expect("Legal point insertion");
        mesh.try_absolute_local_knot_insertion((0.52, 1.00))
            .expect("Legal point insertion");

        // Insert horizontal aspect of the plus
        mesh.try_absolute_local_knot_insertion((0.50, 0.52))
            .expect("Legal point insertion");
        mesh.try_absolute_local_knot_insertion((0.75, 0.52))
            .expect("Legal point insertion");

        // Insert center point of the plus
        let center_point = mesh
            .try_absolute_local_knot_insertion((0.52, 0.52))
            .expect("Legal point insertion");

        // Test absolute knot coordinates of the center point.
        let knot_coords = center_point.read().knot_coordinates();
        assert!(
            (knot_coords.0 + knot_coords.1 - 0.52 - 0.52).so_small(),
            "Knot coordinates for center point do not match expectation."
        );

        // At this point, there is little reason to check if the knot intervals match the expectation, since the
        // center point insertion would have failed, or one of the assertions below would have failed because the
        // LKI is highly sensitive to knot intervals, thus, errors in the algorithm would either lead to a failure
        // in future insertions, a mismatch in absolut knot coordinates, or a missing point connection (or two).
        for dir in TmeshDirection::iter() {
            assert_eq!(
                center_point.read().con_type(dir),
                TmeshConnectionType::Point,
                "Center point is not connected to a point in the direction {}.",
                dir
            );
        }
    }

    /// Constructs the following T-mesh, insreting a point which would require the use of the ray-casting algorithm to
    /// calculate on of the knot intervals used in the calculation of the new cartesian coordinates of the control
    /// points affected by LKI. A comparison much like the other LKI insertion tests is done, where two identical meshes
    ///  are constructed, and then compared using `subs` after one has been modified through the use of LKI. Though a T-junction
    /// technically exists in the LKI, none of the LKI rules are broken, since the four required control points still exist,
    /// and their perpendicular knot vectors are all equal.
    ///
    /// Uses absolute local knot insertion.
    ///
    /// ```
    ///        |   |   |   |   |   |
    /// 1.00 --+---+--[+]--+---+---+--
    ///        |   |   |   |   |   |
    /// 0.75 --+---+--[+]--+---+---+--
    ///        |   |   |   |   |   |
    /// 0.52   |   +--<+>--+   |   |
    ///        |   |   |   |   |   |
    /// 0.50 --+---+--{+}--+---+---+--
    ///        |   |   |   |   |   |
    /// 0.25 --+---+--[+]--+---+---+--
    ///        |   |       |   |   |
    /// 0.00 --+---+-------+---+---+--
    ///        |   |       |   |   |
    ///        0.00|   0.27|   0.75|
    ///            0.25    0.50    1.00
    /// ```
    ///
    /// - `[+]` are control points which are required by LKI
    /// - `{+}` is the control point from which the algorithm will insert the new control point using the `try_local_knot_insertion` function.
    /// - `<+>` is the point that is inserted in one mesh and not the other.
    #[test]
    fn test_t_mesh_local_knot_insertion_force_ray_casting() {
        const N: usize = 25;
        let points = [
            Point3::from((0.0, 0.0, 0.0)),
            Point3::from((1.0, 0.0, 1.0)),
            Point3::from((1.0, 1.0, 0.0)),
            Point3::from((0.0, 1.0, 1.0)),
        ];

        // 5x5
        let mut mesh = Tmesh::new(points, 1.0);
        mesh.subdivide(average_points)
            .expect("Mesh is not malformed.");
        mesh.subdivide(average_points)
            .expect("Mesh is not malformed.");

        // Mangle linearity of control points in cartesian space
        mesh.map_point(
            Point3::from((0.25, 0.25, 0.375)),
            Point3::from((0.10, 0.25, 0.375)),
        )
        .expect("Control point is in mesh");
        mesh.map_point(
            Point3::from((0.25, 0.50, 0.500)),
            Point3::from((0.30, 0.50, 0.300)),
        )
        .expect("Control point is in mesh");
        mesh.map_point(
            Point3::from((0.25, 0.75, 0.625)),
            Point3::from((0.15, 0.75, 0.625)),
        )
        .expect("Control point is in mesh");
        mesh.map_point(
            Point3::from((0.25, 1.00, 0.750)),
            Point3::from((0.25, 1.00, 0.200)),
        )
        .expect("Control point is in mesh");

        // Insert vertical aspect of the plus
        mesh.try_absolute_local_knot_insertion((0.27, 0.25))
            .expect("Legal point insertion");
        mesh.try_absolute_local_knot_insertion((0.27, 0.50))
            .expect("Legal point insertion");
        mesh.try_absolute_local_knot_insertion((0.27, 0.75))
            .expect("Legal point insertion");
        mesh.try_absolute_local_knot_insertion((0.27, 1.00))
            .expect("Legal point insertion");

        // Insert horizontal aspect of the plus
        mesh.try_absolute_local_knot_insertion((0.25, 0.52))
            .expect("Legal point insertion");
        mesh.try_absolute_local_knot_insertion((0.50, 0.52))
            .expect("Legal point insertion");

        let mut test = mesh.clone();
        test.try_absolute_local_knot_insertion((0.27, 0.52))
            .expect("Legal point insertion");

        for s in 0..N {
            let s = s as f64 / N as f64;
            for t in 0..N {
                let t = t as f64 / N as f64;
                let mesh_sub = mesh.subs(s, t).expect("Parametric point is within bounds");
                let test_sub = test.subs(s, t).expect("Parametric point is within bounds");
                assert!(
                    (mesh_sub - test_sub).so_small(),
                    "Surfaces do not match at ({}, {}).",
                    s,
                    t
                );
            }
        }
    }

    /// Tests that `to_bspline_surface` produces a B-spline that closely approximates the original T-mesh.
    #[test]
    fn test_to_bspline_surface_accuracy() {
        use truck_base::cgmath64::*;

        let points = [
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 1.0),
            Point3::new(1.0, 1.0, 2.0),
            Point3::new(0.0, 1.0, 1.0),
        ];
        let mut mesh = Tmesh::new(points, 1.0);
        mesh.subdivide(average_points)
            .expect("Subdivision should succeed");
        mesh.subdivide(average_points)
            .expect("Subdivision should succeed");

        let bsp = mesh.to_bspline_surface(8);

        // Sample both surfaces on an interior grid and check max deviation.
        let n = 20;
        let mut max_err = 0.0f64;
        for i in 1..n {
            let u = i as f64 / n as f64;
            for j in 1..n {
                let v = j as f64 / n as f64;
                let p_tmesh = ParametricSurface::subs(&mesh, u, v);
                let p_bsp = ParametricSurface::subs(&bsp, u, v);
                let err = (p_tmesh - p_bsp).magnitude();
                max_err = max_err.max(err);
            }
        }
        assert!(
            max_err < 1.0e-3,
            "Max deviation between T-mesh and B-spline: {max_err:.2e} (expected < 1e-3)."
        );
    }

    /// Constructs a non-planar T-mesh for derivative testing.
    /// Uses the same pattern as `test_t_mesh_local_knot_insertion_no_edge_conditions`.
    fn make_derivative_test_mesh() -> Tmesh<Point3> {
        let points = [
            Point3::from((0.0, 0.0, 0.0)),
            Point3::from((1.0, 0.0, 1.0)),
            Point3::from((1.0, 1.0, 0.0)),
            Point3::from((0.0, 1.0, 1.0)),
        ];
        let mut mesh = Tmesh::new(points, 1.0);
        mesh.subdivide(average_points)
            .expect("Mesh is not malformed.");
        mesh.subdivide(average_points)
            .expect("Mesh is not malformed.");

        // Mangle linearity so derivatives are nontrivial.
        mesh.map_point(
            Point3::from((0.25, 0.25, 0.375)),
            Point3::from((0.25, 0.10, 0.375)),
        )
        .expect("Control point is in mesh");
        mesh.map_point(
            Point3::from((0.50, 0.25, 0.500)),
            Point3::from((0.50, 0.30, 0.300)),
        )
        .expect("Control point is in mesh");
        mesh.map_point(
            Point3::from((0.75, 0.25, 0.625)),
            Point3::from((0.75, 0.15, 0.625)),
        )
        .expect("Control point is in mesh");
        mesh
    }

    /// Computes a finite-difference derivative for comparison with analytical derivatives.
    fn finite_diff_der(mesh: &Tmesh<Point3>, m: usize, n: usize, u: f64, v: f64) -> Vector3 {
        let h = 1.0e-6;
        if m == 0 && n == 0 {
            let p = mesh.subs(u, v).expect("subs should succeed");
            return Vector3::new(p.x, p.y, p.z);
        }
        if m > 0 {
            let fwd = finite_diff_der(mesh, m - 1, n, u + h, v);
            let bwd = finite_diff_der(mesh, m - 1, n, u - h, v);
            (fwd - bwd) / (2.0 * h)
        } else {
            let fwd = finite_diff_der(mesh, m, n - 1, u, v + h);
            let bwd = finite_diff_der(mesh, m, n - 1, u, v - h);
            (fwd - bwd) / (2.0 * h)
        }
    }

    /// Tests analytical derivatives against finite differences at many sample points.
    #[test]
    fn test_analytical_derivatives_vs_finite_diff() {
        let mesh = make_derivative_test_mesh();
        let n = 10;

        for i in 1..n {
            let u = i as f64 / n as f64;
            for j in 1..n {
                let v = j as f64 / n as f64;
                // Test all 1st and 2nd order derivatives.
                // 2nd-order finite differences are inherently less accurate (O(h^2) error
                // compounds), so use a looser tolerance for them.
                for &(m, ord_n, tol) in &[
                    (1, 0, 1.0e-4),
                    (0, 1, 1.0e-4),
                    (2, 0, 5.0e-4),
                    (0, 2, 5.0e-4),
                    (1, 1, 5.0e-4),
                ] {
                    let analytical = mesh.analytical_der_mn(m, ord_n, u, v);
                    let numerical = finite_diff_der(&mesh, m, ord_n, u, v);
                    let diff = (analytical - numerical).magnitude();
                    assert!(
                        diff < tol,
                        "Derivative d^({},{}) at ({}, {}) differs: analytical={:?}, numerical={:?}, diff={:.2e}",
                        m, ord_n, u, v, analytical, numerical, diff
                    );
                }
            }
        }
    }

    /// Tests derivative continuity at knot boundaries.
    #[test]
    fn test_analytical_derivatives_knot_continuity() {
        let mesh = make_derivative_test_mesh();
        let eps = 1.0e-8;
        let tol = 1.0e-4;

        // Knot boundaries for a 5x5 mesh are at 0.0, 0.25, 0.5, 0.75, 1.0.
        let knots = [0.25, 0.5, 0.75];
        for &k in &knots {
            for &(m, n) in &[(1, 0), (0, 1)] {
                // Test continuity across u-knot boundary.
                let left = mesh.analytical_der_mn(m, n, k - eps, 0.5);
                let right = mesh.analytical_der_mn(m, n, k + eps, 0.5);
                let diff = (left - right).magnitude();
                assert!(
                    diff < tol,
                    "Derivative d^({},{}) discontinuous at u-knot {}: left={:?}, right={:?}, diff={:.2e}",
                    m, n, k, left, right, diff
                );

                // Test continuity across v-knot boundary.
                let below = mesh.analytical_der_mn(m, n, 0.5, k - eps);
                let above = mesh.analytical_der_mn(m, n, 0.5, k + eps);
                let diff = (below - above).magnitude();
                assert!(
                    diff < tol,
                    "Derivative d^({},{}) discontinuous at v-knot {}: below={:?}, above={:?}, diff={:.2e}",
                    m, n, k, below, above, diff
                );
            }
        }
    }

    /// Tests derivatives at parametric boundary values.
    #[test]
    fn test_analytical_derivatives_at_boundaries() {
        let mesh = make_derivative_test_mesh();
        let tol = 1.0e-4;
        let eps = 1.0e-7;

        // Test at near-boundary points (exact boundary 1.0 is excluded from basis support).
        for &(u, v) in &[(eps, 0.5), (0.5, eps), (1.0 - eps, 0.5), (0.5, 1.0 - eps)] {
            for &(m, n) in &[(1, 0), (0, 1)] {
                let analytical = mesh.analytical_der_mn(m, n, u, v);
                let numerical = finite_diff_der(&mesh, m, n, u, v);
                let diff = (analytical - numerical).magnitude();
                assert!(
                    diff < tol,
                    "Boundary derivative d^({},{}) at ({}, {}) differs: diff={:.2e}",
                    m,
                    n,
                    u,
                    v,
                    diff
                );
            }
        }
    }

    /// Tests `refine_at` at a location where an edge already exists — should behave identically
    /// to `try_absolute_local_knot_insertion`.
    #[test]
    fn test_refine_at_existing_edge() {
        let points = [
            Point3::from((0.0, 0.0, 0.0)),
            Point3::from((1.0, 0.0, 1.0)),
            Point3::from((1.0, 1.0, 2.0)),
            Point3::from((0.0, 1.0, 1.0)),
        ];
        let mut mesh = Tmesh::new(points, 1.0);
        mesh.subdivide(average_points)
            .expect("Mesh is not malformed.");
        mesh.subdivide(average_points)
            .expect("Mesh is not malformed.");

        let mut reference = mesh.clone();

        // Direct LKI should succeed at this location (on an existing edge).
        reference
            .try_absolute_local_knot_insertion((0.3, 0.25))
            .expect("Direct LKI should succeed");

        // refine_at should produce the same result.
        mesh.refine_at(0.3, 0.25).expect("refine_at should succeed");

        // Verify surfaces match.
        let n = 20;
        for i in 0..n {
            let u = i as f64 / n as f64;
            for j in 0..n {
                let v = j as f64 / n as f64;
                let a = mesh.subs(u, v).expect("subs should succeed");
                let b = reference.subs(u, v).expect("subs should succeed");
                assert!(
                    (a - b).so_small(),
                    "Surfaces differ at ({}, {}): refine_at={:?}, direct={:?}",
                    u,
                    v,
                    a,
                    b
                );
            }
        }
    }

    /// Tests `refine_at` at a location requiring an intermediate edge insertion.
    #[test]
    fn test_refine_at_with_intermediate_edge() {
        let points = [
            Point3::from((0.0, 0.0, 0.0)),
            Point3::from((1.0, 0.0, 1.0)),
            Point3::from((1.0, 1.0, 2.0)),
            Point3::from((0.0, 1.0, 1.0)),
        ];
        let mut mesh = Tmesh::new(points, 1.0);
        mesh.subdivide(average_points)
            .expect("Mesh is not malformed.");
        mesh.subdivide(average_points)
            .expect("Mesh is not malformed.");

        let original = mesh.clone();

        // (0.3, 0.3) has no straddling edge — it's in the interior of a face.
        // Direct LKI would fail.
        assert!(
            mesh.clone()
                .try_absolute_local_knot_insertion((0.3, 0.3))
                .is_err(),
            "Direct LKI should fail at (0.3, 0.3) — no straddling edge."
        );

        // refine_at should succeed by inserting intermediate edges.
        mesh.refine_at(0.3, 0.3)
            .expect("refine_at should succeed with intermediate edges");

        // Verify surface is unchanged (shape-preserving).
        let n = 20;
        for i in 0..n {
            let u = i as f64 / n as f64;
            for j in 0..n {
                let v = j as f64 / n as f64;
                let a = mesh.subs(u, v).expect("subs should succeed");
                let b = original.subs(u, v).expect("subs should succeed");
                assert!(
                    (a - b).so_small(),
                    "Surface changed after refine_at at ({}, {}): refined={:?}, original={:?}",
                    u,
                    v,
                    a,
                    b
                );
            }
        }
    }

    /// Tests that `refine_at` preserves the surface shape.
    #[test]
    fn test_refine_at_shape_preserving() {
        let points = [
            Point3::from((0.0, 0.0, 0.0)),
            Point3::from((1.0, 0.0, 1.0)),
            Point3::from((1.0, 1.0, 2.0)),
            Point3::from((0.0, 1.0, 1.0)),
        ];
        let mut mesh = Tmesh::new(points, 1.0);
        mesh.subdivide(average_points)
            .expect("Mesh is not malformed.");
        mesh.subdivide(average_points)
            .expect("Mesh is not malformed.");

        let original = mesh.clone();

        // Insert on an existing horizontal edge at t=0.
        mesh.refine_at(0.6, 0.0)
            .expect("refine_at on edge should succeed");

        // Insert requiring intermediate edges.
        mesh.refine_at(0.6, 0.5)
            .expect("refine_at with intermediate should succeed");

        // Verify surface is unchanged.
        let n = 20;
        let mut max_err = 0.0f64;
        for i in 0..n {
            let u = i as f64 / n as f64;
            for j in 0..n {
                let v = j as f64 / n as f64;
                let a = mesh.subs(u, v).expect("subs should succeed");
                let b = original.subs(u, v).expect("subs should succeed");
                let err = (a - b).magnitude();
                max_err = max_err.max(err);
            }
        }
        assert!(
            max_err < 1.0e-10,
            "Surface shape changed after refinement, max error: {max_err:.2e}"
        );
    }

    /// Round-trip test: BSplineSurface → Tmesh → evaluate, compare with original.
    #[test]
    fn test_from_bspline_surface_round_trip() {
        use truck_base::cgmath64::*;

        let u_knots = KnotVec::uniform_knot(3, 3);
        let v_knots = KnotVec::uniform_knot(3, 3);
        // 3 (degree) + 3 (divisions).
        let nu = 6;
        let nv = 6;

        // Build a non-planar cubic B-spline surface.
        let cps: Vec<Vec<Point3>> = (0..nu)
            .map(|i| {
                (0..nv)
                    .map(|j| {
                        let x = i as f64 / (nu - 1) as f64;
                        let y = j as f64 / (nv - 1) as f64;
                        let z = (x * std::f64::consts::PI).sin() * (y * std::f64::consts::PI).sin();
                        Point3::new(x, y, z)
                    })
                    .collect()
            })
            .collect();
        let bsp = BSplineSurface::new((u_knots, v_knots), cps);

        let tmesh = Tmesh::from_bspline_surface(&bsp).expect("Conversion should succeed");

        // Sample both surfaces and compare.
        let n = 15;
        let mut max_err = 0.0f64;
        for i in 1..n {
            let u = i as f64 / n as f64;
            for j in 1..n {
                let v = j as f64 / n as f64;
                let p_bsp = ParametricSurface::subs(&bsp, u, v);
                let p_tmesh = tmesh.subs(u, v).expect("T-mesh eval should succeed");
                let err = (p_bsp - p_tmesh).magnitude();
                max_err = max_err.max(err);
            }
        }
        assert!(
            max_err < 1.0e-6,
            "BSpline → Tmesh round-trip max error: {max_err:.2e} (expected < 1e-6)"
        );
    }

    /// Verifies correct structure: control point count and interior connectivity.
    #[test]
    fn test_from_bspline_surface_structure() {
        let u_knots = KnotVec::uniform_knot(3, 2);
        let v_knots = KnotVec::uniform_knot(3, 3);
        let nu = 5;
        let nv = 6;

        let cps: Vec<Vec<Point3>> = (0..nu)
            .map(|i| {
                (0..nv)
                    .map(|j| Point3::new(i as f64, j as f64, 0.0))
                    .collect()
            })
            .collect();
        let bsp = BSplineSurface::new((u_knots, v_knots), cps);

        let tmesh = Tmesh::from_bspline_surface(&bsp).expect("Conversion should succeed");

        // Correct number of control points.
        assert_eq!(tmesh.control_points().len(), nu * nv);

        // Interior points should have 4 point connections.
        let mut interior_count = 0;
        for cp in tmesh.control_points() {
            let r = cp.read();
            let point_connections: usize = TmeshDirection::iter()
                .filter(|&d| r.con_type(d) == TmeshConnectionType::Point)
                .count();
            if point_connections == 4 {
                interior_count += 1;
            }
        }
        assert_eq!(
            interior_count,
            (nu - 2) * (nv - 2),
            "Expected {} interior points with 4 connections, got {}",
            (nu - 2) * (nv - 2),
            interior_count
        );
    }

    /// Non-cubic B-spline surfaces should be rejected.
    #[test]
    fn test_from_bspline_surface_non_cubic() {
        // Degree 2 in u, degree 2 in v.
        let u_knots = KnotVec::uniform_knot(2, 2);
        let v_knots = KnotVec::uniform_knot(2, 2);
        let nu = 4;
        let nv = 4;

        let cps: Vec<Vec<Point3>> = (0..nu)
            .map(|i| {
                (0..nv)
                    .map(|j| Point3::new(i as f64, j as f64, 0.0))
                    .collect()
            })
            .collect();
        let bsp = BSplineSurface::new((u_knots, v_knots), cps);

        let result = Tmesh::from_bspline_surface(&bsp);
        assert!(result.is_err(), "Non-cubic surface should be rejected");
    }

    /// Flat surface should have zero Gaussian curvature → zero insertions.
    #[test]
    fn test_adaptive_refine_flat_surface() {
        let points = [
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ];
        let mut mesh = Tmesh::new(points, 1.0);
        mesh.subdivide(average_points).expect("Subdivision ok");
        mesh.subdivide(average_points).expect("Subdivision ok");

        let insertions = mesh
            .adaptive_refine(0.01, 3, 5)
            .expect("Adaptive refine should succeed");
        assert_eq!(
            insertions, 0,
            "Flat surface should have 0 insertions, got {insertions}"
        );
    }

    /// Curved surface should get insertions, and the surface shape should be preserved.
    #[test]
    fn test_adaptive_refine_curved_surface() {
        // Use a saddle shape which has intrinsic curvature.
        let points = [
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 1.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 1.0, 1.0),
        ];
        let mut mesh = Tmesh::new(points, 1.0);
        mesh.subdivide(average_points).expect("ok");
        mesh.subdivide(average_points).expect("ok");
        let original = mesh.clone();

        let insertions = mesh
            .adaptive_refine(0.1, 2, 5)
            .expect("Adaptive refine should succeed");
        assert!(
            insertions > 0,
            "Curved surface should have insertions, got 0"
        );

        // Verify shape preservation.
        let n = 15;
        let mut max_err = 0.0f64;
        for i in 0..n {
            let u = i as f64 / n as f64;
            for j in 0..n {
                let v = j as f64 / n as f64;
                let a = mesh.subs(u, v).expect("subs ok");
                let b = original.subs(u, v).expect("subs ok");
                let err = (a - b).magnitude();
                max_err = max_err.max(err);
            }
        }
        assert!(
            max_err < 1.0e-10,
            "Surface changed after adaptive refinement, max error: {max_err:.2e}"
        );
    }

    /// Gaussian curvature should be nonzero at curved regions and ~zero at flat ones.
    #[test]
    fn test_gaussian_curvature_sanity() {
        // Build a flat mesh.
        let flat_points = [
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ];
        let mut flat = Tmesh::new(flat_points, 1.0);
        flat.subdivide(average_points).expect("ok");
        flat.subdivide(average_points).expect("ok");

        let k_flat = flat.gaussian_curvature(0.5, 0.5);
        assert!(
            k_flat.abs() < 1.0e-6,
            "Flat surface curvature should be ~0, got {k_flat:.2e}"
        );

        // Build a curved mesh (saddle shape with mangled points).
        let curved = make_derivative_test_mesh();
        let k_curved = curved.gaussian_curvature(0.3, 0.3);
        assert!(
            k_curved.abs() > 1.0e-3,
            "Curved surface should have significant curvature, got {k_curved:.2e}"
        );
    }
}
