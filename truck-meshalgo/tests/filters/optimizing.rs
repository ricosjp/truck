use truck_meshalgo::filters::*;
use truck_polymesh::*;

#[test]
fn remove_unused_attrs_test() {
    const N: usize = 100;
    let positions: Vec<_> = (0..N)
        .map(|i| {
            let mut p = Point3::origin();
            p[i % 3] = i as f64;
            p
        })
        .collect();
    let uv_coords: Vec<_> = (0..N)
        .map(|i| {
            let mut uv = Vector2::zero();
            uv[i % 2] = i as f64 / N as f64;
            uv
        })
        .collect();
    let normals: Vec<_> = (0..N)
        .map(|i| {
            let mut n = Vector3::zero();
            n[(i + 1) % 3] = i as f64 / N as f64;
            n
        })
        .collect();
    let faces = Faces::from_iter(&[
        [[0, 1, 2], [1, 2, 3], [2, 3, 4]].as_ref(),
        &[[1, 2, 3], [2, 3, 4], [3, 4, 5], [4, 5, 6]],
        &[[2, 3, 4], [3, 4, 5], [4, 5, 6], [5, 6, 7], [6, 7, 8]],
    ]);
    let mut mesh = PolygonMesh::new(
        StandardAttributes {
            positions,
            uv_coords,
            normals,
        },
        faces,
    );
    mesh.remove_unused_attrs();
    assert_eq!(mesh.positions().len(), 7);
    assert_eq!(mesh.uv_coords().len(), 7);
    assert_eq!(mesh.normals().len(), 7);
    assert_eq!(mesh.faces().len(), 3);
}

#[test]
fn remove_degenerate_faces_test() {
    const N: usize = 100;
    let positions: Vec<_> = (0..N)
        .map(|i| {
            let mut p = Point3::origin();
            p[i % 3] = i as f64;
            p
        })
        .collect();
    let faces = Faces::from_iter(&[
        [0, 1, 2].as_ref(),     // survive
        &[0, 0, 0],             // death
        &[1, 0, 0],             // death
        &[0, 1, 0],             // death
        &[0, 0, 1],             // death
        &[0, 1, 2, 3],          // survive
        &[0, 0, 0, 0],          // death
        &[1, 0, 0, 0],          // death
        &[0, 1, 0, 0],          // death
        &[0, 0, 1, 0],          // death
        &[0, 0, 0, 1],          // death
        &[1, 1, 0, 0],          // death
        &[1, 0, 0, 1],          // death
        &[0, 1, 0, 1],          // death
        &[0, 0, 1, 2],          // triangle
        &[0, 1, 2, 0],          // triangle
        &[1, 2, 0, 0],          // triangle
        &[0, 1, 0, 2],          // death
        &[0, 1, 2, 3, 4],       //  survive
        &[0, 1, 2, 3, 0, 1, 2], // quadrangle + triangle
        &[0, 1, 2, 0, 3, 4],    // two triangles
        &[0, 1, 0, 2, 0, 3],    // death
    ]);
    let mut mesh = PolygonMesh::new(
        StandardAttributes {
            positions,
            ..Default::default()
        },
        faces,
    );
    mesh.remove_degenerate_faces();
    assert_eq!(
        mesh.faces().tri_faces().len(),
        7,
        "{:?}",
        mesh.faces().tri_faces()
    );
    assert_eq!(mesh.faces().quad_faces().len(), 2);
    assert_eq!(mesh.faces().other_faces().len(), 1);
}

