use std::f64::consts::FRAC_PI_2;
use truck_geometry::prelude::*;
use truck_stepio::r#in::step_geometry::Sphere as StepSphere;

#[test]
fn step_sphere_preserves_longitude_hint_at_both_poles() {
    let sphere = StepSphere(Sphere::new(Point3::origin(), 10.));
    for latitude in [-FRAC_PI_2, FRAC_PI_2] {
        let expected = (1.2, latitude);
        let point = sphere.subs(expected.0, expected.1);
        let found = sphere.search_parameter(point, Some(expected), 100).unwrap();
        assert!((found.0 - expected.0).abs() < 1e-12);
        assert!((found.1 - expected.1).abs() < 1e-12);
    }
}

#[test]
fn inverted_step_sphere_preserves_longitude_hint() {
    let sphere = StepSphere(Sphere::new(Point3::origin(), 10.));
    let mut surface = Processor::<_, Matrix4>::new(sphere);
    surface.invert();
    for latitude in [-FRAC_PI_2, FRAC_PI_2] {
        let expected = (latitude, 1.2);
        let point = surface.subs(expected.0, expected.1);
        let found = surface
            .search_parameter(point, Some(expected), 100)
            .unwrap();
        assert!((found.0 - expected.0).abs() < 1e-12);
        assert!((found.1 - expected.1).abs() < 1e-12);
    }
}

#[test]
fn nearest_sphere_parameter_is_finite_at_poles() {
    let sphere = Sphere::new(Point3::origin(), 10.);
    for point in [Point3::new(0., 0., 12.), Point3::new(0., 0., -12.)] {
        let found = sphere
            .search_nearest_parameter(point, Some((0., 1.2)), 100)
            .unwrap();
        assert!(found.0.is_finite());
        assert!((found.1 - 1.2).abs() < 1e-12);
        let nearest = sphere.subs(found.0, found.1);
        assert!((nearest.distance(point) - 2.).abs() < 1e-12);
    }
}
