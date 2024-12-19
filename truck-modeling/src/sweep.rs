use crate::topo_impls::*;
use crate::topo_traits::*;
use truck_topology::*;

impl<P, C, T, PC, CC> Sweep<T, PC, CC, Edge<P, C>> for Vertex<P>
where
    P: Clone,
    C: Clone,
    T: GeometricMapping<P> + Copy,
    PC: Connector<P, C>,
{
    /// Transforms a vertex and creates an edge by connecting vertices.
    /// # Examples
    /// ```
    /// truck_topology::prelude!(usize, isize, ());
    /// use truck_modeling::topo_traits::*;
    /// 
    /// let v = Vertex::new(1);
    /// let edge = v.sweep(
    ///     (|i| i + 1) as fn(&usize) -> usize,
    ///     (|i, j| (i * 10 + j) as isize) as fn(&usize, &usize) -> isize,
    ///     (|_, _ | ()) as fn(&isize, &isize) -> (),
    /// );
    /// assert_eq!(edge.front().point(), 1);
    /// assert_eq!(edge.back().point(), 2);
    /// assert_eq!(edge.curve(), 12);
    /// ```
    fn sweep(&self, trans: T, connect_points: PC, _: CC) -> Edge<P, C> {
        let v = self.mapped(trans.mapping());
        connect_vertices(self, &v, &connect_points.connector())
    }
}

impl<P, C, S, T, PC, CC> Sweep<T, PC, CC, Face<P, C, S>> for Edge<P, C>
where
    P: Clone,
    C: Clone,
    S: Clone,
    T: GeometricMapping<P> + GeometricMapping<C> + Copy,
    PC: Connector<P, C>,
    CC: Connector<C, S>,
{
    /// Transforms an edge and creates a face by connecting vertices and edges.
    /// # Examples
    /// ```
    /// truck_topology::prelude!(usize, isize, i64);
    /// use truck_modeling::topo_traits::*;
    /// 
    /// #[derive(Clone, Copy)]
    /// struct Mapping;
    /// impl GeometricMapping<usize> for Mapping {
    ///     fn mapping(self) -> impl Fn(&usize) -> usize { |i| *i + 2 }
    /// }
    /// impl GeometricMapping<isize> for Mapping {
    ///     fn mapping(self) -> impl Fn(&isize) -> isize { |i| *i + 100 }
    /// }
    ///
    /// let v = Vertex::news([1, 2]);
    /// let edge = Edge::new(&v[0], &v[1], 100);
    /// let face = edge.sweep(
    ///     Mapping,
    ///     (|i, j| (i * 10 + j) as isize) as fn(&usize, &usize) -> isize,
    ///     (|i, j| (i + j) as i64) as fn(&isize, &isize) -> i64,
    /// );
    ///
    /// assert_eq!(face.surface(), 300);
    /// assert_eq!(face.boundaries().len(), 1);
    ///
    /// let boundary = face.boundaries()[0].clone();
    /// assert_eq!(boundary.len(), 4);
    ///
    /// assert_eq!(boundary[0], edge);
    ///
    /// assert_eq!(boundary[1].front().point(), 2);
    /// assert_eq!(boundary[1].back().point(), 4);
    /// assert_eq!(boundary[1].curve(), 24);
    ///
    /// assert_eq!(boundary[2].front().point(), 4);
    /// assert_eq!(boundary[2].back().point(), 3);
    /// // the curve of second edge is determined by connect_curves
    /// assert_eq!(boundary[2].curve(), 200);
    ///
    /// assert_eq!(boundary[3].front().point(), 3);
    /// assert_eq!(boundary[3].back().point(), 1);
    /// assert_eq!(boundary[3].curve(), 13);
    /// ```
    fn sweep(&self, trans: T, point_connector: PC, curve_connector: CC) -> Face<P, C, S> {
        let point_mapping = GeometricMapping::<P>::mapping(trans);
        let curve_mapping = GeometricMapping::<C>::mapping(trans);
        let connect_points = point_connector.connector();
        let connect_curves = curve_connector.connector();
        let edge = self.mapped(point_mapping, curve_mapping);
        connect_edges(self, &edge, &connect_points, &connect_curves)
    }
}

