use truck_modeling::*;

fn main() {
	// outer cube
	let v = builder::vertex(Point3::origin());
	let e = builder::tsweep(&v, Vector3::unit_z());
	let f = builder::tsweep(&e, Vector3::unit_x());
	let cube0 = builder::tsweep(&f, Vector3::unit_y());

	// inner cube
	let v = builder::vertex(Point3::new(0.25, 0.25, 0.25));
	let e = builder::tsweep(&v, Vector3::unit_x() * 0.5);
	let f = builder::tsweep(&e, Vector3::unit_z() * 0.5);
	let cube1 = builder::tsweep(&f, Vector3::unit_y() * 0.5);

	let mut boundaries = cube0.into_boundaries();
	boundaries.extend(cube1.into_boundaries());

	let solid = Solid::new(boundaries);
	let json = serde_json::to_string_pretty(&solid).unwrap();
    std::fs::write("cube-in-cube.json", &json).unwrap();
}