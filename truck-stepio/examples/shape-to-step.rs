//! convert from truck shape json to step file.
//!
//! ### usage
//!
//! ```bash
//! shape-to-step <input shape file> [output shape file]
//! ```

use clap::Parser;
use truck_modeling::*;
use truck_stepio::out::{self, StepDesign};
use truck_topology::compress::CompressedSolid;

/// convert from truck shape json to step file.
#[derive(Parser, Debug)]
struct Args {
    /// file name of truck shape json
    input_shape_file: String,
    /// step file name
    #[arg(default_value = "output.stp")]
    output_step_file: String,
}

fn main() {
    let Args {
        input_shape_file,
        output_step_file,
    } = Args::parse();

    let shape_file = std::fs::read(input_shape_file).unwrap();
    let compressed: CompressedSolid<Point3, Curve, Surface> =
        serde_json::from_reader(shape_file.as_slice()).unwrap();
    let repr = StepDesign::<_, Matrix4>::from_model(out::StepModel::from(&compressed));
    let step_string = out::StepDisplay::new(
        out::StepHeaderDescriptor {
            organization_system: "shape-to-step".to_owned(),
            ..Default::default()
        },
        repr,
    )
    .to_string();
    let mut step_file = std::fs::File::create(output_step_file).unwrap();
    std::io::Write::write_all(&mut step_file, step_string.as_ref()).unwrap();
    let _ = ruststep::parser::parse(&step_string).unwrap();
}
