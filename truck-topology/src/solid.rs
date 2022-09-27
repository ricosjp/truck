use crate::errors::Error;
use crate::shell::ShellCondition;
use crate::*;
use std::vec::Vec;

impl<P, C, S> Solid<P, C, S> {
    /// create the shell whose boundaries is boundary.
    /// # Panic
    /// All boundary must be non-empty, connected, and closed manifold.
    #[inline(always)]
    pub fn new(boundaries: Vec<Shell<P, C, S>>) -> Solid<P, C, S> {
        Solid::try_new(boundaries).remove_try()
    }
    /// create the shell whose boundaries is boundary.
    /// # Failure
    /// All boundary must be non-empty, connected, and closed manifold.
    #[inline(always)]
    pub fn try_new(boundaries: Vec<Shell<P, C, S>>) -> Result<Solid<P, C, S>> {
        for shell in &boundaries {
            if shell.is_empty() {
                return Err(Error::EmptyShell);
            } else if !shell.is_connected() {
                return Err(Error::NotConnected);
            } else if shell.shell_condition() != ShellCondition::Closed {
                return Err(Error::NotClosedShell);
            } else if !shell.singular_vertices().is_empty() {
                return Err(Error::NotManifold);
            }
        }
        Ok(Solid::new_unchecked(boundaries))
    }
    /// create the shell whose boundaries is boundary.
    /// # Remarks
    /// This method is prepared only for performance-critical development and is not recommended.
    /// This method does NOT check whether all boundary is non-empty, connected, and closed.
    /// The programmer must guarantee this condition before using this method.
    #[inline(always)]
    pub fn new_unchecked(boundaries: Vec<Shell<P, C, S>>) -> Solid<P, C, S> { Solid { boundaries } }

    /// create the shell whose boundaries is boundary.
    /// # Remarks
    /// This method checks whether all boundary is non-empty, connected, and closed in the debug mode.
    /// The programmer must guarantee this condition before using this method.
    #[inline(always)]
    pub fn debug_new(boundaries: Vec<Shell<P, C, S>>) -> Solid<P, C, S> {
        match cfg!(debug_assertions) {
            true => Solid::new(boundaries),
            false => Solid::new_unchecked(boundaries),
        }
    }

    /// Returns the reference of boundary shells
    #[inline(always)]
    pub fn boundaries(&self) -> &Vec<Shell<P, C, S>> { &self.boundaries }
    /// Returns the boundary shells
    #[inline(always)]
    pub fn into_boundaries(self) -> Vec<Shell<P, C, S>> { self.boundaries }

    /// Returns an iterator over the faces.
    #[inline(always)]
    pub fn face_iter(&self) -> impl Iterator<Item = &Face<P, C, S>> {
        self.boundaries.iter().flatten()
    }

