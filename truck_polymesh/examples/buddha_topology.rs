extern crate truck_io as io;
use truck_polymesh::*;

const INPUT: &str = "tests/data/happy-buddha.obj";
const OUTPUT: &str = "buddha-topology.tts";

fn main() {
    let instant = std::time::Instant::now();
    let file = std::fs::File::open(INPUT).unwrap();
    let mesh = io::obj::read(file).unwrap();
    let read_time = instant.elapsed();
    let instant = std::time::Instant::now();
    let handler = MeshHandler::new(mesh);
    let shell = handler.extract_topology();
    let filter_time = instant.elapsed();
    let instant = std::time::Instant::now();
    let file = std::fs::File::create(OUTPUT).unwrap();
    io::tts::write(&shell, file).unwrap();
    let writing_time = instant.elapsed();

    println!(
        "file reading: {}.{:03} sec",
        read_time.as_secs(),
        read_time.subsec_nanos() / 1_000_000,
    );
    println!(
        "extracting time: {}.{:03} sec",
        filter_time.as_secs(),
        filter_time.subsec_nanos() / 1_000_000
    );
    println!(
        "file writing: {}.{:03} sec",
        writing_time.as_secs(),
        writing_time.subsec_nanos() / 1_000_000,
    );
}
