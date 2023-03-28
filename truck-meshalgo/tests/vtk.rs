#![cfg(not(target_arch = "wasm32"))]
use truck_meshalgo::prelude::*;
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
fn simple_cube() {
    let mesh = truck_simple_cube();
    let dataset = mesh.to_data_set();
    assert_eq!(dataset, vtk_simple_cube());
}

#[test]
fn bottle() {
    const BOTTLE: &[u8] = include_bytes!("../../resources/shape/bottle.json");
    let solid: truck_modeling::Solid = serde_json::from_slice(BOTTLE).unwrap();
    let shell = solid.into_boundaries().pop().unwrap();
    let mesh = shell.triangulation(0.01);
    let _vtk = mesh.to_data_set();
}
