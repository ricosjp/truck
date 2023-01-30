//! Parse STEP data, extract shape, and meshing.

use truck_stepio::r#in::{alias::*, *};

fn main() {
    use truck_meshalgo::tessellation::*;
    let mut args = std::env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        eprintln!("usage: step-to-obj <input step file> [output obj file]");
        return;
    } else if args.len() == 2 {
        args.push("output.obj".to_string());
    }

    println!("reading file...");
    let step_file = std::fs::read_to_string(&args[1]).unwrap();
    let exchange = ruststep::parser::parse(&step_file).unwrap();
    let table = Table::from_data_section(&exchange.data[0]);
    println!("meshing...");
    let mut polymesh = PolygonMesh::default();
    table.shell.iter().for_each(|shell| {
        let shell = table.to_compressed_shell(shell.1).unwrap();
        let new_poly = shell.triangulation(0.05).to_polygon();
        polymesh.merge(new_poly);
    });

    println!("output obj...");
    let obj_file = std::fs::File::create(&args[2]).unwrap();
    obj::write(&polymesh, obj_file).unwrap();
}
