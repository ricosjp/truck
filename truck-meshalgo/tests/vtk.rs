#![cfg(not(target_arch = "wasm32"))]
use truck_meshalgo::prelude::*;
use truck_topology::*;
use vtkio::model::*;

fn truck_simple_cube() -> PolygonMesh {
    let positions = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
    ];
    let faces: Faces = vec![
        [0, 2, 3].as_slice(),
        &[0, 3, 1],
        &[0, 1, 5, 4],
        &[1, 3, 7, 5],
        &[3, 2, 6, 7],
        &[2, 0, 4, 6],
        &[4, 5, 7, 6],
    ]
    .into_iter()
    .collect();
    PolygonMesh::new(
        StandardAttributes {
            positions,
            ..Default::default()
        },
        faces,
    )
}

fn vtk_simple_cube() -> DataSet {
    #[rustfmt::skip]
    let points = IOBuffer::F64(vec![
        0.0, 0.0, 0.0,
        1.0, 0.0, 0.0,
        0.0, 1.0, 0.0,
        1.0, 1.0, 0.0,
        0.0, 0.0, 1.0,
        1.0, 0.0, 1.0,
        0.0, 1.0, 1.0,
        1.0, 1.0, 1.0,
    ]);
    let vertex_numbers = VertexNumbers::XML {
        #[rustfmt::skip]
        connectivity: vec![
            0, 2, 3,
            0, 3, 1,
            0, 1, 5, 4,
            1, 3, 7, 5,
            3, 2, 6, 7,
            2, 0, 4, 6,
            4, 5, 7, 6,
        ],
        offsets: vec![3, 6, 10, 14, 18, 22, 26],
    };
    let piece = PolyDataPiece {
        points,
        polys: Some(vertex_numbers),
        ..Default::default()
    };
    DataSet::PolyData {
        meta: None,
        pieces: vec![Piece::Inline(Box::new(piece))],
    }
}

#[test]
fn polygon() {
    assert_eq!(truck_simple_cube().to_data_set(), vtk_simple_cube());
}

#[test]
fn vertex() {
    let v = Vertex::new(Point3::new(0.0, 0.0, 0.0));
    let data = v.to_data_set();
    let ans_data = DataSet::UnstructuredGrid {
        meta: None,
        pieces: vec![Piece::Inline(Box::new(UnstructuredGridPiece {
            points: IOBuffer::F64(vec![0.0, 0.0, 0.0]),
            cells: Cells {
                cell_verts: VertexNumbers::XML {
                    connectivity: vec![0],
                    offsets: vec![1],
                },
                types: vec![CellType::Vertex],
            },
            data: Default::default(),
        }))],
    };
    assert_eq!(data, ans_data);
}

#[test]
fn edge() {
    let edge = Edge::new(
        &Vertex::new(Point3::new(0.0, 0.0, 0.0)),
        &Vertex::new(Point3::new(0.0, 1.0, 0.0)),
        PolylineCurve(vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ]),
    );
    let data = edge.to_data_set();
    let ans_data = DataSet::UnstructuredGrid {
        meta: None,
        pieces: vec![Piece::Inline(Box::new(UnstructuredGridPiece {
            points: IOBuffer::F64(vec![
                0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0,
            ]),
            cells: Cells {
                cell_verts: VertexNumbers::XML {
                    connectivity: vec![0, 1, 2, 3, 0, 3],
                    offsets: vec![4, 5, 6],
                },
                types: vec![CellType::PolyLine, CellType::Vertex, CellType::Vertex],
            },
            data: Default::default(),
        }))],
    };
    assert_eq!(data, ans_data);

    // inverse case
    let data = edge.inverse().to_data_set();
    let ans_data = DataSet::UnstructuredGrid {
        meta: None,
        pieces: vec![Piece::Inline(Box::new(UnstructuredGridPiece {
            points: IOBuffer::F64(vec![
                0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            ]),
            cells: Cells {
                cell_verts: VertexNumbers::XML {
                    connectivity: vec![0, 1, 2, 3, 0, 3],
                    offsets: vec![4, 5, 6],
                },
                types: vec![CellType::PolyLine, CellType::Vertex, CellType::Vertex],
            },
            data: Default::default(),
        }))],
    };
    assert_eq!(data, ans_data);
}