    /// Returns an iterator over the edges.
    #[inline(always)]
    pub fn edge_iter(&self) -> impl Iterator<Item = Edge<P, C>> + '_ {
        self.face_iter().flat_map(Face::boundaries).flatten()
    }

    /// Returns an iterator over the vertices.
    #[inline(always)]
    pub fn vertex_iter(&self) -> impl Iterator<Item = Vertex<P>> + '_ {
        self.edge_iter().map(|edge| edge.front().clone())
    }

    /// invert all faces
    #[inline(always)]
    pub fn not(&mut self) {
        self.boundaries
            .iter_mut()
            .flat_map(|shell| shell.face_iter_mut())
            .for_each(|face| {
                face.invert();
            })
    }

    /// Returns a new solid whose surfaces are mapped by `surface_mapping`,
    /// curves are mapped by `curve_mapping` and points are mapped by `point_mapping`.
    /// # Remarks
    /// Accessing geometry elements directly in the closure will result in a deadlock.
    /// So, this method does not appear to the document.
    #[doc(hidden)]
    #[inline(always)]
    pub fn try_mapped<Q, D, T>(
        &self,
        mut point_mapping: impl FnMut(&P) -> Option<Q>,
        mut curve_mapping: impl FnMut(&C) -> Option<D>,
        mut surface_mapping: impl FnMut(&S) -> Option<T>,
    ) -> Option<Solid<Q, D, T>> {
        Some(Solid::debug_new(
            self.boundaries()
                .iter()
                .map(move |shell| {
                    shell.try_mapped(&mut point_mapping, &mut curve_mapping, &mut surface_mapping)
                })
                .collect::<Option<Vec<_>>>()?,
        ))
    }

    /// Returns a new solid whose surfaces are mapped by `surface_mapping`,
    /// curves are mapped by `curve_mapping` and points are mapped by `point_mapping`.
    /// # Remarks
    /// Accessing geometry elements directly in the closure will result in a deadlock.
    /// So, this method does not appear to the document.
    #[doc(hidden)]
    #[inline(always)]
    pub fn mapped<Q, D, T>(
        &self,
        mut point_mapping: impl FnMut(&P) -> Q,
        mut curve_mapping: impl FnMut(&C) -> D,
        mut surface_mapping: impl FnMut(&S) -> T,
    ) -> Solid<Q, D, T> {
        Solid::debug_new(
            self.boundaries()
                .iter()
                .map(move |shell| {
                    shell.mapped(&mut point_mapping, &mut curve_mapping, &mut surface_mapping)
                })
                .collect(),
        )
    }

    /// Returns the consistence of the geometry of end vertices
    /// and the geometry of edge.
    #[inline(always)]
    pub fn is_geometric_consistent(&self) -> bool
    where
        P: Tolerance,
        C: BoundedCurve<Point = P>,
        S: IncludeCurve<C>, {
        self.boundaries()
            .iter()
            .all(|shell| shell.is_geometric_consistent())
    }

    /// Cuts one edge into two edges at vertex.
    #[inline(always)]
    pub fn cut_edge(
        &mut self,
        edge_id: EdgeID<C>,
        vertex: &Vertex<P>,
    ) -> Option<(Edge<P, C>, Edge<P, C>)>
    where
        P: Clone,
        C: Cut<Point = P> + SearchParameter<D1, Point = P>,
    {
        let res = self
            .boundaries
            .iter_mut()
            .find_map(|shell| shell.cut_edge(edge_id, vertex));
        #[cfg(debug_assertions)]
        Solid::new(self.boundaries.clone());
        res
    }
    /// Removes `vertex` from `self` by concat two edges on both sides.
    #[inline(always)]
    pub fn remove_vertex_by_concat_edges(&mut self, vertex_id: VertexID<P>) -> Option<Edge<P, C>>
    where
        P: Debug,
        C: Concat<C, Point = P, Output = C> + Invertible + ParameterTransform, {
        let res = self
            .boundaries
            .iter_mut()
            .find_map(|shell| shell.remove_vertex_by_concat_edges(vertex_id));
        #[cfg(debug_assertions)]
        Solid::new(self.boundaries.clone());
        res
    }

    /// Cut a face with `face_id` by edge.
    #[inline(always)]
    pub fn cut_face_by_edge(&mut self, face_id: FaceID<S>, edge: Edge<P, C>) -> bool
    where S: Clone {
        let tuple = self.boundaries.iter_mut().find_map(|shell| {
            let find_res = shell
                .face_iter_mut()
                .enumerate()
                .find(move |(_, face)| face.id() == face_id)
                .map(move |(i, _)| i);
            find_res.map(move |i| (shell, i))
        });
        if let Some((shell, i)) = tuple {
            if let Some(other) = shell[i].cut_by_edge(edge) {
                shell.push(other);
                return true;
            }
        }
        false
    }

    /// Creates display struct for debugging the solid.
    #[inline(always)]
    pub fn display(
        &self,
        format: SolidDisplayFormat,
    ) -> DebugDisplay<'_, Self, SolidDisplayFormat> {
        DebugDisplay {
            entity: self,
            format,
        }
    }
}

