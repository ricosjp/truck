use super::*;

impl Table {
    fn place_holder_edge_any_to_index_and_edge_curve(
        &self,
        edge: &PlaceHolder<EdgeAnyHolder>,
    ) -> Option<(u64, EdgeCurveHolder)> {
        use PlaceHolder::Ref;
        let Ref(Name::Entity(ref idx)) = edge else {
            return None;
        };
        self.oriented_edge
            .get(idx)
            .and_then(|oriented_edge| {
                Some((
                    oriented_edge.edge_element_idx()?,
                    oriented_edge.edge_element_holder(self)?,
                ))
            })
            .or_else(|| {
                self.edge_curve
                    .get(idx)
                    .map(|edge_curve| (*idx, edge_curve.clone()))
            })
    }
    fn face_any_to_orientation_and_face(
        &self,
        face: Option<FaceAnyHolder>,
    ) -> Option<(bool, FaceSurfaceHolder)> {
        match face? {
            FaceAnyHolder::FaceSurface(face) => Some((true, face)),
            FaceAnyHolder::OrientedFace(oriented_face) => {
                let face_element = oriented_face.face_element_holder(self)?;
                Some((oriented_face.orientation, face_element))
            }
        }
    }

    fn shell_vertices(&self, shell: &ShellHolder) -> (Vec<Point3>, HashMap<u64, usize>) {
        use PlaceHolder::Ref;
        let mut vidx_map = HashMap::<u64, usize>::new();
        let vertex_to_point = |v: PlaceHolder<VertexPointHolder>| {
            if let Ref(Name::Entity(ref idx)) = v {
                if !vidx_map.contains_key(idx) {
                    let len = vidx_map.len();
                    vidx_map.insert(*idx, len);
                    let p = EntityTable::<VertexPointHolder>::get_owned(self, *idx)
                        .map_err(|e| eprintln!("{e}"))
                        .ok()?;
                    return Some(Point3::from(&p.vertex_geometry));
                }
            }
            None
        };
        let vertices: Vec<Point3> = shell
            .cfs_faces_holder(self)
            .filter_map(move |face| self.face_any_to_orientation_and_face(face))
            .flat_map(move |(_, face)| face.bounds_holder(self))
            .filter_map(move |bound| bound?.bound_holder(self))
            .flat_map(move |bound| bound.edge_list)
            .filter_map(move |edge| self.place_holder_edge_any_to_index_and_edge_curve(&edge))
            .flat_map(move |(_, edge)| [edge.edge_start, edge.edge_end])
            .filter_map(vertex_to_point)
            .collect();
        (vertices, vidx_map)
    }

    fn shell_edges(
        &self,
        shell: &ShellHolder,
        vidx_map: &HashMap<u64, usize>,
    ) -> (Vec<CompressedEdge<Curve3D>>, HashMap<u64, usize>) {
        use PlaceHolder::Ref;
        let mut eidx_map = HashMap::<u64, usize>::new();
        let edge_curve_to_compressed_edge = |(idx, edge): (u64, EdgeCurveHolder)| {
            if eidx_map.contains_key(&idx) {
                return None;
            }
            let len = eidx_map.len();
            eidx_map.insert(idx, len);
            let edge_curve = edge
                .clone()
                .into_owned(self)
                .map_err(|e| eprintln!("{e}"))
                .ok()?;
            let curve = edge_curve
                .parse_curve3d()
                .map_err(|e| eprintln!("{e}"))
                .ok()?;
            let Ref(Name::Entity(front_idx)) = edge.edge_start else {
                return None;
            };
            let Ref(Name::Entity(back_idx)) = edge.edge_end else {
                return None;
            };
            Some(CompressedEdge {
                vertices: (*vidx_map.get(&front_idx)?, *vidx_map.get(&back_idx)?),
                curve,
            })
        };
        let edges: Vec<CompressedEdge<Curve3D>> = shell
            .cfs_faces_holder(self)
            .filter_map(move |face| self.face_any_to_orientation_and_face(face))
            .flat_map(move |(_, face)| face.bounds_holder(self))
            .filter_map(move |bound| bound?.bound_holder(self))
            .flat_map(move |bound| bound.edge_list)
            .filter_map(move |edge| self.place_holder_edge_any_to_index_and_edge_curve(&edge))
            .filter_map(edge_curve_to_compressed_edge)
            .collect();
        (edges, eidx_map)
    }
    fn face_bound_to_edges(
        &self,
        bound: FaceBoundHolder,
        eidx_map: &HashMap<u64, usize>,
    ) -> Option<Vec<CompressedEdgeIndex>> {
        use PlaceHolder::Ref;
        let ori = bound.orientation;
        let bound = bound.bound_holder(self)?;
        let mut edges: Vec<CompressedEdgeIndex> = bound
            .edge_list
            .into_iter()
            .filter_map(|edge| {
                let Ref(Name::Entity(ref idx)) = edge else {
                    return None;
                };
                let edge_idx = if let Some(oriented_edge) = self.oriented_edge.get(idx) {
                    CompressedEdgeIndex {
                        index: *eidx_map.get(&oriented_edge.edge_element_idx()?)?,
                        orientation: oriented_edge.orientation == ori,
                    }
                } else {
                    CompressedEdgeIndex {
                        index: *eidx_map.get(idx)?,
                        orientation: ori,
                    }
                };
                Some(edge_idx)
            })
            .collect();
        if !ori {
            edges.reverse();
        }
        Some(edges)
    }

