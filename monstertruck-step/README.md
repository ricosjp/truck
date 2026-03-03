# `monstertruck-step`

STEP file import and export.

> Forked from [`truck-stepio`](https://crates.io/crates/truck-stepio) v0.3.0 by [ricosjp](https://github.com/ricosjp/truck).

## Quick Start

```rust
use monstertruck_step::r#in::{*, step_geometry::*};

// Parse a STEP file
let step_string = std::fs::read_to_string("model.step").unwrap();
let table = Table::from_step(&step_string).unwrap();

// Extract a shell and convert to topology
let step_shell = table.shell.values().next().unwrap();
let compressed = table.to_compressed_shell(step_shell).unwrap();
```

## License

Apache License 2.0
