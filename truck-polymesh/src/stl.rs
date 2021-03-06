#![allow(dead_code)]

use crate::*;
use std::io::{BufRead, BufReader, Lines, Read, Write};

/// STL naive mesh
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct STLFace {
    /// normal vector
    pub normal: [f32; 3],
    /// the positions of vertices
    pub vertices: Vec<[f32; 3]>,
}

impl STLFace {
    #[inline(always)]
    fn is_empty(&self) -> bool { self.normal == [0.0, 0.0, 0.0] && self.vertices.is_empty() }
}

/// STL reading iterator
#[derive(Debug)]
pub enum STLReader<R: Read> {
    /// ASCII binary iterator
    ASCII(Lines<BufReader<R>>),
    /// Binary iterator
    Binary(R, usize),
}

/// STL type
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum STLType {
    /// Read the first word and determine stl type automatically.
    Automatic,
    /// text format
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
    /// Cretes new STL reader
    #[inline(always)]
    pub fn new(reader: R, stl_type: STLType) -> Result<Self> {
        match stl_type {
            STLType::Automatic => Self::binary_reader(reader, true),
            STLType::Binary => Self::binary_reader(reader, false),
            STLType::ASCII => Ok(Self::text_reader(reader)),
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
    loop {
        let line = match lines.next() {
            Some(got) => got?,
            None => match face.is_empty() {
                true => return Ok(None),
                false => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "syntax error",
                    )
                    .into())
                }
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
            let position = [
                args[1].parse::<f32>()?,
                args[2].parse::<f32>()?,
                args[3].parse::<f32>()?,
            ];
            face.vertices.push(position);
        } else if &line[0..8] == "endfacet" {
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
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "syntax error").into());
    }
    Ok([
        f32::from_le_bytes(bytes[0]),
        f32::from_le_bytes(bytes[1]),
        f32::from_le_bytes(bytes[2]),
    ])
}

fn binary_one_read<R: Read>(reader: &mut R) -> Result<Option<STLFace>> {
    let normal = read_vector(reader)?;
    let vertices = vec![
        read_vector(reader)?,
        read_vector(reader)?,
        read_vector(reader)?,
    ];
    let mut unuse = [0; 2];
    reader.read(&mut unuse)?;
    Ok(Some(STLFace { normal, vertices }))
}

/// Writes ASCII STL data
pub fn write_ascii<'a, I: Iterator<Item = STLFace>, W: Write>(
    mut iter: I,
    writer: &mut W,
) -> Result<()> {
    writer.write(b"solid\n")?;
    iter.try_for_each(|face| {
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
    })
}

/// Writes binary STL data
pub fn write_binary<'a, I: ExactSizeIterator<Item = STLFace>, W: Write>(
    mut iter: I,
    writer: &mut W,
) -> Result<()> {
    writer.write(&[0u8; 80])?;
    writer.write(&(iter.len() as u32).to_le_bytes())?;
    iter.try_for_each(|face| {
        if face.vertices.len() != 3 {
            return Err(
                std::io::Error::new(std::io::ErrorKind::InvalidData, "syntax error").into(),
            );
        }
        writer.write(bytemuck::cast_slice(&[face.normal]))?;
        writer.write(bytemuck::cast_slice(&face.vertices))?;
        writer.write(&[0u8, 0u8])?;
        Ok(())
    })
}

/// STL face generate from `PolygonMesh`
#[derive(Debug)]
pub struct PolygonMeshSTLFaceIterator<'a> {
    polymesh: &'a PolygonMesh,
    current: &'a [Vertex],
    triangulate: Option<usize>,
}

/// Position merge mode
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum MergeMode {
    /// Do not merge, fast but with lots of wasted memory.
    Naive,
    /// merge near positions.
    ///
    /// [`TOLERANCE`](https://docs.rs/truck-base/0.1.1/truck_base/tolerance/constant.TOLERANCE.html)
    Merge {
        /// tolerance for merge
        tolerance: f64,
        /// tolerance is absolute or not.
        relative: bool,
    },
}

/// STL reading descriptor
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct STLReadDescriptor {
    /// stl format type: text or binary
    pub stl_type: STLType,
    /// parsing
    pub merge: MergeMode,
}
