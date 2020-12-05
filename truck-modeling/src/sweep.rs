use truck_topology::*;
use crate::mapped::Mapped;
use crate::topo_impls::*;

/// Abstruct sweeping
pub trait Sweep<P, C, S> {
    /// The struct of sweeped topology.
    type Sweeped;
    /// Transform topologies and connect vertices and edges in boundaries.
    fn sweep<
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
        connect_curve: &CE,
    ) -> Self::Sweeped;
}

impl<P, C, S> Sweep<P, C, S> for Vertex<P> {
    type Sweeped = Edge<P, C>;
    /// Transforms a vertex and creates an edge by connecting vertices.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// use truck_modeling::sweep::*;
    /// let v = Vertex::new(1);
    /// let edge = v.sweep(
    ///     &move |i: &usize| *i + 1,
    ///     &usize::clone,
    ///     &<()>::clone,
    ///     &move |i: &usize, j: &usize| *i * 10 + j,
    ///     &move |_, _| (),
    /// );
    /// assert_eq!(*edge.front().lock_point().unwrap(), 1);
    /// assert_eq!(*edge.back().lock_point().unwrap(), 2);
    /// assert_eq!(*edge.lock_curve().unwrap(), 12);
    /// ```
    fn sweep<
        FP: Fn(&P) -> P,
        FC: Fn(&C) -> C,
        FS: Fn(&S) -> S,
        CP: Fn(&P, &P) -> C,
        CC: Fn(&C, &C) -> S,
    >(
        &self,
        point_mapping: &FP,
        curve_mapping: &FC,
        surface_mapping: &FS,
        connect_points: &CP,
        _: &CC,
    ) -> Self::Sweeped
    {
        let v = self.mapped(point_mapping, curve_mapping, surface_mapping);
        connect_vertices(self, &v, connect_points)
    }
}

impl<P, C, S> Sweep<P, C, S> for Edge<P, C> {
    type Sweeped = Face<P, C, S>;
    /// Transforms an edge and creates a face by connecting vertices and edges.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// use truck_modeling::sweep::*;
    /// let edge = Edge::new(
    ///     &Vertex::new(1),
    ///     &Vertex::new(2),
    ///     100,
    /// );
    /// let face = edge.sweep(
    ///     &move |i: &usize| *i + 2,
    ///     &move |j: &usize| *j + 100,
    ///     &usize::clone,
    ///     &move |i: &usize, j: &usize| *i * 10 + j,
    ///     &move |i: &usize, j: &usize| *i + *j,
    /// );
    ///
    /// assert_eq!(*face.lock_surface().unwrap(), 300);
    /// assert_eq!(face.boundaries().len(), 1);
    ///
    /// let boundary: Wire<usize, usize> = face.boundaries()[0].clone();
    /// assert_eq!(boundary.len(), 4);
    ///
    /// assert_eq!(boundary[0], edge);
    ///
    /// assert_eq!(*boundary[1].front().lock_point().unwrap(), 2);
    /// assert_eq!(*boundary[1].back().lock_point().unwrap(), 4);
    /// assert_eq!(*boundary[1].lock_curve().unwrap(), 24);
    ///
    /// assert_eq!(*boundary[2].front().lock_point().unwrap(), 4);
    /// assert_eq!(*boundary[2].back().lock_point().unwrap(), 3);
    /// // the curve of second edge is determined by connect_curves  
    /// assert_eq!(*boundary[2].lock_curve().unwrap(), 200);
    ///
    /// assert_eq!(*boundary[3].front().lock_point().unwrap(), 3);
    /// assert_eq!(*boundary[3].back().lock_point().unwrap(), 1);
    /// assert_eq!(*boundary[3].lock_curve().unwrap(), 13);
    /// ```
    fn sweep<
        FP: Fn(&P) -> P,
        FC: Fn(&C) -> C,
        FS: Fn(&S) -> S,
        CP: Fn(&P, &P) -> C,
        CC: Fn(&C, &C) -> S,
    >(
        &self,
        point_mapping: &FP,
        curve_mapping: &FC,
        surface_mapping: &FS,
        connect_points: &CP,
        connect_curves: &CC,
    ) -> Self::Sweeped
    {
        let edge = self.mapped(point_mapping, curve_mapping, surface_mapping);
        connect_edges(self, &edge, connect_points, connect_curves)
    }
}

