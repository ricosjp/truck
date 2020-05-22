use crate::Error;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::convert::TryInto;
use polymesh::PolygonMesh;

/// write obj data to output stream
/// # Examples
/// ```
/// use truck_polymesh::PolygonMesh;
/// let vertices = vec![
///     [0.0, 0.0, 0.0],
///     [1.0, 0.0, 0.0],
///     [0.0, 1.0, 0.0],
///     [0.0, 0.0, 1.0],
///     [1.0, 1.0, 0.0],
///     [1.0, 0.0, 1.0],
///     [0.0, 1.0, 1.0],
///     [1.0, 1.0, 1.0],
/// ];
/// let normals = vec![
///     [1.0, 0.0, 0.0],
///     [0.0, 1.0, 0.0],
///     [0.0, 0.0, 1.0],
///     [-1.0, 0.0, 0.0],
///     [0.0, -1.0, 0.0],
///     [0.0, 0.0, -1.0],
/// ];
/// let faces = vec![
///     [[0, 0, 5], [1, 0, 5], [2, 0, 5]],
///     [[4, 0, 5], [2, 0, 5], [1, 0, 5]],
///     [[1, 0, 4], [0, 0, 4], [3, 0, 4]],
///     [[1, 0, 4], [3, 0, 4], [5, 0, 4]],
///     [[1, 0, 0], [5, 0, 0], [4, 0, 0]],
///     [[4, 0, 0], [5, 0, 0], [7, 0, 0]],
///     [[2, 0, 1], [4, 0, 1], [7, 0, 1]],
///     [[2, 0, 1], [7, 0, 1], [6, 0, 1]],
///     [[0, 0, 3], [2, 0, 3], [6, 0, 3]],
///     [[0, 0, 3], [6, 0, 3], [3, 0, 3]],
///     [[3, 0, 2], [6, 0, 2], [7, 0, 2]],
///     [[3, 0, 2], [7, 0, 2], [5, 0, 2]],
/// ];
/// let mesh = PolygonMesh {
///     vertices: vertices,
///     uv_coords: Vec::new(),
///     normals: normals,
///     tri_faces: faces,
///     quad_faces: Vec::new(),
///     other_faces: Vec::new(),
/// };
/// truck_io::obj::write(&mesh, std::fs::File::create("meshdata.obj").unwrap());
/// ```
pub fn write<W: Write>(mesh: &PolygonMesh, writer: W) -> Result<(), Error> {
    crate::obj::sub_write(mesh, &mut BufWriter::new(writer))
}

pub fn write_vec<W: Write>(mesh: &Vec<PolygonMesh>, writer: W) -> Result<(), Error> {
    let mut writer = BufWriter::new(writer);
    for (i, mesh) in mesh.iter().enumerate() {
        writer.write_fmt(format_args!("g {}\n", i))?;
        sub_write(mesh, &mut writer)?;
    }
    Ok(())
}

