use stl::{STLFace, STLReader, STLType};
use truck_polymesh::*;

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
    stl::write_ascii(mesh.iter().map(|face| face.clone()), &mut ascii).unwrap();
    let mut binary = Vec::new();
    stl::write_binary(mesh.iter().map(|face| face.clone()), &mut binary).unwrap();
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
    let amesh = STLReader::<&[u8]>::new(include_bytes!("data/bunny_ascii.stl"), STLType::Automatic)
        .unwrap()
        .collect::<Result<Vec<_>>>()
        .unwrap();
    let bmesh = STLReader::<&[u8]>::new(include_bytes!("data/bunny_binary.stl"), STLType::Automatic)
        .unwrap()
        .collect::<Result<Vec<_>>>()
        .unwrap();
    assert_eq!(amesh, bmesh);
    let mut bytes = Vec::<u8>::new();
    stl::write_binary(bmesh.iter().map(|face| face.clone()), &mut bytes).unwrap();
    // Binary data is free from notational distortions except for the headers.
    assert_eq!(&bytes[80..], &include_bytes!("data/bunny_binary.stl")[80..]);
}

#[test]
fn teapot_output() {
    let polymesh = obj::read::<&[u8]>(include_bytes!("data/teapot-position.obj")).unwrap();
    let mut file = std::fs::File::create("teapot-binary.stl").unwrap();
    stl::write_ascii(&polymesh, &mut file).unwrap();
}
