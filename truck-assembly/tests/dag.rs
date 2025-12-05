use truck_assembly::dag::*;

#[test]
fn a_little_complex_case() {
    let dag = Dag::<()>::new();
    let node = dag.create_nodes([(); 9]);
    node[0].add_child(node[1]);
    node[0].add_child(node[2]);
    node[1].add_child(node[3]);
    node[1].add_child(node[4]);
    node[2].add_child(node[4]);
    node[2].add_child(node[5]);
    node[3].add_child(node[6]);
    node[3].add_child(node[7]);
    node[4].add_child(node[8]);
    node[7].add_child(node[4]);
    node[8].add_child(node[5]);

    assert_eq!(node[0].paths_iter().count(), 16);
    assert_eq!(node[0].maximal_paths_iter().count(), 5);

    assert!(!node[8].add_child(node[2]));
}
