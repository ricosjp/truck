use super::*;

impl<P> TnurccFace<P> {
    /// Returns a vector containing all the points defining the boundry of the current face, in an
    /// anti-clockwise order starting from the origin of `f`'s reference edge. Returns an empty vector
    /// if `f` does not have a reference `edge`.
    ///
    /// # Borrows
    /// Immutably borrows `f` and all edges which define the edge of `f`
    ///
    /// # Panics
    /// Panics if `f`'s reference `edge` does not reference `f` as a face on either side.
    pub fn boundry_verticies(f: Arc<RwLock<Self>>) -> Vec<Arc<RwLock<TnurccControlPoint<P>>>> {
        if let Some(edge) = f.read().edge.as_ref().map(Arc::clone) {
            let face_side = edge
                .read()
                .face_side(Arc::clone(&f))
                .expect("Face edge should be on either side of that edge.");
            let iter = TnurccAcwFaceIter::try_from_edge(Arc::clone(&edge), face_side)
                .expect("Edge should have face on side if get_face_side succeeded");

            iter.map(|e| {
                match e
                    .read()
                    .face_side(Arc::clone(&f))
                    .expect("Edge on perimeter of face should be connected to the face")
                {
                    TnurccFaceSide::Left => Arc::clone(&e.read().origin),
                    TnurccFaceSide::Right => Arc::clone(&e.read().dest),
                }
            })
            .collect()
        } else {
            Vec::new()
        }
    }

    /// Returns a vector containing all the edges defining the border of the current face, in an
    /// anti-clockwise order starting from `f`'s reference edge. Returns an empty vector if `f`
    /// does not have a reference `edge`.
    ///
    /// # Borrows
    /// Immutably borrows `f` and all edges which define the edge of `f`
    ///
    /// # Panics
    /// Panics if `f`'s reference `edge` does not reference `f` as a face on either side.
    pub fn border_edges(f: Arc<RwLock<Self>>) -> Vec<Arc<RwLock<TnurccEdge<P>>>> {
        if let Some(edge) = f.read().edge.as_ref().map(Arc::clone) {
            let face_side = edge
                .read()
                .face_side(Arc::clone(&f))
                .expect("Face edge should be on either side of that edge.");
            let iter = TnurccAcwFaceIter::try_from_edge(Arc::clone(&edge), face_side)
                .expect("Edge should have face on side if face_side succeeded");

            iter.collect()
        } else {
            Vec::new()
        }
    }
}

impl<P> Drop for TnurccFace<P> {
    fn drop(&mut self) {
        for i in 0..self.corners.len() {
            self.corners[i] = None;
        }
        self.edge = None;
    }
}