#[test]
fn wire() {
    let p = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let v = Vertex::news(p);
    let q = Point3::new(2.0, 0.0, 0.0);
    let wire: Wire<Point3, PolylineCurve<Point3>> = vec![
        Edge::new(&v[0], &v[1], PolylineCurve(vec![p[0], p[1]])),
        Edge::new(&v[2], &v[1], PolylineCurve(vec![p[2], q, p[1]])).inverse(),
        Edge::new(&v[3], &v[0], PolylineCurve(vec![p[3], p[0]])),
    ]
    .into();
    let data = wire.to_data_set();
    let ans_data = DataSet::UnstructuredGrid {
        meta: None,
        pieces: vec![Piece::Inline(Box::new(UnstructuredGridPiece {
            points: IOBuffer::F64(vec![
                0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 2.0, 0.0, 0.0,
            ]),
            cells: Cells {
                cell_verts: VertexNumbers::XML {
                    connectivity: vec![0, 1, 1, 4, 2, 3, 0, 0, 1, 2, 3],
                    offsets: vec![2, 5, 7, 8, 9, 10, 11],
                },
                types: vec![
                    CellType::PolyLine,
                    CellType::PolyLine,
                    CellType::PolyLine,
                    CellType::Vertex,
                    CellType::Vertex,
                    CellType::Vertex,
                    CellType::Vertex,
                ],
            },
            data: Default::default(),
        }))],
    };
    assert_eq!(data, ans_data);
}

fn shell_topology() -> Shell<Point3, PolylineCurve<Point3>, PolygonMesh> {
    let p = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, -1.0),
        Point3::new(1.0, 0.0, -1.0),
        Point3::new(0.5, 0.25, 0.0),
        Point3::new(0.75, 0.5, 0.0),
        Point3::new(0.5, 0.75, 0.0),
        Point3::new(0.25, 0.5, 0.0),
    ];
    let v = Vertex::news(&p);
    let q = [
        Point3::new(0.75, 0.25, 0.0),
        Point3::new(0.75, 0.75, 0.0),
        Point3::new(0.25, 0.75, 0.0),
        Point3::new(0.25, 0.25, 0.0),
    ];
    let e = [
        Edge::new(&v[0], &v[1], PolylineCurve(vec![p[0], p[1]])),
        Edge::new(&v[1], &v[3], PolylineCurve(vec![p[1], p[3]])),
        Edge::new(&v[3], &v[2], PolylineCurve(vec![p[3], p[2]])),
        Edge::new(&v[2], &v[0], PolylineCurve(vec![p[2], p[0]])),
        Edge::new(&v[0], &v[4], PolylineCurve(vec![p[0], p[4]])),
        Edge::new(&v[4], &v[5], PolylineCurve(vec![p[4], p[5]])),
        Edge::new(&v[5], &v[1], PolylineCurve(vec![p[5], p[1]])),
        Edge::new(&v[6], &v[7], PolylineCurve(vec![p[6], q[0], p[7]])),
        Edge::new(&v[7], &v[8], PolylineCurve(vec![p[7], q[1], p[8]])),
        Edge::new(&v[8], &v[9], PolylineCurve(vec![p[8], q[2], p[9]])),
        Edge::new(&v[9], &v[6], PolylineCurve(vec![p[9], q[3], p[6]])),
    ];
    let polygon0 = PolygonMesh::new(
        StandardAttributes {
            positions: [&p[0..4], &p[6..10], &q[0..4]].concat().to_owned(),
            uv_coords: vec![
                p[0].to_vec().truncate(),
                p[1].to_vec().truncate(),
                p[2].to_vec().truncate(),
                p[3].to_vec().truncate(),
                p[6].to_vec().truncate(),
                p[7].to_vec().truncate(),
                p[8].to_vec().truncate(),
                p[9].to_vec().truncate(),
                q[0].to_vec().truncate(),
                q[1].to_vec().truncate(),
                q[2].to_vec().truncate(),
                q[3].to_vec().truncate(),
            ],
            normals: vec![Vector3::new(0.0, 0.0, 1.0)],
        },
        Faces::from_iter([
            [
                (0, Some(0), Some(0)),
                (4, Some(4), Some(0)),
                (11, Some(11), Some(0)),
            ],
            [
                (0, Some(0), Some(0)),
                (1, Some(1), Some(0)),
                (4, Some(4), Some(0)),
            ],
            [
                (1, Some(1), Some(0)),
                (8, Some(8), Some(0)),
                (4, Some(4), Some(0)),
            ],
            [
                (1, Some(1), Some(0)),
                (5, Some(5), Some(0)),
                (8, Some(8), Some(0)),
            ],
            [
                (1, Some(1), Some(0)),
                (3, Some(3), Some(0)),
                (5, Some(5), Some(0)),
            ],
            [
                (3, Some(3), Some(0)),
                (9, Some(9), Some(0)),
                (5, Some(5), Some(0)),
            ],
            [
                (3, Some(3), Some(0)),
                (6, Some(6), Some(0)),
                (9, Some(9), Some(0)),
            ],
            [
                (3, Some(3), Some(0)),
                (2, Some(2), Some(0)),
                (6, Some(6), Some(0)),
            ],
            [
                (2, Some(2), Some(0)),
                (10, Some(10), Some(0)),
                (6, Some(6), Some(0)),
            ],
            [
                (2, Some(2), Some(0)),
                (7, Some(7), Some(0)),
                (10, Some(10), Some(0)),
            ],
            [
                (2, Some(2), Some(0)),
                (0, Some(0), Some(0)),
                (7, Some(7), Some(0)),
            ],
            [
                (0, Some(0), Some(0)),
                (11, Some(11), Some(0)),
                (7, Some(7), Some(0)),
            ],
        ]),
    );
    let polygon1 = PolygonMesh::new(
        StandardAttributes {
            positions: vec![p[0], p[1], p[4], p[5]],
            uv_coords: vec![
                Vector2::new(0.0, 0.0),
                Vector2::new(1.0, 0.0),
                Vector2::new(0.0, 1.0),
                Vector2::new(1.0, 1.0),
            ],
            normals: vec![Vector3::new(0.0, 1.0, 0.0)],
        },
        Faces::from_iter([[
            (1, Some(3), Some(0)),
            (3, Some(1), Some(0)),
            (2, Some(0), Some(0)),
            (0, Some(2), Some(0)),
        ]]),
    );

    vec![
        Face::new(
            vec![
                vec![e[0].clone(), e[1].clone(), e[2].clone(), e[3].clone()].into(),
                vec![
                    e[10].inverse(),
                    e[9].inverse(),
                    e[8].inverse(),
                    e[7].inverse(),
                ]
                .into(),
            ],
            polygon0,
        ),
        Face::new(
            vec![vec![
                e[0].clone(),
                e[6].inverse(),
                e[5].inverse(),
                e[4].inverse(),
                ].into()],
            polygon1,
        ).inverse(),
    ]
    .into()
}

