use crate::topo_impls::*;
use crate::topo_traits::*;
use truck_topology::*;

impl<P, C, T, Pc, Cc> ClosedSweep<T, Pc, Cc, Wire<P, C>> for Vertex<P>
where
    P: Clone,
    C: Clone,
    T: GeometricMapping<P> + Copy,
    Pc: Connector<P, C>,
{
    fn closed_sweep(&self, trans: T, point_connector: Pc, _: Cc, division: usize) -> Wire<P, C> {
        let point_mapping = &trans.mapping();
        let connect_points = &point_connector.connector();
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

impl<P, C, S, T, Pc, Cc> ClosedSweep<T, Pc, Cc, Shell<P, C, S>> for Edge<P, C>
where
    P: Clone,
    C: Clone,
    S: Clone,
    T: GeometricMapping<P> + GeometricMapping<C> + Copy,
    Pc: Connector<P, C>,
    Cc: Connector<C, S>,
{
    fn closed_sweep(
        &self,
        trans: T,
        point_connector: Pc,
        curve_connector: Cc,
        division: usize,
    ) -> Shell<P, C, S> {
        let point_mapping = &GeometricMapping::<P>::mapping(trans);
        let curve_mapping = &GeometricMapping::<C>::mapping(trans);
        let connect_points = &point_connector.connector();
        let connect_curves = &curve_connector.connector();
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

impl<P, C, S, T, Pc, Cc> ClosedSweep<T, Pc, Cc, Shell<P, C, S>> for Wire<P, C>
where
    P: Clone,
    C: Clone,
    S: Clone,
    T: GeometricMapping<P> + GeometricMapping<C> + Copy,
    Pc: Connector<P, C>,
    Cc: Connector<C, S>,
{
    fn closed_sweep(
        &self,
        trans: T,
        point_connector: Pc,
        curve_connector: Cc,
        division: usize,
    ) -> Shell<P, C, S> {
        let point_mapping = &GeometricMapping::<P>::mapping(trans);
        let curve_mapping = &GeometricMapping::<C>::mapping(trans);
        let connect_points = &point_connector.connector();
        let connect_curves = &curve_connector.connector();
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

impl<P, C, S, T, Pc, Cc> ClosedSweep<T, Pc, Cc, Solid<P, C, S>> for Face<P, C, S>
where
    P: Clone,
    C: Clone,
    S: Clone,
    T: GeometricMapping<P> + GeometricMapping<C> + GeometricMapping<S> + Copy,
    Pc: Connector<P, C>,
    Cc: Connector<C, S>,
{
    fn closed_sweep(
        &self,
        trans: T,
        point_connector: Pc,
        curve_connector: Cc,
        division: usize,
    ) -> Solid<P, C, S> {
        Solid::debug_new(
            self.boundaries()
                .iter()
                .map(move |wire| {
                    wire.closed_sweep(trans, point_connector, curve_connector, division)
                })
                .collect(),
        )
    }
}

impl<P, C, S, T, Pc, Cc> ClosedSweep<T, Pc, Cc, Vec<Result<Solid<P, C, S>>>> for Shell<P, C, S>
where
    P: Clone,
    C: Clone,
    S: Clone,
    T: GeometricMapping<P> + GeometricMapping<C> + GeometricMapping<S> + Copy,
    Pc: Connector<P, C>,
    Cc: Connector<C, S>,
{
    fn closed_sweep(
        &self,
        trans: T,
        point_connector: Pc,
        curve_connector: Cc,
        division: usize,
    ) -> Vec<Result<Solid<P, C, S>>> {
        self.connected_components()
            .into_iter()
            .map(move |shell| {
                Solid::try_new(
                    shell
                        .extract_boundaries()
                        .iter()
                        .map(|wire| {
                            wire.closed_sweep(trans, point_connector, curve_connector, division)
                        })
                        .collect(),
                )
            })
            .collect()
    }
}
