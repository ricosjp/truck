use truck_polymesh::*;

const INPUT: &str = "tests/data/happy-buddha.obj";

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
        obj::read(file).unwrap()
    });
    let shell = time_keeper("extracting", || {
        let handler = MeshHandler::new(mesh);
        handler.extract_topology()
    });
    time_keeper("create-solid", || {
        truck_topology::Solid::debug_new(vec![shell]);
    });
}
