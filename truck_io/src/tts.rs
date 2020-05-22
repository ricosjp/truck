use crate::Error;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use topology::WrappedUpShell;

#[derive(Serialize, Deserialize)]
enum TopoElement {
    NumberOfVertex(usize),
    Edge(usize, usize),
    Face(bool, Vec<usize>),
}

pub fn read<R: Read>(reader: R) -> Result<WrappedUpShell, Error> {
    let mut topodata = WrappedUpShell::default();
    let reader = BufReader::new(reader);
    for line in reader.lines() {
        let element: TopoElement = serde_json::from_str(&(line?))?;
        match element {
            TopoElement::NumberOfVertex(number) => topodata.number_of_vertices = number,
            TopoElement::Edge(front, back) => topodata.edges.push((front, back)),
            TopoElement::Face(ori, wire) => topodata.faces.push((ori, wire)),
        }
    }
    Ok(topodata)
}

pub fn write<W: Write>(topodata: &WrappedUpShell, writer: W) -> Result<(), Error> {
    let mut writer = BufWriter::new(writer);

    let number = TopoElement::NumberOfVertex(topodata.number_of_vertices);
    let string = serde_json::to_string(&number)?;
    writer.write_fmt(format_args!("{}\n", string))?;

    for edge in &topodata.edges {
        let edge = TopoElement::Edge(edge.0, edge.1);
        let string = serde_json::to_string(&edge)?;
        writer.write_fmt(format_args!("{}\n", string))?;
    }

    for face in &topodata.faces {
        let face = TopoElement::Face(face.0, face.1.clone());
        let string = serde_json::to_string(&face)?;
        writer.write_fmt(format_args!("{}\n", string))?;
    }

    Ok(())
}
