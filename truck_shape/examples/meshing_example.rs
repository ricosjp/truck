use truck_geometry::*;
use truck_shape::director::TopoGeomIntegrity;
use truck_shape::*;
use truck_topology::*;

fn n_gon_prism(builder: &mut Builder, n: usize) -> Solid {
    let v: Vec<_> = (0..n)
        .map(|i| {
            let t = 2.0 * std::f64::consts::PI * (i as f64) / (n as f64);
            builder.vertex(Vector3::new(t.sin(), 0.0, t.cos())).unwrap()
        })
        .collect();
    let wire: Wire = (0..n)
        .map(|i| builder.line(v[i], v[(i + 1) % n]).unwrap())
        .collect();
    let face = builder.plane(wire).unwrap();
    builder.tsweep(&face, &Vector3::new(0, 2, 0)).unwrap()
}

fn cube(builder: &mut Builder) -> Solid {
    let v: Vertex = builder.vertex(Vector3::new(0.0, 0.0, 0.0)).unwrap();
    let edge = builder.tsweep(&v, &Vector3::new(1.0, 0.0, 0.0)).unwrap();
    let face = builder.tsweep(&edge, &Vector3::new(0.0, 1.0, 0.0)).unwrap();
    builder.tsweep(&face, &Vector3::new(0.0, 0.0, 1.0)).unwrap()
}

fn bottle(builder: &mut Builder) -> Solid {
    let (width, thick, height) = (6.0, 4.0, 10.0);
    let v0 = builder
        .vertex(Vector3::new(-thick / 4.0, 0.0, -width / 2.0))
        .unwrap();
    let v1 = builder
        .vertex(Vector3::new(-thick / 4.0, 0.0, width / 2.0))
        .unwrap();
    let transit = Vector3::new(-thick / 2.0, 0.0, 0.0);
    let edge0 = builder.circle_arc(v0, v1, &transit).unwrap();
    let edge1 = builder
        .create_rotated(
            &edge0,
            &Vector3::new(0.0, 0.0, 0.0),
            &Vector3::new(0.0, 0.0, 1.0),
            std::f64::consts::PI,
        )
        .unwrap();
    let wire0 = Wire::by_slice(&[edge0]);
    let wire1 = Wire::by_slice(&[edge1]);
    let face = builder.homotopy(&wire0, &wire1).unwrap();
    builder
        .tsweep(&face, &Vector3::new(0.0, height, 0.0))
        .unwrap()
        .pop()
        .unwrap()
}

fn tsudsumi(builder: &mut Builder) -> Solid {
    let v = vec![
        builder.vertex(Vector3::new(1.0, 2.0, 0.0)).unwrap(),
        builder.vertex(Vector3::new(-1.0, 2.0, 0.0)).unwrap(),
        builder.vertex(Vector3::new(0.0, 0.0, 1.0)).unwrap(),
        builder.vertex(Vector3::new(0.0, 0.0, -1.0)).unwrap(),
    ];

    let edge = vec![
        builder
            .circle_arc(v[0], v[1], &Vector3::new(0.0, 2.0, 1.0))
            .unwrap(),
        builder
            .circle_arc(v[1], v[0], &Vector3::new(0.0, 2.0, -1.0))
            .unwrap(),
        builder.line(v[0], v[3]).unwrap(),
        builder.line(v[1], v[2]).unwrap(),
        builder
            .circle_arc(v[2], v[3], &Vector3::new(1.0, 0.0, 0.0))
            .unwrap(),
        builder
            .circle_arc(v[3], v[2], &Vector3::new(-1.0, 0.0, 0.0))
            .unwrap(),
    ];

    let wire = vec![
        Wire::by_slice(&[edge[0].inverse(), edge[1].inverse()]),
        Wire::by_slice(&[edge[0], edge[3], edge[4], edge[2].inverse()]),
        Wire::by_slice(&[edge[1], edge[2], edge[5], edge[3].inverse()]),
        Wire::by_slice(&[edge[4].inverse(), edge[5].inverse()]),
    ];

    let shell = wire
        .into_iter()
        .map(|w| builder.plane(w).unwrap())
        .collect();
    Solid::new(vec![shell])
}

