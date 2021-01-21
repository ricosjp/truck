use truck_polymesh::*;

const TEAPOT_POSITION_OBJ: &[u8] = include_bytes!("teapot-position.obj");
const TEAPOT_WITHNORMALS_OBJ: &[u8] = include_bytes!("teapot-with-normals.obj");

// https://sketchfab.com/3d-models/skull-downloadable-1a9db900738d44298b0bc59f68123393
// Skull downloadable - CC Attribution © martinjario
const SKULL_WITHTEXCOORD_OBJ: &[u8] = include_bytes!("skull-with-texcoord.obj");

// https://sketchfab.com/3d-models/pony-cartoon-885d9f60b3a9429bb4077cfac5653cf9
// Pony Cartoon - CC Attribution © Slava Z.
const PONY_COMPLETE_OBJ: &[u8] = include_bytes!("pony-complete.obj");

#[test]
fn position_obj_ioi_test() {
    let read_mesh0 = obj::read(TEAPOT_POSITION_OBJ).unwrap();
    let mut gened_obj: Vec<u8> = Vec::new();
    obj::write(&read_mesh0, &mut gened_obj).unwrap();
    let read_mesh1 = obj::read(AsRef::<[u8]>::as_ref(&gened_obj)).unwrap();
    assert_eq!(read_mesh0, read_mesh1);
}

#[test]
fn withtexcoord_obj_ioi_test() {
    let read_mesh0 = obj::read(SKULL_WITHTEXCOORD_OBJ).unwrap();
    let mut gened_obj: Vec<u8> = Vec::new();
    obj::write(&read_mesh0, &mut gened_obj).unwrap();
    let read_mesh1 = obj::read(AsRef::<[u8]>::as_ref(&gened_obj)).unwrap();
    assert_eq!(read_mesh0, read_mesh1);
}

#[test]
fn withnormals_obj_ioi_test() {
    let read_mesh0 = obj::read(TEAPOT_WITHNORMALS_OBJ).unwrap();
    let mut gened_obj: Vec<u8> = Vec::new();
    obj::write(&read_mesh0, &mut gened_obj).unwrap();
    let read_mesh1 = obj::read(AsRef::<[u8]>::as_ref(&gened_obj)).unwrap();
    assert_eq!(read_mesh0, read_mesh1);
}

#[test]
fn complete_obj_ioi_test() {
    let read_mesh0 = obj::read(PONY_COMPLETE_OBJ).unwrap();
    let mut gened_obj: Vec<u8> = Vec::new();
    obj::write(&read_mesh0, &mut gened_obj).unwrap();
    let read_mesh1 = obj::read(AsRef::<[u8]>::as_ref(&gened_obj)).unwrap();
    assert_eq!(read_mesh0, read_mesh1);
}

mod cube {
    use super::*;
    pub const POSITIONS: [Point3; 8] = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
    ];

    pub const UV_COORDS: [Vector2; 4] = [
        Vector2::new(0.0, 0.0),
        Vector2::new(1.0, 0.0),
        Vector2::new(0.0, 1.0),
        Vector2::new(1.0, 1.0),
    ];

    pub const NORMALS: [Vector3; 6] = [
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(0.0, 0.0, 1.0),
        Vector3::new(-1.0, 0.0, 0.0),
        Vector3::new(0.0, -1.0, 0.0),
        Vector3::new(0.0, 0.0, -1.0),
    ];
}

#[test]
fn positions_obj_oi_test() {
    let faces = Faces::from_iter(&[
        [(0, None, None), (1, None, None), (2, None, None)].as_ref(),
        &[(4, None, None), (2, None, None), (1, None, None)],
        &[(1, None, None), (0, None, None), (3, None, None)],
        &[(1, None, None), (3, None, None), (5, None, None)],
        &[(1, None, None), (5, None, None), (4, None, None), (7, None, None)],
        &[(2, None, None), (4, None, None), (7, None, None)],
        &[(2, None, None), (7, None, None), (6, None, None)],
        &[(0, None, None), (2, None, None), (6, None, None), (3, None, None)],
        &[(3, None, None), (6, None, None), (7, None, None)],
        &[(3, None, None), (7, None, None), (5, None, None)],
    ]);
    let mesh = PolygonMesh::new(cube::POSITIONS.to_vec(), Vec::new(), Vec::new(), faces);
    let mut gened_obj: Vec<u8> = Vec::new();
    obj::write(&mesh, &mut gened_obj).unwrap();
    let read_mesh = obj::read(AsRef::<[u8]>::as_ref(&gened_obj)).unwrap();
    assert_eq!(mesh, read_mesh);
}

