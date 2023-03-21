use truck_meshalgo::prelude::*;

#[test]
fn output_vtk() {
	use vtkio::model::*;
    let mesh = obj::read(
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../resources/obj/teapot-with-normals.obj"
        ))
        .as_slice(),
    )
    .unwrap();
    let data = mesh.to_data_set();
	let vtk = Vtk {
		version: Version::new((2, 0)),
		title: String::new(),
		data,
		file_path: None,
		byte_order: ByteOrder::LittleEndian,
	};	
	let mut buf = Vec::<u8>::new();
	vtk.write_xml(&mut buf).unwrap();
	let string = String::from_utf8(buf).unwrap();
	std::fs::write("teapot-with-normals.vtp", &string).unwrap();
}
