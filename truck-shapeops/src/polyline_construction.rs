use std::collections::{HashMap, HashSet};
use truck_base::{cgmath64::*, tolerance::*};
use truck_meshalgo::prelude::PolylineCurve;

pub fn construct_polylines(lines: &Vec<(Point3, Point3)>) -> Vec<PolylineCurve<Point3>> {
	let mut map = HashMap::<[i64; 3], (Point3, HashSet<[i64; 3]>)>::new();
	for line in lines {
		if line.0.near(&line.1) {
			continue;
		}
		let idx0 = into_index(line.0);
		let idx1 = into_index(line.1);
		if let Some((_, set)) = map.get_mut(&idx0) {
			set.insert(idx1);
		} else {
			let mut set = HashSet::new();
			set.insert(idx1);
			map.insert(idx0, (line.0, set));
		}
		if let Some((_, set)) = map.get_mut(&idx1) {
			set.insert(idx0);
		} else {
			let mut set = HashSet::new();
			set.insert(idx0);
			map.insert(idx1, (line.1, set));
		}
	}
	let mut res = Vec::new();
	while !map.is_empty() {
		let mut wire = Vec::new();
		let (_idx, (pt, _)) = map.iter_mut().next().unwrap();
		wire.push(*pt);
		let mut idx = *_idx;
		while let Some((_, set)) = map.get_mut(&idx) {
			let idx0 = *set.iter().next().unwrap();
			set.remove(&idx0);
			if set.is_empty() {
				map.remove(&idx);
			}
			let (pt, set) = map.get_mut(&idx0).unwrap();
			wire.push(*pt);
			set.remove(&idx);
			if set.is_empty() {
				map.remove(&idx0);
			}
			idx = idx0;
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

	let mut sign = None;
	for line in polyline[0].windows(2) {
		let a = line[0][0] + line[0][1] * 2.0 + line[0][2] * 4.0;
		let b = line[1][0] + line[1][1] * 2.0 + line[1][2] * 4.0;
		let x = b - a;
		assert!(f64::abs(x) == 1.0 || f64::abs(x) == 7.0);
		let s = f64::signum(x * (x - 2.0) * (x + 2.0));
		if let Some(sign) = sign {
			assert!(s == sign);
		} else {
			sign = Some(s);
		}
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

	let mut sign = None;
	for line in polyline[0].windows(2) {
		let a = line[0][0] + line[0][1] * 2.0 + line[0][2] * 4.0;
		let b = line[1][0] + line[1][1] * 2.0 + line[1][2] * 4.0;
		let x = b - a;
		assert!(f64::abs(x) == 1.0 || f64::abs(x) == 7.0);
		let s = f64::signum(x * (x - 2.0) * (x + 2.0));
		if let Some(sign) = sign {
			assert!(s == sign);
		} else {
			sign = Some(s);
		}
	}
}
