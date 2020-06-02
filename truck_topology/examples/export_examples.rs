use truck_topology::*;

fn large_plane() -> Shell {
    const N: usize = 100;

    let v = Vertex::news(N * N);
    let row_edge: Vec<Vec<Edge>> = (0..N)
        .map(|i| {
            (1..N)
                .map(|j| Edge::new(v[i * N + j - 1], v[i * N + j]))
                .collect()
        })
        .collect();
    let col_edge: Vec<Vec<Edge>> = (1..N)
        .map(|i| {
            (0..N)
                .map(|j| Edge::new(v[(i - 1) * N + j], v[(i % N) * N + j]))
                .collect()
        })
        .collect();

    (1..N)
        .flat_map(|i| (1..N).map(move |j| (i, j)))
        .map(|(i, j)| {
            Face::new(Wire::by_slice(&[
                row_edge[i - 1][j - 1],
                col_edge[i - 1][j],
                row_edge[i][j - 1].inverse(),
                col_edge[i - 1][j - 1].inverse(),
            ]))
        })
        .collect()
}

fn large_torus() -> Shell {
    const N: usize = 100;

    let v = Vertex::news(N * N);
    let row_edge: Vec<Vec<Edge>> = (0..N)
        .map(|i| {
            (0..N)
                .map(|j| Edge::new(v[i * N + j], v[i * N + (j + 1) % N]))
                .collect()
        })
        .collect();
    let col_edge: Vec<Vec<Edge>> = (0..N)
        .map(|i| {
            (0..N)
                .map(|j| Edge::new(v[i * N + j], v[((i + 1) % N) * N + j]))
                .collect()
        })
        .collect();

    (0..N)
        .flat_map(|i| (0..N).map(move |j| (i, j)))
        .map(|(i, j)| {
            Face::new(Wire::by_slice(&[
                row_edge[i][j],
                col_edge[i][(j + 1) % N],
                row_edge[(i + 1) % N][j].inverse(),
                col_edge[i][j].inverse(),
            ]))
        })
        .collect()
}

fn cube() -> Shell {
    let v = Vertex::news(8);
    let edge = [
        Edge::new(v[0], v[1]),
        Edge::new(v[1], v[2]),
        Edge::new(v[2], v[3]),
        Edge::new(v[3], v[0]),
        Edge::new(v[0], v[4]),
        Edge::new(v[1], v[5]),
        Edge::new(v[2], v[6]),
        Edge::new(v[3], v[7]),
        Edge::new(v[4], v[5]),
        Edge::new(v[5], v[6]),
        Edge::new(v[6], v[7]),
        Edge::new(v[7], v[4]),
    ];

    let wire = vec![
        Wire::by_slice(&[edge[0], edge[1], edge[2], edge[3]]),
        Wire::by_slice(&[edge[0].inverse(), edge[4], edge[8], edge[5].inverse()]),
        Wire::by_slice(&[edge[1].inverse(), edge[5], edge[9], edge[6].inverse()]),
        Wire::by_slice(&[edge[2].inverse(), edge[6], edge[10], edge[7].inverse()]),
        Wire::by_slice(&[edge[3].inverse(), edge[7], edge[11], edge[4].inverse()]),
        Wire::by_slice(&[
            edge[11].inverse(),
            edge[10].inverse(),
            edge[9].inverse(),
            edge[8].inverse(),
        ]),
    ];

    wire.into_iter().map(|w| Face::new(w)).collect()
}

fn irregular() -> Shell {
    let v = Vertex::news(5);
    let edge = [
        Edge::new(v[0], v[1]),
        Edge::new(v[0], v[2]),
        Edge::new(v[0], v[3]),
        Edge::new(v[0], v[4]),
        Edge::new(v[1], v[2]),
        Edge::new(v[1], v[3]),
        Edge::new(v[1], v[4]),
    ];
    let wire = vec![
        Wire::by_slice(&[edge[0], edge[4], edge[1].inverse()]),
        Wire::by_slice(&[edge[0], edge[5], edge[2].inverse()]),
        Wire::by_slice(&[edge[0], edge[6], edge[3].inverse()]),
    ];
    wire.into_iter().map(|w| Face::new(w)).collect()
}

fn regular() -> Shell {
    let v = Vertex::news(6);
    let edge = [
        Edge::new(v[0], v[1]),
        Edge::new(v[0], v[2]),
        Edge::new(v[1], v[2]),
        Edge::new(v[1], v[3]),
        Edge::new(v[1], v[4]),
        Edge::new(v[2], v[4]),
        Edge::new(v[2], v[5]),
        Edge::new(v[3], v[4]),
        Edge::new(v[4], v[5]),
    ];
    let wire = vec![
        Wire::by_slice(&[edge[0], edge[2], edge[1].inverse()]),
        Wire::by_slice(&[edge[3], edge[7], edge[4].inverse()]),
        Wire::by_slice(&[edge[5], edge[8], edge[6].inverse()]),
        Wire::by_slice(&[edge[2], edge[5], edge[4].inverse()]),
    ];
    wire.into_iter().map(|w| Face::new(w)).collect()
}

fn main() {
    let file = std::fs::File::create("tests/data/irregular.tts").unwrap();
    truck_io::tts::write(&irregular().wrap_up(), file).unwrap();
    let file = std::fs::File::create("tests/data/regular.tts").unwrap();
    truck_io::tts::write(&regular().wrap_up(), file).unwrap();
    let file = std::fs::File::create("tests/data/large_plane.tts").unwrap();
    truck_io::tts::write(&large_plane().wrap_up(), file).unwrap();
    let file = std::fs::File::create("tests/data/large_torus.tts").unwrap();
    truck_io::tts::write(&large_torus().wrap_up(), file).unwrap();
    let file = std::fs::File::create("tests/data/cube.tts").unwrap();
    truck_io::tts::write(&cube().wrap_up(), file).unwrap();
}
