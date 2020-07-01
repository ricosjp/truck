use truck_geometry::*;
use truck_io::GeomDataRef;
type BSplineCurve = truck_geometry::BSplineCurve<[f64; 4]>;
type BSplineSurface = truck_geometry::BSplineSurface<[f64; 4]>;

// exporting sample geometric data to:
const EXPORT_PATH: &str = "tests/data/examples.tgb";

fn typical_2degree_curve() -> BSplineCurve {
    let knot_vec = KnotVec::from(vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 3.0]);
    let control_points = vec![
        vector!(0.0, 0.0, 0.0, 0.0),
        vector!(1.0, 0.0, 0.0, 0.0),
        vector!(0.0, 1.0, 0.0, 0.0),
        vector!(0.0, 0.0, 1.0, 0.0),
        vector!(0.0, 0.0, 0.0, 1.0),
    ];

    BSplineCurve::new(knot_vec, control_points)
}

fn unclamped() -> BSplineCurve {
    let knot_vec = KnotVec::from(vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0]);

    let control_points = vec![
        Vector4::new3(1.0, 0.0, 0.0),
        Vector4::new3(0.0, 1.0, 0.0),
        Vector4::new3(0.0, 0.0, 1.0),
    ];

    BSplineCurve::new(knot_vec, control_points)
}

fn circle_in_projection() -> BSplineCurve {
    let knot_vec = KnotVec::from(vec![
        0.0, 0.0, 0.0, 0.25, 0.25, 0.5, 0.5, 0.75, 0.75, 1.0, 1.0, 1.0,
    ]);

    let control_points = vec![
        Vector4::new3(0.0, 0.0, -1.0) * 2.0,
        Vector4::new3(1.0, 0.0, -1.0),
        Vector4::new3(1.0, 0.0, 0.0),
        Vector4::new3(1.0, 0.0, 1.0),
        Vector4::new3(0.0, 0.0, 1.0) * 2.0,
        Vector4::new3(-1.0, 0.0, 1.0),
        Vector4::new3(-1.0, 0.0, 0.0),
        Vector4::new3(-1.0, 0.0, -1.0),
        Vector4::new3(0.0, 0.0, -1.0) * 2.0,
    ];

    let mut bspcurve = BSplineCurve::new(knot_vec, control_points);
    bspcurve.optimize();
    bspcurve
}

fn full_sphere() -> BSplineSurface {
    // the knot vectors
    let knot_vec0 = KnotVec::from(vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]);
    let knot_vec1 = KnotVec::from(vec![0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.5, 1.0, 1.0, 1.0, 1.0]);

    // sign up the control points in the vector of all points
    let mut v = vec![vec![Vector4::zero(); 7]; 4];
    v[0][0] = Vector4::new3(0.0, 0.0, 1.0);
    v[0][1] = &v[0][0] / 3.0;
    v[0][2] = v[0][1].clone();
    v[0][3] = v[0][0].clone();
    v[0][4] = v[0][1].clone();
    v[0][5] = v[0][1].clone();
    v[0][6] = v[0][0].clone();
    v[1][0] = Vector4::new3(2.0, 0.0, 1.0) / 3.0;
    v[1][1] = Vector4::new3(2.0, 4.0, 1.0) / 9.0;
    v[1][2] = Vector4::new3(-2.0, 4.0, 1.0) / 9.0;
    v[1][3] = Vector4::new3(-2.0, 0.0, 1.0) / 3.0;
    v[1][4] = Vector4::new3(-2.0, -4.0, 1.0) / 9.0;
    v[1][5] = Vector4::new3(2.0, -4.0, 1.0) / 9.0;
    v[1][6] = Vector4::new3(2.0, 0.0, 1.0) / 3.0;
    v[2][0] = Vector4::new3(2.0, 0.0, -1.0) / 3.0;
    v[2][1] = Vector4::new3(2.0, 4.0, -1.0) / 9.0;
    v[2][2] = Vector4::new3(-2.0, 4.0, -1.0) / 9.0;
    v[2][3] = Vector4::new3(-2.0, 0.0, -1.0) / 3.0;
    v[2][4] = Vector4::new3(-2.0, -4.0, -1.0) / 9.0;
    v[2][5] = Vector4::new3(2.0, -4.0, -1.0) / 9.0;
    v[2][6] = Vector4::new3(2.0, 0.0, -1.0) / 3.0;
    v[3][0] = Vector4::new3(0.0, 0.0, -1.0);
    v[3][1] = &v[3][0] / 3.0;
    v[3][2] = v[3][1].clone();
    v[3][3] = v[3][0].clone();
    v[3][4] = v[3][1].clone();
    v[3][5] = v[3][1].clone();
    v[3][6] = v[3][0].clone();

    // construct the B-spline curve
    BSplineSurface::new((knot_vec0, knot_vec1), v)
}

