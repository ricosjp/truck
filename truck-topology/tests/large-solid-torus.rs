use std::iter::FromIterator;

// type definition for test
type Vertex = truck_topology::Vertex<()>;
type Edge = truck_topology::Edge<(), ()>;
type Wire = truck_topology::Wire<(), ()>;
type Face = truck_topology::Face<(), (), ()>;
type Shell = truck_topology::Shell<(), (), ()>;
type Solid = truck_topology::Solid<(), (), ()>;

fn large_torus() -> Solid {
    const N: usize = 1_000;

    let v = Vertex::news(&[(); N * N]);
    let row_edge: Vec<Vec<Edge>> = (0..N)
        .map(|i| {
            (0..N)
                .map(|j| Edge::new(&v[i * N + j], &v[i * N + (j + 1) % N], ()))
                .collect()
        })
        .collect();
    let col_edge: Vec<Vec<Edge>> = (0..N)
        .map(|i| {
            (0..N)
                .map(|j| Edge::new(&v[i * N + j], &v[((i + 1) % N) * N + j], ()))
                .collect()
        })
        .collect();

    let shell: Shell = (0..N)
        .flat_map(|i| (0..N).map(move |j| (i, j)))
        .map(|(i, j)| {
            let wire = Wire::from_iter(vec![
                &row_edge[i][j],
                &col_edge[i][(j + 1) % N],
                &row_edge[(i + 1) % N][j].inverse(),
                &col_edge[i][j].inverse(),
            ]);
            Face::new(vec![wire], ())
        })
        .collect();
    Solid::new(vec![shell])
}

#[test]
#[ignore]
fn main() {
    let instant = std::time::Instant::now();
    large_torus();
    let end_time = instant.elapsed();
    println!(
        "excute time: {}.{:03} sec",
        end_time.as_secs(),
        end_time.subsec_nanos() / 1_000_000,
    );
}
