# `monstertruck`

**M**ultifarious **O**mnificence, **N**omenclature **S**tandardized, **T**erminology **E**nhanced & **R**efactored **Truck** – a **Ru**st **C**ad **K**ernel.

Meta-crate that re-exports all `monstertruck-*` sub-crates via feature flags.
Enable only what you need, or use `full` for everything.

## Quick Start

```toml
[dependencies]
monstertruck = "0.1"                          # default: modeling + meshing
monstertruck = { version = "0.1", features = ["full"] }  # everything
```

```rust
use monstertruck::modeling::builder;

let cube = builder::cube();
```

## Features

| Feature    | Crate                   | Includes               |
| ---------- | ----------------------- | ---------------------- |
| _(always)_ | `monstertruck-core`     |                        |
| `traits`   | `monstertruck-traits`   |                        |
| `derive`   | `monstertruck-derive`   |                        |
| `geometry` | `monstertruck-geometry` | `traits`               |
| `topology` | `monstertruck-topology` |                        |
| `mesh`     | `monstertruck-mesh`     | `traits`               |
| `modeling` | `monstertruck-modeling` | `geometry`, `topology` |
| `meshing`  | `monstertruck-meshing`  | `mesh`, `modeling`     |
| `solid`    | `monstertruck-solid`    | `modeling`             |
| `assembly` | `monstertruck-assembly` |                        |
| `step`     | `monstertruck-step`     | `modeling`             |
| `gpu`      | `monstertruck-gpu`      |                        |
| `render`   | `monstertruck-render`   | `gpu`, `mesh`          |

### Bundles

| Bundle    | Features                                                               |
| --------- | ---------------------------------------------------------------------- |
| `default` | `modeling`, `meshing`                                                  |
| `full`    | `modeling`, `meshing`, `solid`, `assembly`, `step`, `render`, `derive` |

## Re-exported Modules

Each enabled feature exposes a top-level module:

```rust
monstertruck::core        // always available
monstertruck::modeling    // with "modeling"
monstertruck::meshing     // with "meshing"
monstertruck::geometry    // with "geometry"
// ... etc.
```

## License

Apache License 2.0
