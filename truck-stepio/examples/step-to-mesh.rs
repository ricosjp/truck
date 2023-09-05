//! Parse STEP data, extract shape, and meshing.

use std::path::Path;
use truck_meshalgo::prelude::*;
use truck_stepio::r#in::*;
use truck_topology::compress::CompressedShell;

fn main() {
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
    let polyshells = table
        .shell
        .iter()
        .map(|shell| {
            let shell = table.to_compressed_shell(shell.1).unwrap();
            shell.triangulation(0.1)
        })
        .collect::<Vec<_>>();

    let path: &Path = &args[2].as_ref();
    let extension = path.extension().and_then(|e| e.to_str());
    match extension {
        Some("obj") => output_obj(&polyshells, path),
        Some("vtu") => output_vtk(&polyshells, path),
        _ => {}
    }
}

fn output_obj(
    polyshells: &Vec<CompressedShell<Point3, PolylineCurve<Point3>, Option<PolygonMesh>>>,
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
    polyshells: &Vec<CompressedShell<Point3, PolylineCurve<Point3>, Option<PolygonMesh>>>,
    path: &Path,
) {
    use vtkio::model::*;
    let pieces = polyshells
        .iter()
        .flat_map(|shell| {
            let shell = truck_topology::Shell::extract(shell.clone()).unwrap();
            let shell: truck_topology::Shell<Point3, PolylineCurve<Point3>, Option<PolygonMesh>> =
                shell
                    .into_iter()
                    .filter_map(|face| match face.surface() {
                        Some(_) => Some(face),
                        None => None,
                    })
                    .collect();
            let shell = shell.mapped(Point3::clone, PolylineCurve::clone, |x| x.clone().unwrap());
            let DataSet::UnstructuredGrid { pieces, .. } = shell.to_data_set() else {
                unreachable!()
            };
            pieces
        })
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