    fn shell_faces(
        &self,
        shell: &ShellHolder,
        eidx_map: &HashMap<u64, usize>,
    ) -> Vec<CompressedFace<Surface>> {
        shell
            .cfs_faces_holder(self)
            .filter_map(|face| self.face_any_to_orientation_and_face(face))
            .filter_map(|(orientation, face)| {
                let step_surface: SurfaceAny = face
                    .face_geometry
                    .clone()
                    .into_owned(self)
                    .map_err(|e| eprintln!("{e}"))
                    .ok()?;
                let mut surface = Surface::try_from(&step_surface)
                    .map_err(|e| eprintln!("{e}"))
                    .ok()?;
                if !face.same_sense {
                    surface.invert()
                }
                let boundaries: Vec<_> = face
                    .bounds_holder(self)
                    .into_iter()
                    .filter_map(|bound| self.face_bound_to_edges(bound?, eidx_map))
                    .collect();
                Some(CompressedFace {
                    surface,
                    boundaries,
                    orientation,
                })
            })
            .collect()
    }

    /// Constructs `CompressedShell` of `truck` from `Shell` in STEP file
    /// # Example
    /// ```
    /// use truck_stepio::r#in::{*, step_geometry::*};
    /// // read file
    /// let step_string = include_str!(concat!(
    ///     env!("CARGO_MANIFEST_DIR"),
    ///     "/../resources/step/occt-cube.step",
    /// ));
    /// // parse into Rust structs
    /// let table = Table::from_step(&step_string).unwrap();
    /// // take one shell (this is only one shell)
    /// let step_shell = table.shell.values().next().unwrap();
    /// // convert STEP shell to `CompressedShell`
    /// let cshell = table.to_compressed_shell(step_shell).unwrap();
    /// // The cube has 6 faces!
    /// assert_eq!(cshell.faces.len(), 6);
    /// ```
    pub fn to_compressed_shell(
        &self,
        shell: &impl StepShell,
    ) -> Result<CompressedShell<Point3, Curve3D, Surface>, StepConvertingError> {
        shell.to_compressed_shell(self)
    }

    /// Constructs `CompressedShell`s of `truck` from `ShellBasedSurfaceModel` in STEP file
    pub fn to_compressed_shells(
        &self,
        shells: &ShellBasedSurfaceModelHolder,
    ) -> Result<Vec<CompressedShell<Point3, Curve3D, Surface>>, StepConvertingError> {
        let mut res = Vec::new();
        for place_holder in &shells.sbsm_boundary {
            let PlaceHolder::Ref(Name::Entity(idx)) = place_holder else {
                return Err("failed to reference an element of `sbsm_boundary`".into());
            };
            if let Some(shell) = self.shell.get(idx) {
                res.push(self.to_compressed_shell(shell)?);
            } else if let Some(oriented_shell) = self.oriented_shell.get(idx) {
                res.push(self.to_compressed_shell(oriented_shell)?);
            } else {
                return Err("failed to reference an element of `sbsm_boundary`".into());
            }
        }
        Ok(res)
    }

