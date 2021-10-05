use truck_meshalgo::{filters::*, tessellation::*};
use truck_modeling::*;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 2 {
        panic!("usage: tessellate_shape <input json file> <output json file>\nThe default <output file> is output.obj.")
    }
    let file = std::fs::File::open(&args[1]).unwrap();
    let solid = Solid::extract(serde_json::from_reader(file).unwrap()).unwrap();
    let mut poly = solid.triangulation(0.01).unwrap().to_polygon();
    poly.put_together_same_attrs().remove_unused_attrs();
    let mut string = Vec::<u8>::new();
    truck_polymesh::obj::write(&poly, &mut string).unwrap();
    if args.len() > 2 {
        std::fs::write(&args[2], &string).unwrap();
    } else {
        std::fs::write("output.obj", &string).unwrap();
    }
}
