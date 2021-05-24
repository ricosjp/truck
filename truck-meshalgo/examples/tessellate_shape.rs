use truck_meshalgo::{filters::*, tessellation::*};
use truck_modeling::*;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let file = std::fs::File::open(&args[1]).unwrap();
    let solid = Solid::extract(serde_json::from_reader(file).unwrap()).unwrap();
    let mut poly = solid.triangulation(0.01).unwrap();
    poly.put_together_same_attrs().remove_unused_attrs();
    let mut string = Vec::<u8>::new();
    truck_polymesh::obj::write(&poly, &mut string).unwrap();
    std::fs::write("output.obj", &string).unwrap();
}