    /// Constructs `CompressedSolid` of `truck` from `ManifoldSolidBrep` in STEP file
    /// # Example
    /// ```
    /// use truck_stepio::r#in::{*, step_geometry::*};
    /// truck_topology::prelude!(Point3, Curve3D, Surface);
    /// // read file
    /// let step_string = include_str!(concat!(
    ///     env!("CARGO_MANIFEST_DIR"),
    ///     "/../resources/step/occt-cube.step",
    /// ));
    /// // parse into Rust structs
    /// let table = Table::from_step(&step_string).unwrap();
    /// // take the solid
    /// let step_solid = table.manifold_solid_brep.values().next().unwrap();
    /// // convert STEP shell to `CompressedSolid`
    /// let csolid = table.to_compressed_solid(step_solid).unwrap();
    /// // Convert to truck `Solid`
    /// let solid = Solid::extract(csolid).unwrap();
    /// // The cube has 6 faces!
    /// assert_eq!(solid.boundaries()[0].len(), 6);
    /// ```
    pub fn to_compressed_solid(
        &self,
        solid: &ManifoldSolidBrepHolder,
    ) -> Result<CompressedSolid<Point3, Curve3D, Surface>, StepConvertingError> {
        let PlaceHolder::Ref(Name::Entity(outer_idx)) = &solid.outer else {
            return Err("failed to reference `solid.outer`".into());
        };
        let outer_shell = if let Some(step_shell) = self.shell.get(outer_idx) {
            self.to_compressed_shell(step_shell)
        } else if let Some(step_shell) = self.oriented_shell.get(outer_idx) {
            self.to_compressed_shell(step_shell)
        } else {
            Err("failed to reference `solid.outer`".into())
        }?;
        let mut boundaries = vec![outer_shell];
        for shell in &solid.voids {
            let PlaceHolder::Ref(Name::Entity(outer_idx)) = shell else {
                return Err("failed to reference an element of `solid.voids`".into());
            };
            let Some(oriented_shell) = self.oriented_shell.get(outer_idx) else {
                return Err("failed to reference an element of `solid.voids`".into());
            };
            boundaries.push(self.to_compressed_shell(oriented_shell)?);
        }
        Ok(CompressedSolid { boundaries })
    }

    fn product_node_entity(
        &self,
        pds_idx: u64,
        pd: &ProductDefinitionHolder,
    ) -> Result<ProductEntity, StepConvertingError> {
        let PlaceHolder::Ref(Name::Entity(pdf_idx)) = &pd.formation else {
            return Err("failed to reference `product_definition.formation`".into());
        };
        let Some(pdf) = self.product_definition_formation.get(pdf_idx) else {
            return Err("failed to reference `prouct_definition_formation`".into());
        };
        let PlaceHolder::Ref(Name::Entity(p_idx)) = &pdf.of_product else {
            return Err("failed to reference `product_definition_formation.of_product`".into());
        };
        let Some(product) = self.product.get(p_idx) else {
            return Err("failed to reference `product`".into());
        };
        let name = product.name.clone();

        let Some(sdr) = self.shape_definition_representation.values().find(|sdr| {
            let &PlaceHolder::Ref(Name::Entity(idx)) = &sdr.definition else {
                return false;
            };
            pds_idx == idx
        }) else {
            return Err("failed to find `shape_definition_representation` corresp. to `product_definition_shape`".into());
        };
        let PlaceHolder::Ref(Name::Entity(sr_idx)) = &sdr.used_representation else {
            return Err(
                "failed to reference `shape_definition_representation.used_representation`".into(),
            );
        };
        let Some(sr) = self.shape_representation.get(sr_idx) else {
            return Err("failed to reference `shape_representation`".into());
        };
        let Some(shapes) = sr
            .items
            .iter()
            .map(|place_holder| {
                if let &PlaceHolder::Ref(Name::Entity(item_idx)) = place_holder {
                    Some(item_idx)
                } else {
                    None
                }
            })
            .collect::<Option<Vec<_>>>()
        else {
            return Err("failed to reference an element of `shape_representation.items`".into());
        };

        Ok(ProductEntity {
            matrix: NodeMatrix::Identity,
            shapes,
            attrs: name,
        })
    }

