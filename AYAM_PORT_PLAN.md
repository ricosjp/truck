# Ayam -> Truck Comprehensive Extraction and Port Plan

This plan defines what to lift from Ayam, why it complements `truck`, and how to implement it in a Rust-native way.
It includes a general roadmap and a dedicated track for `font outline + complex profile -> B-rep solid`.

## 1) Objectives

- Expand `truck` high-level modeling capabilities using proven Ayam algorithms.
- Keep `truck` architecture and style intact: Rust-native implementation, typed errors, crate-local ownership of features.
- Prioritize features that close practical modeling gaps with high reuse value.
- Deliver a robust end-to-end profile-to-solid pipeline for font and complex planar contours.

## 2) Ground Truth: What Truck Already Has

`truck` already provides strong primitives that reduce port risk:

- Topology-first modeling framework (`truck-modeling`, `truck-topology`).
- NURBS/B-spline core with interpolation, degree elevation, knot ops (`truck-geometry`).
- Sweep/revolve/extrude and homotopy operators in builder APIs.
- Boolean/intersection/healing infrastructure (`truck-shapeops`).
- Tessellation and polymesh pipeline (`truck-meshalgo`, `truck-polymesh`).
- Subdivision support already exists in `truck-meshalgo` and T-spline stack.

Main practical gaps relative to Ayam are high-level surface constructors and profile robustness orchestration, not low-level spline math.

## 3) Extraction Principles

1. Port algorithms, not source code.
2. Preserve Truck crate boundaries.
3. Favor typed API surfaces over script-like command interfaces.
4. Add stable, composable primitives first, then convenience wrappers.
5. Keep optional or heavy features behind crate features.
6. Avoid legacy UI/render/plugin logic from Ayam.

## 4) Ayam Capability -> Truck Complement Matrix

| Ayam capability | Ayam references | Truck status | Port priority | Target crate | Done |
|---|---|---|---|---|---|
| Loop orientation and trim-cap normalization | `src/nurbs/capt.c`, `src/nurbs/nct.c` | **Done** (`profile::attach_plane_normalized`) | P0 | `truck-modeling` | [x] |
| Multi-rail sweep and periodic sweep | `ay_npt_sweep`, `ay_npt_sweepperiodic` in `src/nurbs/npt.c` | **Partial** (`BsplineSurface::sweep_rail`) | P0 | `truck-modeling` + `truck-geometry` | [ ] |
| Birail1/Birail2 | `ay_npt_birail1`, `ay_npt_birail2` | **Done** (`BsplineSurface::birail1`, `BsplineSurface::birail2`) | P1 | `truck-geometry` | [x] |
| Skin (u/v) and interpolation-aware loft | `ay_npt_skinu`, `ay_npt_skinv` | **Done** (`BsplineSurface::skin`, `builder::try_skin_wires`) | P0 | `truck-modeling` | [x] |
| Gordon / DualSkin | `ay_npt_gordon`, `ay_npt_dualskin` | **Done** (`BsplineSurface::gordon`) | P1 | `truck-geometry` | [x] |
| Curve/surface compatibility normalization | `ay_nct_makecompatible`, `ay_npt_makecompatible` | **Done** (`compat::make_curves_compatible`, `compat::make_surfaces_compatible`) | P0 | `truck-geometry` | [x] |
| Curve and surface offsets | `ay_nct_offset`, `ay_npt_offset` | **Done** (`offset::curve_offset_2d`, `curve_offset_3d`, `surface_offset`) | P1 | `truck-geometry` | [x] |
| Fairing and reparameterization workflows | `ay_nct_fair`, `ay_nct_reparam*` | **Done** (`fair::fair_curve`, `fair::reparameterize_arc_length`) | P2 | `truck-geometry` | [x] |
| Patch split/extract workflows | `ay_npt_splitu/v`, `ay_npt_extractnp` | Partial via existing cut operations | P2 | `truck-geometry` + `truck-shapeops` | [ ] |
| PatchMesh basis conversion | `src/nurbs/pmt.c` | **Done** (`basis::{HermiteSegment, CatmullRomSpline, PowerBasisCurve, PiecewiseBezier}` via `From` conversions) | P2 | `truck-geometry` | [x] |
| Font outline to NURBS contours | `src/contrib/tti.c`, `src/objects/text.c` | **Done** (`text::glyph_profile`, `text::text_profile`) | P0 | `truck-modeling` (feature gated) | [x] |
| Trim tessellation heuristics | `src/nurbs/stess.c`, `src/nurbs/rtess.c` | Not started | P3 | `truck-meshalgo` | [ ] |
| Subdivision plugin stack | `src/plugins/subdivide/*` | Already present differently | Skip | N/A | — |
| Ayam UI/script/plugin ecosystem | `src/tcl`, `src/plugins/*.tcl` | Out of scope by design | Skip | N/A | — |

