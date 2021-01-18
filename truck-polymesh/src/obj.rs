use crate::*;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};

/// Writes obj data to output stream
/// # Examples
/// ```
/// use truck_polymesh::*;
/// let positions = vec![
///     Point3::new(0.0, 0.0, 0.0),
///     Point3::new(1.0, 0.0, 0.0),
///     Point3::new(0.0, 1.0, 0.0),
///     Point3::new(0.0, 0.0, 1.0),
///     Point3::new(1.0, 1.0, 0.0),
///     Point3::new(1.0, 0.0, 1.0),
///     Point3::new(0.0, 1.0, 1.0),
///     Point3::new(1.0, 1.0, 1.0),
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
///     [[0, 5], [1, 5], [2, 5]],
///     [[4, 5], [2, 5], [1, 5]],
///     [[1, 4], [0, 4], [3, 4]],
///     [[1, 4], [3, 4], [5, 4]],
///     [[1, 0], [5, 0], [4, 0]],
///     [[4, 0], [5, 0], [7, 0]],
///     [[2, 1], [4, 1], [7, 1]],
///     [[2, 1], [7, 1], [6, 1]],
///     [[0, 3], [2, 3], [6, 3]],
///     [[0, 3], [6, 3], [3, 3]],
///     [[3, 2], [6, 2], [7, 2]],
///     [[3, 2], [7, 2], [5, 2]],
/// ];
/// let mesh = PolygonMesh::from_positions_and_normals(positions, normals, &faces);
/// obj::write(&mesh, std::fs::File::create("meshdata.obj").unwrap());
/// ```
pub fn write<W: Write>(mesh: &PolygonMesh, writer: W) -> Result<()> {
    sub_write(mesh, &mut BufWriter::new(writer))
}

/// Writes obj data to output stream
pub fn write_vec<W: Write>(mesh: &Vec<PolygonMesh>, writer: W) -> Result<()> {
    let mut writer = BufWriter::new(writer);
    for (i, mesh) in mesh.iter().enumerate() {
        writer.write_fmt(format_args!("g {}\n", i))?;
        sub_write(mesh, &mut writer)?;
    }
    Ok(())
}

fn write2vec<V: std::ops::Index<usize, Output = f64>, W: Write>(
    writer: &mut BufWriter<W>,
    vecs: &[V],
    prefix: &str,
) -> Result<()> {
    for vec in vecs {
        writer.write_fmt(format_args!("{} {:.7e} {:.7e}\n", prefix, vec[0], vec[1]))?;
    }
    Ok(())
}

fn write3vec<V: std::ops::Index<usize, Output = f64>, W: Write>(
    writer: &mut BufWriter<W>,
    vecs: &[V],
    prefix: &str,
) -> Result<()> {
    for vec in vecs {
        writer.write_fmt(format_args!(
            "{} {:.7e} {:.7e} {:.7e}\n",
            prefix, vec[0], vec[1], vec[2]
        ))?;
    }
    Ok(())
}

fn write_p_indices<W: Write>(writer: &mut BufWriter<W>, faces: &Faces<[usize; 1]>) -> Result<()> {
    for face in faces.face_iter() {
        writer.write(b"f")?;
        for idx in face {
            writer.write_fmt(format_args!(" {}", idx + 1))?;
        }
        writer.write(b"\n")?;
    }
    Ok(())
}

fn write_pt_indices<W: Write>(writer: &mut BufWriter<W>, faces: &Faces<[usize; 2]>) -> Result<()> {
    for face in faces.face_iter() {
        writer.write(b"f")?;
        for idx in face {
            writer.write_fmt(format_args!(" {}/{}", idx[0] + 1, idx[1] + 1))?;
        }
        writer.write(b"\n")?;
    }
    Ok(())
}

