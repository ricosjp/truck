use crate::mapped::Mapped;
use crate::topo_impls::*;
use std::collections::HashMap;
use truck_topology::*;

pub trait ClosedSweep<P, C, S> {
    type ClosedSweeped;
    fn closed_sweep<
        FP: Fn(&P) -> P,
        FC: Fn(&C) -> C,
        FS: Fn(&S) -> S,
        CP: Fn(&P, &P) -> C,
        CE: Fn(&C, &C) -> S,
    >(
        &self,
        point_mapping: &FP,
        curve_mapping: &FC,
        surface_mapping: &FS,
        connect_points: &CP,
        connect_curves: &CE,
        division: usize,
    ) -> Self::ClosedSweeped;
}

impl<P, C, S> ClosedSweep<P, C, S> for Vertex<P> {
    type ClosedSweeped = Wire<P, C>;
    fn closed_sweep<
        FP: Fn(&P) -> P,
        FC: Fn(&C) -> C,
        FS: Fn(&S) -> S,
        CP: Fn(&P, &P) -> C,
        CE: Fn(&C, &C) -> S,
    >(
        &self,
        point_mapping: &FP,
        curve_mapping: &FC,
        surface_mapping: &FS,
        connect_points: &CP,
        _: &CE,
        division: usize,
    ) -> Self::ClosedSweeped
    {
        let mut wire = Wire::new();
        let mut vertex = self.clone();
        for _ in 1..division {
            let new_vertex = vertex.mapped(point_mapping, curve_mapping, surface_mapping);
            wire.push_back(connect_vertices(&vertex, &new_vertex, connect_points));
            vertex = new_vertex;
        }
        wire.push_back(connect_vertices(&vertex, self, connect_points));
        wire
    }
}

impl<P, C, S> ClosedSweep<P, C, S> for Edge<P, C> {
    type ClosedSweeped = Shell<P, C, S>;
    fn closed_sweep<
        FP: Fn(&P) -> P,
        FC: Fn(&C) -> C,
        FS: Fn(&S) -> S,
        CP: Fn(&P, &P) -> C,
        CE: Fn(&C, &C) -> S,
    >(
        &self,
        point_mapping: &FP,
        curve_mapping: &FC,
        surface_mapping: &FS,
        connect_points: &CP,
        connect_curves: &CE,
        division: usize,
    ) -> Self::ClosedSweeped
    {
        let mut shell = Shell::new();
        shell
    }
}