fn one_sheet_hyperboloid() -> BSplineSurface {
    // the knot vectors
    let knot_vec0 = KnotVec::from(vec![0.0, 0.0, 1.0, 1.0]);
    let knot_vec1 = KnotVec::from(vec![
        0.0, 0.0, 0.0, 0.25, 0.25, 0.5, 0.5, 0.75, 0.75, 1.0, 1.0, 1.0,
    ]);
    let knot_vecs = (knot_vec0, knot_vec1);

    // the control points
    let control_points0 = vec![
        vector!(0.0, -2.0, 2.0, 2.0),
        vector!(1.0, -1.0, 1.0, 1.0),
        vector!(1.0, 0.0, 1.0, 1.0),
        vector!(1.0, 1.0, 1.0, 1.0),
        vector!(0.0, 2.0, 2.0, 2.0),
        vector!(-1.0, 1.0, 1.0, 1.0),
        vector!(-1.0, 0.0, 1.0, 1.0),
        vector!(-1.0, -1.0, 1.0, 1.0),
        vector!(0.0, -2.0, 2.0, 2.0),
    ];
    let control_points1 = vec![
        vector!(2.0, 0.0, -2.0, 2.0),
        vector!(1.0, 1.0, -1.0, 1.0),
        vector!(0.0, 1.0, -1.0, 1.0),
        vector!(-1.0, 1.0, -1.0, 1.0),
        vector!(-2.0, 0.0, -2.0, 2.0),
        vector!(-1.0, -1.0, -1.0, 1.0),
        vector!(0.0, -1.0, -1.0, 1.0),
        vector!(1.0, -1.0, -1.0, 1.0),
        vector!(2.0, 0.0, -2.0, 2.0),
    ];

    // construct the B-spline surface
    BSplineSurface::new(knot_vecs, vec![control_points0, control_points1])
}

fn disk() -> BSplineSurface {
    // the knot vector
    let knot_vec0 = KnotVec::from(vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    let knot_vec1 = knot_vec0.clone();

    let control_points0 = vec![
        vector!(1.0, 0.0, 0.0, 1.0),
        vector!(1.0, 0.0, 1.0, 1.0),
        vector!(0.0, 0.0, 2.0, 2.0),
    ];

    let control_points1 = vec![
        vector!(1.0, 0.0, -1.0, 1.0),
        vector!(0.0, 0.0, 0.0, 1.0),
        vector!(-2.0, 0.0, 2.0, 2.0),
    ];

    let control_points2 = vec![
        vector!(0.0, 0.0, -2.0, 2.0),
        vector!(-2.0, 0.0, -2.0, 2.0),
        vector!(-4.0, 0.0, 0.0, 4.0),
    ];

    let control_points = vec![control_points0, control_points1, control_points2];
    BSplineSurface::new((knot_vec0, knot_vec1), control_points)
}

fn main() {
    let curves = vec![typical_2degree_curve(), unclamped(), circle_in_projection()];
    let curves_ref = curves.iter().collect();

    let surfaces = vec![full_sphere(), one_sheet_hyperboloid(), disk()];
    let surfaces_ref = surfaces.iter().collect();

    let geomdata = GeomDataRef {
        curves: curves_ref,
        surfaces: surfaces_ref,
    };

    let file = std::fs::File::create(EXPORT_PATH).unwrap();
    truck_io::tgb::write(&geomdata, file).unwrap();
}
