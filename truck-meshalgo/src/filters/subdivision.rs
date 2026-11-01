use super::*;

/// subdivision surface algorithms
pub trait Subdivision {
    /// Extended Loop method in-place.
    ///
    /// # Remarks
    /// Confirm:
    /// - All faces are triangles.
    /// - `self.shell_condition()` is `Oriented` or `Closed` before use.
    ///
    /// This method does NOT check these conditions.
    fn loop_subdivision(&mut self) -> &mut Self;
    /// Extended Loop subdivision, returning a new mesh.
    fn loop_subdivide(&self) -> PolygonMesh;
    /// Linear 4-to-1 triangle subdivision (midpoint refinement) in-place.
    fn linear_subdivision(&mut self) -> &mut Self;
    /// Linear 4-to-1 triangle subdivision (midpoint refinement), returning a new mesh.
    fn linear_subdivide(&self) -> PolygonMesh;
}

impl Subdivision for PolygonMesh {
    fn loop_subdivision(&mut self) -> &mut Self {
        *self = self.loop_subdivide();
        self
    }

    fn loop_subdivide(&self) -> PolygonMesh {
        dispatch::Dispatcher::global().loop_subdivide(self)
    }

    fn linear_subdivide(&self) -> PolygonMesh {
        dispatch::Dispatcher::global().linear_subdivide(self)
    }

    fn linear_subdivision(&mut self) -> &mut Self {
        *self = self.linear_subdivide();
        self
    }
}

