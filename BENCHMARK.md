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
