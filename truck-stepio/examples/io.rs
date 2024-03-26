use clap::Parser;
use truck_stepio::{out, r#in::*};
use truck_topology::compress::CompressedSolid;

#[derive(Parser, Debug)]
struct Args {
    /// name of input step file
    input: String,
    /// name of output mesh file
    #[arg(default_value = "output")]
    output: String,
}

fn main() {
    let Args { input, output } = Args::parse();
    let step_string = std::fs::read_to_string(&input).unwrap();
    let table = Table::from_step(&step_string).unwrap();
    table
        .shell
        .values()
        .cloned()
        .enumerate()
        .for_each(|(i, step_shell)| {
            let cshell = table.to_compressed_shell(&step_shell).unwrap();
            /*
            use truck_shapeops::*;
            cshell.robust_split_closed_edges_and_faces(0.05);
            cshell.edges.iter_mut().for_each(|edge| {
                if let alias::Curve3D::PCurve(curve) = &edge.curve {
                    use truck_geometry::prelude::*;
                    if matches!(curve.curve().as_ref(), &alias::Curve2D::Line(_)) {
                    let p = curve.front();
                    let q = curve.back();
                    edge.curve = alias::Curve3D::Line(Line(p, q));
                    }
                }
            });
            */
            let csolid = CompressedSolid {
                boundaries: vec![cshell],
            };
            let step_string =
                out::CompleteStepDisplay::new(out::StepModel::from(&csolid), Default::default())
                    .to_string();
            let filename = output.clone() + &i.to_string() + ".step";
            std::fs::write(&filename, &step_string).unwrap();
        });
}
