use super::*;
use crate::errors::Error;

impl<P> Tnurcc<P>
where
    P: Debug,
{
    /// Creates a new `Tnurcc` instance. `points` is a vector containing the control points in the mesh, and `faces`
    /// describes the connections of the mesh. `faces` must described every face in the mesh, as no `faces` will be
    /// infered by the constructor. Each face in `faces` contains 4 edges. Each edge is described by a tuple containing
    /// an initial point index and a vector containing the other point indicies and knot intervals on the edge.
    /// It is important to note that because there is no sense of orientation in the T-NURCC, the points in edges, and
    /// the edges themselves must be arranged in the correct order prior to instantiating the `Tnurcc` relative to each
    /// other. All point indicies refer to the indecies of the points in the `points` parameter.
    ///
    /// Put together, this means that in an edge, the initial point index must be either the clockwise first corner or
    /// anti-clockwise first corner (Recommended to use anti-clockwise for face culling reasons). That is to say, for the
    /// "top" edge, the "right" corner should be used as the initial point index. Then, the connected points vector should
    /// contain the indicies of the points in order, sweeping across the "top" edge and their knot intervals. For a visual
    /// explenation, see the figure below.
    ///
    /// ```text
    ///     0    6   8  9
    ///   4 +----+---+--+
    ///     |           |
    ///     |           |
    ///   2 |           +
    ///     |           |
    ///   0 +--+---+----+
    ///     0  5   7    9
    /// ```
    /// In the above figure, the numbers represent the cartesian coordinate of the points, while the number of `-`
    /// (minus) or `|` (virtical pipe) characters between them represents the knot interval. In this case, `points`
    /// might be the vector:
    ///
    /// `[(0, 0), (5, 0), (7, 0), (9, 0), (0, 3), (9, 2), (0, 4), (6, 4), (8, 4), (9, 4)]`
    ///
    /// The `points` vector does not need to have a specific order, however, the ordering of the elements in `points`
    /// will change the indicies in `faces`. The above example is also only one face, whereas most T-NURCCs will have
    /// multiple faces. Each point should be included exactly once in the `points` vector regardless of how many faces
    /// it participates in. Then, the edges for the face, if made anti-clockwise, could be:
    ///
    /// `[(0, [(1, 2.0), (2, 3.0), (3, 4.0)]), (3, [(5, 1.0), (9, 2.0)]), (9, [(8, 2.0), (7, 3.0), (6, 4.0)]), (6, [(0, 4.0)])]`
    ///
    /// Notice that the knot interval for points in the connections vector is the relative knot distance between the point it is
    /// tuple'd with and the point *prior* to it. The above face vector is one of four possible identical face elements, where
    /// the other three are the rotations of the edge elements in the face. The edges are also ordered in an anti-clockwise fashion.
    ///
    /// It is mandatory that all faces have the same knot interval spanning opposing edges. In the example above, that means that the
    /// "top" and "bottom" edge must have the same total knot interval, and the same goes for the "left" and "right". In short, all faces
    /// must be rectangular in knot-space (parametrically rectangular).
    ///
    /// # Returns
    ///
    /// - `TnurccNonRectangularFace` if any face is not parametrically rectangular.
    ///
    /// - `TnurccEdgeTrippleFace` if any edge lies between three faces.
    ///
    /// - `TnurccIncompleteFaceEdge` if any edge is comprised of less than 2 points.
    ///
    /// - `Ok(Tnurcc)` if the T-NURCC was succsefully created.
    ///
    /// # Panics
    #[allow(clippy::type_complexity)]
    pub fn try_new(points: Vec<P>, faces: Vec<[(usize, Vec<(usize, f64)>); 4]>) -> Result<Self> {
        let mut control_points = Vec::with_capacity(points.len());

        for (index, point) in points.into_iter().enumerate() {
            control_points.push(Arc::new(RwLock::new(TnurccControlPoint::new(index, point))));
        }

        let mut tnurcc_faces = Vec::new();
        let mut edges: Vec<Arc<RwLock<TnurccEdge<P>>>> = Vec::new();
        for face in faces {
            // Verify that the face has the same knot interval on opposing faces by summing interval for each side
            let dimensions: Vec<f64> = face
                .iter()
                .map(|e| e.1.iter().fold(0.0, |s, p| s + p.1))
                .collect();

            // Subtract opposing side's knot intervals
            if !(dimensions[0] - dimensions[2] + dimensions[1] - dimensions[3]).so_small() {
                return Err(Error::TnurccNonRectangularFace);
            }

            // Produce a vector containing all the point indicies in the face, in order, such that any two adjacent element
            // in the vector should be connected, and a vector containing each connection's weight.
            let (mut connections, knot_intervals) = face
                .into_iter()
                // Converet the format of the side into an array of all points in the side. Remember that sides must,
                // in addition to specifying the initial point, specify the last point in agreement with the first
                // point of the next anti-clockwise side. Thus, we can ignore the initial point, since it is contained
                // in the previous edge's connections vector.
                .flat_map(|e| e.1)
                .collect::<(Vec<_>, Vec<_>)>();

            // The last connection is between the first and last element, which is included in this by adding the
            // first control point to the end of the vector so that a window will pick it up
            connections.push(connections[0]);

            // Collect all edges described by face (This is such a painfully O(N*M) algorithm I don't
            // want to think about it but I'm currently at a loss for what else to do about it...)
            let mut existing_edges: Vec<_> = {
                let map_closure = |c: &[usize]| {
                    edges
                        .iter()
                        .find(|e| {
                            let borrow = e.read();
                            let origin_index = borrow.origin.read().index;
                            let dest_index = borrow.dest.read().index;

                            c.contains(&origin_index) && c.contains(&dest_index)
                        })
                        .map(Arc::clone)
                };

                connections.windows(2).map(map_closure).collect()
            };

            let face = Arc::new(RwLock::new(TnurccFace {
                index: tnurcc_faces.len(),
                edge: None,
                corners: [const { None }; 4],
            }));
            for con_index in 0..existing_edges.len() {
                if let Some(edge) = existing_edges[con_index].as_ref() {
                    if edge.read().face_right.is_some() {
                        return Err(Error::TnurccEdgeTrippleFace);
                    }

                    edge.write().face_right = Some(Arc::clone(&face));
                    if face.read().edge.is_none() {
                        face.write().edge = Some(Arc::clone(edge));
                    }

                    // Connect the current edge to the previous edge in the loop
                    if con_index > 0 {
                        let con_res = TnurccEdge::connect(
                            Arc::clone(edge),
                            Arc::clone(
                                existing_edges[con_index - 1]
                                    .as_ref()
                                    .expect("Previous edge should always exist"),
                            ),
                        );
                        // Map success types
                        con_res?
                    }
                } else {
                    let from = connections[con_index];
                    let to = connections[con_index + 1];
                    let index = edges.len();
                    let edge = TnurccEdge::new(
                        index,
                        knot_intervals[con_index],
                        Arc::clone(&control_points[from]),
                        Arc::clone(&control_points[to]),
                    );

                    edge.write().face_left = Some(Arc::clone(&face));
                    if face.read().edge.is_none() {
                        face.write().edge = Some(Arc::clone(&edge));
                    }

                    if con_index > 0 {
                        let con_res = TnurccEdge::connect(
                            Arc::clone(&edge),
                            Arc::clone(
                                existing_edges[con_index - 1]
                                    .as_ref()
                                    .expect("Previous edge should always exist"),
                            ),
                        );
                        // Map success types
                        con_res?
                    }
                    edges.push(Arc::clone(&edge));
                    existing_edges[con_index].replace(Arc::clone(&edge));
                }
            }

            let first_edge = Arc::clone(
                existing_edges[0]
                    .as_ref()
                    .expect("All edges should exist after loop"),
            );

            let last_edge = Arc::clone(
                existing_edges
                    .last()
                    .expect("existing_edges should contain multiple edges")
                    .as_ref()
                    .expect("All edges should exist after loop"),
            );

            // Connect the first and the last edge
            let con_res = TnurccEdge::connect(Arc::clone(&first_edge), Arc::clone(&last_edge));
            // Map success types
            con_res?;

            tnurcc_faces.push(face);
        }
        for e in edges.iter() {
            if e.read().face_left.is_none() || e.read().face_right.is_none() {
                return Err(Error::TnurccMissingFace);
            }
        }

        // Collect all extraordinary points (points with valence )
        let extraordinary_control_points = control_points
            .iter()
            .filter(|p| p.read().valence != 4)
            .map(Arc::clone)
            .collect();

        Ok(Tnurcc {
            edges,
            control_points,
            extraordinary_control_points,
            faces: tnurcc_faces,
        })
    }

    /// Creates a new `Tnurcc` instance using `try_new`, panicking if it fails. See [`Tnurcc::try_new`] for details on the constructor.
    ///
    /// # Panics
    /// Panics if construction fails.
    #[allow(clippy::type_complexity)]
    pub fn new(points: Vec<P>, faces: Vec<[(usize, Vec<(usize, f64)>); 4]>) -> Self {
        Tnurcc::try_new(points, faces).unwrap()
    }
}

