use truck_geometry::*;
use truck_polymesh::*;
use truck_shape::*;
use truck_topology::*;

fn cube() -> Geometry {
    let v = Vertex::news(8);
    let edge = [
        Edge::new(v[0], v[1]), // 0
        Edge::new(v[1], v[2]), // 1
        Edge::new(v[2], v[3]), // 2
        Edge::new(v[3], v[0]), // 3
        Edge::new(v[0], v[4]), // 4
        Edge::new(v[1], v[5]), // 5
        Edge::new(v[2], v[6]), // 6
        Edge::new(v[3], v[7]), // 7
        Edge::new(v[4], v[5]), // 8
        Edge::new(v[5], v[6]), // 9
        Edge::new(v[6], v[7]), // 10
        Edge::new(v[7], v[4]), // 11
    ];

    let mut wire = vec![Wire::new(); 6];

    wire[0].push_back(edge[0]);
    wire[0].push_back(edge[1]);
    wire[0].push_back(edge[2]);
    wire[0].push_back(edge[3]);

    wire[1].push_back(edge[4]);
    wire[1].push_back(edge[8]);
    wire[1].push_back(edge[5].inverse());
    wire[1].push_back(edge[0].inverse());

    wire[2].push_back(edge[5]);
    wire[2].push_back(edge[9]);
    wire[2].push_back(edge[6].inverse());
    wire[2].push_back(edge[1].inverse());

    wire[3].push_back(edge[6]);
    wire[3].push_back(edge[10]);
    wire[3].push_back(edge[7].inverse());
    wire[3].push_back(edge[2].inverse());
    wire[4].push_back(edge[7]);
    wire[4].push_back(edge[11]);
    wire[4].push_back(edge[4].inverse());
    wire[4].push_back(edge[3].inverse());

    wire[5].push_back(edge[11].inverse());
    wire[5].push_back(edge[10].inverse());
    wire[5].push_back(edge[9].inverse());
    wire[5].push_back(edge[8].inverse());

    let face: Vec<Face> = wire.into_iter().map(|x| Face::new(x)).collect();

    let knot_vec = KnotVec::from(vec![0.0, 0.0, 1.0, 1.0]);
    let pt = vec![
        Vector::new3(0.0, 0.0, 0.0),
        Vector::new3(0.0, 1.0, 0.0),
        Vector::new3(1.0, 1.0, 0.0),
        Vector::new3(1.0, 0.0, 0.0),
        Vector::new3(0.0, 0.0, 1.0),
        Vector::new3(0.0, 1.0, 1.0),
        Vector::new3(1.0, 1.0, 1.0),
        Vector::new3(1.0, 0.0, 1.0),
    ];

    let curve = vec![
        BSplineCurve::new(knot_vec.clone(), vec![pt[0].clone(), pt[1].clone()]),
        BSplineCurve::new(knot_vec.clone(), vec![pt[1].clone(), pt[2].clone()]),
        BSplineCurve::new(knot_vec.clone(), vec![pt[2].clone(), pt[3].clone()]),
        BSplineCurve::new(knot_vec.clone(), vec![pt[3].clone(), pt[0].clone()]),
        BSplineCurve::new(knot_vec.clone(), vec![pt[0].clone(), pt[4].clone()]),
        BSplineCurve::new(knot_vec.clone(), vec![pt[1].clone(), pt[5].clone()]),
        BSplineCurve::new(knot_vec.clone(), vec![pt[2].clone(), pt[6].clone()]),
        BSplineCurve::new(knot_vec.clone(), vec![pt[3].clone(), pt[7].clone()]),
        BSplineCurve::new(knot_vec.clone(), vec![pt[4].clone(), pt[5].clone()]),
        BSplineCurve::new(knot_vec.clone(), vec![pt[5].clone(), pt[6].clone()]),
        BSplineCurve::new(knot_vec.clone(), vec![pt[6].clone(), pt[7].clone()]),
        BSplineCurve::new(knot_vec.clone(), vec![pt[7].clone(), pt[4].clone()]),
    ];

    let surface = vec![
        BSplineSurface::new(
            (knot_vec.clone(), knot_vec.clone()),
            vec![
                vec![pt[0].clone(), pt[3].clone()],
                vec![pt[1].clone(), pt[2].clone()],
            ],
        ),
        BSplineSurface::new(
            (knot_vec.clone(), knot_vec.clone()),
            vec![
                vec![pt[0].clone(), pt[1].clone()],
                vec![pt[4].clone(), pt[5].clone()],
            ],
        ),
        BSplineSurface::new(
            (knot_vec.clone(), knot_vec.clone()),
            vec![
                vec![pt[1].clone(), pt[2].clone()],
                vec![pt[5].clone(), pt[6].clone()],
            ],
        ),
        BSplineSurface::new(
            (knot_vec.clone(), knot_vec.clone()),
            vec![
                vec![pt[2].clone(), pt[3].clone()],
                vec![pt[6].clone(), pt[7].clone()],
            ],
        ),
        BSplineSurface::new(
            (knot_vec.clone(), knot_vec.clone()),
            vec![
                vec![pt[3].clone(), pt[0].clone()],
                vec![pt[7].clone(), pt[4].clone()],
            ],
        ),
        BSplineSurface::new(
            (knot_vec.clone(), knot_vec.clone()),
            vec![
                vec![pt[4].clone(), pt[5].clone()],
                vec![pt[7].clone(), pt[6].clone()],
            ],
        ),
    ];

    let mut geometry = Geometry::new();
    for (it0, it1) in edge.iter().zip(curve.into_iter()) {
        geometry.attach_curve(it0, it1).unwrap();
    }
    for (it0, it1) in face.iter().zip(surface.into_iter()) {
        geometry.attach_surface(it0, it1).unwrap();
    }

    geometry
}

fn tsudsumi() -> Geometry {
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

    let mut geometry = Geometry::new();
    for (v, p) in v.iter().zip(pt.into_iter()) {
        geometry.attach_point(v, p).unwrap();
    }
    for (e, c) in edge.iter().zip(curve.into_iter()) {
        geometry.attach_curve(e, c).unwrap();
    }
    for (f, s) in face.iter().zip(surface.into_iter()) {
        geometry.attach_surface(f, s).unwrap();
    }

    geometry
}

fn main() {
    let mesh = PolygonMesh::from_shape(&mut cube(), 0.01);
    let file = std::fs::File::create("cube.obj").unwrap();
    truck_io::obj::write(&mesh, file).unwrap();

    let mesh = PolygonMesh::from_shape(&mut tsudsumi(), 0.01);
    let file = std::fs::File::create("tsudsumi.obj").unwrap();
    truck_io::obj::write(&mesh, file).unwrap();
}
