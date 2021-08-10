use crate::topo_impls::*;
use crate::topo_traits::*;
use truck_topology::*;

impl<P: Clone, C: Clone, S: Clone> MultiSweep<P, C, S> for Vertex<P> {
    type Swept = Wire<P, C>;
    fn multi_sweep<
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
        let mut wire = Wire::new();
        let mut vertex = self.clone();
        for _ in 0..division {
            let new_vertex = vertex.mapped(point_mapping);
            wire.push_back(connect_vertices(&vertex, &new_vertex, connect_points));
            vertex = new_vertex;
        }
        wire
    }
}

impl<P: Clone, C: Clone, S: Clone> MultiSweep<P, C, S> for Edge<P, C> {
    type Swept = Shell<P, C, S>;
    fn multi_sweep<
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
        let mut shell = Shell::new();
        let mut edge = self.clone();
        for _ in 0..division {
            let new_edge = edge.mapped(point_mapping, curve_mapping);
            shell.push(connect_edges(
                &edge,
                &new_edge,
                connect_points,
                connect_curves,
            ));
            edge = new_edge;
        }
        shell
    }
}

impl<P: Clone, C: Clone, S: Clone> MultiSweep<P, C, S> for Wire<P, C> {
    type Swept = Shell<P, C, S>;
    fn multi_sweep<
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
        let mut shell = Shell::new();
        let mut wire = self.clone();
        for _ in 0..division {
            let new_wire = wire.mapped(point_mapping, curve_mapping);
            shell.extend(connect_wires(
                &wire,
                &new_wire,
                connect_points,
                connect_curves,
            ));
            wire = new_wire;
        }
        shell
    }
}

impl<P: Clone, C: Clone, S: Clone> MultiSweep<P, C, S> for Face<P, C, S> {
    type Swept = Solid<P, C, S>;
    fn multi_sweep<
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
        let mut shell = Shell::new();
        shell.push(self.inverse());
        let mut face_cursor = self.clone();
        for _ in 0..division {
            let seiling = face_cursor.mapped(point_mapping, curve_mapping, surface_mapping);
            let biter0 = face_cursor.boundary_iters().into_iter().flatten();
            let biter1 = seiling.boundary_iters().into_iter().flatten();
            shell.extend(connect_raw_wires(
                biter0,
                biter1,
                connect_points,
                connect_curves,
            ));
            face_cursor = seiling;
        }
        shell.push(face_cursor);
        Solid::debug_new(vec![shell])
    }
}

impl<P: Clone, C: Clone, S: Clone> MultiSweep<P, C, S> for Shell<P, C, S> {
    type Swept = Vec<Result<Solid<P, C, S>>>;
    fn multi_sweep<
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
                let mut bdry = Shell::new();
                bdry.extend(shell.face_iter().map(|face| face.inverse()));
                let mut shell_cursor = shell.clone();
                for _ in 0..division {
                    let seiling =
                        shell_cursor.mapped(point_mapping, curve_mapping, surface_mapping);
                    let bdries0 = shell_cursor.extract_boundaries();
                    let bdries1 = seiling.extract_boundaries();
                    let biter0 = bdries0.iter().flat_map(Wire::edge_iter);
                    let biter1 = bdries1.iter().flat_map(Wire::edge_iter);
                    bdry.extend(connect_wires(
                        biter0,
                        biter1,
                        connect_points,
                        connect_curves,
                    ));
                    shell_cursor = seiling;
                }
                bdry.append(&mut shell_cursor);
                Solid::try_new(vec![bdry])
            })
            .collect()
    }
}
