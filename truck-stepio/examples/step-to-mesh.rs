//! Parse STEP data, extract shape, and meshing.

use std::{cmp::Ordering, path::Path};
use truck_meshalgo::prelude::*;
use truck_stepio::r#in::*;
use truck_topology::compress::*;

fn main() {
    let mut args = std::env::args().collect::<Vec<_>>();
    match args.len().cmp(&2) {
        Ordering::Less => {
            eprintln!("usage: step-to-obj <input step file> [output obj file]");
            return;
        }
        Ordering::Equal => args.push("output.obj".to_string()),
        Ordering::Greater => {}
    }

    println!("reading file...");
    let step_file = std::fs::read_to_string(&args[1]).unwrap();
    let exchange = ruststep::parser::parse(&step_file).unwrap();
    let table = Table::from_data_section(&exchange.data[0]);
    println!("meshing...");
    let polyshells = table
        .shell
        .iter()
        .map(|shell| {
            let shell = table.to_compressed_shell(shell.1).unwrap();
            shell.triangulation(0.05)
        })
        .collect::<Vec<_>>();

    let path: &Path = args[2].as_ref();
    let extension = path.extension().and_then(|e| e.to_str());
    match extension {
        Some("obj") => output_obj(&polyshells, path),
        Some("vtu") => output_vtk(polyshells, path),
        _ => {}
    }
}

fn output_obj(
    polyshells: &[CompressedShell<Point3, PolylineCurve<Point3>, Option<PolygonMesh>>],
    path: &Path,
) {
    let mut polymesh = PolygonMesh::default();
    polyshells
        .iter()
        .for_each(|shell| polymesh.merge(shell.to_polygon()));
    println!("output obj...");
    let obj_file = std::fs::File::create(path).unwrap();
    obj::write(&polymesh, obj_file).unwrap();
}

fn output_vtk(
    polyshells: Vec<CompressedShell<Point3, PolylineCurve<Point3>, Option<PolygonMesh>>>,
    path: &Path,
) {
    use vtkio::model::*;
    let pieces = polyshells
        .into_iter()
        .flat_map(
            |CompressedShell {
                 vertices,
                 edges,
                 faces,
             }| {
                let faces = faces
                    .into_iter()
                    .filter_map(|face| {
                        Some(CompressedFace {
                            boundaries: face.boundaries,
                            orientation: face.orientation,
                            surface: face.surface?,
                        })
                    })
                    .collect();
                let DataSet::UnstructuredGrid { pieces, .. } = CompressedShell {
                    vertices,
                    edges,
                    faces,
                }
                .to_data_set() else {
                    unreachable!()
                };
                pieces
            },
        )
        .collect::<Vec<_>>();
    let vtk = Vtk {
        version: (1, 0).into(),
        title: String::new(),
        byte_order: ByteOrder::LittleEndian,
        file_path: None,
        data: DataSet::UnstructuredGrid { meta: None, pieces },
    };

    let obj_file = std::fs::File::create(path).unwrap();
    vtk.write_xml(obj_file).unwrap();
}
