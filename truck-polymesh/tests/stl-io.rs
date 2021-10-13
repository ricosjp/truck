use stl::{IntoSTLIterator, STLFace, STLReader, STLType};
use truck_base::assert_near;
use truck_polymesh::*;
type Result<T> = std::result::Result<T, errors::Error>;

const ASCII_BUNNY: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../resources/stl/bunny_ascii.stl",
));

const BINARY_BUNNY: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../resources/stl/bunny_binary.stl",
));

#[test]
fn stl_oi_test() {
    let mesh = vec![
        STLFace {
            normal: [0.0, 0.0, 1.0],
            vertices: [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        },
        STLFace {
            normal: [0.0, 1.0, 1.0],
            vertices: [[0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]],
        },
        STLFace {
            normal: [1.0, 0.0, 1.0],
            vertices: [[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        },
    ];
    let mut ascii = Vec::new();
    stl::write(mesh.iter().cloned(), &mut ascii, STLType::ASCII).unwrap();
    let mut binary = Vec::new();
    stl::write(mesh.iter().cloned(), &mut binary, STLType::Binary).unwrap();
    let amesh = STLReader::<&[u8]>::new(&ascii, STLType::Automatic)
        .unwrap()
        .collect::<Result<Vec<_>>>()
        .unwrap();
    let bmesh = STLReader::<&[u8]>::new(&binary, STLType::Automatic)
        .unwrap()
        .collect::<Result<Vec<_>>>()
        .unwrap();
    assert_eq!(mesh, amesh);
    assert_eq!(mesh, bmesh);
}

#[test]
fn stl_io_test() {
    let amesh = STLReader::new(ASCII_BUNNY, STLType::Automatic)
        .unwrap()
        .map(Result::unwrap)
        .collect::<Vec<_>>();
    let bmesh = STLReader::new(BINARY_BUNNY, STLType::Automatic)
        .unwrap()
        .map(Result::unwrap)
        .collect::<Vec<_>>();
    assert_eq!(amesh, bmesh);
    let mut bytes = Vec::<u8>::new();
    stl::write(bmesh.iter().cloned(), &mut bytes, STLType::Binary).unwrap();
    // Binary data is free from notational distortions except for the headers.
    assert_eq!(&bytes[80..], &BINARY_BUNNY[80..]);
}

#[test]
fn through_polymesh() {
    let iter = STLReader::<&[u8]>::new(BINARY_BUNNY, STLType::Automatic).unwrap();
    let polymesh: PolygonMesh = iter.map(|face| face.unwrap()).collect();
    let mesh: Vec<STLFace> = polymesh.into_iter().collect();
    let iter = STLReader::<&[u8]>::new(BINARY_BUNNY, STLType::Automatic).unwrap();
    for (face0, face1) in mesh.iter().zip(iter) {
        let face1 = face1.unwrap();
        assert_near!(face0.vertices[0][0] as f64, face1.vertices[0][0] as f64);
        assert_near!(face0.vertices[0][1] as f64, face1.vertices[0][1] as f64);
        assert_near!(face0.vertices[0][2] as f64, face1.vertices[0][2] as f64);
        assert_near!(face0.vertices[1][0] as f64, face1.vertices[1][0] as f64);
        assert_near!(face0.vertices[1][1] as f64, face1.vertices[1][1] as f64);
        assert_near!(face0.vertices[1][2] as f64, face1.vertices[1][2] as f64);
        assert_near!(face0.vertices[2][0] as f64, face1.vertices[2][0] as f64);
        assert_near!(face0.vertices[2][1] as f64, face1.vertices[2][1] as f64);
        assert_near!(face0.vertices[2][2] as f64, face1.vertices[2][2] as f64);
        // This is not assert_near, since VTK is single precision.
        assert!(f32::abs(face0.normal[0] - face1.normal[0]) < 5.0e-4);
        assert!(f32::abs(face0.normal[1] - face1.normal[1]) < 5.0e-4);
        assert!(f32::abs(face0.normal[2] - face1.normal[2]) < 5.0e-4);
    }
}
