use crate::topo_impls::*;
use crate::topo_traits::*;
use truck_topology::*;

impl<P: Clone, C: Clone, S: Clone> ClosedSweep<P, C, S> for Vertex<P> {
    fn closed_sweep<
        FP: Fn(&P) -> P,
        FC: Fn(&C) -> C,
        FS: Fn(&S) -> S,
        CP: Fn(&P, &P) -> C,
        CE: Fn(&C, &C) -> S,
    >(
        &self,
        point_mapping: &FP,
        _: &FC,
        _: &FS,
        connect_points: &CP,
        _: &CE,
        division: usize,
    ) -> Self::Swept {
        let mut vertex = self.clone();
        let mut wire: Wire<P, C> = (1..division)
            .map(|_| {
                let new_vertex = vertex.mapped(point_mapping);
                let edge = connect_vertices(&vertex, &new_vertex, connect_points);
                vertex = new_vertex;
                edge
            })
            .collect();
        wire.push_back(connect_vertices(&vertex, self, connect_points));
        wire
    }
}

impl<P: Clone, C: Clone, S: Clone> ClosedSweep<P, C, S> for Edge<P, C> {
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
        _: &FS,
        connect_points: &CP,
        connect_curves: &CE,
        division: usize,
    ) -> Self::Swept {
        let mut edge = self.clone();
        let mut shell: Shell<P, C, S> = (1..division)
            .map(|_| {
                let new_edge = edge.mapped(point_mapping, curve_mapping);
                let face = connect_edges(&edge, &new_edge, connect_points, connect_curves);
                edge = new_edge;
                face
            })
            .collect();
        shell.push(connect_edges(&edge, self, connect_points, connect_curves));
        shell
    }
}

impl<P: Clone, C: Clone, S: Clone> ClosedSweep<P, C, S> for Wire<P, C> {
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
        _: &FS,
        connect_points: &CP,
        connect_curves: &CE,
        division: usize,
    ) -> Self::Swept {
        let mut wire = self.clone();
        let mut shell: Shell<P, C, S> = (1..division)
            .flat_map(|_| {
                let new_wire = wire.mapped(point_mapping, curve_mapping);
                let vec: Vec<_> =
                    connect_wires(&wire, &new_wire, connect_points, connect_curves).collect();
                wire = new_wire;
                vec
            })
            .collect();
        shell.extend(connect_wires(&wire, self, connect_points, connect_curves));
        shell
    }
}

impl<P: Clone, C: Clone, S: Clone> ClosedSweep<P, C, S> for Face<P, C, S> {
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
    ) -> Self::Swept {
        Solid::debug_new(
            self.boundaries()
                .iter()
                .map(move |wire| {
                    wire.closed_sweep(
                        point_mapping,
                        curve_mapping,
                        surface_mapping,
                        connect_points,
                        connect_curves,
                        division,
                    )
                })
                .collect(),
        )
    }
}

impl<P: Clone, C: Clone, S: Clone> ClosedSweep<P, C, S> for Shell<P, C, S> {
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
    ) -> Self::Swept {
        self.connected_components()
            .into_iter()
            .map(move |shell| {
                Solid::try_new(
                    shell
                        .extract_boundaries()
                        .iter()
                        .map(|wire| {
                            wire.closed_sweep(
                                point_mapping,
                                curve_mapping,
                                surface_mapping,
                                connect_points,
                                connect_curves,
                                division,
                            )
                        })
                        .collect(),
                )
            })
            .collect()
    }
}
