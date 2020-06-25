extern crate truck_io as io;
use truck_polymesh::*;

const INPUT: &str = "tests/data/happy-buddha.obj";
const OUTPUT: &str = "buddha-topology.tts";

fn time_keeper<Output, F: FnOnce() -> Output>(task_name: &str, task: F) -> Output {
    let instant = std::time::Instant::now();
    let output = task();
    let exec_time = instant.elapsed();
    println!(
        "{} : {}.{:03} sec",
        task_name,
        exec_time.as_secs(),
        exec_time.subsec_nanos() / 1_000_000,
    );
    output
}

fn main() {
    let mesh = time_keeper("file reading", || {
        let file = std::fs::File::open(INPUT).unwrap();
        io::obj::read(file).unwrap()
    });
    let shell = time_keeper("extracting", || {
        let handler = MeshHandler::new(mesh);
        handler.extract_topology()
    });
    time_keeper("file writing", || {
        let file = std::fs::File::create(OUTPUT).unwrap();
        io::tts::write(&shell, file).unwrap();
    });
}