fn shell_unstructured_grid() -> DataSet {
    #[rustfmt::skip]
    let points: Vec<f64> = vec![
        0.0, 0.0, 0.0,
        0.5, 0.25, 0.0,
        0.25, 0.25, 0.0,
        1.0, 0.0, 0.0,
        0.75, 0.25, 0.0,
        0.75, 0.5, 0.0,
        1.0, 1.0, 0.0,
        0.75, 0.75, 0.0,
        0.5, 0.75, 0.0,
        0.0, 1.0, 0.0,
        0.25, 0.75, 0.0,
        0.25, 0.5, 0.0,
    ];
    #[rustfmt::skip]
    let uvs: Vec<f64> = vec![
        0.0, 0.0,
        0.5, 0.25,
        0.25, 0.25,
        1.0, 0.0,
        0.75, 0.25,
        0.75, 0.5,
        1.0, 1.0,
        0.75, 0.75,
        0.5, 0.75,
        0.0, 1.0,
        0.25, 0.75,
        0.25, 0.5,
    ];
    #[rustfmt::skip]
    let normals: Vec<f64> = vec![
        0.0, 0.0, 1.0,
        0.0, 0.0, 1.0,
        0.0, 0.0, 1.0,
        0.0, 0.0, 1.0,
        0.0, 0.0, 1.0,
        0.0, 0.0, 1.0,
        0.0, 0.0, 1.0,
        0.0, 0.0, 1.0,
        0.0, 0.0, 1.0,
        0.0, 0.0, 1.0,
        0.0, 0.0, 1.0,
        0.0, 0.0, 1.0,
    ];
    #[rustfmt::skip]
    let connectivity: Vec<u64> = vec![
        0, 1, 2,
        0, 3, 1,
        3, 4, 1,
        3, 5, 4,
        3, 6, 5,
        6, 7, 5,
        6, 8, 7,
        6, 9, 8,
        9, 10, 8,
        9, 11, 10,
        9, 0, 11,
        0, 2, 11,
        0, 3,
        3, 6,
        6, 9,
        9, 0,
        1, 2, 11,
        11, 10, 8,
        8, 7, 5,
        5, 4, 1,
        0, 3, 6, 9, 1, 11, 8, 5,
    ];
    let offsets: Vec<u64> = vec![
        3, 6, 9, 12, 15, 18, 21, 24, 27, 30, 33, 36, 38, 40, 42, 44, 47, 50, 53, 56, 57, 58, 59,
        60, 61, 62, 63, 64,
    ];
    let types = vec![
        CellType::Polygon,
        CellType::Polygon,
        CellType::Polygon,
        CellType::Polygon,
        CellType::Polygon,
        CellType::Polygon,
        CellType::Polygon,
        CellType::Polygon,
        CellType::Polygon,
        CellType::Polygon,
        CellType::Polygon,
        CellType::Polygon,
        CellType::PolyLine,
        CellType::PolyLine,
        CellType::PolyLine,
        CellType::PolyLine,
        CellType::PolyLine,
        CellType::PolyLine,
        CellType::PolyLine,
        CellType::PolyLine,
        CellType::Vertex,
        CellType::Vertex,
        CellType::Vertex,
        CellType::Vertex,
        CellType::Vertex,
        CellType::Vertex,
        CellType::Vertex,
        CellType::Vertex,
    ];
    let piece0 = Piece::Inline(Box::new(UnstructuredGridPiece {
        points: IOBuffer::F64(points),
        cells: Cells {
            cell_verts: VertexNumbers::XML {
                connectivity,
                offsets,
            },
            types,
        },
        data: vtkio::model::Attributes {
            point: vec![
                Attribute::DataArray(DataArray {
                    name: "TCoords".to_owned(),
                    elem: ElementType::TCoords(2),
                    data: IOBuffer::F64(uvs),
                }),
                Attribute::DataArray(DataArray {
                    name: "Normals".to_owned(),
                    elem: ElementType::Normals,
                    data: IOBuffer::F64(normals),
                }),
            ],
            ..Default::default()
        },
    }));
    #[rustfmt::skip]
    let points: Vec<f64> = vec![
        0.0, 0.0, 0.0,
        0.0, 0.0, -1.0,
        1.0, 0.0, -1.0,
        1.0, 0.0, 0.0,
    ];
    #[rustfmt::skip]
    let uvs: Vec<f64> = vec![
        0.0, 1.0,
        0.0, 0.0,
        1.0, 0.0,
        1.0, 1.0,
    ];
    #[rustfmt::skip]
    let normals: Vec<f64> = vec![
        0.0, -1.0, 0.0,
        0.0, -1.0, 0.0,
        0.0, -1.0, 0.0,
        0.0, -1.0, 0.0,
    ];
    #[rustfmt::skip]
    let connectivity: Vec<u64> = vec![
        0, 1, 2, 3,
        0, 1,
        1, 2,
        2, 3,
        3, 0,
        0, 1, 2, 3,
    ];
    let offsets: Vec<u64> = vec![4, 6, 8, 10, 12, 13, 14, 15, 16];
    let types = vec![
        CellType::Polygon,
        CellType::PolyLine,
        CellType::PolyLine,
        CellType::PolyLine,
        CellType::PolyLine,
        CellType::Vertex,
        CellType::Vertex,
        CellType::Vertex,
        CellType::Vertex,
    ];
    let piece1 = Piece::Inline(Box::new(UnstructuredGridPiece {
        points: IOBuffer::F64(points),
        cells: Cells {
            cell_verts: VertexNumbers::XML {
                connectivity,
                offsets,
            },
            types,
        },
        data: vtkio::model::Attributes {
            point: vec![
                Attribute::DataArray(DataArray {
                    name: "TCoords".to_owned(),
                    elem: ElementType::TCoords(2),
                    data: IOBuffer::F64(uvs),
                }),
                Attribute::DataArray(DataArray {
                    name: "Normals".to_owned(),
                    elem: ElementType::Normals,
                    data: IOBuffer::F64(normals),
                }),
            ],
            ..Default::default()
        },
    }));
    DataSet::UnstructuredGrid {
        meta: None,
        pieces: vec![piece0, piece1],
    }
}

#[test]
fn face() {
    let data = shell_topology()[0].to_data_set();
    let DataSet::UnstructuredGrid { pieces, .. } = shell_unstructured_grid() else { unreachable!() };
    let ans_data = DataSet::UnstructuredGrid { meta: None, pieces: vec![pieces[0].clone()] };
    assert_eq!(data, ans_data);
}

#[test]
fn shell() {
    assert_eq!(shell_topology().to_data_set(), shell_unstructured_grid());
}
