use super::*;
use crate::errors::Error;
use std::fmt;

impl<P> TmeshControlPoint<P> {
    /// Creates a new T-mesh control point located at the real space coordinate `p`, with edge conditions in all directions with knot interval `interval`.
    pub fn new(p: P, interval: f64) -> TmeshControlPoint<P> {
        TmeshControlPoint {
            point: p,
            connections: [
                Some((None, interval)),
                Some((None, interval)),
                Some((None, interval)),
                Some((None, interval)),
            ],
            knot_coordinates: (-1.0, -1.0),
        }
    }

    /// Returns an immutable reference to the location of the control point in real space.
    pub fn point(&self) -> &P { &self.point }

    /// Sets the cartesian point of the control point
    pub fn set_point(&mut self, p: P) { self.point = p; }

    // /// Returns an immutable refence to the connections array.
    // pub fn connections(&self) -> &[Option<TmeshConnection<P>>; 4] {
    //     &self.connections
    // }

    /// Get an immutable reference to the connection on the side `dir`.
    pub fn get(&self, dir: TmeshDirection) -> &Option<TmeshConnection<P>> {
        &self.connections[dir as usize]
    }

    /// Get a mutable reference to the connection on the side `dir`.
    fn connection_mut(&mut self, dir: TmeshDirection) -> &mut Option<TmeshConnection<P>> {
        &mut self.connections[dir as usize]
    }

    /// Returns the knot coordinates for `self`
    pub fn knot_coordinates(&self) -> (f64, f64) { self.knot_coordinates }

    /// Sets the knot coordinates for `self`. Only changes the coordinates if `self`
    /// is not connected to any other points or T-junctions. `t` is the horizontal
    /// knot coordinate, and `s` is the virtical.
    ///
    /// # Returns
    /// - `TmeshExistingConnection` if `self` is connected to anything that is not an edge condition
    ///
    /// - `Ok` if the knot coordinates were changed.
    pub fn set_knot_coordinates(&mut self, t: f64, s: f64) -> Result<()> {
        // Ensure all points are edge conditions
        self.connections
            .iter()
            .all(|e| e.as_ref().is_some_and(|c| c.0.is_none()))
            .then_some(0)
            .ok_or(Error::TmeshExistingConnection)?;

        self.knot_coordinates = (t, s);

        Ok(())
    }

    /// Sets the weight of the edge condition in the direction `dir` to weight `weight`.
    ///
    /// # Returns
    /// - `TmeshConnectionNotFound` if the connection is T-junction.
    ///
    /// - `TmeshExistingConnection` if the connection is connected to another point.
    ///
    /// - `Ok` if the connection was modified.
    pub fn set_edge_con_weight(&mut self, dir: TmeshDirection, weight: f64) -> Result<()> {
        if let Some(connection) = self.connection_mut(dir) {
            // If the connection is not an edge condition, return an error.
            if connection.0.is_some() {
                return Err(Error::TmeshExistingConnection);
            }

            connection.1 = weight;
            Ok(())
        } else {
            Err(Error::TmeshConnectionNotFound)
        }
    }

    /// Removes the connection in the direction `dir`, making it a T-junction. The connection in direection
    /// `dir` can be either a point connection or an edge condition. Modifies the connected point to also
    /// remove the connection, if it exists. In the case of an error, nothing is modified.
    ///
    /// # Returns
    /// - `TmeshConnectionNotFound` if the connection does not exist.
    ///
    /// - `TmeshControlPointNotFound` if the connected control point exists and cannot be
    ///   borrowed mutably. (Does not return this error if the connection is an edge condition,
    ///   that is, if the point does not exist.)
    ///
    /// - `Ok` if the connection was successfully removed.
    ///
    /// # Borrows
    /// The function must be able to mutably borrow the point self is connected to.
    pub fn remove_connection(&mut self, dir: TmeshDirection) -> Result<()> {
        if let Some(connection) = &mut self.connections[dir as usize] {
            // If the connection is not an edge condition, modify the connected point.
            if let Some(ref point) = connection.0 {
                let mut borrow = point.try_write().ok_or(Error::TmeshControlPointNotFound)?;

                // The connected point is connected to self on the opposite side.
                borrow.connections[dir.flip() as usize] = None;
            }

            self.connections[dir as usize] = None;
            Ok(())
        } else {
            Err(Error::TmeshConnectionNotFound)
        }
    }

