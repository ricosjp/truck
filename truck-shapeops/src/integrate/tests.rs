use truck_meshalgo::prelude::*;
use truck_modeling::*;

type AlternativeCurve =
	crate::test_util::Alternatives<Curve, crate::IntersectionCurve<PolylineCurve<Point3>, Surface>>;
crate::impl_from!(
	Curve,
	crate::IntersectionCurve<PolylineCurve<Point3>, Surface>
);

#[test]
fn torus_punched_cube() {
	let v = builder::vertex(Point3::origin());
	let e = builder::tsweep(&v, Vector3::unit_x());
	let f = builder::tsweep(&e, Vector3::unit_y());
	let s = builder::tsweep(&f, Vector3::unit_z());
	let cube = s.mapped(
		Point3::clone,
		|c| AlternativeCurve::from(c.clone()),
		Surface::clone,
	);

	let v = builder::vertex(Point3::new(0.5, 0.0, 0.25));
	let w = builder::rsweep(&v, Point3::new(0.5, 0.0, 0.5), Vector3::unit_y(), Rad(7.0));
	let s = builder::rsweep(&w, Point3::new(0.0, 0.0, 0.0), Vector3::unit_z(), Rad(7.0));
	let s = Solid::new(vec![s]);
	let mut torus = s.mapped(
		Point3::clone,
		|c| AlternativeCurve::from(c.clone()),
		Surface::clone,
	);
	torus.not();
	let and = crate::and(&cube, &torus, 0.05).unwrap();

	let poly = and.triangulation(0.01).unwrap().into_polygon();
	let file = std::fs::File::create("torus-punched-cube.obj").unwrap();
	obj::write(&poly, file).unwrap();
}
