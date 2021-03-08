use crate::*;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Lines, Read, Write};

fn syntax_error() -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, "syntax error")
}

/// STL naive mesh
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct STLFace {
    /// normal vector
    pub normal: [f32; 3],
    /// the positions of vertices
    pub vertices: [[f32; 3]; 3],
}

impl STLFace {
    #[inline(always)]
    fn is_empty(&self) -> bool { self == &STLFace::default() }
}

/// STL reading iterator
#[derive(Debug)]
pub enum STLReader<R: Read> {
    #[doc(hidden)]
    ASCII(Lines<BufReader<R>>),
    #[doc(hidden)]
    Binary(R, usize),
}

/// STL type
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum STLType {
    /// Determine stl type automatically.
    /// 
    /// **Reading**: if the first 5 bytes are..
    /// - "solid" => ascii format
    /// - otherwise => binary format
    /// 
    /// **Writing**: always binary format.
    Automatic,
    /// ascii format
    ASCII,
    /// binary format
    Binary,
}

impl Default for STLType {
    #[inline(always)]
    fn default() -> STLType { STLType::Automatic }
}

impl<R: Read> STLReader<R> {
    #[inline(always)]
    fn text_reader(reader: R) -> STLReader<R> { STLReader::ASCII(BufReader::new(reader).lines()) }
    fn binary_reader(mut reader: R, header_judge: bool) -> Result<STLReader<R>> {
        let mut header = [0; 5];
        reader.read(&mut header)?;
        if header_judge && &header == b"solid" {
            return Ok(Self::text_reader(reader));
        }
        let mut header = [0; 75];
        reader.read(&mut header)?;
        let mut length_bytes = [0; 4];
        reader.read(&mut length_bytes)?;
        let length = u32::from_le_bytes(length_bytes) as usize;
        Ok(STLReader::Binary(reader, length))
    }
    /// Creates new STL reader
    #[inline(always)]
    pub fn new(reader: R, stl_type: STLType) -> Result<Self> {
        match stl_type {
            STLType::Automatic => Self::binary_reader(reader, true),
            STLType::Binary => Self::binary_reader(reader, false),
            STLType::ASCII => Ok(Self::text_reader(reader)),
        }
    }
    /// Returns the STL type
    #[inline(always)]
    pub fn stl_type(&self) -> STLType {
        match self {
            STLReader::ASCII(_) => STLType::ASCII,
            STLReader::Binary(_, _) => STLType::Binary,
        }
    }
}

impl<R: Read> Iterator for STLReader<R> {
    type Item = Result<STLFace>;
    fn next(&mut self) -> Option<Self::Item> {
        let res = match self {
            STLReader::Binary(reader, length) => match *length == 0 {
                true => return None,
                false => {
                    *length -= 1;
                    binary_one_read(reader)
                }
            },
            STLReader::ASCII(lines) => ascii_one_read(lines),
        };
        match res {
            Ok(got) => match got {
                Some(got) => Some(Ok(got)),
                None => None,
            },
            Err(error) => Some(Err(error)),
        }
    }
}

fn ascii_one_read<R: BufRead>(lines: &mut Lines<R>) -> Result<Option<STLFace>> {
    let mut face = STLFace::default();
    let mut num_ver = 0;
    loop {
        let line = match lines.next() {
            Some(got) => got?,
            None => match face.is_empty() {
                true => return Ok(None),
                false => return Err(syntax_error().into()),
            },
        };
        let line = line.trim();
        if line.len() < 8 {
            continue;
        } else if &line[0..5] == "facet" {
            let args: Vec<_> = line.split_whitespace().collect();
            face.normal = [
                args[2].parse::<f32>()?,
                args[3].parse::<f32>()?,
                args[4].parse::<f32>()?,
            ];
        } else if &line[0..6] == "vertex" {
            let args: Vec<_> = line.split_whitespace().collect();
            if num_ver > 2 {
                return Err(syntax_error().into());
            }
            face.vertices[num_ver] = [
                args[1].parse::<f32>()?,
                args[2].parse::<f32>()?,
                args[3].parse::<f32>()?,
            ];
            num_ver += 1;
        } else if &line[0..8] == "endfacet" {
            if num_ver != 3 {
                return Err(syntax_error().into());
            }
            return Ok(Some(face));
        }
    }
}

