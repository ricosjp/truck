use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use truck_geometry::*;
use crate::{GeomData, GeomDataRef};
use std::result::Result;

#[derive(Serialize, Deserialize)]
enum GeomElement {
    BSplineCurve(Vec<f64>, Vec<usize>, Vec<[f64; 4]>),
    BSplineSurface(
        Vec<f64>,
        Vec<usize>,
        Vec<f64>,
        Vec<usize>,
        Vec<Vec<[f64; 4]>>,
    ),
}

pub fn read<R: Read>(reader: R) -> Result<GeomData, crate::Error> {
    let mut geomdata = GeomData::default();
    let reader = BufReader::new(reader);
    for line in reader.lines() {
        let element: GeomElement = serde_json::from_str(&(line?))?;
        match element {
            GeomElement::BSplineCurve(knots, mults, control_points) => {
                let knot_vec = KnotVec::from_single_multi(knots, mults)?;
                let control_points = control_points.into_iter().map(|x| x.into()).collect();
                geomdata
                    .curves
                    .push(BSplineCurve::try_new(knot_vec, control_points)?);
            }
            GeomElement::BSplineSurface(uknots, umults, vknots, vmults, control_points) => {
                let knot_vec0 = KnotVec::from_single_multi(uknots, umults)?;
                let knot_vec1 = KnotVec::from_single_multi(vknots, vmults)?;
                let control_points = control_points
                    .into_iter()
                    .map(|row| row.into_iter().map(|x| x.into()).collect())
                    .collect();
                geomdata.surfaces.push(BSplineSurface::try_new(
                    (knot_vec0, knot_vec1),
                    control_points,
                )?);
            }
        }
    }
    Ok(geomdata)
}

pub fn write<W: Write>(geomdata: &GeomDataRef, writer: W) -> Result<(), crate::Error> {
    let mut writer = BufWriter::new(writer);

    for curve in &geomdata.curves {
        let (knots, mults) = curve.knot_vec().to_single_multi();
        let control_points = curve
            .control_points()
            .iter()
            .map(|x| x.clone().into())
            .collect();
        let curve = GeomElement::BSplineCurve(knots, mults, control_points);
        let string = serde_json::to_string(&curve)?;
        writer.write_fmt(format_args!("{}\n", string))?;
    }

    for surface in &geomdata.surfaces {
        let (knot_vec0, knot_vec1) = surface.knot_vecs();
        let (knots0, mults0) = knot_vec0.to_single_multi();
        let (knots1, mults1) = knot_vec1.to_single_multi();
        let control_points = surface
            .control_points()
            .iter()
            .map(|row| row.iter().map(|x| x.clone().into()).collect())
            .collect();
        let surface = GeomElement::BSplineSurface(knots0, mults0, knots1, mults1, control_points);
        let string = serde_json::to_string(&surface)?;
        writer.write_fmt(format_args!("{}\n", string))?;
    }

    Ok(())
}
