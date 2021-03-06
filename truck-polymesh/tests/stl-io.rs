use stl::{STLFace, STLReader, STLType};
use truck_polymesh::*;

#[test]
fn stl_oi_test() {
    let mesh = vec![
        STLFace {
            normal: [0.0, 0.0, 1.0],
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        },
        STLFace {
            normal: [0.0, 1.0, 1.0],
            vertices: vec![[0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]],
        },
        STLFace {
            normal: [1.0, 0.0, 1.0],
            vertices: vec![[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
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
