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
/// let faces = Faces::from_iter(&[
///     [(0, None, Some(5)), (1, None, Some(5)), (2, None, Some(5))],
///     [(4, None,Some(5)), (2, None, Some(5)), (1, None, Some(5))],
///     [(1, None, Some(4)), (0, None, Some(4)), (3, None, Some(4))],
///     [(1, None, Some(4)), (3, None, Some(4)), (5, None, Some(4))],
///     [(1, None, Some(0)), (5, None, Some(0)), (4, None, Some(0))],
///     [(4, None, Some(0)), (5, None, Some(0)), (7, None, Some(0))],
///     [(2, None, Some(1)), (4, None, Some(1)), (7, None, Some(1))],
///     [(2, None, Some(1)), (7, None, Some(1)), (6, None, Some(1))],
///     [(0, None, Some(3)), (2, None, Some(3)), (6, None, Some(3))],
///     [(0, None, Some(3)), (6, None, Some(3)), (3, None, Some(3))],
///     [(3, None, Some(2)), (6, None, Some(2)), (7, None, Some(2))],
///     [(3, None, Some(2)), (7, None, Some(2)), (5, None, Some(2))],
/// ]);
/// let mesh = PolygonMesh::new(positions, Vec::new(), normals, faces);
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

impl Vertex {
    fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match (self.uv, self.nor) {
            (None, None) => writer.write_fmt(format_args!("{}", self.pos + 1)),
            (Some(uv), None) => writer.write_fmt(format_args!("{}/{}", self.pos + 1, uv + 1)),
            (None, Some(nor)) => writer.write_fmt(format_args!("{}//{}", self.pos + 1, nor + 1)),
            (Some(uv), Some(nor)) => {
                writer.write_fmt(format_args!("{}/{}/{}", self.pos + 1, uv + 1, nor + 1))
            }
        }
    }
}

impl Faces {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        for face in self.face_iter() {
            writer.write(b"f")?;
            for v in face {
                writer.write(b" ")?;
                v.write(writer)?;
            }
            writer.write(b"\n")?;
        }
        Ok(())
    }
}

fn sub_write<W: Write>(mesh: &PolygonMesh, writer: &mut BufWriter<W>) -> Result<()> {
    write3vec(writer, mesh.positions(), "v")?;
    write2vec(writer, mesh.uv_coords(), "vt")?;
    write3vec(writer, mesh.normals(), "vn")?;
    mesh.faces.write(writer)
}

/// Reads mesh data from wavefront obj file.
pub fn read<R: Read>(reader: R) -> Result<PolygonMesh> {
    let mut positions = Vec::new();
    let mut uv_coords = Vec::new();
    let mut normals = Vec::new();
    let mut faces = Faces::default();
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
                let mut face = Vec::new();
                for vert_str in args {
                    if &vert_str[0..1] == "#" {
                        break;
                    }
                    let mut iter = vert_str.split("/");
                    let pos = iter
                        .next()
                        .map(|val| val.parse::<usize>().ok())
                        .unwrap_or(None);
                    let uv = iter
                        .next()
                        .map(|val| val.parse::<usize>().ok())
                        .unwrap_or(None);
                    let nor = iter
                        .next()
                        .map(|val| val.parse::<usize>().ok())
                        .unwrap_or(None);
                    let vert = match (pos, uv, nor) {
                        (None, _, _) => continue,
                        (Some(pos), uv, nor) => Vertex { pos, uv, nor },
                    };
                    face.push(vert);
                }
                faces.push(face);
            }
        }
    }
    PolygonMesh::try_new(positions, uv_coords, normals, faces)
}