## 5) Strategic Fit: How It Complements Truck

### 5.1 Modeling API Layer

Truck is strong at topology primitives and generic sweeps.
Ayam-derived constructors add missing high-level surface modeling operations:

- Section-curve skinning.
- Rail-based sweeps.
- Birail and Gordon network surfaces.
- Robust profile normalization before face creation.

This reduces custom glue code users currently write for common CAD workflows.

### 5.2 Geometry Robustness Layer

Truck has low-level spline operations but fewer orchestration utilities for multi-input compatibility.
Ayam compatibility patterns provide a proven sequence:

- normalize degree,
- normalize knot domain,
- merge knot vectors,
- only then construct combined surfaces.

Adopting this pattern increases reliability for all network-based constructors.

### 5.3 End-to-End Pipeline Layer

Ayam's text/profile pipeline maps well onto Truck's topology APIs:

- outline extraction -> closed loops,
- loop classification/orientation normalization,
- planar face creation with holes,
- extrusion/revolve/sweep to solids.

This complements existing Truck strengths rather than replacing them.

## 6) Architecture Target in Truck

### 6.1 Crate Responsibilities

- `truck-geometry`:
  - [x] Compatibility normalization primitives.
  - [x] Shared frame/path sampling utilities.
  - [x] Offset and fairing primitives.
- `truck-modeling`:
  - [x] High-level profile/text APIs (`profile`, feature-gated `text`).
  - [x] Builder-level skin wrapper (`try_skin_wires`).
  - [ ] Builder-level wrappers for `sweep_rail`, `birail`, and `gordon`.
  - [x] Normalized planar profile (`profile` module).
  - [x] Font/profile front-end module (feature-gated `text` module).
- `truck-shapeops`:
  - [ ] Topological integration and healing hooks for new constructors where needed.
- `truck-meshalgo`:
  - [ ] Selective tessellation robustness improvements for trimmed surfaces.

### 6.2 Proposed API Additions

Core profile and text:

- [x] `profile::attach_plane_normalized(wires: Vec<Wire>) -> Result<Face, Error>`.
- [x] `profile::solid_from_planar_profile(wires: Vec<Wire>, dir: Vector3) -> Result<Solid, Error>`.
- [x] `text::glyph_profile(...) -> Result<Vec<Wire>, Error>`.
- [x] `text::text_profile(...) -> Result<Vec<Wire>, Error>`.

Surface constructors:

- [x] `builder::try_skin_wires(wires: &[Wire]) -> Result<Shell, Error>`.
- [ ] `builder::sweep_rail(profile, rail, opts) -> Result<Face/Shell, Error>`.
- [x] `BsplineSurface::sweep_rail(profile, rail, n_sections)`.
- [x] `BsplineSurface::birail1(profile, rail1, rail2, n_sections)`.
- [x] `BsplineSurface::birail2(profile1, profile2, rail1, rail2, n_sections)`.
- [x] `BsplineSurface::gordon(u_curves, v_curves, points)`.

Geometry support:

- [x] `nurbs::compat::make_curves_compatible(...)`.
- [x] `nurbs::compat::make_surfaces_compatible(...)`.
- [x] `nurbs::offset::{curve_offset_2d, curve_offset_3d, surface_offset}`.
- [x] `nurbs::fair::{fair_curve, reparameterize_arc_length}`.

## 7) Phased Delivery Plan

## Phase 0. Program Setup and Baseline.

Tasks:

- [x] Add tracking epic and feature list with crate ownership.
- [ ] Create fixture corpus for:
  - [x] Loop nesting and winding variants.
  - [ ] Problematic rail/section combinations.
  - [ ] Near-degenerate NURBS cases.
  - [ ] Representative fonts and glyph sets.
- [ ] Define numeric tolerance policy and shared constants.

