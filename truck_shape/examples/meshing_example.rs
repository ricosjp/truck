use std::f64::consts::PI;
use std::fs::DirBuilder;
use std::iter::FromIterator;
use std::convert::TryInto;
use truck_geometry::*;
use truck_polymesh::PolygonMesh;
use truck_shape::elements::{Integrity, TopoGeomIntegrity};
use truck_shape::mesher::Meshed;
use truck_shape::*;
use truck_topology::*;

#[allow(dead_code)]
fn n_gon_prism(builder: &mut Builder, n: usize) -> Solid {
    let v: Vec<_> = (0..n)
        .map(|i| {
            let t = 2.0 * PI * (i as f64) / (n as f64);
            builder.vertex(Point3::new(t.sin(), 0.0, t.cos())).unwrap()
        })
        .collect();
    let wire: Wire = (0..n)
        .map(|i| builder.line(v[i], v[(i + 1) % n]).unwrap())
        .collect();
    let face = builder.plane(wire).unwrap();
    builder.tsweep(face, Vector3::new(0.0, 2.0, 0.0)).unwrap()
}

#[allow(dead_code)]
fn cube(builder: &mut Builder) -> Solid {
    let v: Vertex = builder.vertex(Point3::origin()).unwrap();
    let edge = builder.tsweep(v, Vector3::unit_x()).unwrap();
    let face = builder.tsweep(edge, Vector3::unit_y()).unwrap();
    builder.tsweep(face, Vector3::unit_z()).unwrap()
}

#[allow(dead_code)]
fn bottle(builder: &mut Builder) -> Solid {
    let (width, thick, height) = (6.0, 4.0, 10.0);
    let v0 = builder
        .vertex(Point3::new(-thick / 4.0, 0.0, -width / 2.0))
        .unwrap();
    let v1 = builder
        .vertex(Point3::new(-thick / 4.0, 0.0, width / 2.0))
        .unwrap();
    let transit = Point3::new(-thick / 2.0, 0.0, 0.0);
    let edge0 = builder.circle_arc(v0, v1, transit).unwrap();
    let edge1 = builder
        .rotated(&edge0, Point3::new(0.0, 0.0, 0.0), Vector3::unit_z(), PI)
        .unwrap();
    let wire0 = Wire::from_iter(&[edge0]);
    let wire1 = Wire::from_iter(&[edge1]);
    let face = builder.homotopy(&wire0, &wire1).unwrap();
    builder
        .tsweep(face, Vector3::new(0.0, height, 0.0))
        .unwrap()
        .pop()
        .unwrap()
}

#[allow(dead_code)]
fn tsudsumi(builder: &mut Builder) -> Solid {
    let v0 = builder.vertex(Point3::new(1.0, 2.0, 0.0)).unwrap();
    let v1 = builder.vertex(Point3::new(0.0, 0.0, 1.0)).unwrap();
    let edge = builder.line(v0, v1).unwrap();
    let mut shell = builder
        .rsweep(edge, Point3::origin(), Vector3::unit_y(), PI * 2.0)
        .unwrap();
    let wire = shell.extract_boundaries();
    for mut wire in wire {
        wire.invert();
        shell.push(builder.plane(wire).unwrap());
    }
    Solid::try_new(vec![shell]).unwrap()
}

#[allow(dead_code)]
fn truck3d(builder: &mut Builder) -> Solid {
    let v: Vec<Vertex> = vec![
        builder.vertex(Point3::new(0.0, 0.0, 0.0)).unwrap(),
        builder.vertex(Point3::new(4.0, 0.0, 0.0)).unwrap(),
        builder.vertex(Point3::new(1.0, 0.0, 2.0)).unwrap(),
        builder.vertex(Point3::new(3.0, 0.0, 2.0)).unwrap(),
    ];
    let edge = vec![
        builder.line(v[1], v[0]).unwrap(),
        builder.circle_arc(v[3], v[2], Point3::new(2.0, 0.0, 1.0)).unwrap(),
    ];
    let mut shell = builder.homotopy(&edge[0], &edge[1]).unwrap();
    let face1 = builder
        .rotated(
            &shell[0],
            Point3::new(2.0, 0.0, 3.5),
            Vector3::unit_y(),
            std::f64::consts::PI,
        )
        .unwrap();
    let wire0 = shell[0].boundary();
    let wire1 = face1.boundary();
    let face2 = builder.homotopy(&wire0[1].inverse(), &wire1[3]).unwrap()[0].clone();
    let face3 = builder.homotopy(&wire0[3].inverse(), &wire1[1]).unwrap()[0].clone();
    shell.append(&mut vec![face1, face2, face3].into());
    builder
        .tsweep(shell, Vector3::new(0.0, 3.0, 0.0))
        .unwrap()
        .pop()
        .unwrap()
}

