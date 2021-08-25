use super::*;

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