    fn assy_node_entity(
        &self,
        pds_idx: u64,
        next_assy: &NextAssemblyUsageOccurrenceHolder,
    ) -> Result<(ProductEntity, (u64, u64)), StepConvertingError> {
        let &PlaceHolder::Ref(Name::Entity(parent_idx)) = &next_assy.relating_product_definition
        else {
            return Err("failed to reference the parent node".into());
        };
        let &PlaceHolder::Ref(Name::Entity(child_idx)) = &next_assy.related_product_definition
        else {
            return Err("failed to reference the child node".into());
        };

        let name = next_assy.name.clone();

        let Some(cdsr) = self
            .context_dependent_shape_representation
            .values()
            .find(|cdsr| {
                let &PlaceHolder::Ref(Name::Entity(idx)) = &cdsr.represented_product_relation
                else {
                    return false;
                };
                pds_idx == idx
            })
        else {
            return Err("".into());
        };

        let PlaceHolder::Ref(Name::Entity(srrwt_idx)) = &cdsr.representation_relation else {
            return Err("failed to reference `context_dependent_shape_representation.representation_relation`".into());
        };

        let Some(srrwt) = self
            .shape_representation_relationship_with_transformation
            .get(srrwt_idx)
        else {
            return Err("failed to reference `shape_representation_relationship`".into());
        };
        let idtf = srrwt.transformation_operator.clone().into_owned(self)?;

        let entity = ProductEntity {
            matrix: NodeMatrix::Transform(idtf.into()),
            shapes: Vec::new(),
            attrs: name,
        };

        Ok((entity, (parent_idx, child_idx)))
    }

    pub fn step_assy<'a>(&self, assy: &'a StepAssembly<'a>) -> Result<(), StepConvertingError> {
        let mut product_nodes = HashMap::<u64, ProductEntity>::new();
        let mut assy_nodes = Vec::<(ProductEntity, (u64, u64))>::new();
        for (&pds_idx, pds) in &self.product_definition_shape {
            let &PlaceHolder::Ref(Name::Entity(idx)) = &pds.definition else {
                return Err("failed to reference `product_definition_shape.definition`".into());
            };
            if let Some(pd) = self.product_definition.get(&idx) {
                product_nodes.insert(idx, self.product_node_entity(pds_idx, pd)?);
            } else if let Some(next_assy) = self.next_assembly_usage_occurrence.get(&idx) {
                assy_nodes.push(self.assy_node_entity(pds_idx, next_assy)?);
            }
        }

        let node_map: HashMap<_, _> = product_nodes
            .into_iter()
            .map(|(idx, entity)| (idx, assy.create_node(entity)))
            .collect();
        for (entity, (parent_idx, child_idx)) in assy_nodes {
            let node = assy.create_node(entity);
            let Some(&parent) = node_map.get(&parent_idx) else {
                return Err("assembly structure is invalid.".into());
            };
            node.add_parent(parent);
            let Some(&child) = node_map.get(&child_idx) else {
                return Err("assembly structure is invalid.".into());
            };
            node.add_child(child);
        }

        Ok(())
    }
}

pub trait StepShell {
    fn to_compressed_shell(
        &self,
        table: &Table,
    ) -> Result<CompressedShell<Point3, Curve3D, Surface>, StepConvertingError>;
}

impl StepShell for ShellHolder {
    fn to_compressed_shell(
        &self,
        table: &Table,
    ) -> Result<CompressedShell<Point3, Curve3D, Surface>, StepConvertingError> {
        let (vertices, vidx_map) = table.shell_vertices(self);
        let (edges, eidx_map) = table.shell_edges(self, &vidx_map);
        Ok(CompressedShell {
            vertices,
            edges,
            faces: table.shell_faces(self, &eidx_map),
        })
    }
}

impl StepShell for OrientedShellHolder {
    fn to_compressed_shell(
        &self,
        table: &Table,
    ) -> Result<CompressedShell<Point3, Curve3D, Surface>, StepConvertingError> {
        let PlaceHolder::Ref(Name::Entity(idx)) = &self.shell_element else {
            return Err("failed to reference shell".into());
        };
        let Some(shell) = table.shell.get(idx) else {
            return Err("failed to reference shell".into());
        };
        let mut res = shell.to_compressed_shell(table)?;
        if !self.orientation {
            for face in &mut res.faces {
                face.orientation = !face.orientation;
            }
        }
        Ok(res)
    }
}

impl StepShell for ShellAnyHolder {
    fn to_compressed_shell(
        &self,
        table: &Table,
    ) -> Result<CompressedShell<Point3, Curve3D, Surface>, StepConvertingError> {
        match self {
            ShellAnyHolder::OrientedShell(shell) => shell.to_compressed_shell(table),
            ShellAnyHolder::Shell(shell) => shell.to_compressed_shell(table),
        }
    }
}