    /// Removes the edge condition in direction `dir` from a point, replacing it with a T-juction.
    ///
    /// # WARNING
    /// USE THIS FUNCTION WITH CARE. Using this function incorrectly will create bugs which
    /// are extremely difficult to squash, since many T-mesh functions rely on a properly
    /// formatted T-mesh and DO NOT CHECK to make sure they are given one.
    ///
    /// # Returns
    /// - `TmeshExistingConnection` if `dir` is not an edge condition.
    ///
    /// - `Ok` if the edge codition was successfully removed.
    pub fn remove_edge_condition(&mut self, dir: TmeshDirection) -> Result<()> {
        let is_edge = self
            .connection_mut(dir)
            .as_ref()
            .is_some_and(|c| c.0.is_none());
        if is_edge {
            *self.connection_mut(dir) = None;
            Ok(())
        } else {
            Err(Error::TmeshExistingConnection)
        }
    }

    /// Connect `point` to `other` in the direction `dir` and give the connection knot interval `ki`.
    /// `other` is also connected to `point` in the corresponding manner. Knot coordinates are calculated
    /// by taking the knot coordinates of `point` and applying the delta `ki` to the relevant coordinate.
    /// Only `other`'s knot coordinates will change
    ///
    /// # Returns
    /// - `TmeshControlPointNotFound` if either `point` or `other` could not be borrowed mutably.
    ///
    /// - `TmeshExistingConnection` if either point has an existing connection in the
    ///   corresponding directions.
    ///
    /// - `TmeshExistingControlPoint` if `point` and `other` are the same control point.
    ///
    /// - `Ok` if the connection was successfully created between the two points.
    ///
    /// # Borrows
    /// `connect` borrows both `point` and `other` mutably.
    pub fn connect(
        point: Arc<RwLock<TmeshControlPoint<P>>>,
        other: Arc<RwLock<TmeshControlPoint<P>>>,
        dir: TmeshDirection,
        ki: f64,
    ) -> Result<()> {
        if std::ptr::eq(point.as_ref(), other.as_ref()) {
            return Err(Error::TmeshExistingControlPoint);
        }

        // is connection dir for point none?
        let con = {
            let borrow = point.try_read().ok_or(Error::TmeshControlPointNotFound)?;

            borrow.get(dir).is_none()
        };

        // is connection dir.flip() for other none?
        let other_con = {
            let borrow = other.try_read().ok_or(Error::TmeshControlPointNotFound)?;

            borrow.get(dir.flip()).is_none()
        };

        // If both points have no connections in the relevant directions, connect them together
        if con && other_con {
            // point -> other
            {
                let mut borrow = point.try_write().ok_or(Error::TmeshControlPointNotFound)?;

                borrow.connections[dir as usize] = Some((Some(Arc::clone(&other)), ki));
            }

            // point <-> other
            {
                let mut borrow = other.try_write().ok_or(Error::TmeshControlPointNotFound)?;

                // The connected point is connected to self on the opposite side
                borrow.connections[dir.flip() as usize] = Some((Some(Arc::clone(&point)), ki));
            }

            // Knot coordinates
            let mut delta = ki;
            other.write().knot_coordinates = point.write().knot_coordinates;

            // Subtract the knot interval if needed (dir is LEFT or DOWN).
            if dir.knot_delta_negative() {
                delta *= -1.0;
            }

            // Delta the correct knot coordinate based on dir.
            if dir.horizontal() {
                other.write().knot_coordinates.0 += delta;
            } else {
                other.write().knot_coordinates.1 += delta;
            }
        } else {
            // If either point has a connection in the relevant direction, return an error
            return Err(Error::TmeshExistingConnection);
        }

        Ok(())
    }

    /// Returns the connection type for `self` in the direction `dir`.
    pub fn con_type(&self, dir: TmeshDirection) -> TmeshConnectionType {
        // The first option differentiates between a T-junction and a knotted (weighted)
        // connection (edge condition or connection to another point).
        if let Some(con) = self.get(dir).as_ref() {
            // This option differentiates between a point connection and an edge condition.
            if con.0.is_some() {
                TmeshConnectionType::Point
            } else {
                TmeshConnectionType::Edge
            }
        } else {
            TmeshConnectionType::Tjunction
        }
    }

    /// Returns the knot interval for a connection in the direction `dir`.
    ///
    /// # Returns
    /// - `None` if a T-junction is found in the directoin `dir`.
    ///
    /// - `Some(f64)` otherwise.
    pub fn connection_knot(&self, dir: TmeshDirection) -> Option<f64> {
        match self.con_type(dir) {
            TmeshConnectionType::Edge | TmeshConnectionType::Point => Some(
                self.get(dir)
                    .as_ref()
                    .expect("Edge and Point connection types must have a Some(TmeshConnection<P>)")
                    .1,
            ),
            TmeshConnectionType::Tjunction => None,
        }
    }