#[allow(dead_code)]
fn large_box(builder: &mut Builder) -> Solid {
    const N: usize = 100;

    let v: Vec<_> = (0..N)
        .flat_map(|i| (0..N).map(move |j| (i, j)))
        .map(|(i, j)| builder.vertex(Point3::new(i as f64, j as f64, 0.0)).unwrap())
        .collect();
    let row_edge: Vec<Vec<_>> = (0..N)
        .map(|i| {
            (1..N)
                .map(|j| builder.line(v[i * N + j - 1], v[i * N + j]).unwrap())
                .collect()
        })
        .collect();
    let col_edge: Vec<Vec<_>> = (1..N)
        .map(|i| {
            (0..N)
                .map(|j| {
                    builder
                        .line(v[(i - 1) * N + j], v[(i % N) * N + j])
                        .unwrap()
                })
                .collect()
        })
        .collect();

    let shell: Shell = (1..N)
        .flat_map(|i| (1..N).map(move |j| (i, j)))
        .map(|(i, j)| {
            let wire = Wire::from_iter(&[
                row_edge[i - 1][j - 1],
                col_edge[i - 1][j],
                row_edge[i][j - 1].inverse(),
                col_edge[i - 1][j - 1].inverse(),
            ]);
            builder.plane(wire).unwrap()
        })
        .collect();
    builder
        .tsweep(shell, Vector3::unit_z())
        .unwrap()
        .pop()
        .unwrap()
}

#[allow(dead_code)]
fn torus(builder: &mut Builder) -> Shell {
    let v = vec![
        builder.vertex(Point3::new(0.0, 0.0, 1.0)).unwrap(),
        builder.vertex(Point3::new(0.0, 0.0, 3.0)).unwrap(),
    ];
    let wire = Wire::from_iter(&[
        builder
            .circle_arc(v[0], v[1], Point3::new(0.0, 1.0, 2.0))
            .unwrap(),
        builder
            .circle_arc(v[1], v[0], Point3::new(0.0, -1.0, 2.0))
            .unwrap(),
    ]);
    builder
        .rsweep(
            wire,
            Point3::origin(),
            Vector3::unit_y(),
            std::f64::consts::PI * 2.0,
        )
        .unwrap()
}

#[allow(dead_code)]
fn half_torus(builder: &mut Builder) -> Solid {
    let v = vec![
        builder.vertex(Point3::new(0.0, 0.0, 1.0)).unwrap(),
        builder.vertex(Point3::new(0.0, 0.0, 3.0)).unwrap(),
    ];
    let wire = Wire::from_iter(&[
        builder
            .circle_arc(v[0], v[1], Point3::new(0.0, 1.0, 2.0))
            .unwrap(),
        builder
            .circle_arc(v[1], v[0], Point3::new(0.0, -1.0, 2.0))
            .unwrap(),
    ]);
    let face = builder.plane(wire).unwrap();
    builder
        .rsweep(
            face,
            Point3::origin(),
            Vector3::unit_y(),
            std::f64::consts::PI,
        )
        .unwrap()
}

#[allow(dead_code)]
fn truck_torus(builder: &mut Builder) -> Solid {
    let v: Vec<Vertex> = vec![
        builder.vertex(Point3::new(0.0, 0.0, 4.0)).unwrap(),
        builder.vertex(Point3::new(4.0, 0.0, 4.0)).unwrap(),
        builder.vertex(Point3::new(1.0, 0.0, 6.0)).unwrap(),
        builder.vertex(Point3::new(3.0, 0.0, 6.0)).unwrap(),
    ];
    let edge = vec![
        builder.line(v[1], v[0]).unwrap(),
        builder.circle_arc(v[3], v[2], Point3::new(2.0, 0.0, 5.0)).unwrap(),
    ];
    let mut shell = builder.homotopy(&edge[0], &edge[1]).unwrap();
    let face1 = builder
        .rotated(
            &shell[0],
            Point3::new(2.0, 0.0, 7.5),
            Vector3::unit_y(),
            std::f64::consts::PI,
        )
        .unwrap();
    let wire0 = shell[0].boundary();
    let wire1 = face1.boundary();
    let face2 = builder.homotopy(&wire0[1].inverse(), &wire1[3]).unwrap()[0].clone();
    let face3 = builder.homotopy(&wire0[3].inverse(), &wire1[1]).unwrap()[0].clone();
    shell.append(&mut vec![face1, face2, face3].into());
    builder
        .rsweep(shell, Point3::origin(), Vector3::unit_x(), -PI * 2.0)
        .unwrap()
        .pop()
        .unwrap()
}

#[allow(dead_code)]
fn vase(builder: &mut Builder) -> Shell {
    let v0 = builder.vertex(Point3::new(0.0, 0.0, 0.0)).unwrap();
    let v1 = builder.vertex(Point3::new(1.0, 0.0, 0.0)).unwrap();
    let v2 = builder.vertex(Point3::new(1.5, 3.0, 0.0)).unwrap();
    let origin = Point3::origin();
    let axis = Vector3::unit_y();
    let edge0 = builder.line(v0, v1).unwrap();
    let inter_points = vec![
        Vector3::new(2.0, 0.5, 0.0),
        Vector3::new(1.2, 3.5, 0.0),
        Vector3::new(1.5, 3.5, 0.0),
    ];
    let edge1 = builder.bezier(v1, v2, inter_points).unwrap();
    let wire = Wire::from_iter(&[edge0, edge1]);
    builder.rsweep(wire, origin, axis, -PI * 2.0).unwrap()
}

