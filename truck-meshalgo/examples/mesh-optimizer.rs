use truck_meshalgo::prelude::*;

fn main() {
	let args: Vec<String> = std::env::args().collect();
	if args.len() < 2 {
		eprintln!("usage: mesh-optimizer <input file>");
		return;
	}
	let reader = std::fs::File::open(&args[1]).unwrap();
	let mut mesh = obj::read(reader).unwrap();
	mesh.normalize_normals()
		.add_naive_normals(false)
		.put_together_same_attrs();
	println!("{:?}", mesh.shell_condition());
}
