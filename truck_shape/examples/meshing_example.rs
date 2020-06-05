use truck_geometry::*;
use truck_polymesh::*;
use truck_shape::director::TopoGeomIntegrity;
use truck_shape::*;
use truck_topology::*;

fn cube(director: &mut Director) -> Solid {
    let v = director.create_vertex(Vector::new3(0.0, 0.0, 0.0));
    let edge = director.tsweep(&v, &Vector3::new(1.0, 0.0, 0.0)).unwrap();
    let face = director.tsweep(&edge, &Vector3::new(0.0, 1.0, 0.0)).unwrap();
    let solid = director.tsweep(&face, &Vector3::new(0.0, 0.0, 1.0)).unwrap();
    director.remove_surface(face);
    solid
}

fn bottle(director: &mut Director, width: f64, thick: f64, height: f64) -> Solid {
    let v0 = director.create_vertex(Vector::new3(-width / 2.0, -thick / 4.0, 0.0));
    let v1 = director.create_vertex(Vector::new3(width / 2.0, -thick / 4.0, 0.0));
    let transit = Vector3::new(0.0, -thick / 2.0, 0.0);
    let edge0 = director.circle_arc(v0, v1, &transit).unwrap();
    let edge1 = director.create_rotated(
        &edge0,
        &Vector3::new(0.0, 0.0, 0.0),
        &Vector3::new(0.0, 0.0, 1.0),
        std::f64::consts::PI,
    ).unwrap();
    let wire0 = Wire::by_slice(&[edge0]);
    let wire1 = Wire::by_slice(&[edge1.inverse()]);
    let face = director.homotopy(&wire0, &wire1).unwrap();
    director.tsweep(&face, &Vector3::new(0.0, 0.0, height)).unwrap()
}

