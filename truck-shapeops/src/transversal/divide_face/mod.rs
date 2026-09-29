#![allow(clippy::many_single_char_names)]

use super::faces_classification::FacesClassification;
use super::loops_store::*;
use rustc_hash::FxHashMap as HashMap;
use std::ops::Deref;
use truck_meshalgo::prelude::*;
use truck_topology::*;

fn create_parameter_boundary<P, C, S>(
    face: &Face<P, C, S>,
    wire: &Wire<P, C>,
    polys: &mut HashMap<EdgeID<C>, PolylineCurve<P>>,
    tol: f64,
) -> Option<PolylineCurve<Point2>>
where
    P: Copy,
    C: BoundedCurve<Point = P> + ParameterDivision1D<Point = P>,
    S: Clone + SearchParameter<D2, Point = P>,
{
    let surface = face.surface();
    let pt = wire.front_vertex().unwrap().point();
    let p: Point2 = surface.search_parameter(pt, None, 100)?.into();
    let vec = wire.edge_iter().try_fold(vec![p], |mut vec, edge| {
        let poly = polys.entry(edge.id()).or_insert_with(|| {
            let curve = edge.curve();
            let div = curve.parameter_division(curve.range_tuple(), tol).1;
            PolylineCurve(div)
        });
        let mut p = *vec.last().unwrap();
        let closure = |q: &P| -> Option<Point2> {
            p = surface.search_parameter(*q, Some(p.into()), 100)?.into();
            Some(p)
        };
        let add: Option<Vec<Point2>> = match edge.orientation() {
            true => poly.iter().skip(1).map(closure).collect(),
            false => poly.iter().rev().skip(1).map(closure).collect(),
        };
        vec.append(&mut add?);
        Some(vec)
    })?;
    Some(PolylineCurve(vec))
}

#[derive(Clone, Debug)]
struct WireChunk<'a, C> {
    poly: PolylineCurve<Point2>,
    wire: &'a BoundaryWire<Point3, C>,
}

type FaceWithShapesOpStatus<C, S> = (Face<Point3, C, S>, ShapesOpStatus);
fn divide_one_face<C, S>(
    face: &Face<Point3, C, S>,
    loops: &Loops<Point3, C>,
    tol: f64,
) -> Option<Vec<FaceWithShapesOpStatus<C, S>>>
where
    C: BoundedCurve<Point = Point3> + ParameterDivision1D<Point = Point3>,
    S: Clone + SearchParameter<D2, Point = Point3>,
{
    let (mut pre_faces, mut negative_wires) = (Vec::new(), Vec::new());
    let mut map = HashMap::default();
    loops.iter().try_for_each(|wire| {
        let poly = create_parameter_boundary(face, wire, &mut map, tol)?;
        let area = poly.area();
        if area.abs() < tol {
            // eprintln!("Ignored small area loop: {:?}", area);
            return Some(());
        }
        match area > 0.0 {
            true => pre_faces.push(vec![WireChunk { poly, wire }]),
            false => negative_wires.push(WireChunk { poly, wire }),
        }
        Some(())
    })?;
    negative_wires.into_iter().try_for_each(|chunk| {
        let pt = chunk.poly.front();
        let idx = pre_faces.iter().position(|face| face[0].poly.include(pt));
        if let Some(i) = idx {
            let outer_area = pre_faces[i][0].poly.area();
            let chunk_area = chunk.poly.area();
            // access to the same area with opposite orientation
            // if the sum of areas is zero, the face is canceled.
            // This happens when an intersection loop exactly matches the face boundary.
            if (outer_area + chunk_area).abs() < tol {
                pre_faces[i].clear();
            } else {
                pre_faces[i].push(chunk);
            }
        }
        Some(())
    })?;
    let vec: Vec<_> = pre_faces
        .into_iter()
        .filter(|pre_face| !pre_face.is_empty())
        .map(|pre_face| {
            let surface = face.surface();
            let op = pre_face
                .iter()
                .find(|chunk| chunk.wire.status() != ShapesOpStatus::Unknown);
            let status = match op {
                Some(chunk) => chunk.wire.status(),
                None => ShapesOpStatus::Unknown,
            };
            let wires: Vec<Wire<Point3, C>> = pre_face
                .into_iter()
                .map(|chunk| chunk.wire.deref().clone())
                .collect();
            let mut new_face = Face::debug_new(wires, surface);
            if !face.orientation() {
                new_face.invert();
            }
            (new_face, status)
        })
        .collect();
    Some(vec)
}

pub fn divide_faces<C, S>(
    shell: &Shell<Point3, C, S>,
    loops_store: &LoopsStore<Point3, C>,
    tol: f64,
) -> Option<FacesClassification<Point3, C, S>>
where
    C: BoundedCurve<Point = Point3> + ParameterDivision1D<Point = Point3>,
    S: Clone + SearchParameter<D2, Point = Point3>,
{
    let mut res = FacesClassification::<Point3, C, S>::default();
    shell
        .iter()
        .zip(loops_store)
        .try_for_each(|(face, loops)| {
            if loops
                .iter()
                .all(|wire| wire.status() == ShapesOpStatus::Unknown)
            {
                res.push(face.clone(), ShapesOpStatus::Unknown);
            } else {
                let vec = divide_one_face(face, loops, tol)?;
                vec.into_iter()
                    .for_each(|(face, status)| res.push(face, status));
            }
            Some(())
        })?;
    Some(res)
}

#[cfg(test)]
mod tests;