impl<P, C, S> PartialEq for Solid<P, C, S> {
    fn eq(&self, other: &Self) -> bool { self.boundaries == other.boundaries }
}

impl<P, C, S> Eq for Solid<P, C, S> {}

impl<'a, P: Debug, C: Debug, S: Debug> Debug
    for DebugDisplay<'a, Solid<P, C, S>, SolidDisplayFormat>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.format {
            SolidDisplayFormat::ShellsList { shell_format } => f
                .debug_list()
                .entries(
                    self.entity
                        .boundaries
                        .iter()
                        .map(|shell| shell.display(shell_format)),
                )
                .finish(),
            SolidDisplayFormat::ShellsListTuple { shell_format } => f
                .debug_tuple("Solid")
                .field(&DebugDisplay {
                    entity: self.entity,
                    format: SolidDisplayFormat::ShellsList { shell_format },
                })
                .finish(),
            SolidDisplayFormat::Struct { shell_format } => f
                .debug_struct("Solid")
                .field(
                    "boundaries",
                    &DebugDisplay {
                        entity: self.entity,
                        format: SolidDisplayFormat::ShellsList { shell_format },
                    },
                )
                .finish(),
        }
    }
}

#[cfg(test)]
pub(super) fn cube() -> Solid<(), (), ()> {
    use crate::*;
    let v = Vertex::news(&[(); 8]);
    let edge = [
        Edge::new(&v[0], &v[1], ()), // 0
        Edge::new(&v[1], &v[2], ()), // 1
        Edge::new(&v[2], &v[3], ()), // 2
        Edge::new(&v[3], &v[0], ()), // 3
        Edge::new(&v[0], &v[4], ()), // 4
        Edge::new(&v[1], &v[5], ()), // 5
        Edge::new(&v[2], &v[6], ()), // 6
        Edge::new(&v[3], &v[7], ()), // 7
        Edge::new(&v[4], &v[5], ()), // 8
        Edge::new(&v[5], &v[6], ()), // 9
        Edge::new(&v[6], &v[7], ()), // 10
        Edge::new(&v[7], &v[4], ()), // 11
    ];

    let wire0 = Wire::from_iter(vec![&edge[0], &edge[1], &edge[2], &edge[3]]);
    let face0 = Face::new(vec![wire0], ());

    let wire1 = Wire::from_iter(vec![
        &edge[4],
        &edge[8],
        &edge[5].inverse(),
        &edge[0].inverse(),
    ]);
    let face1 = Face::new(vec![wire1], ());

    let wire2 = Wire::from_iter(vec![
        &edge[5],
        &edge[9],
        &edge[6].inverse(),
        &edge[1].inverse(),
    ]);
    let face2 = Face::new(vec![wire2], ());

    let wire3 = Wire::from_iter(vec![
        &edge[6],
        &edge[10],
        &edge[7].inverse(),
        &edge[2].inverse(),
    ]);
    let face3 = Face::new(vec![wire3], ());
    let wire4 = Wire::from_iter(vec![
        &edge[7],
        &edge[11],
        &edge[4].inverse(),
        &edge[3].inverse(),
    ]);
    let face4 = Face::new(vec![wire4], ());
    let wire5 = Wire::from_iter(vec![
        &edge[11].inverse(),
        &edge[10].inverse(),
        &edge[9].inverse(),
        &edge[8].inverse(),
    ]);
    let face5 = Face::new(vec![wire5], ());

    let mut shell = Shell::new();
    shell.push(face0);
    shell.push(face5);
    assert!(!shell.is_connected());
    shell.push(face1);
    assert_eq!(shell.shell_condition(), ShellCondition::Oriented);
    assert!(shell.is_connected());
    shell.push(face2);
    shell.push(face3);
    shell.push(face4);

    Solid::new(vec![shell])
}

#[test]
fn cube_test() { cube(); }
