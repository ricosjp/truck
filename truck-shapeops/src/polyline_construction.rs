use std::collections::HashMap;
use truck_meshalgo::prelude::PolylineCurve;
use truck_base::{cgmath64::*, tolerance::*};

pub fn construct_polylines(lines: &Vec<(Point3, Point3)>) -> Vec<PolylineCurve<Point3>> {
	let mut lines: HashMap<[i64; 3], (Point3, Point3)> = lines
		.iter()
		.filter(|(pt0, pt1)| pt0.distance2(*pt1) > TOLERANCE)
		.map(|(pt0, pt1)| (into_index(*pt0), (*pt0, *pt1)))
		.collect();
	let mut res = Vec::new();
	while !lines.is_empty() {
		let mut wire = Vec::new();
		let mut idx = *lines.iter().next().unwrap().0;
		let line = lines.remove(&idx).unwrap();
		wire.push(line.0);
		wire.push(line.1);
		idx = into_index(line.1);
		while let Some(line) = lines.remove(&idx) {
			wire.push(line.1);
			idx = into_index(line.1);
		}
		res.push(PolylineCurve(wire));
	}
	res
}

fn into_index(pt: Point3) -> [i64; 3] {
	let idx = pt.add_element_wise(TOLERANCE) / (2.0 * TOLERANCE);
	idx.cast::<i64>().unwrap().into()
}

#[test]
fn construct_polylines_positive0() {
	let lines = vec![
		(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)),
		(Point3::new(1.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0)),
		(Point3::new(1.0, 1.0, 0.0), Point3::new(0.0, 0.0, 1.0)),
		(Point3::new(0.0, 1.0, 1.0), Point3::new(1.0, 1.0, 1.0)),
		(Point3::new(0.0, 0.0, 1.0), Point3::new(1.0, 0.0, 1.0)),
		(Point3::new(0.0, 1.0, 0.0), Point3::new(1.0, 1.0, 0.0)),
		(Point3::new(1.0, 1.0, 1.0), Point3::new(0.0, 0.0, 0.0)),
		(Point3::new(1.0, 0.0, 1.0), Point3::new(0.0, 1.0, 1.0)),
	];
	let polyline = construct_polylines(&lines);
	assert_eq!(polyline.len(), 1);
	assert_eq!(polyline[0].len(), 9);

	for line in polyline[0].windows(2) {
		let a = line[0][0] + line[0][1] * 2.0 + line[0][2] * 4.0;
		let b = line[1][0] + line[1][1] * 2.0 + line[1][2] * 4.0;
		assert!((b - a) == 1.0 || (b - a) == -7.0);
	}
}

#[test]
fn construct_polylines_positive1() {
	let lines = vec![
		(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)),
		(Point3::new(1.0, 0.0, 0.0), Point3::new(1.0, 1.0, 0.0)),
		(Point3::new(1.0, 0.0, 1.0), Point3::new(1.0, 1.0, 1.0)),
		(Point3::new(1.0, 1.0, 1.0), Point3::new(0.0, 1.0, 1.0)),
		(Point3::new(1.0, 1.0, 0.0), Point3::new(0.0, 1.0, 0.0)),
		(Point3::new(0.0, 1.0, 0.0), Point3::new(0.0, 0.0, 0.0)),
		(Point3::new(0.0, 0.0, 1.0), Point3::new(1.0, 0.0, 1.0)),
		(Point3::new(0.0, 1.0, 1.0), Point3::new(0.0, 0.0, 1.0)),
	];
	let polyline = construct_polylines(&lines);
	assert_eq!(polyline.len(), 2);
	assert_eq!(polyline[0].len(), 5);
	assert_eq!(polyline[1].len(), 5);
}

#[test]
fn construct_polylines_positive2() {
	let lines = vec![
		(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)),
		(Point3::new(1.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0)),
		(Point3::new(1.0, 1.0, 0.0), Point3::new(0.0, 0.0, 1.0)),
		(Point3::new(0.0, 1.0, 1.0), Point3::new(1.0, 1.0, 1.0)),
		(Point3::new(0.0, 0.0, 1.0), Point3::new(1.0, 0.0, 1.0)),
		(Point3::new(1.0, 1.0, 0.0), Point3::new(1.0, 1.0, 0.0)),
		(Point3::new(0.0, 1.0, 0.0), Point3::new(1.0, 1.0, 0.0)),
		(Point3::new(1.0, 1.0, 1.0), Point3::new(0.0, 0.0, 0.0)),
		(Point3::new(1.0, 0.0, 1.0), Point3::new(0.0, 1.0, 1.0)),
	];
	let polyline = construct_polylines(&lines);
	assert_eq!(polyline.len(), 1);
	assert_eq!(polyline[0].len(), 9);

	for line in polyline[0].windows(2) {
		let a = line[0][0] + line[0][1] * 2.0 + line[0][2] * 4.0;
		let b = line[1][0] + line[1][1] * 2.0 + line[1][2] * 4.0;
		assert!((b - a) == 1.0 || (b - a) == -7.0);
	}
}
