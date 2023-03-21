use crate::*;
pub use vtkio;
use vtkio::model::{Attributes, *};

/// Trait for convert to [`DataSet`]
pub trait ToDataSet {
    /// Converts to [`DataSet`]
    /// # Remarks
    /// The `DataSet` is generated assuming output in XML format.
    fn to_data_set(&self) -> DataSet;
}

impl ToDataSet for PolygonMesh<usize, Vec<Point3>> {
    fn to_data_set(&self) -> DataSet {
        let flatten_points = self
            .attributes()
            .iter()
            .flat_map(|p| AsRef::<[f64; 3]>::as_ref(p))
            .copied()
            .collect::<Vec<_>>();
        let points = IOBuffer::F64(flatten_points);
        let connectivity = self
            .face_iter()
            .flat_map(|face| face.iter().map(|idx| *idx as u64))
            .collect::<Vec<_>>();
        let mut offset: u64 = 0;
        let offsets = self
            .face_iter()
            .map(|face| {
                offset += face.len() as u64;
                offset
            })
            .collect::<Vec<_>>();
        let polys = Some(VertexNumbers::XML {
            connectivity,
            offsets,
        });
        DataSet::PolyData {
            meta: None,
            pieces: vec![Piece::Inline(Box::new(PolyDataPiece {
                points,
                polys,
                ..Default::default()
            }))],
        }
    }
}

impl ToDataSet for PolygonMesh<usize, Vec<(Point3, Vector2)>> {
    fn to_data_set(&self) -> DataSet {
        let flatten_points = self
            .attributes()
            .iter()
            .flat_map(|(p, _)| AsRef::<[f64; 3]>::as_ref(p))
            .copied()
            .collect::<Vec<_>>();
        let points = IOBuffer::F64(flatten_points);
        let flatten_uvs = self
            .attributes()
            .iter()
            .flat_map(|(_, uv)| AsRef::<[f64; 2]>::as_ref(uv))
            .copied()
            .collect::<Vec<_>>();
        let uvs = Attribute::DataArray(DataArray {
            name: "TCoords".to_owned(),
            elem: ElementType::TCoords(2),
            data: IOBuffer::F64(flatten_uvs),
        });
        let connectivity = self
            .face_iter()
            .flat_map(|face| face.iter().map(|idx| *idx as u64))
            .collect::<Vec<_>>();
        let mut offset: u64 = 0;
        let offsets = self
            .face_iter()
            .map(|face| {
                offset += face.len() as u64;
                offset
            })
            .collect::<Vec<_>>();
        let polys = Some(VertexNumbers::XML {
            connectivity,
            offsets,
        });
        DataSet::PolyData {
            meta: None,
            pieces: vec![Piece::Inline(Box::new(PolyDataPiece {
                points,
                polys,
                data: Attributes {
                    point: vec![uvs],
                    ..Default::default()
                },
                ..Default::default()
            }))],
        }
    }
}

impl ToDataSet for PolygonMesh<usize, Vec<(Point3, Vector3)>> {
    fn to_data_set(&self) -> DataSet {
        let flatten_points = self
            .attributes()
            .iter()
            .flat_map(|(p, _)| AsRef::<[f64; 3]>::as_ref(p))
            .copied()
            .collect::<Vec<_>>();
        let points = IOBuffer::F64(flatten_points);
        let flatten_normals = self
            .attributes()
            .iter()
            .flat_map(|(_, n)| AsRef::<[f64; 3]>::as_ref(n))
            .copied()
            .collect::<Vec<_>>();
        let normals = Attribute::DataArray(DataArray {
            name: "Normals".to_owned(),
            elem: ElementType::Normals,
            data: IOBuffer::F64(flatten_normals),
        });
        let connectivity = self
            .face_iter()
            .flat_map(|face| face.iter().map(|idx| *idx as u64))
            .collect::<Vec<_>>();
        let mut offset: u64 = 0;
        let offsets = self
            .face_iter()
            .map(|face| {
                offset += face.len() as u64;
                offset
            })
            .collect::<Vec<_>>();
        let polys = Some(VertexNumbers::XML {
            connectivity,
            offsets,
        });
        DataSet::PolyData {
            meta: None,
            pieces: vec![Piece::Inline(Box::new(PolyDataPiece {
                points,
                polys,
                data: Attributes {
                    point: vec![normals],
                    ..Default::default()
                },
                ..Default::default()
            }))],
        }
    }
}

impl ToDataSet for PolygonMesh<usize, Vec<StandardAttribute>> {
    fn to_data_set(&self) -> DataSet {
        const NAN2: [f64; 2] = [f64::NAN; 2];
        const NAN3: [f64; 3] = [f64::NAN; 3];
        let flatten_points = self
            .attributes()
            .iter()
            .flat_map(|attr| AsRef::<[f64; 3]>::as_ref(&attr.position))
            .copied()
            .collect::<Vec<_>>();
        let points = IOBuffer::F64(flatten_points);
        let flatten_uvs = self
            .attributes()
            .iter()
            .flat_map(|attr| match &attr.uv_coord {
                Some(uv) => AsRef::<[f64; 2]>::as_ref(uv),
                None => &NAN2,
            })
            .copied()
            .collect::<Vec<_>>();
        let uvs = Attribute::DataArray(DataArray {
            name: "TCoords".to_owned(),
            elem: ElementType::TCoords(2),
            data: IOBuffer::F64(flatten_uvs),
        });
        let flatten_normals = self
            .attributes()
            .iter()
            .flat_map(|attr| match &attr.normal {
                Some(normal) => AsRef::<[f64; 3]>::as_ref(normal),
                None => &NAN3,
            })
            .copied()
            .collect::<Vec<_>>();
        let normals = Attribute::DataArray(DataArray {
            name: "Normals".to_owned(),
            elem: ElementType::Normals,
            data: IOBuffer::F64(flatten_normals),
        });
        let connectivity = self
            .face_iter()
            .flat_map(|face| face.iter().map(|idx| *idx as u64))
            .collect::<Vec<_>>();
        let mut offset: u64 = 0;
        let offsets = self
            .face_iter()
            .map(|face| {
                offset += face.len() as u64;
                offset
            })
            .collect::<Vec<_>>();
        let polys = Some(VertexNumbers::XML {
            connectivity,
            offsets,
        });
        DataSet::PolyData {
            meta: None,
            pieces: vec![Piece::Inline(Box::new(PolyDataPiece {
                points,
                polys,
                data: Attributes {
                    point: vec![uvs, normals],
                    ..Default::default()
                },
                ..Default::default()
            }))],
        }
    }
}

impl ToDataSet for PolygonMesh {
    fn to_data_set(&self) -> DataSet {
        let attrs = self.attributes();
        match (attrs.uv_coords().is_empty(), attrs.normals().is_empty()) {
            (true, true) => self.to_positions_mesh().to_data_set(),
            (false, true) => self
                .expands(|attr| {
                    let uv = match attr.uv_coord {
                        Some(uv) => uv,
                        None => Vector2::new(f64::NAN, f64::NAN),
                    };
                    (attr.position, uv)
                })
                .to_data_set(),
            (true, false) => self
                .expands(|attr| {
                    let normal = match attr.normal {
                        Some(normal) => normal,
                        None => Vector3::new(f64::NAN, f64::NAN, f64::NAN),
                    };
                    (attr.position, normal)
                })
                .to_data_set(),
            (false, false) => self.expands(std::convert::identity).to_data_set(),
        }
    }
}
