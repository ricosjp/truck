use truck_polymesh::prelude::*;

const INPUT: &str = "examples/data/happy-buddha.obj";
const OUTPUT: &str = "requadrangulated-buddha.obj";

fn main() {
    let instant = std::time::Instant::now();
    let file = std::fs::File::open(INPUT).unwrap();
    let mut mesh = obj::read(file).unwrap();
    let read_time = instant.elapsed();
    let first_quads = mesh.quad_faces().len();
    let instant = std::time::Instant::now();
    mesh.triangulate().quadrangulate(0.01, 1.0);
    let filter_time = instant.elapsed();
    let mesh: PolygonMesh = mesh.into();
    let tris = mesh.tri_faces().len();
    let quads = mesh.quad_faces().len();
    let instant = std::time::Instant::now();
    let file = std::fs::File::create(OUTPUT).unwrap();
    obj::write(&mesh, file).unwrap();
    let writing_time = instant.elapsed();

    println!("--- Excuting Status ---");
    println!("happy-buddha");
    println!("quadrangle:   {}\n", first_quads);
    println!("requadranglated-buddha");
    println!("triangle:     {}", tris);
    println!("quadrangle:   {}\n", quads);
    println!(
        "file reading: {}.{:03} sec",
        read_time.as_secs(),
        read_time.subsec_nanos() / 1_000_000,
    );
    println!(
        "filter run time: {}.{:03} sec",
        filter_time.as_secs(),
        filter_time.subsec_nanos() / 1_000_000
    );
    println!(
        "file writing: {}.{:03} sec",
        writing_time.as_secs(),
        writing_time.subsec_nanos() / 1_000_000,
    );
}
