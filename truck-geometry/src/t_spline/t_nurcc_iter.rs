use super::*;

impl<P> TnurccAcwPointIter<P> {
    /// Creates a new `TnurccAcwPointIter` which iterates the edges around `e`'s vertex `end` in an anti-clockwise manner.
    /// Returns `None` without making a full rotation, that is, will return `e` as the first element, but not the last.
    ///
    /// # Borrows
    /// Immutably borrows every edge connected to the point at `e`'s `end` when calling `next`.
    pub fn from_edge(e: Arc<RwLock<TnurccEdge<P>>>, end: TnurccVertexEnd) -> Self {
        return TnurccAcwPointIter {
            point: Arc::clone(&e.read().point_at_end(end)),
            start: Arc::clone(&e),
            cur: Some(Arc::clone(&e)),
        };
    }
}

impl<P> TnurccAcwFaceIter<P> {
    /// Creates a new `TnurccAcwFaceIter` which iterates the edges around `e`'s face `side` in an anti-clockwise manner.
    /// `next` returns `None` without making a full rotation, that is, will return `e` as the first element, but not the last.
    ///
    /// # Returns
    /// - `None` if `e` does not have a face on `side`.
    ///
    /// - `Some(iter)` otherwise.
    pub fn try_from_edge(e: Arc<RwLock<TnurccEdge<P>>>, side: TnurccFaceSide) -> Option<Self> {
        if let Some(face) = e.read().face_from_side(side) {
            Some(TnurccAcwFaceIter {
                face,
                start: Arc::clone(&e),
                cur: Some(Arc::clone(&e)),
            })
        } else {
            None
        }
    }

    /// Creates a new `TnurccAcwFaceIter` which iterates the edges around `f` in an anti-clockwise manner.
    /// `next` returns `None` without making a full rotation, that is, will not return the first edge twice.
    ///
    /// # Returns
    /// - `None` if `f` does not have a reference edge.
    ///
    /// - `Some(iter)` otherwise.
    #[allow(dead_code)]
    pub fn try_from_face(f: Arc<RwLock<TnurccFace<P>>>) -> Option<Self> {
        if let Some(edge) = f.read().edge.as_ref() {
            Some(TnurccAcwFaceIter {
                face: Arc::clone(&f),
                start: Arc::clone(edge),
                cur: Some(Arc::clone(edge)),
            })
        } else {
            None
        }
    }
}

impl<P> Iterator for TnurccAcwPointIter<P> {
    type Item = Arc<RwLock<TnurccEdge<P>>>;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.cur.as_ref().map(Arc::clone);

        if let Some(edge) = self.cur.as_ref() {
            // Is point the origin or dest?
            let end = edge.read().point_end(Arc::clone(&self.point));

            end?;
            let end = end.unwrap();

            // Get the next ACW edge for point
            let new_edge = edge.read().acw_edge_from_end(end);

            // If the new edge is the starting edge, stop the iterator by setting cur to none
            // Otherwise, keep going
            if std::ptr::eq(self.start.as_ref(), new_edge.as_ref()) {
                self.cur = None;
            } else {
                self.cur = Some(new_edge);
            }
        } else {
            return None;
        }

        ret
    }
}

impl<P> Iterator for TnurccAcwFaceIter<P> {
    type Item = Arc<RwLock<TnurccEdge<P>>>;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.cur.as_ref().map(Arc::clone);

        if let Some(edge) = self.cur.as_ref() {
            // Is point the origin or dest?
            let side = edge.read().face_side(Arc::clone(&self.face));

            side?;
            let side = side.unwrap();

            // Get the next ACW edge for point
            let new_edge = edge.read().acw_edge_from_side(side);

            // If the new edge is the starting edge, stop the iterator by setting cur to none
            // Otherwise, keep going
            if std::ptr::eq(self.start.as_ref(), new_edge.as_ref()) {
                self.cur = None;
            } else {
                self.cur = Some(new_edge);
            }
        } else {
            return None;
        }

        ret
    }
}
