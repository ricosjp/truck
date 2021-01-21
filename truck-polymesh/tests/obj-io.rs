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
fn position_obj_io_test() {
    let read_mesh0 = obj::read(TEAPOT_POSITION_OBJ).unwrap();
    let mut gened_obj: Vec<u8> = Vec::new();
    obj::write(&read_mesh0, &mut gened_obj).unwrap();
    let read_mesh1 = obj::read(AsRef::<[u8]>::as_ref(&gened_obj)).unwrap();
    assert_eq!(read_mesh0.positions(), read_mesh1.positions());
    assert_eq!(read_mesh0.uv_coords(), read_mesh1.uv_coords());
    assert_eq!(read_mesh0.normals(), read_mesh1.normals());
    assert_eq!(read_mesh0.faces(), read_mesh1.faces());
}

#[test]
fn withtexcoord_obj_io_test() {
    let read_mesh0 = obj::read(SKULL_WITHTEXCOORD_OBJ).unwrap();
    let mut gened_obj: Vec<u8> = Vec::new();
    obj::write(&read_mesh0, &mut gened_obj).unwrap();
    let read_mesh1 = obj::read(AsRef::<[u8]>::as_ref(&gened_obj)).unwrap();
    assert_eq!(read_mesh0.positions(), read_mesh1.positions());
    assert_eq!(read_mesh0.uv_coords(), read_mesh1.uv_coords());
    assert_eq!(read_mesh0.normals(), read_mesh1.normals());
    assert_eq!(read_mesh0.faces(), read_mesh1.faces());
}

#[test]
fn withnormals_obj_io_test() {
    let read_mesh0 = obj::read(TEAPOT_WITHNORMALS_OBJ).unwrap();
    let mut gened_obj: Vec<u8> = Vec::new();
    obj::write(&read_mesh0, &mut gened_obj).unwrap();
    let read_mesh1 = obj::read(AsRef::<[u8]>::as_ref(&gened_obj)).unwrap();
    assert_eq!(read_mesh0.positions(), read_mesh1.positions());
    assert_eq!(read_mesh0.uv_coords(), read_mesh1.uv_coords());
    assert_eq!(read_mesh0.normals(), read_mesh1.normals());
    assert_eq!(read_mesh0.faces(), read_mesh1.faces());
}

#[test]
fn complete_obj_io_test() {
    let read_mesh0 = obj::read(PONY_COMPLETE_OBJ).unwrap();
    let mut gened_obj: Vec<u8> = Vec::new();
    obj::write(&read_mesh0, &mut gened_obj).unwrap();
    let read_mesh1 = obj::read(AsRef::<[u8]>::as_ref(&gened_obj)).unwrap();
    assert_eq!(read_mesh0.positions(), read_mesh1.positions());
    assert_eq!(read_mesh0.uv_coords(), read_mesh1.uv_coords());
    assert_eq!(read_mesh0.normals(), read_mesh1.normals());
    assert_eq!(read_mesh0.faces(), read_mesh1.faces());
}
