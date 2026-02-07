use super::*;

impl<P> TnurccControlPoint<P> {
    /// Createss a new `TnurccControlPoint` instance with index `index`, cartesian point `point`,
    /// valence `0`, and no incoming edge (`None`).
    pub fn new(index: usize, point: P) -> Self {
        TnurccControlPoint {
            index,
            valence: 0,
            point,
            incoming_edge: None,
        }
    }

    /// Returns a vector containing all edges connected to `p`. Returns an empty vector if `p` does
    /// not have an `incoming_edge`. Collects in an anti-clockwise fashion starting from `p`'s
    /// reference `incoming_edge`.
    ///
    /// # Panics
    /// Panics if `p`'s `incoming_edge` does not have `p` as an end.
    pub fn radial_edges(p: Arc<RwLock<TnurccControlPoint<P>>>) -> Vec<Arc<RwLock<TnurccEdge<P>>>> {
        if let Some(edge) = p.read().incoming_edge.as_ref().map(Arc::clone) {
            let point_end = edge
                .read()
                .point_end(Arc::clone(&p))
                .expect("Vertex should be on either end of reference edge.");

            TnurccAcwPointIter::from_edge(Arc::clone(&edge), point_end).collect()
        } else {
            Vec::new()
        }
    }

    /// Returns the edge connecting two points, if it exists.
    ///
    /// # Returns
    /// - `None` if `center` has no `incoming_edge`, if that `incoming_edge` is incorrectly assigned to `center`.
    ///   or if no edge connects `center` to `op`.
    ///
    /// - `Some(edge)` if the edge is found.
    ///
    /// # Borrows
    /// Immutably borrows `center`, `op`, and any edges connected to `center`.
    ///
    /// # Panics
    /// Panics if any borrow fails
    pub fn edge_from_opposing_point(
        center: Arc<RwLock<Self>>,
        op: Arc<RwLock<Self>>,
    ) -> Option<Arc<RwLock<TnurccEdge<P>>>> {
        if let Some(in_edge) = center.read().incoming_edge.as_ref().map(Arc::clone) {
            TnurccAcwPointIter::from_edge(
                Arc::clone(&in_edge),
                in_edge.read().point_end(Arc::clone(&center))?,
            )
            .find(|e| e.read().point_end(Arc::clone(&op)).is_some())
        } else {
            None
        }
    }
}

impl<P> Drop for TnurccControlPoint<P> {
    fn drop(&mut self) { self.incoming_edge = None; }
}
