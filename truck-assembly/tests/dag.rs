use proptest::{prelude::*, property_test};
use std::collections::BTreeSet;
use truck_assembly::dag::*;

#[test]
fn a_little_complex_case() {
    let mut dag = Dag::<(), ()>::new();
    let node = dag.create_nodes([(); 9]);
    dag.create_edge(node[0], node[1], ());
    dag.create_edge(node[0], node[2], ());
    dag.create_edge(node[1], node[3], ());
    dag.create_edge(node[1], node[4], ());
    dag.create_edge(node[2], node[4], ());
    dag.create_edge(node[2], node[5], ());
    dag.create_edge(node[3], node[6], ());
    dag.create_edge(node[3], node[7], ());
    dag.create_edge(node[4], node[8], ());
    dag.create_edge(node[7], node[4], ());
    dag.create_edge(node[8], node[5], ());

    assert_eq!(dag.paths_iter(node[0]).count(), 16);
    assert_eq!(dag.maximal_paths_iter(node[0]).count(), 5);

    assert!(!dag.create_edge(node[8], node[2], ()));
}

const NODE_RANGE: std::ops::Range<usize> = 0..8;
const NUM_EDGE: usize = 32;
const NUM_REMOVE: usize = 10;

#[property_test]
fn random_graph(
    #[strategy = prop::array::uniform32(NODE_RANGE)] from: [usize; NUM_EDGE],
    #[strategy = prop::array::uniform32(NODE_RANGE)] to: [usize; NUM_EDGE],
    #[strategy = prop::array::uniform32(NODE_RANGE)] entity: [usize; NUM_EDGE],
    #[strategy = prop::array::uniform10(NODE_RANGE)] node4remove: [usize; NUM_REMOVE],
    #[strategy = prop::array::uniform10(0f64..1f64)] edge4remove: [f64; NUM_REMOVE],
) {
    let mut dag = Dag::<usize, usize>::new();
    let node = dag.create_nodes(NODE_RANGE);

    let mut num_edge = 0;
    for i in 0..NUM_EDGE {
        if dag.create_edge(node[from[i]], node[to[i]], entity[i]) {
            num_edge += 1;
        }
    }
    prop_assume!(num_edge > 20);

    for i in 0..NUM_REMOVE {
        let node_index = node[node4remove[i]];
        let node0 = dag.node(node_index);
        let len = node0.edges().len() as f64;
        let edge_index = (len * edge4remove[i]) as usize;
        dag.remove_edge(node_index, edge_index);
    }

    // Consistency of parents
    for edge in dag.all_edges() {
        let (node0, node1) = edge.nodes();
        assert!(dag.node(node1).parents().contains(&node0));
    }

    assert!(!has_cycle(&dag));
}

fn has_cycle<NE, EE>(dag: &Dag<NE, EE>) -> bool {
    let mut seen = BTreeSet::new();
    let mut finished = BTreeSet::new();
    let indices = dag.node_indices().collect::<Vec<_>>();

    while let Some(first) = indices.iter().copied().find(|x| !finished.contains(x)) {
        if sub_has_cycle(first, dag, &mut seen, &mut finished) {
            return true;
        }
    }
    false
}

fn sub_has_cycle<NE, EE>(
    cursor: NodeIndex,
    dag: &Dag<NE, EE>,
    seen: &mut BTreeSet<NodeIndex>,
    finished: &mut BTreeSet<NodeIndex>,
) -> bool {
    seen.insert(cursor);
    for edge in dag.node(cursor).edges() {
        let (_, child) = edge.nodes();
        if finished.contains(&child) {
            continue;
        } else if seen.contains(&child) || sub_has_cycle(child, dag, seen, finished) {
            return true;
        }
    }
    finished.insert(cursor);
    false
}