impl<P, C, S, T, PC, CC> Sweep<T, PC, CC, Shell<P, C, S>> for Wire<P, C>
where
    P: Clone,
    C: Clone,
    S: Clone,
    T: GeometricMapping<P> + GeometricMapping<C> + Copy,
    PC: Connector<P, C>,
    CC: Connector<C, S>,
{
    /// Transforms a wire and creates a shell by connecting vertices and edges.
    /// # Examples
    /// ```
    /// truck_topology::prelude!(usize, isize, i64);
    /// use truck_modeling::topo_traits::*;
    /// 
    /// #[derive(Clone, Copy)]
    /// struct Mapping;
    /// impl GeometricMapping<usize> for Mapping {
    ///     fn mapping(self) -> impl Fn(&usize) -> usize { |i| *i + 4 }
    /// }
    /// impl GeometricMapping<isize> for Mapping {
    ///     fn mapping(self) -> impl Fn(&isize) -> isize { |j| *j + 100 }
    /// }
    /// 
    /// let v = Vertex::news(&[1, 2, 3, 4]);
    /// let wire = Wire::from(vec![
    ///     Edge::new(&v[0], &v[1], 100),
    ///     Edge::new(&v[1], &v[2], 110),
    ///     Edge::new(&v[3], &v[2], 120).inverse(),
    ///     Edge::new(&v[3], &v[1], 130),
    /// ]);
    /// let shell = wire.sweep(
    ///     Mapping,
    ///     (|i, j| (i * 10 + j) as isize) as fn(&usize, &usize) -> isize,
    ///     (|i, j| (i + j) as i64) as fn(&isize, &isize) -> i64,
    /// );
    /// assert!(shell.is_connected());
    ///
    /// let face1 = &shell[1];
    /// assert_eq!(face1.surface(), 320);
    /// let boundary1 = &face1.boundaries()[0];
    /// assert_eq!(boundary1[0].curve(), 110);
    /// assert_eq!(boundary1[1].curve(), 37);
    /// assert_eq!(boundary1[2].curve(), 210);
    /// assert_eq!(boundary1[3].curve(), 26);
    /// assert_eq!(boundary1[0].front().point(), 2);
    /// assert_eq!(boundary1[1].front().point(), 3);
    /// assert_eq!(boundary1[2].front().point(), 7);
    /// assert_eq!(boundary1[3].front().point(), 6);
    ///
    /// let face2 = &shell[2];
    /// assert_eq!(face2.surface(), 340);
    /// let boundary2 = &face2.boundaries()[0];
    /// assert_eq!(boundary2[0].curve(), 120);
    /// assert_eq!(boundary2[1].curve(), 48);
    /// assert_eq!(boundary2[2].curve(), 220);
    /// assert_eq!(boundary2[3].curve(), 37);
    /// assert_eq!(boundary2[0].front().point(), 3);
    /// assert_eq!(boundary2[1].front().point(), 4);
    /// assert_eq!(boundary2[2].front().point(), 8);
    /// assert_eq!(boundary2[3].front().point(), 7);
    ///
    /// assert_eq!(boundary1[1].id(), boundary2[3].id());
    /// assert_ne!(boundary1[1], boundary2[3]);
    /// ```
    fn sweep(&self, trans: T, point_connector: PC, curve_connector: CC) -> Shell<P, C, S> {
        let point_mapping = GeometricMapping::<P>::mapping(trans);
        let curve_mapping = GeometricMapping::<C>::mapping(trans);
        let connect_points = point_connector.connector();
        let connect_curves = curve_connector.connector();
        let wire = self.mapped(point_mapping, curve_mapping);
        connect_wires(self, &wire, &connect_points, &connect_curves).collect()
    }
}

