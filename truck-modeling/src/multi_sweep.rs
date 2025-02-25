use crate::topo_impls::*;
use crate::topo_traits::*;
use truck_topology::*;

impl<P, C, T, Pc, Cc> MultiSweep<T, Pc, Cc, Wire<P, C>> for Vertex<P>
where
    P: Clone,
    C: Clone,
    T: GeometricMapping<P> + Copy,
    Pc: Connector<P, C>,
{
    fn multi_sweep(&self, trans: T, point_connector: Pc, _: Cc, division: usize) -> Wire<P, C> {
        let point_mapping = &trans.mapping();
        let connect_points = &point_connector.connector();
        let mut vertex = self.clone();
        (0..division)
            .map(move |_| {
                let new_vertex = vertex.mapped(point_mapping);
                let edge = connect_vertices(&vertex, &new_vertex, connect_points);
                vertex = new_vertex;
                edge
            })
            .collect()
    }
}

impl<P, C, S, T, Pc, Cc> MultiSweep<T, Pc, Cc, Shell<P, C, S>> for Edge<P, C>
where
    P: Clone,
    C: Clone,
    S: Clone,
    T: GeometricMapping<P> + GeometricMapping<C> + Copy,
    Pc: Connector<P, C>,
    Cc: Connector<C, S>,
{
    fn multi_sweep(
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
        (0..division)
            .map(move |_| {
                let new_edge = edge.mapped(point_mapping, curve_mapping);
                let face = connect_edges(&edge, &new_edge, connect_points, connect_curves);
                edge = new_edge;
                face
            })
            .collect()
    }
}

impl<P, C, S, T, Pc, Cc> MultiSweep<T, Pc, Cc, Shell<P, C, S>> for Wire<P, C>
where
    P: Clone,
    C: Clone,
    S: Clone,
    T: GeometricMapping<P> + GeometricMapping<C> + Copy,
    Pc: Connector<P, C>,
    Cc: Connector<C, S>,
{
    fn multi_sweep(
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
        (0..division)
            .flat_map(move |_| {
                let new_wire = wire.mapped(point_mapping, curve_mapping);
                let shell: Vec<_> =
                    connect_wires(&wire, &new_wire, connect_points, connect_curves).collect();
                wire = new_wire;
                shell
            })
            .collect()
    }
}

impl<P, C, S, T, Pc, Cc> MultiSweep<T, Pc, Cc, Solid<P, C, S>> for Face<P, C, S>
where
    P: Clone,
    C: Clone,
    S: Clone,
    T: GeometricMapping<P> + GeometricMapping<C> + GeometricMapping<S> + Copy,
    Pc: Connector<P, C>,
    Cc: Connector<C, S>,
{
    fn multi_sweep(
        &self,
        trans: T,
        point_connector: Pc,
        curve_connector: Cc,
        division: usize,
    ) -> Solid<P, C, S> {
        let point_mapping = &GeometricMapping::<P>::mapping(trans);
        let curve_mapping = &GeometricMapping::<C>::mapping(trans);
        let surface_mapping = &GeometricMapping::<S>::mapping(trans);
        let connect_points = &point_connector.connector();
        let connect_curves = &curve_connector.connector();
        let mut shell = Shell::from(vec![self.inverse()]);
        let mut face_cursor = self.clone();
        shell.extend((0..division).flat_map(|_| {
            let seiling = face_cursor.mapped(point_mapping, curve_mapping, surface_mapping);
            let biter0 = face_cursor.boundary_iters().into_iter().flatten();
            let biter1 = seiling.boundary_iters().into_iter().flatten();
            let vec: Vec<_> =
                connect_raw_wires(biter0, biter1, connect_points, connect_curves).collect();
            face_cursor = seiling;
            vec
        }));
        shell.push(face_cursor);
        Solid::debug_new(vec![shell])
    }
}

impl<P, C, S, T, Pc, Cc> MultiSweep<T, Pc, Cc, Vec<Result<Solid<P, C, S>>>> for Shell<P, C, S>
where
    P: Clone,
    C: Clone,
    S: Clone,
    T: GeometricMapping<P> + GeometricMapping<C> + GeometricMapping<S> + Copy,
    Pc: Connector<P, C>,
    Cc: Connector<C, S>,
{
    fn multi_sweep(
        &self,
        trans: T,
        point_connector: Pc,
        curve_connector: Cc,
        division: usize,
    ) -> Vec<Result<Solid<P, C, S>>> {
        let point_mapping = &GeometricMapping::<P>::mapping(trans);
        let curve_mapping = &GeometricMapping::<C>::mapping(trans);
        let surface_mapping = &GeometricMapping::<S>::mapping(trans);
        let connect_points = &point_connector.connector();
        let connect_curves = &curve_connector.connector();
        self.connected_components()
            .into_iter()
            .map(move |shell| {
                let mut bdry: Shell<P, C, S> = shell.face_iter().map(Face::inverse).collect();
                let mut shell_cursor = shell;
                bdry.extend((0..division).flat_map(|_| {
                    let seiling =
                        shell_cursor.mapped(point_mapping, curve_mapping, surface_mapping);
                    let bdries0 = shell_cursor.extract_boundaries();
                    let bdries1 = seiling.extract_boundaries();
                    let biter0 = bdries0.iter().flat_map(Wire::edge_iter);
                    let biter1 = bdries1.iter().flat_map(Wire::edge_iter);
                    let vec: Vec<_> =
                        connect_wires(biter0, biter1, connect_points, connect_curves).collect();
                    shell_cursor = seiling;
                    vec
                }));
                bdry.append(&mut shell_cursor);
                Solid::try_new(vec![bdry])
            })
            .collect()
    }
}