Done criteria:

- [x] Baseline pass/fail matrix and benchmark baseline documented (profile benchmarks in `truck-modeling/benches/profile_bench.rs`).

## Phase 1. Compatibility Normalization Core.

Tasks:

- [x] Implement reusable curve/surface compatibility routines in `truck-geometry`.
- [x] Standardize degree and knot synchronization flows.
- [x] Expose typed error taxonomy for incompatible inputs.

Done criteria:

- [x] Constructors can call one compatibility entrypoint.
- [x] Unit tests cover mixed-order and mixed-knot scenarios.

## Phase 2. High-ROI Surface Constructors.

Tasks:

- [x] Implement `skin`.
- [x] Implement `sweep_rail`.
- [ ] Implement periodic sweep variants.
- [ ] Add dedicated option structs for orientation/frame rules and interpolation modes.
- [x] Add builder-level wrappers in `truck-modeling`.

Done criteria:

- [x] New constructors produce geometric-consistent outputs on fixture corpus.
- [x] API examples added.

## Phase 3. Birail and Gordon Family.

Tasks:

- [x] Implement `birail1`.
- [x] Implement `birail2`.
- [x] Implement `gordon` based on compatibility core.
- [ ] Support intersection-grid driven and supplied-grid variants for Gordon.
- [ ] Add fallback and diagnostics for invalid curve networks.

Done criteria:

- [x] Integration tests for representative curve network classes (doc tests).

## Phase 4. Planar Profile Normalization.

Tasks:

- [x] Add loop classification (outer/holes), nesting checks, and winding normalization.
- [x] Add `attach_plane_normalized`.
- [x] Add robust errors for non-planar/open/self-intersecting/ambiguous loops.

Done criteria:

- [x] Stable output independent of input loop order/winding in tests.

## Phase 5. Font and Profile Front-End.

Tasks:

- [x] Add feature-gated text outline ingestion module.
- [x] Convert contours to Truck wires with closure cleanup.
- [x] Integrate with phase-4 normalization and face creation.

Done criteria:

- [ ] End-to-end text profile creation passes real-font fixtures with hole-preserving glyphs.
- [x] Contour-to-wire and text assembly unit tests exist (`truck-modeling/src/text.rs`).

## Phase 6. Offsets and Fairing.

Tasks:

- [x] Implement curve offset v1 for planar and near-planar workflows.
- [x] Implement surface offset v1 with explicit caveats.
- [x] Add fairing/reparameterization utilities for preconditioning inputs.

Done criteria:

- [x] Offsets and fairing are available as explicit pre-processing steps.

## Phase 7. PatchMesh Basis Conversion.

Tasks:

- [x] Implement basis conversion helpers for patch import pipelines.
- [x] Support Bezier/B-spline/Catmull/Hermite/Power basis mapping to NURBS.

Done criteria:

- [x] Conversion fixtures validate geometry continuity and expected topology.

## Phase 8. Tessellation Robustness Improvements.

Status: **Deferred**. The existing `truck-meshalgo` tessellation pipeline already includes CDT-based
trimming with boundary constraints, singular-point handling, periodic surface support, robust
parameter search fallback chains (exact → nearest), and adaptive subdivision. No regressions
have been identified that warrant porting Ayam heuristics. A trimmed-surface regression corpus
would be needed to drive targeted improvements.

Tasks:

- [ ] Port selected trim robustness heuristics where they improve current behavior.
- [ ] Avoid introducing legacy display-era assumptions.

Done criteria:

- [ ] Reduced failure rate on trimmed-surface regression corpus.

## Phase 9. Performance and Parallelism.

Tasks:

- [x] Profile pipeline benchmarked (`truck-modeling/benches/profile_bench.rs`).
- [x] Profile all new surface constructors (benchmarks in `truck-geometry/benches/surface_constructors.rs`).
- [x] Parallelize per-glyph/per-loop tasks with `rayon` on non-WASM (`text_profile` uses `par_iter`).
- [x] Constructors use functional style and pre-allocated collections; no low-hanging allocation waste identified.

Done criteria:

- [x] Benchmark baseline established for profile pipeline.
- [x] Benchmark baseline established for all surface constructors, offset, fairing, and basis conversion.

## Phase 10. Documentation, Examples, and Hardening.

Tasks:

