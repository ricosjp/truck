//! Parse STEP data, extract shape, and meshing.

use clap::Parser;
use std::path::Path;
use truck_assembly::assy::*;
use truck_meshalgo::prelude::*;
use truck_stepio::r#in::*;
use truck_topology::compress::*;

type MeshedCShell = CompressedShell<Point3, PolylineCurve<Point3>, Option<PolygonMesh>>;

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
    let step_file = std::fs::read_to_string(input_step_file).unwrap();
    let table = Table::from_step(&step_file).unwrap();
    println!("meshing...");
    let polyshells = step_to_mesh(&table);

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

fn step_to_mesh<'a>(table: &Table) -> Vec<MeshedCShell> {
    let assy = table.step_assy().unwrap();

    let node_map = |ProductEntity {
                        shape: indices,
                        attrs: name,
                    }: &ProductEntity| {
        let shape = indices
            .iter()
            .filter_map(|idx| {
                let shells = if let Some(step_solid) = table.manifold_solid_brep.get(idx) {
                    table
                        .to_compressed_solid(step_solid)
                        .map_err(|err| eprintln!("failed to convert solid: {err}"))
                        .ok()?
                        .boundaries
                } else if let Some(step_shells) = table.shell_based_surface_model.get(idx) {
                    table
                        .to_compressed_shells(step_shells)
                        .map_err(|err| eprintln!("failed to convert shells: {err}"))
                        .ok()?
                } else {
                    return None;
                };
                let meshed_shells = shells
                    .into_iter()
                    .map(|shell| {
                        let pre = shell.robust_triangulation(0.01).to_polygon();
                        let bdd = pre.bounding_box();
                        shell.robust_triangulation(bdd.diameter() * 0.001)
                    })
                    .collect::<Vec<_>>();
                Some(meshed_shells)
            })
            .collect::<Vec<_>>();
        NodeEntity {
            shape,
            attrs: name.clone(),
        }
    };
    let edge_map = |EdgeEntity { matrix: trans, .. }: &AssembleEntity| EdgeEntity {
        matrix: Matrix4::try_from(trans).unwrap(),
        attrs: (),
    };

    let meshed_assy = assy.par_map(node_map, edge_map);

    meshed_assy
        .top_nodes()
        .into_iter()
        .flat_map(|top| meshed_assy.paths_iter(top.index()))
        .flat_map(|path| {
            let matrix = path.matrix();
            let shapes = path.terminal_node().shape().clone();
            shapes.into_iter().flatten().map(move |mut shell| {
                shell.vertices.iter_mut().for_each(|v| {
                    *v = matrix.transform_point(*v);
                });
                shell.edges.iter_mut().for_each(|edge| {
                    edge.curve.transform_by(matrix);
                });
                shell.faces.iter_mut().for_each(|face| {
                    face.surface
                        .as_mut()
                        .map(|surface| surface.transform_by(matrix));
                });
                shell
            })
        })
        .collect()
}

fn output_obj(polyshells: &[MeshedCShell], path: &Path, condition_check: bool) {
    let mut polymesh = PolygonMesh::default();
    polyshells.iter().for_each(|shell| {
        let mut poly = shell.to_polygon();
        poly.put_together_same_attrs(TOLERANCE * 50.0)
            .remove_degenerate_faces()
            .remove_unused_attrs();
        if condition_check {
            println!("{:?}", poly.shell_condition());
        }
        polymesh.merge(poly);
    });
    let obj_file = std::fs::File::create(path).unwrap();
    obj::write(&polymesh, obj_file).unwrap();
}

fn output_vtk(polyshells: Vec<MeshedCShell>, path: &Path) {
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

    let vtk_file = std::fs::File::create(path).unwrap();
    vtk.write_xml(vtk_file).unwrap();
}

fn output_vtk_edge(polyshells: Vec<MeshedCShell>, path: &Path) {
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

    let vtk_file = std::fs::File::create(path).unwrap();
    vtk.write_xml(vtk_file).unwrap();
}