fn sub_screw(director: &mut Director, radius: f64) -> Wire {
    let mut v = director.get_builder().vertex(Point3::new(0.0, 0.0, radius)).unwrap();
    let circle = director.get_builder().rsweep(v, Point3::origin(), Vector3::unit_y(), 2.0 * PI).unwrap();
    let cylinder = director.get_builder().tsweep(circle, Vector3::unit_y()).unwrap();
    let n = cylinder.len();
    let mut wire = Wire::new();
    for i in 1..=2 {
        let surface = director.get_geometry(&cylinder[i % n]).unwrap().clone();
        if i == 1 {
            v = director.get_builder().create_topology(surface.subs(0.0, i as f64 / (2 * n) as f64)).unwrap();
        }
        let new_v: Vertex = director.get_builder().create_topology(surface.subs(1.0, (i + 1) as f64 / (2 * n) as f64)).unwrap();
        let bnd_box = BoundingBox::from_iter(&[
            Vector2::new(0.0, i as f64 / (2 * n) as f64),
            Vector2::new(1.0, (i + 1) as f64 / (2 * n) as f64),
        ]);
        let curve = surface.sectional_curve(bnd_box);
        let edge: Edge = Edge::new(v, new_v);
        director.attach(&edge, curve);
        wire.push_back(edge);
        v = new_v;
    }
    wire
}

#[allow(dead_code)]
fn screw(director: &mut Director) -> Shell {
    let wire = sub_screw(director, 1.0);
    let mut shell = director.get_builder().tsweep(wire, -Vector3::unit_y() * 0.2).unwrap();
    let wire = sub_screw(director, 1.1);
    let mut shell0 = director.get_builder().tsweep(wire, Vector3::unit_y() * 0.1).unwrap();
    shell0 = director.get_builder().translated(&shell0, -0.15 * Vector3::unit_y()).unwrap();
    shell.append(&mut shell0);
    let mut shell3 = director.get_builder().translated(&shell, 0.5 * Vector3::unit_y()).unwrap();
    shell.append(&mut shell3);
    shell
} 

#[allow(dead_code)]
fn assert_integrity<T: Integrity>(elem: &T, director: &mut Director, filename: &str) {
    let integrity = director.check_integrity(elem);
    assert_eq!(
        integrity,
        TopoGeomIntegrity::Integrate,
        "Integrate Error: {}",
        filename
    );
}

fn output_mesh<F, T>(director: &mut Director, function: F, filename: &str)
where
    F: FnOnce(&mut Builder) -> T,
    T: Meshed<MeshType = PolygonMesh> + Integrity, {
    let path = "./output/".to_string() + filename;
    let instant = std::time::Instant::now();
    let solid = director.building(function);
    //assert_integrity(&solid, director, filename);
    let mesh = director.get_mesher().meshing(&solid, 0.02);
    let end_time = instant.elapsed();
    println!(
        "{}: {}.{:03} sec",
        filename,
        end_time.as_secs(),
        end_time.subsec_nanos() / 1_000_000,
    );
    let file = std::fs::File::create(path).unwrap();
    truck_io::obj::write(&mesh, file).unwrap();
}

fn semi_output_mesh<F, T>(director: &mut Director, function: F, filename: &str)
where
    F: FnOnce(&mut Director) -> T,
    T: Meshed<MeshType = PolygonMesh> + Integrity, {
    let path = "./output/".to_string() + filename;
    let instant = std::time::Instant::now();
    let solid = function(director);
    //assert_integrity(&solid, director, filename);
    let mesh = director.get_mesher().meshing(&solid, 0.02);
    let end_time = instant.elapsed();
    println!(
        "{}: {}.{:03} sec",
        filename,
        end_time.as_secs(),
        end_time.subsec_nanos() / 1_000_000,
    );
    let file = std::fs::File::create(path).unwrap();
    truck_io::obj::write(&mesh, file).unwrap();
}

fn main() {
    let mut director = Director::new();
    DirBuilder::new().recursive(true).create("output").unwrap();
    output_mesh(&mut director, cube, "cube.obj");
    output_mesh(&mut director, bottle, "bottle.obj");
    output_mesh(&mut director, tsudsumi, "tsudsumi.obj");
    output_mesh(&mut director, truck3d, "truck3d.obj");
    for n in 3..=8 {
        let filename = format!("{}-gon-prism.obj", n);
        output_mesh(&mut director, |d| n_gon_prism(d, n), &filename);
    }
    //output_mesh(&mut director, large_box, "large_plane.obj");
    output_mesh(&mut director, torus, "torus.obj");
    output_mesh(&mut director, half_torus, "half_torus.obj");
    //output_mesh(&mut director, truck_torus, "truck_torus.obj");
    //output_mesh(&mut director, vase, "vase.obj");
    semi_output_mesh(&mut director, screw, "screw.obj");
}