#[inline(always)]
fn read_vector<R: Read>(reader: &mut R) -> Result<[f32; 3]> {
    let mut bytes = [[0; 4]; 3];
    let size0 = reader.read(&mut bytes[0])?;
    let size1 = reader.read(&mut bytes[1])?;
    let size2 = reader.read(&mut bytes[2])?;
    if size0 != 4 || size1 != 4 || size2 != 4 {
        return Err(syntax_error().into());
    }
    Ok([
        f32::from_le_bytes(bytes[0]),
        f32::from_le_bytes(bytes[1]),
        f32::from_le_bytes(bytes[2]),
    ])
}

fn binary_one_read<R: Read>(reader: &mut R) -> Result<Option<STLFace>> {
    let normal = read_vector(reader)?;
    let vertices = [
        read_vector(reader)?,
        read_vector(reader)?,
        read_vector(reader)?,
    ];
    let mut unuse = [0; 2];
    reader.read(&mut unuse)?;
    Ok(Some(STLFace { normal, vertices }))
}

/// write STL file in `stl_type` format.
/// 
/// If `stl_type == STLType::Automatic`, write the binary format.
#[inline(always)]
pub fn write<I: IntoSTLIterator, W: Write>(
    iter: I,
    writer: &mut W,
    stl_type: STLType,
) -> Result<()> {
    match stl_type {
        STLType::ASCII => write_ascii(iter, writer),
        _ => write_binary(iter, writer),
    }
}

/// Writes ASCII STL data
fn write_ascii<I: IntoSTLIterator, W: Write>(iter: I, writer: &mut W) -> Result<()> {
    let mut iter = iter.into_iter();
    writer.write(b"solid\n")?;
    iter.try_for_each::<_, Result<()>>(|face| {
        writer.write_fmt(format_args!(
            "  facet normal {:e} {:e} {:e}\n",
            face.normal[0], face.normal[1], face.normal[2]
        ))?;
        writer.write(b"    outer loop\n")?;
        face.vertices.iter().try_for_each(|pt| {
            writer.write_fmt(format_args!(
                "      vertex {:e} {:e} {:e}\n",
                pt[0], pt[1], pt[2]
            ))
        })?;
        writer.write(b"    endloop\n  endfacet\n")?;
        Ok(())
    })?;
    writer.write(b"endsolid\n")?;
    Ok(())
}

/// Writes binary STL data
#[inline(always)]
fn write_binary<I: IntoSTLIterator, W: Write>(iter: I, writer: &mut W) -> Result<()> {
    let mut iter = iter.into_iter();
    let len = iter.len() as u32;
    writer.write(&[0u8; 80])?;
    writer.write(&len.to_le_bytes())?;
    iter.try_for_each(|face| {
        writer.write(bytemuck::cast_slice(&[face.normal]))?;
        writer.write(bytemuck::cast_slice(&face.vertices))?;
        writer.write(&[0u8, 0u8])?;
        Ok(())
    })
}

/// By implementing `IntoSTLIterator` for a type, you define how it will be converted to an iterator.
/// This is common for types which describe a collection of some kind.
pub trait IntoSTLIterator {
    /// Which kind of iterator are we turning this into?
    type IntoIter: ExactSizeIterator<Item = STLFace>;
    /// Creates an iterator from a value.
    fn into_iter(self) -> Self::IntoIter;
}

/// STL face generate from `PolygonMesh`
#[derive(Debug)]
pub struct PolygonMeshSTLFaceIterator<'a> {
    positions: &'a Vec<Point3>,
    tri_faces: std::slice::Iter<'a, [Vertex; 3]>,
    quad_faces: std::slice::Iter<'a, [Vertex; 4]>,
    other_faces: std::slice::Iter<'a, Vec<Vertex>>,
    current_face: Option<&'a [Vertex]>,
    current_vertex: usize,
    len: usize,
}

#[inline(always)]
fn pos_to_face(a: Point3, b: Point3, c: Point3) -> STLFace {
    let normal = (b - a).cross(c - a).normalize().cast().unwrap().into();
    let vertices = [
        a.cast().unwrap().into(),
        b.cast().unwrap().into(),
        c.cast().unwrap().into(),
    ];
    STLFace { normal, vertices }
}