impl<P> Tnurcc<P>
where
    P: ControlPoint<f64>,
{
    /// Performs the global subdivide algorithm required by \[Sederberg et al. 2003\] and described
    /// in \[Sederberg et al. 1998\], dubbed "refinement".
    ///
    /// # Returns
    /// - `Ok(())` on succesfull subdivision.
    /// - `TnurccMalformedFace` if boundary vertecies for a face cannot be collected.
    ///
    /// # Panics
    /// - If any borrow fails.
    /// - If any reference geometry does not correctly point to the object it is referencing.
    /// - If any new connections or edge splits fail.
    ///
    /// # Borrows
    /// Mutably borrows `self.edges`, `self.control_points`, and `self.faces`, as well as all elements contained within.
    pub fn global_subdivide(&mut self) -> Result<()> {
        use TnurccConnection::*;
        let mut face_points = Vec::with_capacity(self.faces.len());
        let mut edge_points = Vec::with_capacity(self.edges.len());
        let mut edge_m_points = Vec::with_capacity(self.edges.len());
        let mut split_edge = vec![false; self.edges.len()];
        // let mut vertex_points = Vec::with_capacity(self.control_points.len());
        // Used for creation of emtpy edges whose control points are not yet known
        let dummy_point = Arc::new(RwLock::new(TnurccControlPoint::new(0, P::origin())));

        // Some wrapped functions which apply in the specific case here where edges are being
        // retrieved from control points, guaranteeing membership.
        let nth_acw_int = |e, p, n| {
            TnurccEdge::nth_acw_edge_from_point(e, p, n)
                .expect("Edge should contain point it was synthesized from")
                .read()
                .knot_interval
        };
        let nth_cw_int = |e, p, n| {
            TnurccEdge::nth_cw_edge_from_point(e, p, n)
                .expect("Edge should contain point it was synthesized from")
                .read()
                .knot_interval
        };

        // Calculate new face points
        for face in self.faces.iter() {
            let mut w_vec = Vec::with_capacity(self.faces.len());
            let mut w_points = Vec::with_capacity(self.faces.len());

            let mut cir_points = TnurccFace::boundry_verticies(Arc::clone(face));
            // If there are no points defining a face, that is a problem.
            if cir_points.is_empty() {
                return Err(Error::TnurccMalformedFace);
            }
            // Copy the first 4 points to the end of the array to extend the windows of the array so that the
            // first and last two points in the array are given a window in which they are the "center" element.
            cir_points.push(Arc::clone(&cir_points[0]));
            cir_points.push(Arc::clone(&cir_points[1]));
            cir_points.push(Arc::clone(&cir_points[2]));
            cir_points.push(Arc::clone(&cir_points[3]));

            for win in cir_points.windows(5) {
                // let (pm2, pm1, p, pp1, pp2) = (win[0], win[1], win[2], win[3], win[4]);
                // Array structures as [d_{i-2, i-2}, d_{i-1, i}, d_{i, i+1}, d_{i+1, i+2}]
                let edges = win
                    .windows(2)
                    .map(|a| {
                        TnurccControlPoint::edge_from_opposing_point(
                            Arc::clone(&a[0]),
                            Arc::clone(&a[1]),
                        )
                        .expect("adjacent vertecies in a face should be connected by an edge")
                    })
                    .collect::<Vec<_>>();

                let first_sum = edges[2].read().knot_interval                 // d_{i+1, i}^{0}
                    + nth_acw_int(Arc::clone(&edges[2]), Arc::clone(&win[3]), 2)  // d_{i+1, i}^{2}
                    + nth_cw_int(Arc::clone(&edges[2]), Arc::clone(&win[3]), 2)   // d_{i+1, i}^{-2}
                    + edges[0].read().knot_interval                           // d_{i-2, i-1}^{0}
                    + nth_acw_int(Arc::clone(&edges[0]), Arc::clone(&win[0]), 2)  // d_{i-2, i-1}^{2}
                    + nth_cw_int(Arc::clone(&edges[0]), Arc::clone(&win[0]), 2); // d_{i-2, i-1}^{-2}

                let second_sum = edges[1].read().knot_interval                 // d_{i-1, i}^{0}
                    + nth_acw_int(Arc::clone(&edges[1]), Arc::clone(&win[1]), 2)  // d_{i-1, i}^{2}
                    + nth_cw_int(Arc::clone(&edges[1]), Arc::clone(&win[1]), 2)   // d_{i-1, i}^{-2}
                    + edges[3].read().knot_interval                           // d_{i+2, i+1}^{0}
                    + nth_acw_int(Arc::clone(&edges[3]), Arc::clone(&win[4]), 2)  // d_{i+2, i+1}^{2}
                    + nth_cw_int(Arc::clone(&edges[3]), Arc::clone(&win[4]), 2); // d_{i+2, i+1}^{-2}

                let w = first_sum * second_sum;
                let w_p = win[2].read().point * w;
                w_vec.push(w);
                w_points.push(w_p);
            }

            let face_point: P = w_points
                .into_iter()
                .fold(P::origin(), |sum, p| sum + p.to_vec())
                / w_vec.iter().sum();
            face_points.push(face_point);
        }

        // Compute the location of the new point which splits every edge in mesh
        // (Equation 13  in \[Sederberg et al. 1998\])
        for edge in self.edges.iter() {
            // Equivalen to F_{ij} in Equation 13 of \[Sederberg et al. 1998\]
            let f_od = face_points[edge
                .read()
                .face_left
                .as_ref()
                .expect("All edges should have faces on both sides")
                .read()
                .index];
            // Equivalen to F_{ji} in Equation 13 of \[Sederberg et al. 1998\]
            let f_do = face_points[edge
                .read()
                .face_right
                .as_ref()
                .expect("All edges should have faces on both sides")
                .read()
                .index];

            // Denominator of equation 14 in \[Sederberg et al. 1998\]
            let a_denom: f64 = (0..4)
                .map(|n| {
                    edge.read()
                        .connection(TnurccConnection::from_usize(n))
                        .read()
                        .knot_interval
                })
                .sum::<f64>()
                * 2.0;

            // Equation 14 in \[Sederberg et al. 1998\] for alpha_{ij}
            // TODO: Check that the TnurccConnection parity is correct (L/R)
            let a_od = {
                if a_denom.so_small() {
                    0.0
                } else {
                    [LeftAcw, LeftCw]
                        .iter()
                        .map(|c| edge.read().connection(*c).read().knot_interval)
                        .sum::<f64>()
                        / a_denom
                }
            };
            // Equation 14 in \[Sederberg et al. 1998\] for alpha_{ji}
            // TODO: Check that the TnurccConnection parity is correct (L/R)
            let a_do = {
                if a_denom.so_small() {
                    0.0
                } else {
                    [RightAcw, RightCw]
                        .iter()
                        .map(|c| edge.read().connection(*c).read().knot_interval)
                        .sum::<f64>()
                        / a_denom
                }
            };
            // Equation 15 in \[Sederberg et al. 1998\]
            let m: P = {
                let num_dest_sum = edge.read().knot_interval
                    + nth_acw_int(Arc::clone(edge), Arc::clone(&edge.read().origin), 2)
                    + nth_cw_int(Arc::clone(edge), Arc::clone(&edge.read().origin), 2);
                let num_origin_sum = edge.read().knot_interval
                    + nth_acw_int(Arc::clone(edge), Arc::clone(&edge.read().dest), 2)
                    + nth_cw_int(Arc::clone(edge), Arc::clone(&edge.read().dest), 2);
                let m_denom = num_dest_sum + num_origin_sum;

                let origin = edge.read().origin.read().point;
                let dest = edge.read().dest.read().point;

                // If block not in the paper, but seems nescessary
                if m_denom.so_small() {
                    (origin + dest.to_vec()) * 0.5
                } else {
                    (origin * num_dest_sum + (dest * num_origin_sum).to_vec()) / m_denom
                }
            };
            edge_m_points.push(m);

            let e = m * (1.0 - a_do - a_od) + (f_od * a_od).to_vec() + (f_do * a_do).to_vec();
            edge_points.push(e);
        }

        // Compute the new location of every vertex in the mesh
        // (Equation 16  in \[Sederberg et al. 1998\])
        for vertex in self.control_points.iter() {
            let p_naught = vertex.read().point;
            let valence = vertex.read().valence as f64;

            // Get radial edges and push first again as last for last window
            let mut radial_edges = TnurccControlPoint::radial_edges(Arc::clone(vertex));
            if radial_edges.is_empty() {
                // Probably needs its own error
                // TODO: Fix the Tnurcc errors?
                return Err(Error::TnurccMalformedFace);
            }
            radial_edges.push(Arc::clone(&radial_edges[0]));

            // Group radial edges into windows of 2. This is not strictly nesessary for the calculations
            // (in \[Sederberg et al. 1998\] they do not do this), however, it makes aquiring the radial
            // faces around the vertex much easier, as the common face between two edges can be used, and
            // then the first edge in the window will always be the "actionable" edge which is used for
            // the equations specified in \[Sederberg et al. 1998\]. The accumulator variable is a tuple
            // containing the numerator and denominator from equation 16 in \[Sederberg et al. 1998\],
            // respectively, sans the integer multiples 3 and n.
            let factional_components =
                radial_edges
                    .windows(2)
                    .fold((P::origin(), 0.0), |acc, win| {
                        let face = win[0]
                            .read()
                            .common_face(Arc::clone(&win[1]))
                            .expect("Adjacent edges should share a face");
                        // Get the face point calculated in equation 11 in \[Sederberg et al. 1998\]
                        let f_point = face_points[face.read().index];
                        // Equation 18 in \[Sederberg et al. 1998\]. Radial faces are in ACW order, so win[0] is the
                        // edge from which the next ACW edge is retrieved, and vice-versa
                        let f_scalar = nth_acw_int(Arc::clone(&win[0]), Arc::clone(vertex), 1)
                            * nth_cw_int(Arc::clone(&win[1]), Arc::clone(vertex), 1);

                        // Get the edge point calculated in equation 15 in \[Sederberg et al. 1998\]
                        let m_point = edge_points[win[0].read().index];
                        // Equation 17 in \[Sederberg et al. 1998\]
                        let m_scalar = 0.5
                            * (nth_acw_int(Arc::clone(&win[0]), Arc::clone(vertex), 1)
                                + nth_cw_int(Arc::clone(&win[0]), Arc::clone(vertex), 1))
                            * (nth_acw_int(Arc::clone(&win[0]), Arc::clone(vertex), 2)
                                + nth_cw_int(Arc::clone(&win[0]), Arc::clone(vertex), 2));

                        (
                            acc.0 + (m_point * m_scalar).to_vec() + (f_point * f_scalar).to_vec(),
                            acc.1 + m_scalar + f_scalar,
                        )
                    });

            // Equation 16 in \[Sederberg et al. 1998\], with help from equation 19 from the same.
            vertex.write().point = if factional_components.1.so_small() {
                p_naught
            } else {
                let c = (valence - 3.0) / valence;
                p_naught * c
                    + ((factional_components.0 * 3.0) / (valence * factional_components.1)).to_vec()
            };
        }

        // Calculate the new know spacings (Section 4.2.1 in \[Sederberg et al. 1998\]) and create the new
        // edges and faces which complete the subdivision.
        // let original_vertex_count = self.control_points.len();
        for (face_i, f_p) in face_points.into_iter().enumerate() {
            // Recall that for each face, a face point was constructed and stored in the
            // vector such that a given index maps the face to and from the face point.
            let face = Arc::clone(&self.faces[face_i]);
            // perim will only contain the edges which were originally part of the perimiter for the face
            let perim = TnurccFace::border_edges(Arc::clone(&face))
                .into_iter()
                .filter(|e| split_edge.get(e.read().index).is_some())
                .collect::<Vec<_>>();

            // New control point for the face
            let f_cp = Arc::new(RwLock::new(TnurccControlPoint::new(
                self.control_points.len(),
                f_p,
            )));
            f_cp.write().valence = perim.len();

            // In essance, a new face and edge is created for every original edge on the perimiter.
            // Allocate and create them here, then, when splitting and connecting the edges, assign
            // edges and so forth. For every new face, the 0th corner is the new face point. Note that
            // the assignment of corners to the new face is not as trivial as it may initially seem,
            // since the edge splitting process is not orientation agnostic. Furthermore, when iterating
            // on the perimiter, we are able to access every point which will be asssigned as a corner to
            // one of the new faces, but we will not not have (easy) access to all corners for a single
            // face at once. Thus, the corner assignment algorithm is a little unexpected and should not
            // be attempted without a diagram (One may be provided in the comments if I am feeling
            // diligent). As a guiding principle, remember that as the index for the perimeter edge
            // increases, we traverse the perimeter in an anti-clockwise fashion.
            let mut new_faces = Vec::with_capacity(perim.len());
            let mut radial_edges = Vec::with_capacity(perim.len()); // Indexes with perim_i
            let mut edge_conjugates = Vec::with_capacity(perim.len());

            let perim_knots = perim
                .iter()
                .map(|e| e.read().knot_interval)
                .collect::<Vec<_>>();
            self.control_points.push(Arc::clone(&f_cp));

            (0..perim.len()).for_each(|i| {
                new_faces.push(Arc::new(RwLock::new(TnurccFace::<P> {
                    index: self.faces.len() + i,
                    edge: None,
                    corners: [Some(Arc::clone(&f_cp)), None, None, None],
                })));

                radial_edges.push(Arc::new(RwLock::new(TnurccEdge::<P> {
                    index: 0,
                    connections: [None, None, None, None],
                    face_left: None,
                    face_right: None,
                    origin: Arc::clone(&f_cp),
                    dest: Arc::clone(&dummy_point),
                    knot_interval: 0.0,
                })));
            });

            // In order to guarantee that auto-connecting the faces works as expected, we need to iterate through the
            // perimeter twice. Once to split all edges which aren't already split, collecting split conjugates as we
            // go, and another to assign faces and connect edges. Doing this all at once is difficult, as having too much
            // of the mesh up in the air at once makes knowing exactly what is where almost impossible.
            for (perim_i, edge) in perim.iter().enumerate() {
                // Some helper variables to make life a little nicer
                let edge_index = edge.read().index;

                // Only split edge if it shas not been split before.
                if *split_edge
                    .get(edge_index)
                    .expect("All out of bounds edges should have been filtered out on construction of perim") 
                {
                    // If the edge is already split, then the edge which was generated by the split is connected to the 
                    // current edge, since none of the radial edges have been connected yet. The match is nesessary
                    // because the conjugate may be located in different positioins depending on which face we are
                    // currently iterating.
                    edge_conjugates.push(
                        match edge
                            .read()
                            .face_side(Arc::clone(&face))
                            .expect("Edge on face perimiter should be connected to face")
                        {
                            TnurccFaceSide::Left => edge.read().connection(LeftAcw),
                            TnurccFaceSide::Right => edge.read().connection(RightCw),
                        },
                    );

                    // If the edge has been split, that means that the edge's destination is the center of the split (by convention)
                    // Guaranteed to be the case because new edges created by this algorithm were filtered out. Set the corresponding radial edge's destination to the 
                    radial_edges[perim_i].write().dest = Arc::clone(&edge.read().dest);
                } else {
                    // If it is not split, split it
                    split_edge[edge_index] = true;
                    // No need to check which side is which for knot intervals because the interval is symetrical
                    let edge_control_point = TnurccEdge::split_edge(
                        Arc::clone(edge),
                        self.edges.len(),
                        edge_points[edge_index],
                        self.control_points.len(),
                        0.5,
                    )
                    .expect("Subdivide should always be able to split an edge");
                    // The edge that results from the split
                    let pair = edge.read().connection(LeftAcw);

                    // Push split edge and new control point
                    self.edges.push(Arc::clone(&pair));
                    self.control_points.push(Arc::clone(&edge_control_point));

                    edge_conjugates.push(pair);
                    radial_edges[perim_i].write().dest = edge_control_point;
                };
            }

            for (perim_i, edge) in perim.iter().enumerate() {
                // Some helper variables to make life a little nicer
                let next_perim_index = (perim_i + 1) % perim.len();
                let prev_perim_index = (perim_i + perim.len() - 1) % perim.len();
                let edge_face_side = edge
                    .read()
                    .face_side(Arc::clone(&self.faces[face_i]))
                    .expect("Edge on face perimiter should be connected to face");

                // Depending on the orientation of the current edge, different points must be assigned to be the corners of the face.
                // Recall that corners[0] was set in the constructor. Also note that corners are assigned in an anti-clockwise fashion,
                // which is not strictly nescessary, but makes keeping track of what is where simpler. Notice that the same corner
                // indicies for the sme new face indicies are modified every iteration regardless of orientation.
                // This is also the time to assign the correct faces to the edges, including the face comming out of the new face point.
                match edge_face_side {
                    TnurccFaceSide::Left => {
                        // Assign corners
                        new_faces[perim_i].write().corners[2] =
                            Some(Arc::clone(&edge.read().origin));
                        new_faces[perim_i].write().corners[3] =
                            Some(Arc::clone(&edge.read().dest));

                        new_faces[next_perim_index].write().corners[1] =
                            Some(Arc::clone(&edge.read().dest));

                        // Update incoming edge on face
                        new_faces[perim_i].write().edge = Some(Arc::clone(edge));

                        // Assign faces
                        edge.write().face_left = Some(Arc::clone(&new_faces[perim_i]));
                        edge_conjugates[perim_i].write().face_left =
                            Some(Arc::clone(&new_faces[next_perim_index]));
                    }
                    TnurccFaceSide::Right => {
                        // Assign corners
                        new_faces[perim_i].write().corners[2] =
                            Some(Arc::clone(&edge_conjugates[perim_i].read().dest));
                        new_faces[perim_i].write().corners[3] =
                            Some(Arc::clone(&edge_conjugates[perim_i].read().origin));

                        new_faces[next_perim_index].write().corners[1] =
                            Some(Arc::clone(&edge_conjugates[perim_i].read().origin));

                        // Update incoming edge on face
                        new_faces[perim_i].write().edge =
                            Some(Arc::clone(&edge_conjugates[perim_i]));

                        // Assign faces
                        edge.write().face_right =
                            Some(Arc::clone(&new_faces[next_perim_index]));
                        edge_conjugates[perim_i].write().face_right =
                            Some(Arc::clone(&new_faces[perim_i]));
                    }
                };

                // Knot interval of face connecting edge point to face point.
                // Figure 10 in \[Sederberg et al. 1998\]
                let knot_interval = {
                    // We unfourtunatly require the knot interval of the original (unsplit) previous and next edge.
                    // However, since the knot interval is evenly split, if we know an edge is split, we can just
                    // multiply the knot interval by two. Edge is split if it is out of bounds for split_edge or
                    // split_edge is true.
                    let mut multiplier = 0.25;
                    if split_edge
                        .get(perim[next_perim_index].read().index)
                        .is_none_or(|&b| b)
                    {
                        multiplier *= 2.0;
                    }

                    if split_edge
                        .get(perim[prev_perim_index].read().index)
                        .is_none_or(|&b| b)
                    {
                        multiplier *= 2.0;
                    }

                    multiplier * (perim_knots[next_perim_index] + perim_knots[prev_perim_index])
                };

                // Mutate the face point to split midpoint edge so that it is ready for connection
                {
                    let mut borrow = radial_edges[perim_i].write();
                    borrow.index = self.edges.len() + perim_i;
                    borrow.connections.fill_with(|| Some(Arc::clone(edge)));
                    borrow.face_left = Some(Arc::clone(&new_faces[next_perim_index]));
                    borrow.face_right = Some(Arc::clone(&new_faces[perim_i]));
                    // borrow.origin (already correctly assigned in constructor)
                    // borrow.dest (assigned on edge splitting)
                    borrow.knot_interval = knot_interval;
                }
            }

            for (perim_i, edge) in perim.iter().enumerate() {
                // Some helper variables to make life a little nicer
                let next_perim_index = (perim_i + 1) % perim.len();

                // Connect radial to the perimiter
                TnurccEdge::connect(Arc::clone(edge), Arc::clone(&radial_edges[perim_i]))
                    .expect("Edges around the perimeter should always succesfully connect to the radial edge");
                TnurccEdge::connect(Arc::clone(&edge_conjugates[perim_i]), Arc::clone(&radial_edges[perim_i]))
                    .expect("Edges around the perimeter should always succesfully connect to the radial edge");

                // Connect the radials between each other
                TnurccEdge::connect(
                    Arc::clone(&radial_edges[next_perim_index]),
                    Arc::clone(&radial_edges[perim_i]),
                )
                .expect("Radial edges should always succesfully connect between each other");

                // Update valence of edge point
                edge.read()
                    .common_point(Arc::clone(&edge_conjugates[perim_i]))
                    .expect("Edges should have a common point with their split conjugates")
                    .write()
                    .valence += 1;

                // This will be overidden several times, not sure exactly how to deal with it without crying...
                f_cp.write()
                    .incoming_edge
                    .replace(Arc::clone(&radial_edges[perim_i]));
                self.edges.push(Arc::clone(&radial_edges[perim_i]));
            }

            // Adding the faces into the self array is a little tricky. The first face in the new_faces arrays
            //substitutes the old face, and the rest are tacked onto the end of the faces array.
            new_faces[0].write().index = face_i;
            self.faces[face_i] = Arc::clone(&new_faces[0]);
            new_faces.into_iter().skip(1).for_each(|f| {
                f.write().index = self.faces.len();
                self.faces.push(f);
            });

            f_cp.write().valence = perim.len();
        }

        Ok(())
    }
}

