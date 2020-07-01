use crate::Error;
use polymesh::PolygonMesh;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};

/// write obj data to output stream
/// # Examples
/// ```
/// use truck_polymesh::PolygonMesh;
/// use truck_geometry::{Vector2, Vector3};
/// let positions = vec![
///     Vector3::new(0.0, 0.0, 0.0),
///     Vector3::new(1.0, 0.0, 0.0),
///     Vector3::new(0.0, 1.0, 0.0),
///     Vector3::new(0.0, 0.0, 1.0),
///     Vector3::new(1.0, 1.0, 0.0),
///     Vector3::new(1.0, 0.0, 1.0),
///     Vector3::new(0.0, 1.0, 1.0),
///     Vector3::new(1.0, 1.0, 1.0),
/// ];
/// let normals = vec![
///     Vector3::new(1.0, 0.0, 0.0),
///     Vector3::new(0.0, 1.0, 0.0),
///     Vector3::new(0.0, 0.0, 1.0),
///     Vector3::new(-1.0, 0.0, 0.0),
///     Vector3::new(0.0, -1.0, 0.0),
///     Vector3::new(0.0, 0.0, -1.0),
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
///     positions: positions,
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
    for vertex in &mesh.positions {
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
        let mut args = line.split_whitespace();
        if let Some(first_str) = args.next() {
            if first_str == "v" {
                let x = args.next().unwrap().parse::<f64>()?;
                let y = args.next().unwrap().parse::<f64>()?;
                let z = args.next().unwrap().parse::<f64>()?;
                mesh.positions.push(vector!(x, y, z));
            } else if first_str == "vt" {
                let u = args.next().unwrap().parse::<f64>()?;
                let v = args.next().unwrap().parse::<f64>()?;
                mesh.uv_coords.push(vector!(u, v));
            } else if first_str == "vn" {
                let x = args.next().unwrap().parse::<f64>()?;
                let y = args.next().unwrap().parse::<f64>()?;
                let z = args.next().unwrap().parse::<f64>()?;
                mesh.normals.push(vector!(x, y, z));
            } else if first_str == "f" {
                let mut face = Vec::new();
                for vert_str in args {
                    if &vert_str[0..1] == "#" {
                        break;
                    }
                    let mut vert = Vec::new();
                    for val in vert_str.split("/") {
                        vert.push(val.parse::<usize>().unwrap_or(1) - 1);
                    }
                    vert.resize(3, 0);
                    face.push([vert[0], vert[1], vert[2]]);
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
    }
    Ok(mesh)
}