impl<'a> Iterator for PolygonMeshSTLFaceIterator<'a> {
    type Item = STLFace;
    fn next(&mut self) -> Option<STLFace> {
        if self.current_face.is_none() {
            self.current_face = if let Some(face) = self.tri_faces.next() {
                Some(face)
            } else if let Some(face) = self.quad_faces.next() {
                Some(face)
            } else if let Some(face) = self.other_faces.next() {
                Some(face)
            } else {
                return None;
            }
        }
        let face = self.current_face.unwrap();
        let res = pos_to_face(
            self.positions[face[0].pos],
            self.positions[face[self.current_vertex + 1].pos],
            self.positions[face[self.current_vertex + 2].pos],
        );
        if self.current_vertex + 3 == face.len() {
            self.current_face = None;
            self.current_vertex = 0;
        } else {
            self.current_vertex += 1;
        }
        self.len -= 1;
        Some(res)
    }
    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) { (self.len, Some(self.len)) }
}

impl<'a> ExactSizeIterator for PolygonMeshSTLFaceIterator<'a> {}

impl<'a> IntoSTLIterator for &'a PolygonMesh {
    type IntoIter = PolygonMeshSTLFaceIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        let len = self.face_iter().fold(0, |len, face| len + face.len());
        Self::IntoIter {
            positions: self.positions(),
            tri_faces: self.tri_faces().iter(),
            quad_faces: self.quad_faces().iter(),
            other_faces: self.other_faces().iter(),
            current_face: None,
            current_vertex: 0,
            len,
        }
    }
}

impl<I> IntoSTLIterator for I
where
    I: IntoIterator<Item = STLFace>,
    I::IntoIter: ExactSizeIterator,
{
    type IntoIter = I::IntoIter;
    fn into_iter(self) -> I::IntoIter { self.into_iter() }
}

fn signup_vector(vector: [f32; 3], map: &mut HashMap<[i64; 3], usize>) -> usize {
    let vector = [
        ((vector[0] as f64 + TOLERANCE * 0.25) / (TOLERANCE * 0.5)) as i64,
        ((vector[1] as f64 + TOLERANCE * 0.25) / (TOLERANCE * 0.5)) as i64,
        ((vector[2] as f64 + TOLERANCE * 0.25) / (TOLERANCE * 0.5)) as i64,
    ];
    match map.get(&vector) {
        Some(res) => *res,
        None => {
            let res = map.len();
            map.insert(vector, res);
            res
        }
    }
}

impl std::iter::FromIterator<STLFace> for PolygonMesh {
    fn from_iter<I: IntoIterator<Item = STLFace>>(iter: I) -> PolygonMesh {
        let mut positions = HashMap::<[i64; 3], usize>::new();
        let mut normals = HashMap::<[i64; 3], usize>::new();
        let faces: Vec<[Vertex; 3]> = iter
            .into_iter()
            .map(|face| {
                let n = signup_vector(face.normal, &mut normals);
                let p = [
                    signup_vector(face.vertices[0], &mut positions),
                    signup_vector(face.vertices[1], &mut positions),
                    signup_vector(face.vertices[2], &mut positions),
                ];
                [
                    (p[0], None, Some(n)).into(),
                    (p[1], None, Some(n)).into(),
                    (p[2], None, Some(n)).into(),
                ]
            })
            .collect();
        let faces = Faces::from_tri_and_quad_faces(faces, Vec::new());
        let mut positions: Vec<([i64; 3], usize)> = positions.into_iter().collect();
        positions.sort_by(|a, b| a.1.cmp(&b.1));
        let positions: Vec<Point3> = positions
            .into_iter()
            .map(|(p, _)| {
                Point3::new(
                    p[0] as f64 * TOLERANCE * 0.5,
                    p[1] as f64 * TOLERANCE * 0.5,
                    p[2] as f64 * TOLERANCE * 0.5,
                )
            })
            .collect();
        let mut normals: Vec<([i64; 3], usize)> = normals.into_iter().collect();
        normals.sort_by(|a, b| a.1.cmp(&b.1));
        let normals: Vec<Vector3> = normals
            .into_iter()
            .map(|(p, _)| {
                Vector3::new(
                    p[0] as f64 * TOLERANCE * 0.5,
                    p[1] as f64 * TOLERANCE * 0.5,
                    p[2] as f64 * TOLERANCE * 0.5,
                )
            })
            .collect();
        PolygonMesh::debug_new(positions, Vec::new(), normals, faces)
    }
}

/// Read STL file and parse to `PolygonMesh`.
#[inline(always)]
pub fn read<P: AsRef<std::path::Path>>(path: P, stl_type: STLType) -> Result<PolygonMesh> {
    STLReader::new(std::fs::File::open(path)?, stl_type)?.collect()
}
