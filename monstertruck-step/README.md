# monstertruck-step

[![Crates.io](https://img.shields.io/crates/v/monstertruck-step.svg)](https://crates.io/crates/monstertruck-step) [![Docs.rs](https://docs.rs/monstertruck-step/badge.svg)](https://docs.rs/monstertruck-step)

Reads/writes STEP files from/to truck.

## Sample Codes

### shape-to-step

Convert a truck shape JSON file to a STEP file.

#### usage

```bash
shape-to-step <input shape file> [output shape file]
```

### step-to-mesh

Parse STEP data, extract shapes, and generate meshes.