fn truck3d(builder: &mut Builder) -> Solid {
    let v: Vec<Vertex> = vec![
        builder.vertex(Vector3::new(0, 0, 0)).unwrap(),
        builder.vertex(Vector3::new(4, 0, 0)).unwrap(),
        builder.vertex(Vector3::new(1, 0, 2)).unwrap(),
        builder.vertex(Vector3::new(3, 0, 2)).unwrap(),
    ];
    let edge = vec![
        builder.line(v[0], v[1]).unwrap(),
        builder
            .circle_arc(v[2], v[3], &Vector3::new(2, 0, 1))
            .unwrap(),
    ];
    let mut shell = builder.homotopy(&edge[0], &edge[1]).unwrap();
    let face1 = builder
        .create_rotated(
            &shell[0],
            &Vector3::new(2.0, 0.0, 3.5),
            &Vector3::new(0.0, 1.0, 0.0),
            std::f64::consts::PI,
        )
        .unwrap();
    let wire0 = shell[0].boundary();
    let wire1 = face1.boundary();
    let face2 = builder.homotopy(&wire0[1].inverse(), &wire1[3]).unwrap()[0].clone();
    let face3 = builder.homotopy(&wire0[3].inverse(), &wire1[1]).unwrap()[0].clone();
    shell.append(&mut vec![face1, face2, face3].into());
    builder
        .tsweep(&shell, &Vector3::new(0, 3, 0))
        .unwrap()
        .pop()
        .unwrap()
}

fn large_box(builder: &mut Builder) -> Solid {
    const N: usize = 100;

    let v: Vec<_> = (0..N)
        .flat_map(|i| (0..N).map(move |j| (i, j)))
        .map(|(i, j)| {
            builder
                .vertex(Vector3::new(i as f64, j as f64, 0.0))
                .unwrap()
        })
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
                .map(|j| builder.line(v[(i - 1) * N + j], v[(i % N) * N + j]).unwrap())
                .collect()
        })
        .collect();

    let shell: Shell = (1..N)
        .flat_map(|i| (1..N).map(move |j| (i, j)))
        .map(|(i, j)| {
            let wire = Wire::by_slice(&[
                row_edge[i - 1][j - 1],
                col_edge[i - 1][j],
                row_edge[i][j - 1].inverse(),
                col_edge[i - 1][j - 1].inverse(),
            ]);
            builder.plane(wire).unwrap()
        })
        .collect();
    builder.tsweep(&shell, &Vector3::new(0, 0, 1)).unwrap().pop().unwrap()
}

fn output_mesh<F>(director: &mut Director, function: F, filename: &str)
where F: FnOnce(&mut Builder) -> Solid {
    let instant = std::time::Instant::now();
    let solid = director.building(function);
    let integrity = director.check_integrity(&solid);
    assert_eq!(
        integrity,
        TopoGeomIntegrity::Integrate,
        "Integrate Error: {}",
        filename
    );
    let mesh = director.get_mesher().meshing(&solid, 0.01);
    let end_time = instant.elapsed();
    println!(
        "{}: {}.{:03} sec",
        filename,
        end_time.as_secs(),
        end_time.subsec_nanos() / 1_000_000,
    );
    let file = std::fs::File::create(filename).unwrap();
    truck_io::obj::write(&mesh, file).unwrap();
}

fn main() {
    let mut director = Director::new();
    output_mesh(&mut director, cube, "cube.obj");
    output_mesh(&mut director, bottle, "bottle.obj");
    output_mesh(&mut director, tsudsumi, "tsudsumi.obj");
    output_mesh(&mut director, truck3d, "truck3d.obj");
    for n in 3..=8 {
        let filename = format!("{}-gon-prism.obj", n);
        output_mesh(&mut director, |d| n_gon_prism(d, n), &filename);
    }
    output_mesh(&mut director, large_box, "large_plane.obj");
}