impl<P, C, S> Sweep<P, C, S> for Wire<P, C> {
    type Sweeped = Shell<P, C, S>;
    /// Transforms a wire and creates a shell by connecting vertices and edges.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// use truck_modeling::sweep::*;
    /// use shell::ShellCondition;
    /// let v = Vertex::news(&[1, 2, 3, 4]);
    /// let wire = Wire::from(vec![
    ///     Edge::new(&v[0], &v[1], 100),
    ///     Edge::new(&v[1], &v[2], 110),
    ///     Edge::new(&v[3], &v[2], 120).inverse(),
    ///     Edge::new(&v[3], &v[1], 130),
    /// ]);
    /// let shell = wire.sweep(
    ///     &move |i: &usize| *i + 4,
    ///     &move |j: &usize| *j + 100,
    ///     &usize::clone,
    ///     &move |i: &usize, j: &usize| *i * 10 + j,
    ///     &move |i: &usize, j: &usize| *i + *j,
    /// );
    /// assert!(shell.is_connected());
    /// 
    /// let face1 = &shell[1];
    /// assert_eq!(*face1.lock_surface().unwrap(), 320);
    /// let boundary1 = &face1.boundaries()[0];
    /// assert_eq!(*boundary1[0].lock_curve().unwrap(), 110);
    /// assert_eq!(*boundary1[1].lock_curve().unwrap(), 37);
    /// assert_eq!(*boundary1[2].lock_curve().unwrap(), 210);
    /// assert_eq!(*boundary1[3].lock_curve().unwrap(), 26);
    /// assert_eq!(*boundary1[0].front().lock_point().unwrap(), 2);
    /// assert_eq!(*boundary1[1].front().lock_point().unwrap(), 3);
    /// assert_eq!(*boundary1[2].front().lock_point().unwrap(), 7);
    /// assert_eq!(*boundary1[3].front().lock_point().unwrap(), 6);
    /// 
    /// let face2 = &shell[2];
    /// assert_eq!(*face2.lock_surface().unwrap(), 340);
    /// let boundary2 = &face2.boundaries()[0];
    /// assert_eq!(*boundary2[0].lock_curve().unwrap(), 120);
    /// assert_eq!(*boundary2[1].lock_curve().unwrap(), 48);
    /// assert_eq!(*boundary2[2].lock_curve().unwrap(), 220);
    /// assert_eq!(*boundary2[3].lock_curve().unwrap(), 37);
    /// assert_eq!(*boundary2[0].front().lock_point().unwrap(), 3);
    /// assert_eq!(*boundary2[1].front().lock_point().unwrap(), 4);
    /// assert_eq!(*boundary2[2].front().lock_point().unwrap(), 8);
    /// assert_eq!(*boundary2[3].front().lock_point().unwrap(), 7);
    /// 
    /// assert_eq!(boundary1[1].id(), boundary2[3].id());
    /// assert_ne!(boundary1[1], boundary2[3]);
    /// ```
    fn sweep<
        FP: Fn(&P) -> P,
        FC: Fn(&C) -> C,
        FS: Fn(&S) -> S,
        CP: Fn(&P, &P) -> C,
        CC: Fn(&C, &C) -> S,
    >(
        &self,
        point_mapping: &FP,
        curve_mapping: &FC,
        surface_mapping: &FS,
        connect_points: &CP,
        connect_curves: &CC,
    ) -> Self::Sweeped
    {
        let wire = self.mapped(point_mapping, curve_mapping, surface_mapping);
        connect_wires(self, &wire, connect_points, connect_curves).collect()
    }
}

