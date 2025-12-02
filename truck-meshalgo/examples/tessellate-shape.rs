//! Tessellate a shape and output an obj file.
//!
//! ```bash
//! usage: tessellate-shape <input json file> <output json file>
//! ```
//!
//! The default `<output file>` is output.obj.

use truck_meshalgo::{analyzers::*, filters::*, tessellation::*};
use truck_modeling::{geometry::*, Point3};
use truck_polymesh::{algo::DefaultSplitParams, TOLERANCE};
use truck_topology::compress::*;
type CShell = CompressedShell<Point3, Curve, Surface>;
type CSolid = CompressedSolid<Point3, Curve, Surface>;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 2 {
        panic!("usage: tessellate-shape <input json file> <output json file>\nThe default <output file> is output.obj.")
    }
    let file = std::fs::read_to_string(&args[1]).unwrap();
    let mut poly = {
        if let Ok(solid) = serde_json::from_str::<CSolid>(&file) {
            solid.triangulation(DefaultSplitParams::new(0.005)).to_polygon()
        } else if let Ok(shell) = serde_json::from_str::<CShell>(&file) {
            shell.triangulation(DefaultSplitParams::new(0.005)).to_polygon()
        } else {
            panic!("Your json file is something wrong.");
        }
    };
    poly.put_together_same_attrs(TOLERANCE * 10.0)
        .remove_degenerate_faces()
        .remove_unused_attrs();
    println!("polygon shell condition: {:?}", poly.shell_condition());
    let path = match args.len() > 2 {
        true => &args[2],
        false => "output.obj",
    };
    let file = std::fs::File::create(path).unwrap();
    truck_polymesh::obj::write(&poly, file).unwrap();
}
