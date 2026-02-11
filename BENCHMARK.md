# Tessellation Benchmark Log

All times are wall-clock, debug profile, on the same machine.
Median of 3 runs shown.

## Baseline (pre-optimization)

### Shape tessellation (`bench-tessellation` example, tol=0.005)

| Shape                 | Time    | Triangles | Condition |
| --------------------- | ------- | --------- | --------- |
| bottle                | 21.6ms  | 1120      | Closed    |
| cube                  | 747us   | 12        | Closed    |
| cube-in-cube          | 1.4ms   | 24        | Closed    |
| cylinder              | 20.3ms  | 1024      | Closed    |
| large-torus           | 5.0ms   | 0         | Closed    |
| punched-cube-shapeops | 137.1ms | 712       | Closed    |
| punched-cube          | 7.4ms   | 112       | Closed    |
| sphere                | 8.2ms   | 1088      | Closed    |
| torus-punched-cube    | 11.7ms  | 734       | Closed    |
| torus                 | 26.1ms  | 2896      | Closed    |
| **total**             | 699.6ms |           |           |

### STEP assembly (`step-to-mesh` on `occt-assy.step`)

| Metric      | Value  |
| ----------- | ------ |
| Wall time   | ~0.31s |
| Shell count | 5      |
| All closed  | Yes    |

## Post-optimization (Commits 1–5)

### Shape tessellation (`bench-tessellation` example, tol=0.005)

| Shape                 | Time    | Triangles | Condition | vs Baseline |
| --------------------- | ------- | --------- | --------- | ----------- |
| bottle                | 16.5ms  | 1120      | Closed    | -24%        |
| cube                  | 533us   | 12        | Closed    | ~same       |
| cube-in-cube          | 1.1ms   | 24        | Closed    | ~same       |
| cylinder              | 15.3ms  | 1024      | Closed    | -25%        |
| large-torus           | 3.7ms   | 0         | Closed    | ~same       |
| punched-cube-shapeops | 75.6ms  | 712       | Closed    | -45%        |
| punched-cube          | 5.1ms   | 112       | Closed    | ~same       |
| sphere                | 5.4ms   | 1088      | Closed    | -34%        |
| torus-punched-cube    | 7.8ms   | 734       | Closed    | -33%        |
| torus                 | 15.7ms  | 2896      | Closed    | -40%        |
| **total**             | 464.1ms |           |           | **-34%**    |

### STEP assembly (`step-to-mesh` on `occt-assy.step`)

| Metric      | Value  | vs Baseline |
| ----------- | ------ | ----------- |
| Wall time   | ~0.23s | -26%        |
| Shell count | 5      |             |
| All closed  | Yes    |             |

### Key optimizations applied

1. **Commit 1**: Removed double tessellation in `step-to-mesh` (bbox from geometry sampling)
2. **Commit 2**: UV AABB early reject in `PolyBoundary::include`
3. **Commit 3**: Configurable search trials via `TessellationConfig`
4. **Commit 4**: Parallel `StructuredMesh::from_surface_par` (rayon, non-WASM)
5. **Commit 5**: Untrimmed face fast path (skip CDT for untrimmed faces)