#[test]
fn put_together_same_attrs_test() {
    const N_POS: usize = 12 * 19;
    const N_UV: usize = 13 * 18;
    const N_NOR: usize = 14 * 17; // <- Largest!

    let positions = (0..N_POS)
        .map(|i| Point3::new((i * 12 % 19) as f64, 0.0, 0.0))
        .collect::<Vec<_>>();
    let uv_coords = (0..N_UV)
        .map(|i| Vector2::new((i * 13 % 18) as f64, 0.0))
        .collect::<Vec<_>>();
    let normals = (0..N_NOR)
        .map(|i| Vector3::new((i * 14 % 17) as f64, 0.0, 0.0))
        .collect::<Vec<_>>();
    let mut faces = Faces::default();
    for i in 0..N_NOR {
        match i % 4 {
            0 => faces.push([
                [i % N_POS, (i + 15) % N_UV, (i * 24) % N_NOR],
                [(i + 1) % N_POS, (i + 16) % N_UV, (i * 24 + 1) % N_NOR],
                [(i + 2) % N_POS, (i + 17) % N_UV, (i * 24 + 2) % N_NOR],
            ]),
            1 => faces.push([
                [i % N_POS, (i + 15) % N_UV, (i * 24) % N_NOR],
                [(i + 1) % N_POS, (i + 16) % N_UV, (i * 24 + 1) % N_NOR],
                [(i + 2) % N_POS, (i + 17) % N_UV, (i * 24 + 2) % N_NOR],
                [(i + 3) % N_POS, (i + 18) % N_UV, (i * 24 + 3) % N_NOR],
            ]),
            2 => faces.push([
                [i % N_POS, (i + 15) % N_UV, (i * 24) % N_NOR],
                [(i + 1) % N_POS, (i + 16) % N_UV, (i * 24 + 1) % N_NOR],
                [(i + 2) % N_POS, (i + 17) % N_UV, (i * 24 + 2) % N_NOR],
                [(i + 3) % N_POS, (i + 18) % N_UV, (i * 24 + 3) % N_NOR],
                [(i + 4) % N_POS, (i + 19) % N_UV, (i * 24 + 4) % N_NOR],
            ]),
            3 => faces.push([
                [i % N_POS, (i + 15) % N_UV, (i * 24) % N_NOR],
                [(i + 1) % N_POS, (i + 16) % N_UV, (i * 24 + 1) % N_NOR],
                [(i + 2) % N_POS, (i + 17) % N_UV, (i * 24 + 2) % N_NOR],
                [(i + 3) % N_POS, (i + 18) % N_UV, (i * 24 + 3) % N_NOR],
                [(i + 4) % N_POS, (i + 19) % N_UV, (i * 24 + 4) % N_NOR],
                [(i + 5) % N_POS, (i + 20) % N_UV, (i * 24 + 5) % N_NOR],
            ]),
            _ => {}
        }
    }
    let mut mesh = PolygonMesh::new(
        StandardAttributes {
            positions,
            uv_coords,
            normals,
        },
        faces,
    );
    let attrs: Vec<_> = mesh
        .faces()
        .face_iter()
        .flatten()
        .map(|v| {
            (
                mesh.positions()[v.pos],
                mesh.uv_coords()[v.uv.unwrap()],
                mesh.normals()[v.nor.unwrap()],
            )
        })
        .collect();

    mesh.put_together_same_attrs();
    mesh.faces()
        .face_iter()
        .flatten()
        .zip(&attrs)
        .for_each(|(v, attr)| {
            assert!(mesh.positions()[v.pos].near(&attr.0));
            assert!(mesh.uv_coords()[v.uv.unwrap()].near(&attr.1));
            assert!(mesh.normals()[v.nor.unwrap()].near(&attr.2));
        });

    mesh.remove_unused_attrs();
    mesh.faces()
        .face_iter()
        .flatten()
        .zip(&attrs)
        .for_each(|(v, attr)| {
            assert!(mesh.positions()[v.pos].near(&attr.0));
            assert!(mesh.uv_coords()[v.uv.unwrap()].near(&attr.1));
            assert!(mesh.normals()[v.nor.unwrap()].near(&attr.2));
        });
    assert_eq!(mesh.positions().len(), 19);
    assert_eq!(mesh.uv_coords().len(), 18);
    assert_eq!(mesh.normals().len(), 17);
}