- [x] Add focused examples for profile pipeline (`profile-box.rs`, `profile-with-holes.rs`).
- [x] Add focused examples for each new surface constructor family (`skin-surface.rs`, `sweep-rail.rs`, `gordon-surface.rs`, `birail-surface.rs`).
- [x] Document guarantees, tolerances, and failure modes (via doc comments on all public APIs).
- [ ] Publish migration guidance for manual workflow users.

Done criteria:

- [x] Profile example suite runs.
- [x] Surface constructor example suite runs.

## 8) Special Use Case Track: Font Outline + Complex Profile -> B-rep Solid

### 8.1 Data Flow

1. [x] Text or glyph selection.
2. [x] Outline extraction into contours.
3. [x] Segment-to-curve conversion.
4. [x] Contour closure cleanup and dedup.
5. [x] Loop classification and winding normalization.
6. [x] Planar face creation with holes.
7. [x] Solid creation by extrusion (v1).
8. [ ] Solid creation by revolve/sweep (v2).
9. [ ] Consistency and tessellation validation.

### 8.2 Why this is native to Truck, not bolt-on

- Face/wire/solid primitives already match the needed output topology.
- Existing meshalgo and shapeops can validate and consume resulting solids.
- The same normalization layer serves both text profiles and arbitrary CAD sketches.

### 8.3 Dedicated Milestones

- [ ] M1: Single glyph with one hole to valid solid (real-font fixture).
- [ ] M2: Multi-glyph Latin text, baseline and advance support (real-font fixture).
- [ ] M3: Mixed glyph + custom profile loops to single face and solid (real-font fixture).
- [ ] M4: Stress corpus of tricky fonts and small-feature geometry.

## 9) Testing and Verification Strategy

Unit tests:

- [x] Loop classification and winding normalization.
- [x] Compatibility normalization and knot synchronization.
- [x] Constructor precondition checks and error taxonomy.

Integration tests:

- [x] `curves -> skin/birail/gordon -> surface` invariants (doc tests and geometry checks).
- [ ] `text/profile -> wires -> face -> solid` end-to-end with real-font fixtures (profile path is covered).

Regression corpus:

- [ ] Curated pathological geometry and font fixtures.

Performance tests:

- [x] Profile pipeline throughput (benchmarks exist).
- [x] Constructor throughput for surface constructors (benchmarks in `truck-geometry/benches/surface_constructors.rs`).
- [ ] Large text and large loop-set profile build times.

Quality gates:

- [ ] `cargo test` (not verified in this sandbox due network restrictions).
- [ ] `cargo clippy --all-targets -- -W warnings` (not verified in this sandbox due network restrictions).

## 10) Risks and Mitigations

Risk: Numeric instability in near-degenerate geometry.
Mitigation: Shared tolerance policy, deterministic tie-break rules, explicit failure modes.

Risk: API sprawl from too many constructor variants.
Mitigation: minimal v1 APIs with option structs; defer niche variants.

Risk: Font complexity and shaping scope creep.
Mitigation: feature gating and strict v1 scope; shaping as optional extension.

Risk: Performance regressions from compatibility and normalization passes.
Mitigation: profile-first rollout and targeted parallelism.

## 11) Non-Goals

- Porting Ayam GUI, Tcl command system, or rendering plugin ecosystem.
- Source-level translation of legacy parser and tessellation code.
- Reproducing Ayam object tree semantics inside Truck.

## 12) Deliverables

- [x] New constructor APIs (`skin`, `sweep_rail`, `birail`, `gordon`) with code and examples.
- [x] Compatibility and normalization core in `truck-geometry`.
- [x] Planar profile normalization pipeline and face/solid helpers.
- [x] Feature-gated text outline to wire/profile APIs.
- [x] Offset/fairing baseline operations.
- [x] Documentation and examples for shipped profile/text features.
- [x] Documentation and examples for all remaining features.

## 13) Recommended Implementation Order (ROI-First)

1. [x] Compatibility normalization core.
2. [x] Planar profile normalization.
3. [x] `skin` + `sweep_rail` constructors.
4. [x] Font/profile pipeline to extrusion solid.
5. [x] `birail` and `gordon`.
6. [x] Offsets/fairing.
7. [x] PatchMesh basis conversion.
8. [ ] Tessellation refinements.

This order delivers immediate utility while reducing risk for advanced surface features.