    /// Navigates from `self` in the direction `traverse` until a connection is found in the direction `monitor`,
    /// returning a tuple of the point at which that connection was found and the knot interval traversed.
    /// Assumes that `self` has no connection in direction `monitor`.
    ///
    /// # Returns
    /// - `TmeshConnectionNotFound` if a T junction is found in the direction `traverse` while traversing.
    ///
    /// - `TmeshControlPointNotFound` if an edge condition is found in the direction `traverse` while traversing.
    ///
    /// - `Ok((Arc<RwLock<TmeshControlPoint<P>>>, f64))` if a connection was successfully found.
    ///
    /// # Borrows
    /// Immutably borrows all points that are connected to `self` in direction `traverse` and connected to the
    /// face that `self` is connected to.
    pub fn navigate_until_con(
        &self,
        traverse: TmeshDirection,
        monitor: TmeshDirection,
    ) -> Result<(Arc<RwLock<Self>>, f64)> {
        // Begin traversing (Think of this as a do while loop, where this let block is the
        // first "do" iteration)
        let (mut cur_point, mut knot_acc) = {
            let first = self.get(traverse);

            // Check initial conditions
            if let Some(con) = first {
                if let Some(point) = con.0.as_ref() {
                    let point = Arc::clone(point);
                    let interval = con.1;
                    (point, interval)
                } else {
                    return Err(Error::TmeshControlPointNotFound);
                }
            } else {
                return Err(Error::TmeshConnectionNotFound);
            }
        };

        'traverse: loop {
            // Traverse to the next point
            cur_point = {
                let borrow = cur_point.read();

                // Found the desired connection
                if borrow.get(monitor).as_ref().is_some_and(|c| c.0.is_some()) {
                    break 'traverse;
                }

                // Check for T-junction
                if let Some(con) = borrow.get(traverse) {
                    // Check for edge condition
                    if let Some(point) = con.0.as_ref() {
                        knot_acc += con.1;
                        Arc::clone(point)
                    } else {
                        return Err(Error::TmeshControlPointNotFound);
                    }
                } else {
                    return Err(Error::TmeshConnectionNotFound);
                }
            };
        }

        Ok((cur_point, knot_acc))
    }

    /// Returns the connected control point in the direction `dir` if it exists.
    ///
    /// # Returns
    /// - `TmeshConnectionNotFound` if the connection in direction `dir` is a T-junction.
    ///
    /// - `TmeshControlPointNotFound` if the connection in direction `dir` is an edge condition.
    ///
    /// - `Ok(point)` if the connection in direction `dir` is a point, where `point` is the corresponding control point.
    pub fn try_connected_point(
        &self,
        dir: TmeshDirection,
    ) -> Result<Arc<RwLock<TmeshControlPoint<P>>>> {
        let connected_point = &self
            .get(dir)
            .as_ref()
            .ok_or(Error::TmeshConnectionNotFound)?
            .0
            .as_ref()
            .ok_or(Error::TmeshControlPointNotFound)?;

        Ok(Arc::clone(connected_point))
    }

    /// Returns the connected control point in the direction `dir`, panicking if it does not exist.
    ///
    /// # Returns
    /// The point connected to `self`
    ///
    /// # Panics
    /// If there is no point connected to self in direction `dir`.
    pub fn connected_point(&self, dir: TmeshDirection) -> Arc<RwLock<TmeshControlPoint<P>>> {
        let connected_point = &self
            .get(dir)
            .as_ref()
            .ok_or(Error::TmeshConnectionNotFound)
            .expect("Point connections require a connection")
            .0
            .as_ref()
            .ok_or(Error::TmeshControlPointNotFound)
            .expect("Point connections require a connected point");

        Arc::clone(connected_point)
    }
}

impl<P> fmt::Display for TmeshControlPoint<P>
where P: Debug
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write self's point
        writeln!(f, "{:?} => {{", self.point())?;

        // Go through all directions for self
        for dir in TmeshDirection::iter() {
            // Write direction
            write!(f, "\t{}: ", dir)?;

            if let Some(con) = self.get(dir).as_ref() {
                if let Some(ref point_rc) = con.0 {
                    // Connection to another point.
                    let point_borrow = point_rc.read();
                    let point = point_borrow.point();
                    write!(f, "{:?} => {}", point, con.1)?;
                } else {
                    // Edge condition.
                    write!(f, "{}; Edge", con.1)?;
                }
            } else {
                // T-junction.
                write!(f, "~")?;
            }
            writeln!(f)?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}