impl<P, C, S, T, PC, CC> Sweep<T, PC, CC, Solid<P, C, S>> for Face<P, C, S>
where
    P: Clone,
    C: Clone,
    S: Clone,
    T: GeometricMapping<P> + GeometricMapping<C> + GeometricMapping<S> + Copy,
    PC: Connector<P, C>,
    CC: Connector<C, S>,
{
    /// Transforms a face and creates a solid by connecting vertices, edges and faces.
    /// # Examples
    /// ```
    /// truck_topology::prelude!(usize, isize, i64);
    /// use truck_modeling::topo_traits::*;
    /// 
    /// #[derive(Clone, Copy)]
    /// struct Mapping(usize, isize, i64);
    /// impl GeometricMapping<usize> for Mapping {
    ///     fn mapping(self) -> impl Fn(&usize) -> usize { move |i| *i + self.0 }
    /// }
    /// impl GeometricMapping<isize> for Mapping {
    ///     fn mapping(self) -> impl Fn(&isize) -> isize { move |i| *i + self.1 }
    /// }
    /// impl GeometricMapping<i64> for Mapping {
    ///     fn mapping(self) -> impl Fn(&i64) -> i64 { move |i| *i + self.2 }
    /// }
    /// 
    /// let connect_points: fn(&usize, &usize) -> isize = |i, j| (*i * 10 + *j) as isize;
    /// let connect_curves: fn(&isize, &isize) -> i64 = |i, j| (*i * 100 + *j) as i64;
    /// 
    /// let v = Vertex::news(&[1, 2]);
    /// let edge = Edge::new(&v[0], &v[1], 12);
    /// let face = edge.sweep(Mapping(2, 22, 0), connect_points, connect_curves);
    /// let solid = face.sweep(Mapping(4, 44, 3333), connect_points, connect_curves);
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
    /// assert_eq!(shell[2].surface(), 2468);
    /// let bdry = &shell[2].boundaries()[0];
    /// assert_eq!(bdry[0].curve(), 24);
    /// assert_eq!(bdry[1].curve(), 48);
    /// assert_eq!(bdry[2].curve(), 68);
    /// assert_eq!(bdry[3].curve(), 26);
    ///
    /// // Check the last face: seiling.
    /// assert_eq!(shell[5].surface(), 4567);
    /// ```
    fn sweep(&self, trans: T, point_connector: PC, curve_connector: CC) -> Solid<P, C, S> {
        let point_mapping = GeometricMapping::<P>::mapping(trans);
        let curve_mapping = GeometricMapping::<C>::mapping(trans);
        let surface_mapping = GeometricMapping::<S>::mapping(trans);
        let connect_points = point_connector.connector();
        let connect_curves = curve_connector.connector();
        let mut shell = shell![self.inverse()];
        let seiling = self.mapped(point_mapping, curve_mapping, surface_mapping);
        let biter0 = self.boundary_iters().into_iter().flatten();
        let biter1 = seiling.boundary_iters().into_iter().flatten();
        shell.extend(connect_raw_wires(
            biter0,
            biter1,
            &connect_points,
            &connect_curves,
        ));
        shell.push(seiling);
        Solid::debug_new(vec![shell])
    }
}

impl<P, C, S, T, PC, CC> Sweep<T, PC, CC, Vec<Result<Solid<P, C, S>>>> for Shell<P, C, S>
where
    P: Clone,
    C: Clone,
    S: Clone,
    T: GeometricMapping<P> + GeometricMapping<C> + GeometricMapping<S> + Copy,
    PC: Connector<P, C>,
    CC: Connector<C, S>,
{
    /// Transforms a shell and tries to create solids by connecting vertices, edges and faces.
    ///
    /// In this function, the shell is broken down into connected components and each of components
    /// extruded to form a solid.
    ///
    /// # Remarks
    /// For each component, this method returns `Result` of sweeping,
    /// since there is no clear guarantee that a solid can be formed by the extrusion of the shell.
    /// At least, a component must be oriented and not be closed to be extruded.
    fn sweep(
        &self,
        trans: T,
        point_connector: PC,
        curve_connector: CC,
    ) -> Vec<Result<Solid<P, C, S>>> {
        let point_mapping = GeometricMapping::<P>::mapping(trans);
        let curve_mapping = GeometricMapping::<C>::mapping(trans);
        let surface_mapping = GeometricMapping::<S>::mapping(trans);
        let connect_points = point_connector.connector();
        let connect_curves = curve_connector.connector();
        self.connected_components()
            .into_iter()
            .map(move |shell| {
                let mut bdry = Shell::new();
                let mut seiling = shell.mapped(&point_mapping, &curve_mapping, &surface_mapping);
                bdry.extend(shell.face_iter().map(|face| face.inverse()));
                let bdries0 = shell.extract_boundaries();
                let bdries1 = seiling.extract_boundaries();
                let biter0 = bdries0.iter().flat_map(Wire::edge_iter);
                let biter1 = bdries1.iter().flat_map(Wire::edge_iter);
                bdry.extend(connect_wires(
                    biter0,
                    biter1,
                    &connect_points,
                    &connect_curves,
                ));
                bdry.append(&mut seiling);
                Solid::try_new(vec![bdry])
            })
            .collect()
    }
}
