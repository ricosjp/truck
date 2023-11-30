//! Parse STEP data, extract shape, and meshing.

use clap::Parser;
use std::path::Path;
use truck_meshalgo::prelude::*;
use truck_stepio::r#in::*;
use truck_topology::compress::*;

#[derive(Parser, Debug)]
struct Args {
    /// name of input step file
    input_step_file: String,
    /// name of output mesh file
    #[arg(default_value = "output.obj")]
    output_mesh_file: String,
    /// output only edge file
    /// Ignored when outputting obj files
    #[arg(long("only-edge"))]
    only_edge: bool,
    /// output the condition of meshes
    /// Ignored when outputting vtk files
    #[arg(long("condition-check"))]
    condition_check: bool,
}

fn main() {
    let Args {
        input_step_file,
        output_mesh_file,
        only_edge,
        condition_check,
    } = Args::parse();

    println!("reading file...");
    let step_file = std::fs::read_to_string(&input_step_file).unwrap();
    let exchange = ruststep::parser::parse(&step_file).unwrap();
    let table = Table::from_data_section(&exchange.data[0]);
    println!("meshing...");
    let polyshells = table
        .shell
        .iter()
        .map(|shell| {
            let shell = table.to_compressed_shell(shell.1).unwrap();
            shell.robust_triangulation(0.05)
        })
        .collect::<Vec<_>>();

    let path: &Path = output_mesh_file.as_ref();
    let extension = path.extension().and_then(|e| e.to_str());
    match extension {
        Some("obj") => output_obj(&polyshells, path, condition_check),
        Some("vtu") => match only_edge {
            true => output_vtk_edge(polyshells, path),
            false => output_vtk(polyshells, path),
        },
        _ => {}
    }
}

fn output_obj(
    polyshells: &[CompressedShell<Point3, PolylineCurve<Point3>, Option<PolygonMesh>>],
    path: &Path,
    condition_check: bool,
) {
    let mut polymesh = PolygonMesh::default();
    polyshells.iter().for_each(|shell| {
        let mut poly = shell.to_polygon();
        poly.put_together_same_attrs().remove_unused_attrs();
        if condition_check {
            println!("{:?}", poly.shell_condition());
        }
        polymesh.merge(poly);
    });
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

fn output_vtk_edge(
    polyshells: Vec<CompressedShell<Point3, PolylineCurve<Point3>, Option<PolygonMesh>>>,
    path: &Path,
) {
    use vtkio::model::*;
    let pieces = polyshells
        .into_iter()
        .flat_map(|shell| shell.edges)
        .map(|CompressedEdge { curve, .. }| {
            let points = curve
                .0
                .iter()
                .copied()
                .flat_map(Into::<[f64; 3]>::into)
                .collect::<Vec<_>>();
            let len = curve.0.len() as u64;
            let connectivity = (0..len).chain(vec![0, len - 1]).collect();
            Piece::Inline(Box::new(UnstructuredGridPiece {
                points: IOBuffer::F64(points),
                cells: Cells {
                    cell_verts: VertexNumbers::XML {
                        connectivity,
                        offsets: vec![len, len + 1, len + 2],
                    },
                    types: vec![CellType::PolyLine, CellType::Vertex, CellType::Vertex],
                },
                data: Default::default(),
            }))
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
