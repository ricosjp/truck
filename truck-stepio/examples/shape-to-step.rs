use std::env;
use truck_modeling::*;
use truck_stepio::out;
use truck_topology::CompressedSolid;

fn main() {
	let mut step_string = String::new();
	std::io::Read::read_to_string(&mut std::fs::File::open("problem.stp").unwrap(), &mut step_string).unwrap();
	let _ = ruststep::parser::parse(&step_string).unwrap();

	let mut args = env::args().collect::<Vec<_>>();
	if args.len() < 2 {
		eprintln!("usage: shape-to-step <input shape file> [output shape file]");
		return;
	} else if args.len() == 2 {
		args.push("output.stp".to_string());
	}

	let shape_file = std::fs::File::open(&args[1]).unwrap();
	let compressed: CompressedSolid<Point3, Curve, Surface> =
		serde_json::from_reader(shape_file).unwrap();
	let step_string = out::CompleteStepDisplay::new(&compressed).to_string();
	let mut step_file = std::fs::File::create(&args[2]).unwrap();
	std::io::Write::write_all(&mut step_file, step_string.as_ref()).unwrap();
	let _ = ruststep::parser::parse(&step_string).unwrap();
}
