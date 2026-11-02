use std::f64::consts::FRAC_PI_2;
use truck_geometry::prelude::*;
use truck_meshalgo::prelude::*;
use truck_topology::compress::*;

fn assert_pole_boundary_matches_its_parameters(invert: bool) {
    let mut surface = Processor::<_, Matrix4>::new(Sphere::new(Point3::origin(), 10.));
    if invert {
        surface.invert();
    }
    let uv = |u, v| {
        if invert {
            Point2::new(v, u)
        } else {
            Point2::new(u, v)
        }
    };
    let a = uv(FRAC_PI_2, 0.);
    let b = uv(0., 0.);
    let b_other_meridian = uv(0., FRAC_PI_2);
    let c = uv(FRAC_PI_2, FRAC_PI_2);
    let vertices = [a, b, c].map(|p| surface.subs(p.x, p.y)).to_vec();
    // The two parameter values at the north pole name the same 3D vertex.
    let curves = [(a, b), (b_other_meridian, c), (c, a)];
    let edges = curves
        .into_iter()
        .enumerate()
        .map(|(i, (p, q))| CompressedEdge {
            vertices: (i, (i + 1) % 3),
            curve: PCurve::new(Line(p, q), surface),
        })
        .collect();
    let shell = CompressedShell {
        vertices,
        edges,
        faces: vec![CompressedFace {
            surface,
            orientation: true,
            boundaries: vec![(0..3)
                .map(|index| CompressedEdgeIndex {
                    index,
                    orientation: true,
                })
                .collect()],
        }],
    };
    let polygon = shell.robust_triangulation(0.1).to_polygon();
    assert!(polygon.faces().triangle_iter().next().is_some());
    for triangle in polygon.faces().triangle_iter() {
        for vertex in triangle {
            let p = polygon.positions()[vertex.pos];
            let uv = polygon.uv_coords()[vertex.uv.unwrap()];
            assert!(
                p.distance(surface.subs(uv.x, uv.y)) < 1e-6,
                "pole UV and 3D vertex disagree: p={p:?} uv={uv:?}, error={}",
                p.distance(surface.subs(uv.x, uv.y))
            );
        }
    }
}

#[test]
fn leaving_a_degenerate_v_boundary_preserves_the_previous_point() {
    assert_pole_boundary_matches_its_parameters(false);
}

#[test]
fn leaving_a_degenerate_u_boundary_preserves_the_previous_point() {
    assert_pole_boundary_matches_its_parameters(true);
}