fn tsudsumi() -> Director {
    let v = Vertex::news(4);
    let edge = [
        Edge::new(v[0], v[1]),
        Edge::new(v[0], v[1]),
        Edge::new(v[0], v[2]),
        Edge::new(v[1], v[3]),
        Edge::new(v[2], v[3]),
        Edge::new(v[2], v[3]),
    ];

    let mut wire = Vec::new();

    wire.push(Wire::new());
    wire[0].push_back(edge[0]);
    wire[0].push_back(edge[1].inverse());
    wire.push(Wire::new());
    wire[1].push_back(edge[2]);
    wire[1].push_back(edge[4]);
    wire[1].push_back(edge[3].inverse());
    wire[1].push_back(edge[0].inverse());

    wire.push(Wire::new());
    wire[2].push_back(edge[1]);
    wire[2].push_back(edge[3]);
    wire[2].push_back(edge[5].inverse());
    wire[2].push_back(edge[2].inverse());

    wire.push(Wire::new());
    wire[3].push_back(edge[5]);
    wire[3].push_back(edge[4].inverse());

    let mut face = Vec::new();
    for w in wire.into_iter() {
        face.push(Face::new(w));
    }

    let pt = vec![
        Vector::new3(0.0, 2.0, 0.0),
        Vector::new3(0.0, 2.0, 2.0),
        Vector::new3(-1.0, 0.0, 1.0),
        Vector::new3(1.0, 0.0, 1.0),
    ];

    let curve = vec![
        BSplineCurve::new(
            KnotVec::try_from(vec![0.0, 0.0, 0.0, 0.5, 0.5, 1.0, 1.0, 1.0]).unwrap(),
            vec![
                Vector::new(0.0, 2.0, 0.0, 1.0),
                Vector::new(-1.0, 2.0, 0.0, 1.0),
                Vector::new(-1.0, 2.0, 1.0, 1.0) * 2.0,
                Vector::new(-1.0, 2.0, 2.0, 1.0),
                Vector::new(0.0, 2.0, 2.0, 1.0),
            ],
        ),
        BSplineCurve::new(
            KnotVec::try_from(vec![0.0, 0.0, 0.0, 0.5, 0.5, 1.0, 1.0, 1.0]).unwrap(),
            vec![
                Vector::new(0.0, 2.0, 0.0, 1.0) * 2.0,
                Vector::new(1.0, 2.0, 0.0, 1.0),
                Vector::new(1.0, 2.0, 1.0, 1.0),
                Vector::new(1.0, 2.0, 2.0, 1.0),
                Vector::new(0.0, 2.0, 2.0, 1.0) * 2.0,
            ],
        ),
        BSplineCurve::new(
            KnotVec::try_from(vec![0.0, 0.0, 1.0, 1.0]).unwrap(),
            vec![pt[0].clone(), pt[2].clone()],
        ),
        BSplineCurve::new(
            KnotVec::try_from(vec![0.0, 0.0, 1.0, 1.0]).unwrap(),
            vec![pt[1].clone(), pt[3].clone()],
        ),
        BSplineCurve::new(
            KnotVec::try_from(vec![0.0, 0.0, 0.0, 0.5, 0.5, 1.0, 1.0, 1.0]).unwrap(),
            vec![
                Vector::new(-1.0, 0.0, 1.0, 1.0),
                Vector::new(-1.0, 0.0, 2.0, 1.0),
                Vector::new(0.0, 0.0, 2.0, 1.0) * 2.0,
                Vector::new(1.0, 0.0, 2.0, 1.0),
                Vector::new(1.0, 0.0, 1.0, 1.0),
            ],
        ),
        BSplineCurve::new(
            KnotVec::try_from(vec![0.0, 0.0, 0.0, 0.5, 0.5, 1.0, 1.0, 1.0]).unwrap(),
            vec![
                Vector::new(-1.0, 0.0, 1.0, 1.0) * 2.0,
                Vector::new(-1.0, 0.0, 0.0, 1.0),
                Vector::new(0.0, 0.0, 0.0, 1.0),
                Vector::new(1.0, 0.0, 0.0, 1.0),
                Vector::new(1.0, 0.0, 1.0, 1.0) * 2.0,
            ],
        ),
    ];

    let mut surface = vec![
        BSplineSurface::homotopy(&curve[0], &curve[1]),
        BSplineSurface::homotopy(&curve[0], &curve[4]),
        BSplineSurface::homotopy(&curve[1], &curve[5]),
        BSplineSurface::homotopy(&curve[5], &curve[4]),
    ];
    surface[1].swap_axes();

    let mut geometry = Director::new();
    for (v, p) in v.iter().zip(pt.into_iter()) {
        geometry.insert_point(v, p);
    }
    for (e, c) in edge.iter().zip(curve.into_iter()) {
        geometry.insert_curve(e, c);
    }
    for (f, s) in face.iter().zip(surface.into_iter()) {
        geometry.insert_surface(f, s);
    }

    geometry
}

fn main() {
    let mut director = Director::new();
    let solid = cube(&mut director);
    let integrity = director.check_solid_integrity(&solid);
    assert_eq!(integrity, TopoGeomIntegrity::Integrate);
    let mesh = StructuredMesh::from_shape(&mut director, 0.01);
    let file = std::fs::File::create("cube.obj").unwrap();
    truck_io::obj::write(&mesh, file).unwrap();
    
    let mut director = Director::new();
    let solid = bottle(&mut director, 6.0, 4.0, 10.0);
    let integrity = director.check_solid_integrity(&solid);
    assert_eq!(integrity, TopoGeomIntegrity::Integrate);
    let mesh = StructuredMesh::from_shape(&mut director, 0.01);
    let file = std::fs::File::create("bottle.obj").unwrap();
    truck_io::obj::write(&mesh, file).unwrap();

    let mut geom = tsudsumi();
    let mesh = StructuredMesh::from_shape(&mut geom, 0.01);
    let file = std::fs::File::create("tsudsumi.obj").unwrap();
    truck_io::obj::write(&mesh, file).unwrap();
}
