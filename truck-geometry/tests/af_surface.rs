use std::f64::consts::PI;
use truck_geometry::prelude::{rbf_surface::RadiusFunction, *};

#[test]
fn approx_fillet_between_two_spheres() {
    let sphere0 = Sphere::new(Point3::new(0.0, 0.0, 10.0), 20.0);
    let sphere1 = Sphere::new(Point3::new(0.0, 0.0, -10.0), 20.0);
    let edge_circle = Processor::with_transform(
        UnitCircle::<Point3>::new(),
        Matrix4::from_scale(10.0 * f64::sqrt(3.0)),
    );

    #[derive(Clone, Copy, Debug)]
    struct Radius;
    impl RadiusFunction for Radius {
        fn der_n(&self, n: usize, t: f64) -> f64 {
            let o = if n == 0 { 10.0 } else { 0.0 };
            let x = match n % 4 {
                0 => f64::cos(t),
                1 => -f64::sin(t),
                2 => -f64::cos(t),
                _ => f64::sin(t),
            };
            o + 5.0 * x
        }
    }

    let fillet = RbfSurface::new(edge_circle, sphere0, sphere1, Radius);

    let instance = std::time::Instant::now();
    let approx =
        ApproxFilletSurface::approx_rolling_ball_fillet(&fillet, (PI * 0.1, PI * 1.9), 0.001)
            .unwrap();
    println!("fillet approximation: {}ms", instance.elapsed().as_millis());

    let instance = std::time::Instant::now();
    let _ = fillet.parameter_division(((0.0, 1.0), (PI * 0.1, PI * 1.9)), 0.005);
    println!(
        "tessellate strict fillet: {}ms",
        instance.elapsed().as_millis()
    );

    let instance = std::time::Instant::now();
    let _ = approx.parameter_division(((0.0, 1.0), (PI * 0.1, PI * 1.9)), 0.005);
    println!(
        "tessellate fillet approx: {}ms",
        instance.elapsed().as_millis()
    );
}
