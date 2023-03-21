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

fn to_vertex_numbers(faces: &Faces<usize>) -> VertexNumbers {
    let connectivity = faces
        .face_iter()
        .flat_map(|face| face.iter().map(|idx| *idx as u64))
        .collect::<Vec<_>>();
    let mut offset: u64 = 0;
    let offsets = faces
        .face_iter()
        .map(|face| {
            offset += face.len() as u64;
            offset
        })
        .collect::<Vec<_>>();
    VertexNumbers::XML {
        connectivity,
        offsets,
    }
}

impl ToDataSet for PolygonMesh<usize, Vec<Point3>> {
    fn to_data_set(&self) -> DataSet {
        let flatten_points = self
            .attributes()
            .iter()
            .flat_map(|p| Into::<[f64; 3]>::into(*p))
            .collect::<Vec<_>>();
        DataSet::PolyData {
            meta: None,
            pieces: vec![Piece::Inline(Box::new(PolyDataPiece {
                points: IOBuffer::F64(flatten_points),
                polys: Some(to_vertex_numbers(self.faces())),
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
            .flat_map(|(p, _)| Into::<[f64; 3]>::into(*p))
            .collect::<Vec<_>>();
        let flatten_uvs = self
            .attributes()
            .iter()
            .flat_map(|(_, uv)| Into::<[f64; 2]>::into(*uv))
            .collect::<Vec<_>>();
        let uvs = Attribute::DataArray(DataArray {
            name: "TCoords".to_owned(),
            elem: ElementType::TCoords(2),
            data: IOBuffer::F64(flatten_uvs),
        });
        DataSet::PolyData {
            meta: None,
            pieces: vec![Piece::Inline(Box::new(PolyDataPiece {
                points: IOBuffer::F64(flatten_points),
                polys: Some(to_vertex_numbers(self.faces())),
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
            .flat_map(|(p, _)| Into::<[f64; 3]>::into(*p))
            .collect::<Vec<_>>();
        let flatten_normals = self
            .attributes()
            .iter()
            .flat_map(|(_, n)| Into::<[f64; 3]>::into(*n))
            .collect::<Vec<_>>();
        let normals = Attribute::DataArray(DataArray {
            name: "Normals".to_owned(),
            elem: ElementType::Normals,
            data: IOBuffer::F64(flatten_normals),
        });
        DataSet::PolyData {
            meta: None,
            pieces: vec![Piece::Inline(Box::new(PolyDataPiece {
                points: IOBuffer::F64(flatten_points),
                polys: Some(to_vertex_numbers(self.faces())),
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
        let flatten_points = self
            .attributes()
            .iter()
            .flat_map(|attr| Into::<[f64; 3]>::into(attr.position))
            .collect::<Vec<_>>();
        let flatten_uvs = self
            .attributes()
            .iter()
            .flat_map(|attr| match attr.uv_coord {
                Some(uv) => Into::<[f64; 2]>::into(uv),
                None => [f64::NAN; 2],
            })
            .collect::<Vec<_>>();
        let uvs = Attribute::DataArray(DataArray {
            name: "TCoords".to_owned(),
            elem: ElementType::TCoords(2),
            data: IOBuffer::F64(flatten_uvs),
        });
        let flatten_normals = self
            .attributes()
            .iter()
            .flat_map(|attr| match attr.normal {
                Some(normal) => Into::<[f64; 3]>::into(normal),
                None => [f64::NAN; 3],
            })
            .collect::<Vec<_>>();
        let normals = Attribute::DataArray(DataArray {
            name: "Normals".to_owned(),
            elem: ElementType::Normals,
            data: IOBuffer::F64(flatten_normals),
        });
        DataSet::PolyData {
            meta: None,
            pieces: vec![Piece::Inline(Box::new(PolyDataPiece {
                points: IOBuffer::F64(flatten_points),
                polys: Some(to_vertex_numbers(self.faces())),
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
