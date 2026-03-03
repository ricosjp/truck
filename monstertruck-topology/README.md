# `monstertruck-topology`

Topological data structures: vertices, edges, wires, faces, shells, and solids.

> Forked from [`truck-topology`](https://crates.io/crates/truck-topology) v0.6.0 by [ricosjp](https://github.com/ricosjp/truck).

## Quick Start

```rust
monstertruck_topology::prelude!((), (), ());

// Build a tetrahedron from vertices and edges
let v = Vertex::news(&[(); 4]);
let edge = [
    Edge::new(&v[0], &v[1], ()), Edge::new(&v[0], &v[2], ()),
    Edge::new(&v[0], &v[3], ()), Edge::new(&v[1], &v[2], ()),
    Edge::new(&v[1], &v[3], ()), Edge::new(&v[2], &v[3], ()),
];
let wire = vec![
    wire![&edge[0], &edge[3], &edge[1].inverse()],
    wire![&edge[1], &edge[5], &edge[2].inverse()],
    wire![&edge[2], &edge[4].inverse(), &edge[0].inverse()],
    wire![&edge[3], &edge[5], &edge[4].inverse()],
];
let mut face: Vec<Face> = wire.into_iter().map(|w| Face::new(vec![w], ())).collect();
face[3].invert();

let solid = Solid::new(vec![face.into()]);
```

## License

Apache License 2.0
