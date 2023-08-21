# truck-stepio

[![Crates.io](https://img.shields.io/crates/v/truck-stepio.svg)](https://crates.io/crates/truck-stepio) [![Docs.rs](https://docs.rs/truck-stepio/badge.svg)](https://docs.rs/truck-stepio)

Reads/writes STEP files from/to truck.

## Sample Codes

### shape-to-step

convert from truck shape json to step file.

#### usage

```bash
shape-to-step <input shape file> [output shape file]
```

### step-to-mesh

Parse STEP data, extract shape, and meshing.