impl<P> Clone for Tnurcc<P>
where
    P: Clone,
{
    fn clone(&self) -> Self {
        let mut new_control_points = Vec::new();
        let mut new_faces = Vec::new();
        let mut new_edges = Vec::new();
        

        // Clone control points (need still to set the reference edge)
        for p in self.control_points.iter() {
            let borrow = p.read();
            let mut ncp = TnurccControlPoint::new(borrow.index, borrow.point.clone());

            ncp.valence = borrow.valence;
            new_control_points.push(Arc::new(RwLock::new(ncp)));
        }

        // Clone faces (need still to set the reference edge)
        for f in self.faces.iter() {
            let borrow = f.read();

            // Transfer corners over from the control points of self to the new control points
            let corners = [
                borrow.corners[0]
                    .as_ref()
                    .map(|p| Arc::clone(&new_control_points[p.read().index])),
                borrow.corners[1]
                    .as_ref()
                    .map(|p| Arc::clone(&new_control_points[p.read().index])),
                borrow.corners[2]
                    .as_ref()
                    .map(|p| Arc::clone(&new_control_points[p.read().index])),
                borrow.corners[3]
                    .as_ref()
                    .map(|p| Arc::clone(&new_control_points[p.read().index])),
            ];
            let index = borrow.index;

            let nf = TnurccFace {
                corners,
                index,
                edge: None,
            };
            new_faces.push(Arc::new(RwLock::new(nf)));
        }

        // Clone edges (need still to set the connections)
        for e in self.edges.iter() {
            let borrow = e.read();

            let origin = Arc::clone(&new_control_points[borrow.origin.read().index]);
            let dest = Arc::clone(&new_control_points[borrow.dest.read().index]);

            let fl = borrow
                .face_left
                .as_ref()
                .map(|f| Arc::clone(&new_faces[f.read().index]));
            let fr = borrow
                .face_right
                .as_ref()
                .map(|f| Arc::clone(&new_faces[f.read().index]));

            let ne = TnurccEdge {
                index: borrow.index,
                connections: [const { None }; 4],
                face_left: fl,
                face_right: fr,
                origin: Arc::clone(&origin),
                dest: Arc::clone(&dest),
                knot_interval: borrow.knot_interval,
            };
            let ne: Arc<RwLock<TnurccEdge<P>>> = Arc::new(RwLock::new(ne));
            new_edges.push(ne);
        }

        // Set control point reference edges
        new_control_points.iter().for_each(|p| {
            let index = p.read().index;
            p.write().incoming_edge = self.control_points[index]
                .read()
                .incoming_edge
                .as_ref()
                .map(|e| Arc::clone(&new_edges[e.read().index]));
        });

        // Set face reference edges
        new_faces.iter().for_each(|f| {
            let index = f.read().index;
            f.write().edge = self.faces[index]
                .read()
                .edge
                .as_ref()
                .map(|e| Arc::clone(&new_edges[e.read().index]));
        });

        // Set edge connections
        new_edges.iter().for_each(|e| {
            let index = e.read().index;
            e.write().connections.iter_mut().enumerate().for_each(|(i, c)| {
                *c = self.edges[index]
                    .read()
                    .connections[i]
                    .as_ref()
                    .map(|map_e| Arc::clone(&new_edges[map_e.read().index]));
            });
        });

        let new_extraordinary_control_points = self
            .extraordinary_control_points
            .iter()
            .map(|p| Arc::clone(&new_control_points[p.read().index]))
            .collect::<Vec<_>>();

        Tnurcc {
            edges: new_edges,
            control_points: new_control_points,
            extraordinary_control_points: new_extraordinary_control_points,
            faces: new_faces,
        }
    }
}