#[test]
fn withtexcoords_obj_oi_test() {
    let faces = Faces::from_iter(&[
        [(0, Some(0), None), (1, Some(1), None), (2, Some(2), None)].as_ref(),
        &[(4, Some(3), None), (2, Some(2), None), (1, Some(1), None)],
        &[(1, Some(0), None), (0, Some(1), None), (3, Some(2), None)],
        &[(1, Some(3), None), (3, Some(2), None), (5, Some(1), None)],
        &[(1, Some(0), None), (5, Some(1), None), (4, Some(2), None), (7, Some(1), None)],
        &[(2, Some(0), None), (4, Some(1), None), (7, Some(2), None)],
        &[(2, Some(3), None), (7, Some(2), None), (6, Some(1), None)],
        &[(0, Some(0), None), (2, Some(1), None), (6, Some(2), None), (3, Some(1), None)],
        &[(3, Some(0), None), (6, Some(1), None), (7, Some(2), None)],
        &[(3, Some(3), None), (7, Some(2), None), (5, Some(1), None)],
    ]);
    let mesh = PolygonMesh::new(cube::POSITIONS.to_vec(), cube::UV_COORDS.to_vec(), Vec::new(), faces);
    let mut gened_obj: Vec<u8> = Vec::new();
    obj::write(&mesh, &mut gened_obj).unwrap();
    let read_mesh = obj::read(AsRef::<[u8]>::as_ref(&gened_obj)).unwrap();
    assert_eq!(mesh, read_mesh);
}

#[test]
fn withnormals_obj_oi_test() {
    let faces = Faces::from_iter(&[
        [(0, None, Some(5)), (1, None, Some(5)), (2, None, Some(5))].as_ref(),
        &[(4, None, Some(5)), (2, None, Some(5)), (1, None, Some(5))],
        &[(1, None, Some(4)), (0, None, Some(4)), (3, None, Some(4))],
        &[(1, None, Some(4)), (3, None, Some(4)), (5, None, Some(4))],
        &[(1, None, Some(0)), (5, None, Some(0)), (4, None, Some(0)), (7, None, Some(0))],
        &[(2, None, Some(1)), (4, None, Some(1)), (7, None, Some(1))],
        &[(2, None, Some(1)), (7, None, Some(1)), (6, None, Some(1))],
        &[(0, None, Some(3)), (2, None, Some(3)), (6, None, Some(3)), (3, None, Some(3))],
        &[(3, None, Some(2)), (6, None, Some(2)), (7, None, Some(2))],
        &[(3, None, Some(2)), (7, None, Some(2)), (5, None, Some(2))],
    ]);
    let mesh = PolygonMesh::new(cube::POSITIONS.to_vec(), Vec::new(), cube::NORMALS.to_vec(), faces);
    let mut gened_obj: Vec<u8> = Vec::new();
    obj::write(&mesh, &mut gened_obj).unwrap();
    let read_mesh = obj::read(AsRef::<[u8]>::as_ref(&gened_obj)).unwrap();
    assert_eq!(mesh, read_mesh);
}

#[test]
fn incomplete_obj_oi_test() {
    let faces = Faces::from_iter(&[
        [(0, Some(0), Some(5)), (1, Some(1), Some(5)), (2, Some(2), Some(5))].as_ref(),
        &[(4, Some(3), Some(5)), (2, Some(2), Some(5)), (1, Some(1), Some(5))],
        &[(1, Some(0), Some(4)), (0, None, Some(4)), (3, Some(2), Some(4))],
        &[(1, Some(3), Some(4)), (3, Some(2), Some(4)), (5, None, Some(4))],
        &[(1, Some(0), None), (5, Some(1), Some(0)), (4, Some(2), Some(0)), (7, Some(1), Some(0))],
        &[(2, Some(3), Some(1)), (7, Some(2), Some(1)), (6, Some(1), Some(1)), (0, Some(0), Some(3)), (2, None, None)],
        &[(0, Some(3), Some(3)), (6, Some(2), Some(3)), (3, Some(1), Some(3))],
        &[(3, None, Some(2)), (6, Some(1), Some(2)), (7, Some(2), Some(2)), (0, None, None), (1, Some(0), Some(4))],
        &[(3, Some(3), Some(2)), (7, Some(2), Some(2)), (5, Some(1), Some(2))],
    ]);
    let mesh = PolygonMesh::new(cube::POSITIONS.to_vec(), cube::UV_COORDS.to_vec(), cube::NORMALS.to_vec(), faces);
    let mut gened_obj: Vec<u8> = Vec::new();
    obj::write(&mesh, &mut gened_obj).unwrap();
    let read_mesh = obj::read(AsRef::<[u8]>::as_ref(&gened_obj)).unwrap();
    assert_eq!(mesh, read_mesh);
}
