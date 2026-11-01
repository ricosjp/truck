use std::time::{Duration, Instant};
use truck_base::cgmath64::*;
use truck_meshalgo::prelude::*;
use truck_modeling::*;

fn bench_fn<F: FnMut()>(name: &str, mut f: F, iters: usize) -> Duration {
    // Warmup
    f();

    let start = Instant::now();
    for _ in 0..iters {
        f();
    }
    let elapsed = start.elapsed() / iters as u32;
    println!("  {:<35} : {:>8.2} ms", name, elapsed.as_secs_f64() * 1000.0);
    elapsed
}

fn create_punched_cube_solids() -> (Solid, Solid) {
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

    (cube, cylinder)
}

fn main() {
    println!("\n=================================================================================================================================");
    println!("                                      TRUCK SOLID BOOLEAN & TESSELLATION BENCHMARK                                               ");
    println!("=================================================================================================================================\n");

    let (cube, cylinder) = create_punched_cube_solids();
    let tol = 0.05;

    println!(">>> Benchmarking End-to-End Solid Boolean AND Operation (Cube AND Inverted Cylinder)...");
    let t_bool = bench_fn("Solid Boolean Operation (crate::and)", || {
        let res = truck_shapeops::and(&cube, &cylinder, tol).unwrap();
        std::hint::black_box(res);
    }, 5);

    let res = truck_shapeops::and(&cube, &cylinder, tol).unwrap();
    println!("    Resulting solid contains {} boundary shells, {} faces.\n", res.boundaries().len(), res.boundaries()[0].len());

    println!(">>> Benchmarking Resulting B-Rep Solid Tessellation (tol = 0.01)...");
    let t_tess = bench_fn("Solid Tessellation (triangulation)", || {
        let poly = res.triangulation(0.01).to_polygon();
        std::hint::black_box(poly);
    }, 5);

    let poly = res.triangulation(0.01).to_polygon();
    println!("    Tessellated mesh: {} vertices, {} faces.\n", poly.positions().len(), poly.faces().len());

    println!("=================================================================================================================================");
    println!("                                                    BENCHMARK SUMMARY                                                            ");
    println!("=================================================================================================================================");
    println!("| Operation                          | Latency (ms) | Vertices / Faces     |");
    println!("|------------------------------------|--------------|----------------------|");
    println!("| Solid Boolean (Cube AND Cylinder)  | {:>9.2} ms | {:>7} faces        |", t_bool.as_secs_f64() * 1000.0, res.boundaries()[0].len());
    println!("| Solid B-Rep Tessellation           | {:>9.2} ms | {:>7} verts, {:>5} f |", t_tess.as_secs_f64() * 1000.0, poly.positions().len(), poly.faces().len());
    println!("=================================================================================================================================\n");
}
