use truck_polymesh::algo::DefaultSplitParams;

use super::*;

type CompressedSolid = truck_topology::compress::CompressedSolid<Point3, Curve, Surface>;

macro_rules! dir ( () => { concat!(env!("CARGO_MANIFEST_DIR"), "/../resources/shape/") });

const SHAPE_JSONS: &[&str] = &[
    concat!(dir!(), "bottle.json"),
    concat!(dir!(), "punched-cube.json"),
    concat!(dir!(), "torus-punched-cube.json"),
    concat!(dir!(), "sphere.json"),
    concat!(dir!(), "torus.json"),
];

fn read_jsons() -> Vec<Vec<u8>> {
    let closure = |json| std::fs::read(json).unwrap();
    SHAPE_JSONS.iter().map(closure).collect()
}

#[test]
fn solid_is_closed() {
    for (i, json) in read_jsons().into_iter().enumerate() {
        let solid: Solid = serde_json::from_reader(json.as_slice()).unwrap();
        let mut poly = solid.triangulation(DefaultSplitParams::new(0.01)).to_polygon();
        poly.put_together_same_attrs(TOLERANCE * 2.0)
            .remove_degenerate_faces()
            .remove_unused_attrs();
        assert_eq!(
            poly.shell_condition(),
            ShellCondition::Closed,
            "not closed: file no. {i}"
        );
    }
}

#[test]
fn csolid_is_closed() {
    for (i, json) in read_jsons().into_iter().enumerate() {
        let solid: CompressedSolid = serde_json::from_reader(json.as_slice()).unwrap();
        let mut poly = solid.triangulation(DefaultSplitParams::new(0.01)).to_polygon();
        poly.put_together_same_attrs(TOLERANCE * 2.0)
            .remove_degenerate_faces()
            .remove_unused_attrs();
        assert_eq!(
            poly.shell_condition(),
            ShellCondition::Closed,
            "not closed: file no. {i}"
        );
    }
}

#[test]
fn compare_occt_mesh() {
    let jsons = read_jsons();
    let solid: Solid = serde_json::from_slice(jsons[2].as_slice()).unwrap();
    let res = solid.triangulation(DefaultSplitParams::new(0.005)).to_polygon();
    let path = concat!(dir!(), "../obj/by_occt.obj");
    let ans = obj::read(std::fs::read(path).unwrap().as_slice()).unwrap();
    assert!(res.is_clung_to_by(ans.positions(), 0.05));
    assert!(ans.is_clung_to_by(res.positions(), 0.05));
}

#[test]
fn compare_occt_mesh_csolid() {
    let jsons = read_jsons();
    let solid: CompressedSolid = serde_json::from_slice(jsons[2].as_slice()).unwrap();
    let res = solid.triangulation(DefaultSplitParams::new(0.005)).to_polygon();
    let path = concat!(dir!(), "../obj/by_occt.obj");
    let ans = obj::read(std::fs::read(path).unwrap().as_slice()).unwrap();
    assert!(res.is_clung_to_by(ans.positions(), 0.05));
    assert!(ans.is_clung_to_by(res.positions(), 0.05));
}

#[test]
fn large_number_meshing() {
    let json = std::fs::read(concat!(dir!(), "large-torus.json")).unwrap();
    let torus: Solid = serde_json::from_slice(json.as_slice()).unwrap();
    let _ = torus.triangulation(DefaultSplitParams::new(1.0)).to_polygon();
}

fn special_cylinder_model() -> Shell {
    let v0 = builder::vertex(Point3::new(0.0, 1.0, 0.0));
    let v1 = builder::vertex(Point3::new(0.0, 1.0, 1.0));
    let v2 = builder::vertex(Point3::new(0.0, -1.0, 0.0));
    let v3 = builder::vertex(Point3::new(0.0, -1.0, 1.0));

    let edge0 = builder::line(&v0, &v1);
    let edge1 = builder::line(&v2, &v3);
    let edge2 = builder::circle_arc(&v0, &v2, Point3::new(-1.0, 0.0, 0.0));
    let edge3 = builder::circle_arc(&v2, &v0, Point3::new(1.0, 0.0, 0.0));
    let edge4 = builder::circle_arc(&v1, &v3, Point3::new(-1.0, 0.0, 1.0));
    let edge5 = builder::circle_arc(&v3, &v1, Point3::new(1.0, 0.0, 1.0));

    let face0 =
        builder::try_attach_plane(&[vec![edge2.inverse(), edge3.inverse()].into()]).unwrap();
    let face1 = builder::try_attach_plane(&[vec![edge4.clone(), edge5.clone()].into()]).unwrap();

    let surface_row = RevolutedCurve::<Curve>::by_revolution(
        Line(Point3::new(1.0, 0.0, 1.0), Point3::new(1.0, 0.0, 0.0)).into(),
        Point3::origin(),
        Vector3::unit_z(),
    );
    let surface: Surface = Processor::new(surface_row).into();

    let face2 = Face::new(
        vec![vec![edge2, edge1.clone(), edge4.inverse(), edge0.inverse()].into()],
        surface.clone(),
    );
    let face3 = Face::new(
        vec![vec![edge3, edge0, edge5.inverse(), edge1.inverse()].into()],
        surface,
    );

    vec![face0, face1, face2, face3].into()
}

#[test]
fn special_cylinder() {
    let shell = special_cylinder_model();
    let mut mesh = shell.triangulation(DefaultSplitParams::new(0.01)).to_polygon();
    mesh.put_together_same_attrs(TOLERANCE)
        .remove_degenerate_faces()
        .remove_unused_attrs();
    assert_eq!(mesh.shell_condition(), ShellCondition::Closed);
}

#[test]
fn special_cylinder_csolid() {
    let shell = special_cylinder_model().compress();
    let mut mesh = shell.triangulation(DefaultSplitParams::new(0.01)).to_polygon();
    mesh.put_together_same_attrs(TOLERANCE)
        .remove_degenerate_faces()
        .remove_unused_attrs();
    assert_eq!(mesh.shell_condition(), ShellCondition::Closed);
}

#[test]
fn robust_closed() {
    let cube: Solid = {
        let v = builder::vertex(Point3::origin());
        let e = builder::tsweep(&v, Vector3::unit_x());
        let f = builder::tsweep(&e, Vector3::unit_y());
        builder::tsweep(&f, Vector3::unit_z())
    };

    let o = Point3::new(0.5, 0.5, 0.5);
    cube.edge_iter().for_each(|edge| {
        let curve = edge.curve();

        if let Curve::Line(line) = curve {
            let m = line.subs(0.5);
            let p = m + 0.2 * (o - m);
            let bsp = BSplineCurve::new(KnotVec::bezier_knot(2), vec![line.0, p, line.1]);
            edge.set_curve(bsp.into());
        }
    });

    assert!(cube
        .triangulation(DefaultSplitParams::new(0.01))
        .face_iter()
        .all(|face| face.surface().is_none()));

    let mut mesh = cube.robust_triangulation(DefaultSplitParams::new(0.01)).to_polygon();
    mesh.put_together_same_attrs(TOLERANCE2)
        .remove_unused_attrs();
    assert_eq!(mesh.shell_condition(), ShellCondition::Closed);
}