fn write_pn_indices<W: Write>(writer: &mut BufWriter<W>, faces: &Faces<[usize; 2]>) -> Result<()> {
    for face in faces.face_iter() {
        writer.write(b"f")?;
        for idx in face {
            writer.write_fmt(format_args!(" {}//{}", idx[0] + 1, idx[1] + 1))?;
        }
        writer.write(b"\n")?;
    }
    Ok(())
}

fn write_ptn_indices<W: Write>(writer: &mut BufWriter<W>, faces: &Faces<[usize; 3]>) -> Result<()> {
    for face in faces.face_iter() {
        writer.write(b"f")?;
        for idx in face {
            writer.write_fmt(format_args!(" {}/{}/{}", idx[0] + 1, idx[1] + 1, idx[2] + 1))?;
        }
        writer.write(b"\n")?;
    }
    Ok(())
}

fn sub_write<W: Write>(mesh: &PolygonMesh, writer: &mut BufWriter<W>) -> Result<()> {
    write3vec(writer, mesh.positions(), "v")?;
    write2vec(writer, mesh.uv_coords(), "vt")?;
    write3vec(writer, mesh.normals(), "vn")?;
    match mesh.faces() {
        FacesRef::Positions(faces) => write_p_indices(writer, faces),
        FacesRef::Textured(faces) => write_pt_indices(writer, faces),
        FacesRef::WithNormals(faces) => write_pn_indices(writer, faces),
        FacesRef::Complete(faces) => write_ptn_indices(writer, faces),
    }
}

/// Reads mesh data from wavefront obj file.
pub fn read<R: Read>(reader: R) -> Result<PolygonMesh> {
    let mut positions = Vec::new();
    let mut uv_coords = Vec::new();
    let mut normals = Vec::new();
    let mut faces1 = Vec::new();
    let mut faces2 = Vec::new();
    let mut faces3 = Vec::new();
    let reader = BufReader::new(reader);
    for line in reader.lines().map(|s| s.unwrap()) {
        let mut args = line.split_whitespace();
        if let Some(first_str) = args.next() {
            if first_str == "v" {
                let x = args.next().unwrap().parse::<f64>()?;
                let y = args.next().unwrap().parse::<f64>()?;
                let z = args.next().unwrap().parse::<f64>()?;
                positions.push(Point3::new(x, y, z));
            } else if first_str == "vt" {
                let u = args.next().unwrap().parse::<f64>()?;
                let v = args.next().unwrap().parse::<f64>()?;
                uv_coords.push(Vector2::new(u, v));
            } else if first_str == "vn" {
                let x = args.next().unwrap().parse::<f64>()?;
                let y = args.next().unwrap().parse::<f64>()?;
                let z = args.next().unwrap().parse::<f64>()?;
                normals.push(Vector3::new(x, y, z));
            } else if first_str == "f" {
                let mut face1 = Vec::new();
                let mut face2 = Vec::new();
                let mut face3 = Vec::new();
                for vert_str in args {
                    if &vert_str[0..1] == "#" {
                        break;
                    }
                    let mut vert = Vec::new();
                    for val in vert_str.split("/") {
                        match val.parse::<usize>() {
                            Ok(got) => vert.push(got - 1),
                            Err(_) => {}
                        }
                    }
                    match vert.len() {
                        1 => face1.push([vert[0]]),
                        2 => face2.push([vert[0], vert[1]]),
                        3 => face3.push([vert[0], vert[1], vert[2]]),
                        _ => {}
                    }
                }
                faces1.push(face1);
                faces2.push(face2);
                faces3.push(face3);
            }
        }
    }
    match (uv_coords.len(), normals.len()) {
        (0, 0) => PolygonMesh::try_from_positions(positions, &faces1),
        (_, 0) => PolygonMesh::try_from_positions_and_uvs(positions, uv_coords, &faces2),
        (0, _) => PolygonMesh::try_from_positions_and_normals(positions, normals, &faces2),
        (_, _) => PolygonMesh::try_new(positions, uv_coords, normals, &faces3),
    }
}
