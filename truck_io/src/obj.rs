use crate::Error;
use extern_obj::{Obj, SimplePolygon};
use std::path::Path;
use std::io::{BufWriter, Write};
use truck_polymesh::PolygonMesh;

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
                    "f {} {} {}\n",
                    face[0][0] + 1,
                    face[1][0] + 1,
                    face[3][0] + 1,
                ))?;
                writer.write_fmt(format_args!(
                    "f {} {} {}\n",
                    face[2][0] + 1,
                    face[3][0] + 1,
                    face[1][0] + 1,
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
                    "f {}//{} {}//{} {}//{}\n",
                    face[0][0] + 1,
                    face[0][2] + 1,
                    face[1][0] + 1,
                    face[1][2] + 1,
                    face[3][0] + 1,
                    face[3][2] + 1
                ))?;
                writer.write_fmt(format_args!(
                    "f {}//{} {}//{} {}//{}\n",
                    face[2][0] + 1,
                    face[2][2] + 1,
                    face[3][0] + 1,
                    face[3][2] + 1,
                    face[1][0] + 1,
                    face[1][2] + 1
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
                    "f {}/{} {}/{} {}/{}\n",
                    face[0][0] + 1,
                    face[0][1] + 1,
                    face[1][0] + 1,
                    face[1][1] + 1,
                    face[3][0] + 1,
                    face[3][1] + 1
                ))?;
                writer.write_fmt(format_args!(
                    "f {}/{} {}/{} {}/{}\n",
                    face[2][0] + 1,
                    face[2][1] + 1,
                    face[3][0] + 1,
                    face[3][1] + 1,
                    face[1][0] + 1,
                    face[1][1] + 1
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
                    "f {}/{}/{} {}/{}/{} {}/{}/{}\n",
                    face[0][0] + 1,
                    face[0][1] + 1,
                    face[0][2] + 1,
                    face[1][0] + 1,
                    face[1][1] + 1,
                    face[1][2] + 1,
                    face[3][0] + 1,
                    face[3][1] + 1,
                    face[3][2] + 1
                ))?;
                writer.write_fmt(format_args!(
                    "f {}/{}/{} {}/{}/{} {}/{}/{}\n",
                    face[2][0] + 1,
                    face[2][1] + 1,
                    face[2][2] + 1,
                    face[3][0] + 1,
                    face[3][1] + 1,
                    face[3][2] + 1,
                    face[1][0] + 1,
                    face[1][1] + 1,
                    face[1][2] + 1
                ))?;
            }
        }
    }
    Ok(())
}

/// read obj data to output stream
pub fn read<P: AsRef<Path>>(path: &P) -> Result<PolygonMesh, Error> {
    let obj = Obj::<SimplePolygon>::load(path.as_ref())?;
    let vertices: Vec<[f64; 3]> = obj.position
        .iter()
        .map(|x| [x[0] as f64, x[1] as f64, x[2] as f64])
        .collect();
    let uv_coords: Vec<[f64; 2]> = obj.texture
        .iter()
        .map(|x| [x[0] as f64, x[1] as f64])
        .collect();
    let normals: Vec<[f64; 3]> = obj.normal
        .iter()
        .map(|x| [x[0] as f64, x[1] as f64, x[2] as f64])
        .collect();

    let mut faces = Vec::new();
    for object in obj.objects.iter() {
        for grp in object.groups.iter() {
            for tri in grp.polys.iter() {
                let face = [
                    [tri[0].0, tri[0].1.unwrap_or(0), tri[0].2.unwrap_or(0)],
                    [tri[1].0, tri[1].1.unwrap_or(0), tri[1].2.unwrap_or(0)],
                    [tri[2].0, tri[2].1.unwrap_or(0), tri[2].2.unwrap_or(0)],
                ];
                faces.push(face);
            }
        }
    }

    Ok(PolygonMesh {
        vertices: vertices,
        uv_coords: uv_coords,
        normals: normals,
        tri_faces: faces,
        quad_faces: Vec::new(),
    })
}
