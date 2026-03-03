# `monstertruck-assembly`

Assembly data structures using a directed acyclic graph (DAG).

> Forked from [`truck-assembly`](https://crates.io/crates/truck-assembly) v0.1.0 by [ricosjp](https://github.com/ricosjp/truck).

## Quick Start

```rust
use monstertruck_assembly::assy::*;

let mut assy = Assembly::<(), (), f64, ()>::new();

// Create nodes and connect with transform edges
let nodes = assy.create_nodes([().into(); 4]);
assy.create_edge(nodes[0], nodes[1], 2.0.into());
assy.create_edge(nodes[1], nodes[2], 3.0.into());
assy.create_edge(nodes[2], nodes[3], 5.0.into());

// Walk the path and accumulate transforms
let path = assy.maximal_paths_iter(nodes[0]).next().unwrap();
assert_eq!(path.matrix(), 30.0); // 2 * 3 * 5
```

## License

Apache License 2.0