fn sub_write<W: Write>(mesh: &PolygonMesh, writer: &mut BufWriter<W>) -> Result<(), Error> {
    for vertex in &mesh.vertices {
        writer.write_fmt(format_args!(
            "v {:.7e} {:.7e} {:.7e}\n",
            vertex[0], vertex[1], vertex[2]
        ))?;
    }
    for uv in &mesh.uv_coords {
        writer.write_fmt(format_args!("vt {:.7e} {:.7e}\n", uv[0], uv[1]))?;
    }
    for normal in &mesh.normals {
        writer.write_fmt(format_args!(
            "vn {:.7e} {:.7e} {:.7e}\n",
            normal[0], normal[1], normal[2]
        ))?;
    }
    if mesh.uv_coords.is_empty() {
        if mesh.normals.is_empty() {
            for face in &mesh.tri_faces {
                writer.write_fmt(format_args!(
                    "f {} {} {}\n",
                    face[0][0] + 1,
                    face[1][0] + 1,
                    face[2][0] + 1
                ))?;
            }
            for face in &mesh.quad_faces {
                writer.write_fmt(format_args!(
                    "f {} {} {} {}\n",
                    face[0][0] + 1,
                    face[1][0] + 1,
                    face[2][0] + 1,
                    face[3][0] + 1,
                ))?;
            }
        } else {
            for face in &mesh.tri_faces {
                writer.write_fmt(format_args!(
                    "f {}//{} {}//{} {}//{}\n",
                    face[0][0] + 1,
                    face[0][2] + 1,
                    face[1][0] + 1,
                    face[1][2] + 1,
                    face[2][0] + 1,
                    face[2][2] + 1
                ))?;
            }
            for face in &mesh.quad_faces {
                writer.write_fmt(format_args!(
                    "f {}//{} {}//{} {}//{} {}//{}\n",
                    face[0][0] + 1,
                    face[0][2] + 1,
                    face[1][0] + 1,
                    face[1][2] + 1,
                    face[2][0] + 1,
                    face[2][2] + 1,
                    face[3][0] + 1,
                    face[3][2] + 1
                ))?;
            }
        }
    } else {
        if mesh.normals.is_empty() {
            for face in &mesh.tri_faces {
                writer.write_fmt(format_args!(
                    "f {}/{} {}/{} {}/{}\n",
                    face[0][0] + 1,
                    face[0][1] + 1,
                    face[1][0] + 1,
                    face[1][1] + 1,
                    face[2][0] + 1,
                    face[2][1] + 1
                ))?;
            }
            for face in &mesh.quad_faces {
                writer.write_fmt(format_args!(
                    "f {}/{} {}/{} {}/{} {}/{}\n",
                    face[0][0] + 1,
                    face[0][1] + 1,
                    face[1][0] + 1,
                    face[1][1] + 1,
                    face[2][0] + 1,
                    face[2][1] + 1,
                    face[3][0] + 1,
                    face[3][1] + 1
                ))?;
            }
        } else {
            for face in &mesh.tri_faces {
                writer.write_fmt(format_args!(
                    "f {}/{}/{} {}/{}/{} {}/{}/{}\n",
                    face[0][0] + 1,
                    face[0][1] + 1,
                    face[0][2] + 1,
                    face[1][0] + 1,
                    face[1][1] + 1,
                    face[1][2] + 1,
                    face[2][0] + 1,
                    face[2][1] + 1,
                    face[2][2] + 1
                ))?;
            }
            for face in &mesh.quad_faces {
                writer.write_fmt(format_args!(
                    "f {}/{}/{} {}/{}/{} {}/{}/{} {}/{}/{}\n",
                    face[0][0] + 1,
                    face[0][1] + 1,
                    face[0][2] + 1,
                    face[1][0] + 1,
                    face[1][1] + 1,
                    face[1][2] + 1,
                    face[2][0] + 1,
                    face[2][1] + 1,
                    face[2][2] + 1,
                    face[3][0] + 1,
                    face[3][1] + 1,
                    face[3][2] + 1
                ))?;
            }
        }
    }
    Ok(())
}

pub fn read<R: Read>(reader: R) -> Result<PolygonMesh, Error> {
    let mut mesh = PolygonMesh::default();
    let reader = BufReader::new(reader);
    for line in reader.lines().map(|s| s.unwrap()) {
        let args: Vec<_> = line.split_whitespace().collect();
        if args.is_empty() {
            continue;
        } else if args[0] == "v" {
            mesh.vertices
                .push([args[1].parse()?, args[2].parse()?, args[3].parse()?].into());
        } else if args[0] == "vt" {
            mesh.uv_coords.push([args[1].parse()?, args[2].parse()?].into());
        } else if args[0] == "vn" {
            mesh.normals
                .push([args[1].parse()?, args[2].parse()?, args[3].parse()?].into());
        } else if args[0] == "f" {
            let mut face = Vec::new();
            for i in 1..args.len() {
                if &args[i][0..1] == "#" {
                    break;
                }
                let mut vert = Vec::new();
                for val in args[i].split("/") {
                    if val.len() == 0 {
                        vert.push(0);
                    } else {
                        vert.push(val.parse::<usize>()? - 1);
                    }
                }
                vert.resize(3, 0);
                face.push(vert.as_slice().try_into().unwrap());
            }
            if face.len() == 3 {
                mesh.tri_faces.push([face[0], face[1], face[2]]);
            } else if face.len() == 4 {
                mesh.quad_faces.push([face[0], face[1], face[2], face[3]]);
            } else {
                mesh.other_faces.push(face);
            }
        }
    }
    Ok(mesh)
}
