use crate::topo_traits::*;
use truck_topology::*;

impl<P: Clone, C: Clone, S: Clone> Mapped<P, C, S> for Vertex<P> {
    #[inline(always)]
    fn mapped<FP: Fn(&P) -> P, FC: Fn(&C) -> C, FS: Fn(&S) -> S>(
        &self,
        point_mapping: &FP,
        _: &FC,
        _: &FS,
    ) -> Self {
        self.mapped(point_mapping)
    }
}

impl<P: Clone, C: Clone, S: Clone> Mapped<P, C, S> for Edge<P, C> {
    #[inline(always)]
    fn mapped<FP: Fn(&P) -> P, FC: Fn(&C) -> C, FS: Fn(&S) -> S>(
        &self,
        point_mapping: &FP,
        curve_mapping: &FC,
        _: &FS,
    ) -> Self {
        self.mapped(point_mapping, curve_mapping)
    }
}

impl<P: Clone, C: Clone, S: Clone> Mapped<P, C, S> for Wire<P, C> {
    #[inline(always)]
    fn mapped<FP: Fn(&P) -> P, FC: Fn(&C) -> C, FS: Fn(&S) -> S>(
        &self,
        point_mapping: &FP,
        curve_mapping: &FC,
        _: &FS,
    ) -> Self {
        self.mapped(point_mapping, curve_mapping)
    }
}

impl<P: Clone, C: Clone, S: Clone> Mapped<P, C, S> for Face<P, C, S> {
    #[inline(always)]
    fn mapped<FP: Fn(&P) -> P, FC: Fn(&C) -> C, FS: Fn(&S) -> S>(
        &self,
        point_mapping: &FP,
        curve_mapping: &FC,
        surface_mapping: &FS,
    ) -> Self {
        self.mapped(point_mapping, curve_mapping, surface_mapping)
    }
}

impl<P: Clone, C: Clone, S: Clone> Mapped<P, C, S> for Shell<P, C, S> {
    #[inline(always)]
    fn mapped<FP: Fn(&P) -> P, FC: Fn(&C) -> C, FS: Fn(&S) -> S>(
        &self,
        point_mapping: &FP,
        curve_mapping: &FC,
        surface_mapping: &FS,
    ) -> Self {
        self.mapped(point_mapping, curve_mapping, surface_mapping)
    }
}

impl<P: Clone, C: Clone, S: Clone> Mapped<P, C, S> for Solid<P, C, S> {
    /// Returns a new solid whose surfaces are mapped by `surface_mapping`,
    /// curves are mapped by `curve_mapping` and points are mapped by `point_mapping`.
    #[inline(always)]
    fn mapped<FP: Fn(&P) -> P, FC: Fn(&C) -> C, FS: Fn(&S) -> S>(
        &self,
        point_mapping: &FP,
        curve_mapping: &FC,
        surface_mapping: &FS,
    ) -> Self {
        Solid::debug_new(
            self.boundaries()
                .iter()
                .map(|shell| shell.mapped(point_mapping, curve_mapping, surface_mapping))
                .collect(),
        )
    }
}