impl<P> Drop for Tnurcc<P> {
    fn drop(&mut self) {
        for face in self.faces.iter() {
            face.write().corners.iter_mut().for_each(|o| *o = None);
            face.write().edge = None;
        }

        for cp in self.control_points.iter() {
            cp.write().incoming_edge = None;
        }

        for edge in self.edges.iter() {
            edge.write()
                .connections
                .iter_mut()
                .for_each(|o| *o = None);
            edge.write().face_left = None;
            edge.write().face_right = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates a T-NURCC cube with sides of lenth `1`, lower front left point at `(0, 0, 0)`,
    /// and all verticies in the first octant.
    fn make_cube() -> Result<Tnurcc<Point3>> {
        use crate::prelude::Point3;
        let points = vec![
            Point3::from((0.0, 0.0, 0.0)), // 0
            Point3::from((0.0, 0.0, 1.0)), // 1
            Point3::from((1.0, 0.0, 1.0)), // 2
            Point3::from((1.0, 0.0, 0.0)), // 3
            Point3::from((0.0, 1.0, 0.0)), // 4
            Point3::from((0.0, 1.0, 1.0)), // 5
            Point3::from((1.0, 1.0, 1.0)), // 6
            Point3::from((1.0, 1.0, 0.0)), // 7
        ];

        let faces = vec![
            [
                // Front
                (0, vec![(3, 1.0)]),
                (3, vec![(2, 1.0)]),
                (2, vec![(1, 1.0)]),
                (1, vec![(0, 1.0)]),
            ],
            [
                // Left
                (0, vec![(1, 1.0)]),
                (1, vec![(5, 1.0)]),
                (5, vec![(4, 1.0)]),
                (4, vec![(0, 1.0)]),
            ],
            [
                // Top
                (1, vec![(2, 1.0)]),
                (2, vec![(6, 1.0)]),
                (6, vec![(5, 1.0)]),
                (5, vec![(1, 1.0)]),
            ],
            [
                // Back
                (4, vec![(5, 1.0)]),
                (5, vec![(6, 1.0)]),
                (6, vec![(7, 1.0)]),
                (7, vec![(4, 1.0)]),
            ],
            [
                // Right
                (2, vec![(3, 1.0)]),
                (3, vec![(7, 1.0)]),
                (7, vec![(6, 1.0)]),
                (6, vec![(2, 1.0)]),
            ],
            [
                // Bottom
                (0, vec![(4, 1.0)]),
                (4, vec![(7, 1.0)]),
                (7, vec![(3, 1.0)]),
                (3, vec![(0, 1.0)]),
            ],
        ];

        Tnurcc::try_new(points, faces)
    }

    fn verify_tnurcc_control_points(t: &Tnurcc<Point3>) {
        for (i, p) in t.control_points.iter().enumerate() {
            // Incoming edge of the point
            let point_edge = Arc::clone(p.read().incoming_edge.as_ref().expect(&format!(
                "Point {} should have an incoming edge",
                p.read().index,
            )));

            // Point-based iter will rotate around the current control point
            // Incedentally verifies that the control point is referenced by the edge
            let iter = TnurccAcwPointIter::from_edge(
                Arc::clone(&point_edge),
                point_edge
                    .read()
                    .point_end(Arc::clone(&p))
                    .expect(&format!(
                        "Point {} should be a side of its incoming edge",
                        p.read().index,
                    )),
            );
            let next = iter.last().expect(&format!(
                "Point {} edge-rotation iterator should wrap around and end.",
                p.read().index,
            ));

            // Assert the next acw edge (from the last one returned by the iter)
            // is the same edge as the one it started at
            let next_point_end = next.read().point_end(Arc::clone(&p)).expect(&format!(
                "Edges reached through point {} iter should be connected to that point",
                p.read().index,
            ));
            let final_edge = next.read().acw_edge_from_end(next_point_end);
            assert!(
                std::ptr::eq(final_edge.as_ref(), point_edge.as_ref()),
                "Iter does not rotate around point {} correctly. Reached {}, expected {}",
                p.read().index,
                final_edge.read().index,
                point_edge.read().index,
            );

            // Calculate the anti-clockwise valence of the point and verify it matches the
            // recorded valence of the point.
            let iter = TnurccAcwPointIter::from_edge(
                Arc::clone(&point_edge),
                point_edge
                    .read()
                    .point_end(Arc::clone(&p))
                    .expect(&format!(
                        "Point {} should be a side of its incoming edge",
                        p.read().index,
                    )),
            );
            let acw_calc_valence = iter.count();
            assert!(
                acw_calc_valence == p.read().valence,
                "Point {} anti-clockwise valence {} does not match recorded valence {}",
                p.read().index,
                acw_calc_valence,
                p.read().valence,
            );

            // Check that the index field matches the index of the point
            assert!(
                p.read().index == i,
                "Point {} index field must match index in mesh points array",
                p.read().index,
            );
        }
    }

    fn verify_tnurcc_edges(t: &Tnurcc<Point3>) {
        for (i, e) in t.edges.iter().enumerate() {
            // Check index field
            assert!(
                i == e.read().index,
                "Tnurcc edge index field must be equal to edge index in edge array"
            );

            let common_faces = [
                TnurccFaceSide::Left,
                TnurccFaceSide::Left,
                TnurccFaceSide::Right,
                TnurccFaceSide::Right,
            ];

            let common_points = [
                [TnurccVertexEnd::Dest, TnurccVertexEnd::Origin],
                [TnurccVertexEnd::Origin, TnurccVertexEnd::Dest],
                [TnurccVertexEnd::Dest, TnurccVertexEnd::Origin],
                [TnurccVertexEnd::Origin, TnurccVertexEnd::Dest],
            ];

            // Check connected edges
            for (dir_index, &dir) in [
                TnurccConnection::LeftAcw,
                TnurccConnection::LeftCw,
                TnurccConnection::RightAcw,
                TnurccConnection::RightCw,
            ]
            .iter()
            .enumerate()
            {
                // Get edge in the direction under investigation
                let con = e.read().connection(dir);

                // Check the face between the two is the same and correct
                let common_face = e
                    .read()
                    .common_face(Arc::clone(&con))
                    .expect("Connected edges must have a common face between them");
                assert!(std::ptr::eq(
                    common_face.as_ref(),
                    e.read()
                        .face_from_side(common_faces[dir_index])
                        .expect("Tnurcc must be closed on all edges")
                        .as_ref()
                ));

                // Check that the point between them is the same and correct
                let common_point = e
                    .read()
                    .common_point(Arc::clone(&con))
                    .expect("Connected edges must have a common point between them");

                // In order to check to make sure that the common points is the correct one,
                // both the connection and orientation of the connected edge relative to the
                // common face needs to be computed in order to know what the relative
                // orientation of the two edges is to each other.
                let other_common_point = con.read().point_at_end(
                    common_points[dir as usize][con
                        .read()
                        .face_side(Arc::clone(&common_face))
                        .expect("Common face must be a side on con")
                        as usize],
                );

                assert!(
                    std::ptr::eq(common_point.as_ref(), other_common_point.as_ref()),
                    "Connected edges {} and {} do not share the correct point.",
                    e.read().index,
                    con.read().index
                );
            }
        }
    }

    fn verify_tnurcc_faces(t: &Tnurcc<Point3>) {
        for face in t.faces.iter() {
            // Get reference edge for face
            let face_edge = Arc::clone(
                face.read()
                    .edge
                    .as_ref()
                    .expect("All faces should have a reference edge in T-NURCC"),
            );

            // Assert the next acw edge (from the last one returned by the iter)
            // is the same edge as the one it started at
            let last_edge = TnurccAcwFaceIter::try_from_edge(
                Arc::clone(&face_edge),
                face_edge.read().face_side(Arc::clone(face)).unwrap(),
            )
            .expect("Prevously tested assertion")
            .last()
            .expect("Iter of size greater than 0 should have a last element");

            // Assert that the face is closed (The next edge around the face after exhausting the iterator
            // should be the original reference edge)
            let next_face_side = last_edge
                .read()
                .face_side(Arc::clone(&face))
                .expect("Edges reached through a face iter should be connected to that face");
            let final_edge = last_edge.read().acw_edge_from_side(next_face_side);
            assert!(
                std::ptr::eq(final_edge.as_ref(), face_edge.as_ref()),
                "Iter does not rotate around face correctly. Reached {}, expected {}",
                final_edge.read().index,
                face_edge.read().index,
            );
        }
    }

    #[test]
    fn t_nurcc_test_make_cube_euclidiean_geometry() {
        // Sanity check that the cube is (probably) actually a cube

        let surface = make_cube();
        assert!(
            surface.is_ok(),
            "Surface was unsuccesfully created with error: {}.",
            surface.err().unwrap()
        );
        let surface = make_cube().unwrap();

        assert_eq!(surface.faces.len(), 6, "Cube does not contain 6 faces.");
        assert_eq!(
            surface.control_points.len(),
            8,
            "Cube does not contain 8 verticies."
        );
        assert_eq!(surface.edges.len(), 12, "Cube does not contain 12 edges.");
    }

    #[test]
    fn t_nurcc_test_cube_control_point_properties() {
        let t = make_cube().expect("Cube should be succesfully created");

        verify_tnurcc_control_points(&t);

        for p in t.control_points.iter() {
            // Check valencies
            assert_eq!(
                p.read().valence,
                3,
                "Point {} does not have a valence of 3.",
                p.read().index,
            );
        }
    }

    #[test]
    fn t_nurcc_test_cube_edge_properties() {
        let t = make_cube().expect("Cube should be succesfully created");

        verify_tnurcc_edges(&t);
    }

    #[test]
    fn t_nurcc_test_cube_face_properties() {
        let surface = make_cube().unwrap();

        verify_tnurcc_faces(&surface);

        for face in surface.faces.iter() {
            let face_edge = Arc::clone(
                face.read()
                    .edge
                    .as_ref()
                    .expect("All faces should have a reference edge in T-NURCC"),
            );

            // Assert that each face has four edges
            let edge_count = TnurccAcwFaceIter::try_from_edge(
                Arc::clone(&face_edge),
                face_edge.read().face_side(Arc::clone(face)).unwrap(),
            )
            .expect("face_edge should have Some(face) because it was cloned from face")
            .count();

            assert!(
                edge_count == 4,
                "Rectangular faces should have 4 faces to rotate around"
            );
        }
    }

    fn t_nurcc_subdivded_cube() -> Tnurcc<Point3> {
        let mut surface = make_cube().unwrap();
        surface
            .global_subdivide()
            .expect("Subdivision of cube is possible");
        surface
    }

    #[test]
    fn t_nurcc_test_subdivide_euclidean_geometry() {
        let surface = t_nurcc_subdivded_cube();

        // Check basic geometric properties
        assert_eq!(
            surface.faces.len(),
            6 * 4,
            "Number of faces after subdivide should be 4 times the original quantity"
        );
        assert_eq!(
            surface.control_points.len(),
            (8 + 12 + 6),
            "Number of points after subdivide should be the sum of points, edges, and faces prior to subdividing");
        assert_eq!(
            surface.edges.len(),
            (12 * 2 + 4 * 6),
            "Number of edges after subdivide should be the sum of twice the count of edges prior subdividing and the sum of the number of edges on each face for each face");
    }

    #[test]
    fn t_nurcc_test_subdivide_edges() {
        let surface = t_nurcc_subdivded_cube();

        verify_tnurcc_edges(&surface);
    }

    #[test]
    fn t_nurcc_test_subdivide_faces() {
        let surface = t_nurcc_subdivded_cube();

        verify_tnurcc_faces(&surface);

        // Make sure the faces are well formed (a little redundant but better be thorough)
        surface.faces.iter().for_each(|f| {
            let start_edge = Arc::clone(
                f.read()
                    .edge
                    .as_ref()
                    .expect("All faces should have an edge"),
            );

            // Anticlockwise traversal
            let mut acw_traverse_edge = Arc::clone(&start_edge);
            // Each face is 4 sided
            for i in 0..4 {
                acw_traverse_edge = {
                    let side = acw_traverse_edge
                        .read()
                        .face_side(Arc::clone(f))
                        .expect(format!("Face should be connected to reference edge, error on ACW traversal {} face {}", i, f.read().index).as_str());
                    match side {
                        TnurccFaceSide::Left => acw_traverse_edge
                            .read()
                            .connection(TnurccConnection::LeftAcw),
                        TnurccFaceSide::Right => acw_traverse_edge
                            .read()
                            .connection(TnurccConnection::RightAcw),
                    }
                };
            }

            // Clockwise traversal
            let mut cw_traverse_edge = Arc::clone(&start_edge);
            // Each face is 4 sided
            for i in 0..4 {
                cw_traverse_edge = {
                    let side = cw_traverse_edge
                        .read()
                        .face_side(Arc::clone(f))
                        .expect(format!("Face should be connected to reference edge, error on CW traversal {} face {}", i, f.read().index).as_str());
                    match side {
                        TnurccFaceSide::Left => cw_traverse_edge
                            .read()
                            .connection(TnurccConnection::LeftCw),
                        TnurccFaceSide::Right => cw_traverse_edge
                            .read()
                            .connection(TnurccConnection::RightCw),
                    }
                };
            }

            assert!(
                std::ptr::eq(start_edge.as_ref(), acw_traverse_edge.as_ref()),
                "Anticlockwise traversal around face index {} did not return to the start edge.",
                f.read().index
            );

            assert!(
                std::ptr::eq(start_edge.as_ref(), cw_traverse_edge.as_ref()),
                "Clockwise traversal around face index {} did not return to the start edge.",
                f.read().index
            );
        });
    }

    #[test]
    fn t_nurcc_test_subdivide_points() {
        let surface = t_nurcc_subdivded_cube();

        verify_tnurcc_control_points(&surface);
    }

    #[test]
    fn t_nurcc_test_double_subdivide() {
        let mut surface = t_nurcc_subdivded_cube();
        surface
            .global_subdivide()
            .expect("Double subdivide should succeed");
        verify_tnurcc_control_points(&surface);
        verify_tnurcc_edges(&surface);
        verify_tnurcc_faces(&surface);
    }

    #[test]
    fn t_nurcc_test_clone() {
        use std::mem::drop;
        let mut surface;
        {
            let clone = make_cube().unwrap();
            surface = clone.clone();
            drop(clone);
        }

        surface
            .global_subdivide()
            .expect("Cloned subdivide should succeed");

        surface
            .global_subdivide()
            .expect("Cloned double subdivide should succeed");
        
        verify_tnurcc_control_points(&surface);
        verify_tnurcc_edges(&surface);
        verify_tnurcc_faces(&surface);
    }
}