impl<P, C, S> Sweep<P, C, S> for Face<P, C, S> {
    type Sweeped = Solid<P, C, S>;
    /// Transforms a face and creates a solid by connecting vertices, edges and faces.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// use truck_modeling::sweep::*;
    /// let v = Vertex::news(&[1, 2]);
    /// let edge = Edge::new(&v[0], &v[1], 12);
    /// let face = edge.sweep(
    ///     &move |i: &usize| *i + 2,
    ///     &move |i: &usize| *i + 22,
    ///     &usize::clone,
    ///     &move |i: &usize, j: &usize| *i * 10 + *j,
    ///     &move |i: &usize, j: &usize| *i * 100 + *j,
    /// );
    /// let solid = face.sweep(
    ///     &move |i: &usize| *i + 4,
    ///     &move |i: &usize| *i + 44,
    ///     &move |i: &usize| *i + 3333,
    ///     &move |i: &usize, j: &usize| *i * 10 + *j,
    ///     &move |i: &usize, j: &usize| *i * 100 + *j,
    /// );
    /// let shell = &solid.boundaries()[0];
    /// # assert_eq!(shell.shell_condition(), shell::ShellCondition::Closed);
    /// 
    /// // The boundary shell has 6 faces since this solid is a cube.
    /// assert_eq!(shell.len(), 6);
    /// 
    /// // the first face of the boundary shell is the inversed original face.
    /// assert_eq!(shell[0].id(), face.id());
    /// assert_ne!(shell[0].orientation(), face.orientation());
    /// 
    /// // Check the condition of the third face.
    /// assert_eq!(*shell[2].lock_surface().unwrap(), 2468);
    /// let bdry = &shell[2].boundaries()[0];
    /// assert_eq!(*bdry[0].lock_curve().unwrap(), 24);
    /// assert_eq!(*bdry[1].lock_curve().unwrap(), 48);
    /// assert_eq!(*bdry[2].lock_curve().unwrap(), 68);
    /// assert_eq!(*bdry[3].lock_curve().unwrap(), 26);
    /// 
    /// // Check the last face: seiling.
    /// assert_eq!(*shell[5].lock_surface().unwrap(), 4567);
    /// ```
    fn sweep<
        FP: Fn(&P) -> P,
        FC: Fn(&C) -> C,
        FS: Fn(&S) -> S,
        CP: Fn(&P, &P) -> C,
        CC: Fn(&C, &C) -> S,
    >(
        &self,
        point_mapping: &FP,
        curve_mapping: &FC,
        surface_mapping: &FS,
        connect_points: &CP,
        connect_curves: &CC,
    ) -> Self::Sweeped
    {
        let mut shell = Shell::new();
        shell.push(self.inverse());
        let seiling = self.mapped(point_mapping, curve_mapping, surface_mapping);        
        let biter0 = self.boundary_iters().into_iter().flatten();
        let biter1 = seiling.boundary_iters().into_iter().flatten();
        shell.extend(connect_raw_wires(biter0, biter1, connect_points, connect_curves));
        shell.push(seiling);
        Solid::debug_new(vec![shell])
    }
}

impl<P, C, S> Sweep<P, C, S> for Shell<P, C, S> {
    type Sweeped = Vec<Result<Solid<P, C, S>>>;
    /// Transforms a shell and tries to create solids by connecting vertices, edges and faces.
    /// 
    /// In this function, the shell is broken down into connected components and each of components
    /// extruded to form a solid.
    /// 
    /// # Remarks
    /// For each component, this method returns `Result` of sweeping,
    /// since there is no clear guarantee that a solid can be formed by the extrusion of the shell.
    /// At least, a component must be oriented and not be closed to be extruded.
    fn sweep<
        FP: Fn(&P) -> P,
        FC: Fn(&C) -> C,
        FS: Fn(&S) -> S,
        CP: Fn(&P, &P) -> C,
        CC: Fn(&C, &C) -> S,
    >(
        &self,
        point_mapping: &FP,
        curve_mapping: &FC,
        surface_mapping: &FS,
        connect_points: &CP,
        connect_curves: &CC,
    ) -> Self::Sweeped
    {
        self.connected_components().into_iter().map(move|shell| {
            let mut bdry = Shell::new();
            let mut seiling = shell.mapped(point_mapping, curve_mapping, surface_mapping);
            bdry.extend(shell.face_iter().map(|face| face.inverse()));
            let bdries0 = shell.extract_boundaries();
            let bdries1 = seiling.extract_boundaries();
            let biter0 = bdries0.iter().flat_map(Wire::edge_iter);
            let biter1 = bdries1.iter().flat_map(Wire::edge_iter);
            bdry.extend(connect_wires(biter0, biter1, connect_points, connect_curves));
            bdry.append(&mut seiling);
            Solid::try_new(vec![bdry])
        }).collect()
    }
}

