use std::collections::{HashMap, HashSet, VecDeque};
use truck_base::{cgmath64::*, tolerance::*};
use truck_meshalgo::prelude::PolylineCurve;

pub fn construct_polylines(lines: &Vec<(Point3, Point3)>) -> Vec<PolylineCurve<Point3>> {
	let mut graph: Graph = lines.into_iter().collect();
	let mut res = Vec::new();
	while !graph.is_empty() {
		let (mut idx, node) = graph.get_one();
		let mut wire: VecDeque<_> = vec![node.coord].into();
		while let Some((idx0, pt)) = graph.get_a_next_node(idx) {
			idx = idx0;
			wire.push_back(pt);
		}
		let mut idx = PointIndex::from(wire[0]);
		while let Some((idx0, pt)) = graph.get_a_next_node(idx) {
			idx = idx0;
			wire.push_front(pt);
		}
		res.push(PolylineCurve(wire.into()));
	}
	res
}

#[derive(Clone, Debug, Copy, Hash, PartialEq, Eq)]
struct PointIndex([i64; 3]);

impl From<Point3> for PointIndex {
	#[inline(always)]
	fn from(pt: Point3) -> PointIndex {
		let idx = pt.add_element_wise(TOLERANCE) / (2.0 * TOLERANCE);
		PointIndex(idx.cast::<i64>().unwrap().into())
	}
}

struct Node {
	coord: Point3,
	adjacency: HashSet<PointIndex>,
}

impl Node {
	#[inline(always)]
	fn new(coord: Point3, adjacency: HashSet<PointIndex>) -> Node { Node { coord, adjacency } }

	fn pop_one_adjacency(&mut self) -> PointIndex {
		let idx = *self.adjacency.iter().next().unwrap();
		self.adjacency.remove(&idx);
		idx
	}
}

struct Graph(HashMap<PointIndex, Node>);

impl std::ops::Deref for Graph {
	type Target = HashMap<PointIndex, Node>;
	#[inline(always)]
	fn deref(&self) -> &Self::Target { &self.0 }
}

impl std::ops::DerefMut for Graph {
	#[inline(always)]
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl Graph {
	fn add_half_edge(&mut self, pt0: Point3, pt1: Point3) {
		let idx0 = pt0.into();
		let idx1 = pt1.into();
		if let Some(node) = self.get_mut(&idx0) {
			node.adjacency.insert(idx1);
		} else {
			let mut set = HashSet::new();
			set.insert(idx1);
			self.insert(idx0, Node::new(pt0, set));
		}
	}

	fn add_edge(&mut self, line: (Point3, Point3)) {
		if !line.0.near(&line.1) {
			self.add_half_edge(line.0, line.1);
			self.add_half_edge(line.1, line.0);
		}
	}

	#[inline(always)]
	fn get_one(&self) -> (PointIndex, &Node) {
		let (idx, node) = self.iter().next().unwrap();
		(*idx, node)
	}

	fn get_a_next_node(&mut self, idx: PointIndex) -> Option<(PointIndex, Point3)> {
		let node = self.get_mut(&idx)?;
		let idx0 = node.pop_one_adjacency();
		if node.adjacency.is_empty() {
			self.remove(&idx);
		}
		let node = self.get_mut(&idx0)?;
		node.adjacency.remove(&idx);
		let pt = node.coord;
		if node.adjacency.is_empty() {
			self.remove(&idx0);
		}
		Some((idx0, pt))
	}
}

impl<'a> std::iter::FromIterator<&'a (Point3, Point3)> for Graph {
	fn from_iter<I: IntoIterator<Item = &'a (Point3, Point3)>>(iter: I) -> Graph {
		let mut res = Graph(HashMap::new());
		iter.into_iter().for_each(|line| res.add_edge(*line));
		res
	}
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

#[test]
fn construct_polylines_positive3() {
	let lines = vec![
		(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)),
		(Point3::new(1.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0)),
		(Point3::new(1.0, 1.0, 0.0), Point3::new(0.0, 0.0, 1.0)),
		(Point3::new(0.0, 1.0, 1.0), Point3::new(1.0, 1.0, 1.0)),
		(Point3::new(0.0, 0.0, 1.0), Point3::new(1.0, 0.0, 1.0)),
		(Point3::new(1.0, 1.0, 0.0), Point3::new(1.0, 1.0, 0.0)),
		(Point3::new(0.0, 1.0, 0.0), Point3::new(1.0, 1.0, 0.0)),
		(Point3::new(1.0, 0.0, 1.0), Point3::new(0.0, 1.0, 1.0)),
	];
	let polyline = construct_polylines(&lines);
	assert_eq!(polyline.len(), 1);
	assert_eq!(polyline[0].len(), 8);

	let mut sign = None;
	for line in polyline[0].windows(2) {
		let a = line[0][0] + line[0][1] * 2.0 + line[0][2] * 4.0;
		let b = line[1][0] + line[1][1] * 2.0 + line[1][2] * 4.0;
		let x = b - a;
		assert!(f64::abs(x) == 1.0);
		let s = f64::signum(x * (x - 2.0) * (x + 2.0));
		if let Some(sign) = sign {
			assert!(s == sign);
		} else {
			sign = Some(s);
		}
	}
}
