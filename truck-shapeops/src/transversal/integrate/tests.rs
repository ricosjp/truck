use truck_meshalgo::prelude::*;
use truck_modeling::*;

#[test]
fn punched_cube() {
    let v = builder::vertex(Point3::origin());
    let e = builder::tsweep(&v, Vector3::unit_x());
    let f = builder::tsweep(&e, Vector3::unit_y());
    let cube: Solid = builder::tsweep(&f, Vector3::unit_z());

    let v = builder::vertex(Point3::new(0.5, 0.25, -0.5));
    let w = builder::rsweep(
        &v,
        Point3::new(0.5, 0.5, 0.0),
        Vector3::unit_z(),
        Rad(7.0),
        3,
    );
    let f = builder::try_attach_plane(&[w]).unwrap();
    let mut cylinder = builder::tsweep(&f, Vector3::unit_z() * 2.0);
    cylinder.not();
    let and = crate::and(&cube, &cylinder, 0.05).unwrap();

    let poly = and.triangulation(0.01).to_polygon();
    let file = std::fs::File::create("punched-cube.obj").unwrap();
    obj::write(&poly, file).unwrap();
}

#[test]
fn adjacent_cubes_or() {
    let v = builder::vertex(Point3::origin());
    let e = builder::tsweep(&v, Vector3::unit_x());
    let f = builder::tsweep(&e, Vector3::unit_y());
    let cube: Solid = builder::tsweep(&f, Vector3::unit_z());

    let v = builder::vertex(Point3::new(0.5, 0.5, 1.0));
    let w = builder::tsweep(&v, Vector3::unit_x());
    let f = builder::tsweep(&w, Vector3::unit_y());
    let cube2: Solid = builder::tsweep(&f, Vector3::unit_z());

    let result = crate::or(&cube, &cube2, 0.05);
    assert!(
        result.is_some(),
        "Boolean OR of adjacent cubes should succeed"
    );
    let solid = result.unwrap();

    // The result should be a single solid (one shell)
    assert_eq!(solid.boundaries().len(), 1);

    // Triangulate to check volume and bounding box
    let poly = solid.triangulation(0.01).to_polygon();

    // The volume of two unit cubes sharing only a portion of their boundary should be 2.0
    assert_near!(poly.volume(), 2.0);

    // Check the center of gravity
    let homog = poly.center_of_gravity();
    assert_near!(homog.to_point(), Point3::new(0.75, 0.75, 1.0));

    // Check the bounding box
    let bbx = poly.bounding_box();
    assert_near!(bbx.min(), Point3::new(0.0, 0.0, 0.0));
    assert_near!(bbx.max(), Point3::new(1.5, 1.5, 2.0));

    // Optional: check face count.
    // Two cubes normally have 6 faces each.
    // After union, the internal shared part is removed.
    // We expect around 12 faces as analyzed.
    assert_eq!(solid.face_iter().count(), 12);
}
