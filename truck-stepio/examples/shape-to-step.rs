//! convert from truck shape json to step file.
//!
//! ### usage
//!
//! ```bash
//! shape-to-step <input shape file> [output shape file]
//! ```

use std::{cmp::Ordering, env};
use truck_modeling::*;
use truck_stepio::out;
use truck_topology::compress::CompressedSolid;

fn main() {
    let mut args = env::args().collect::<Vec<_>>();
    match args.len().cmp(&2) {
        Ordering::Less => {
            eprintln!("usage: shape-to-step <input shape file> [output shape file]");
            return;
        }
        Ordering::Equal => args.push("output.stp".to_string()),
        Ordering::Greater => {}
    }

    let shape_file = std::fs::read(&args[1]).unwrap();
    let compressed: CompressedSolid<Point3, Curve, Surface> =
        serde_json::from_reader(shape_file.as_slice()).unwrap();
    let step_string = out::CompleteStepDisplay::new(
        out::StepModel::from(&compressed),
        out::StepHeaderDescriptor {
            organization_system: "shape-to-step".to_owned(),
            ..Default::default()
        },
    )
    .to_string();
    let mut step_file = std::fs::File::create(&args[2]).unwrap();
    std::io::Write::write_all(&mut step_file, step_string.as_ref()).unwrap();
    let _ = ruststep::parser::parse(&step_string).unwrap();
}
