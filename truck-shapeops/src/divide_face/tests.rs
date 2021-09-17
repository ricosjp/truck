use super::*;
use truck_geometry::*;
use truck_topology::Vertex;

fn line(v0: &Vertex<Point3>, v1: &Vertex<Point3>) -> Edge<Point3, BSplineCurve<Point3>> {
	let curve = BSplineCurve::new(
		KnotVec::bezier_knot(1),
		vec![v0.get_point(), v1.get_point()],
	);
	Edge::new(&v0, &v1, curve)
}

fn parabola(
	v0: &Vertex<Point3>,
	v1: &Vertex<Point3>,
	pt: Point3,
) -> Edge<Point3, BSplineCurve<Point3>> {
	let curve = BSplineCurve::new(
		KnotVec::bezier_knot(2),
		vec![v0.get_point(), pt, v1.get_point()],
	);
	Edge::new(v0, v1, curve)
}

#[test]
fn divide_plane_test() {
	let v = Vertex::news(&[
		Point3::new(0.0, 0.0, 0.0),
		Point3::new(0.0, 4.0, 0.0),
		Point3::new(-1.0, 1.0, 0.0),
		Point3::new(-1.0, 3.0, 0.0),
		Point3::new(1.0, 1.0, 0.0),
		Point3::new(1.0, 3.0, 0.0),
	]);
	let edge = vec![
		parabola(&v[0], &v[1], Point3::new(-4.0, 2.0, 0.0)),
		parabola(&v[0], &v[1], Point3::new(4.0, 2.0, 0.0)),
		line(&v[0], &v[1]),
		parabola(&v[2], &v[3], Point3::new(-3.0, 2.0, 0.0)),
		parabola(&v[2], &v[3], Point3::new(-1.0, 2.0, 0.0)),
		parabola(&v[4], &v[5], Point3::new(1.0, 2.0, 0.0)),
		parabola(&v[4], &v[5], Point3::new(3.0, 2.0, 0.0)),
	];
	let wire: Vec<Wire<_, _>> = vec![
		vec![edge[1].clone(), edge[0].inverse()].into(),
		vec![edge[2].clone(), edge[0].inverse()].into(),
		vec![edge[1].clone(), edge[2].inverse()].into(),
		vec![edge[3].clone(), edge[4].inverse()].into(),
		vec![edge[5].clone(), edge[6].inverse()].into(),
	];
	let face = Face::new(
		vec![wire[0].clone(), wire[3].clone(), wire[4].clone()],
		Plane::new(
			Point3::origin(),
			Point3::new(1.0, 0.0, 0.0),
			Point3::new(0.0, 1.0, 0.0),
		),
	);
	let loops: Loops<_, _> = vec![
		BoundaryWire::new(wire[1].clone(), BoundaryStatus::Or),
		BoundaryWire::new(wire[2].clone(), BoundaryStatus::And),
		BoundaryWire::new(wire[3].clone(), BoundaryStatus::Unknown),
		BoundaryWire::new(wire[4].clone(), BoundaryStatus::Unknown),
	]
	.into_iter()
	.collect();
	let res = divide_one_face(&face, &loops, 0.01).unwrap();
	assert_eq!(res.len(), 2);
	let (mut or, mut and) = (true, true);
	for (face, status) in res {
		let bdd = face.absolute_boundaries();
		match status {
			BoundaryStatus::Or => {
				assert_eq!(bdd.len(), 2);
				assert!(bdd[0] == wire[1] || bdd[0] == wire[3]);
				assert!(bdd[1] == wire[1] || bdd[1] == wire[3]);
				assert_ne!(bdd[0], bdd[1]);
				assert!(or);
				or = false;
			}
			BoundaryStatus::And => {
				assert_eq!(bdd.len(), 2);
				assert!(bdd[0] == wire[2] || bdd[0] == wire[4]);
				assert!(bdd[1] == wire[2] || bdd[1] == wire[4]);
				assert_ne!(bdd[0], bdd[1]);
				assert!(and);
				and = false;
			}
			_ => panic!("There must be no unknown!"),
		}
	}
}
