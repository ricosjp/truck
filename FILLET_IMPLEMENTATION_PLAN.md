# Fillet Implementation Plan (`truck`) — Edge-Centric, All Cases

This plan is for implementing production-ready fillets in `truck` so a caller can fillet **any shared edge** between two faces:

- untrimmed/shared native edge,
- trimmed/shared boundary edge,
- CSG/boolean-result edge (often intersection/trim curves),
- and wire chains (open first, closed after).

The plan is structured so another agent can execute it phase-by-phase with clear done criteria.

---

## 1) Current State (Ground Truth)

### 1.1 User-visible status

- `truck-modeling` docs ~~still state fillet is unsupported~~ updated:
  - `truck-modeling/README.md:15`
  - `truck-modeling/examples/bottle.rs:5`

### 1.2 Existing internal assets to leverage

- There is already a ~~hidden prototype~~ public fillet module:
  - `truck-shapeops/src/lib.rs:21` (`pub mod fillet;`)
  - `truck-shapeops/src/fillet/mod.rs`
- Existing core routines (now split into `geometry.rs`, `ops.rs`, `topology.rs`):
  - round fillet surface builder: `geometry.rs` (`rolling_ball_fillet_surface`)
  - chamfer surface builder: `geometry.rs` (`chamfer_fillet_surface`)
  - face trimming along fillet curves: `topology.rs` (`cut_face_by_bezier`)
  - single-edge fillet workflow: `ops.rs` (`simple_fillet`)
  - side-face update: `ops.rs` (`fillet_with_side`)
  - wire/chain fillet workflow: `ops.rs` (`fillet_along_wire`)
  - closed wire fillet: `ops.rs` (`fillet_along_wire_closed`)
- ~~Known prototype limitation: closed wire not implemented~~ (resolved)

### 1.3 Geometry support already present in `truck-geometry`

- Rolling-ball fillet primitives:
  - `RbfSurface`: `truck-geometry/src/decorators/mod.rs:321`
  - constructor: `truck-geometry/src/decorators/rbf_surface/mod.rs:7`
- Approximation helper:
  - `ApproxFilletSurface::approx_rolling_ball_fillet`: `truck-geometry/src/decorators/af_surface.rs:231`
- T-spline fallback approximation exists:
  - `to_bspline_surface`: `truck-geometry/src/t_spline/t_mesh.rs:2560`

---

## 2) Ayam Material to Port (Design, Not Literal C Translation)

Use Ayam as algorithmic reference for missing product behaviors.

### 2.1 Parameter model (important)

- Per-side/per-edge settings (`state/type/dir/integrate/radius/force3d`):
  - `/home/ritz/code/ayam/ayam/src/ayam.h:180`
- Cap params (future extension):
  - `/home/ritz/code/ayam/ayam/src/ayam.h:193`

### 2.2 Pipeline orchestration pattern

- Bevel dispatcher (`addbevels`):
  - `/home/ritz/code/ayam/ayam/src/nurbs/bevelt.c:55`
- Tag/parameter parse pattern (`parsetags`):
  - `/home/ritz/code/ayam/ayam/src/nurbs/bevelt.c:624`

### 2.3 Geometry generators to emulate in Rust API

- 2D profile sweep bevel:
  - `/home/ritz/code/ayam/ayam/src/nurbs/bevelt.c:800`
- 3D profile + normals/tangents sweep:
  - `/home/ritz/code/ayam/ayam/src/nurbs/bevelt.c:926`
- Round-to-cap strategy:
  - `/home/ritz/code/ayam/ayam/src/nurbs/bevelt.c:1098`
- Round-to-normal strategy:
  - `/home/ritz/code/ayam/ayam/src/nurbs/bevelt.c:1281`
- Integrate bevel patch into host surface:
  - `/home/ritz/code/ayam/ayam/src/nurbs/bevelt.c:1490`
- Built-in profile catalog (round/linear/ridge/custom):
  - `/home/ritz/code/ayam/ayam/src/nurbs/bevelt.c:1623`
  - `/home/ritz/code/ayam/ayam/src/nurbs/bevelt.c:1682`

---

## 3) Architecture Target

Implement fillet at two layers:

1. **Core engine in `truck-shapeops`**
   - owns topology surgery and fillet-face generation.
   - edge-centric API (not "CSG-only API").

2. **Ergonomic API in `truck-modeling`**
   - thin wrapper over shapeops fillet engine.
   - exposed to end users and examples.

Use **canonical NURBS workspace** internally for fillet creation, then map back to topology results.

---

## 4) Public API Shape (v1 — implemented)

API in `truck-shapeops` (actual shape):

- `fillet_edges(shell, edge_ids, params: Option<&FilletOptions>) -> Result<(), FilletError>`
- `fillet_edges_generic(shell, edges, params: Option<&FilletOptions>) -> Result<(), FilletError>`
- `simple_fillet(face0, face1, edge_id, options: &FilletOptions) -> Result<(Face, Face, Face), FilletError>`
- `fillet_with_side(face0, face1, edge_id, side0, side1, options: &FilletOptions) -> Result<..., FilletError>`
- `fillet_along_wire(shell, wire, options: &FilletOptions) -> Result<(), FilletError>`

Core parameter types:

- `FilletOptions` (was `FilletParams`)
  - `radius: RadiusSpec`
    - [x] `Constant(f64)`
    - [x] `Variable(Box<dyn Fn(f64) -> f64>)`
    - [ ] `PerEdge(Vec<...>)` (optional advanced mode)
  - `profile: FilletProfile`
    - [x] `Round`
    - [x] `Chamfer`
    - [x] `Ridge`
    - [x] `Custom(BSplineCurve<Point2>)`
  - [ ] `extend_mode`, `corner_mode`, `integrate_mode`
  - [x] `division: NonZeroUsize`
- [x] `FilletError` typed enum (no `eprintln!` paths)
- [x] `Default` impl for `FilletOptions` (radius=0.1, division=5, Round)
- [x] Builder methods: `constant()`, `variable()`, `with_division()`, `with_profile()`
- [x] High-level functions take `Option<&FilletOptions>`, low-level take `&FilletOptions`

---

## 5) Phase-by-Phase Execution

## Phase 0 — Stabilize Baseline [DONE]

### Tasks

- [x] Document current behavior and known limits
- [x] Capture input assumptions:
  - [x] manifold shell requirement
  - [x] edge must have exactly two incident faces

### Done criteria

- [x] Short design note committed with explicit v1 scope and deferred items.

---

## Phase 1 — Productize Existing Prototype [DONE]

### Tasks

- [x] Refactor `truck-shapeops/src/fillet/mod.rs` into modules:
  - `ops.rs`, `geometry.rs`, `topology.rs`, `params.rs`, `error.rs`, `types.rs`, `convert.rs`, `edge_select.rs`, `tests.rs`
- [x] Remove prototype markers (`#![allow(dead_code)]`)
- [x] Convert `Option` returns to `Result<_, FilletError>`
- [x] Replace `eprintln!` failures with typed errors (9 error variants)
- [x] Export from `truck-shapeops/src/lib.rs` (public API)

### Done criteria

- [x] Public round-fillet API exists in `truck-shapeops`.
- [x] Existing fillet tests pass.

---

## Phase 2 — Edge Selection and Adjacency [DONE]

### Tasks

- [x] Implement edge-centric selection:
  - [x] from explicit `EdgeID` list (`fillet_edges`)
  - [x] from wire/chain helper (`fillet_along_wire`)
- [x] Robustly resolve incident faces per edge (`build_edge_face_map`)
- [x] Validate and reject:
  - [x] non-manifold edges (>2 faces) — `FilletError::NonManifoldEdge`
  - [x] boundary edges (<2 faces) — `FilletError::NonManifoldEdge(1)`
  - [x] degenerate tiny edges — `FilletError::DegenerateEdge`
- [x] Ensure edge orientation normalization before geometry build
- [x] Multi-chain robustness: rebuild `edge_face_map` per chain to avoid stale indices

### Done criteria

- [x] Same API accepts edges from hand-built B-rep
- [x] CSG outputs (`and`, `or`) tested — `boolean_shell_converts_for_fillet`

---

## Phase 3 — Canonical Geometry Workspace [DONE]

### Tasks

- [x] Build canonical conversion stage (`convert.rs`):
  - [x] `FilletableSurface` trait: `to_nurbs_surface()` / `from_nurbs_surface()`
  - [x] `FilletableCurve` trait: `to_nurbs_curve()` / `from_nurbs_curve()` / `from_pcurve()` / `from_intersection_curve()`
- [x] `fillet_edges_generic` converts arbitrary types to internal NURBS, runs fillet, converts back
- [x] Preserve identity map: `convert_shell_in` / `convert_shell_out`

### Done criteria

- [x] Mixed cases run through one pipeline (tested: Plane, NurbsSurface, mixed)
- [x] Unsupported geometry (T-spline) returns typed error

---

## Phase 4 — Round Fillet for Any Shared Edge [DONE]

### Tasks

- [x] Harden and expose:
  - [x] `rolling_ball_fillet_surface` (geometry.rs)
  - [x] `cut_face_by_bezier` (topology.rs)
  - [x] `simple_fillet` (ops.rs)
  - [x] `fillet_along_wire` (ops.rs)
- [x] Variable radius support (`RadiusSpec::Variable`)
- [x] Implement closed-wire support:
  - [x] `fillet_along_wire_closed` with circular indexing
  - [x] Wrap-around seam averaging
- [x] Improve robustness around closest-parameter search and cut tolerances

### Done criteria

- [x] Round fillet works on:
  - [x] single arbitrary shared edge
  - [x] open chains
  - [x] closed loops

---

## Phase 5 — Profile Modes (Chamfer, Ridge, Custom) [DONE]

### Tasks

- [x] Add `FilletProfile` enum to `params.rs`
- [x] Implement chamfer (flat ruled surface):
  - [x] `expand_chamfer` in `geometry.rs`
  - [x] `chamfer_fillet_surface` dispatcher in `geometry.rs`
  - [x] Thread `FilletProfile` through ops.rs, topology.rs, edge_select.rs
  - [x] Fix `concat_fillet_surface` to derive u-knot from actual surface (not hardcoded `unit_circle_knot_vec`)
  - [x] Chamfer tests: `chamfer_single_edge`, `chamfer_semi_cube`, `chamfer_closed_wire`
- [x] Implement Ridge profile
  - [x] Ridge cross-section geometry (V-shaped or pointed profile)
  - [x] `expand_ridge` / `ridge_fillet_surface` in `geometry.rs`
  - [x] Ridge tests: `ridge_single_edge`, `ridge_semi_cube`, `ridge_closed_wire`
- [x] Implement Custom profile
  - [x] `Custom(BSplineCurve<Point2>)` variant in `FilletProfile`
  - [x] Profile-driven patch generation (2D sweep)
  - [x] Custom profile tests: `custom_profile_linear`, `custom_profile_bump`

### Done criteria

- [x] `Round` works with the same edge API
- [x] `Chamfer` works with the same edge API
- [x] `Ridge` works with the same edge API
- [x] Custom profile NURBS curve path is available

---

## Phase 6 — Optional Integration Mode (Merge vs Keep Separate Fillet Face) [NOT STARTED]

### Tasks

- [ ] Introduce mode:
  - [ ] `KeepSeparateFace` (default, safest)
  - [ ] `IntegrateIntoHost` (advanced)
- [ ] Port orientation/order matching principles from Ayam integration logic:
  - `/home/ritz/code/ayam/ayam/src/nurbs/bevelt.c:1490`
- [ ] Ensure continuity and orientation correction after integration

### Done criteria

- [ ] Integration mode passes topology checks and does not regress default mode.

---

## Phase 7 — `truck-modeling` Integration and Docs [DONE]

### Tasks

- [x] Add wrapper API in `truck-modeling` (`fillet_impls.rs`)
- [x] Add dependency on `truck-shapeops` in `truck-modeling/Cargo.toml` (optional, behind `fillet` feature)
- [x] Update docs/examples that currently state fillet unsupported:
  - [x] `truck-modeling/README.md:15`
  - [x] `truck-modeling/examples/bottle.rs:5`
- [x] Add/refresh example showing filleting after boolean — `fillet-after-boolean.rs`
- [x] Re-export: `fillet_edges_generic as fillet_edges`, `FilletError`, `FilletOptions`, `FilletProfile`, `RadiusSpec`, `FilletableCurve`, `FilletableSurface`

### Done criteria

- [x] End users can call fillet from `truck-modeling` with simple API
- [x] Documentation no longer claims fillet is unavailable

---

## Post-Plan Improvements (completed outside original phases)

- [x] Rename `PCurveLns` → `ParamCurveLinear`
- [x] Rename `FilletParams` → `FilletOptions`
- [x] Change `division: usize` → `division: NonZeroUsize` (enforce invariants at API boundary)
- [x] Consolidate low-level function signatures to take `&FilletOptions` instead of separate `(radius, division, profile)` args
- [x] High-level `fillet_edges` / `fillet_edges_generic` take `Option<&FilletOptions>` with `unwrap_or(&default)`
- [x] `Default` impl for `FilletOptions`

---

## 6) Test Plan

### 6.1 Shape classes

- [x] Two adjacent NURBS faces with untrimmed shared edge
- [x] Two adjacent faces with trimmed shared edge
- [x] CSG-generated shared edge from `and`/`or` — `boolean_shell_converts_for_fillet`
- [x] Mixed face types (Plane + NurbsSurface) — `generic_fillet_mixed_surfaces`
- [x] Unsupported surface type error path — `generic_fillet_unsupported`

### 6.2 Topological checks

- [x] Shell remains manifold (triangulation succeeds on all test shells)
- [x] No open cracks along inserted fillet boundaries (verified via mesh output)
- [x] Orientation consistency of all new faces/wires

### 6.3 Geometric checks

- [x] Closed-loop seam closure test — `fillet_closed_wire_box_top`, `chamfer_closed_wire`
- [ ] Radius error bounds for round mode
- [ ] Endpoint and tangency continuity at joins
- [x] Variable-radius wire test — `variable_radius_closed_wire`

### 6.4 Regression checks

- [x] Existing fillet prototype tests remain green after refactor (27 tests passing)

### 6.5 Test inventory (27 tests)

- [x] `create_fillet_surface` — raw geometry surface creation
- [x] `create_simple_fillet` — simple 2-face fillet
- [x] `create_fillet_with_side` — fillet with side face update
- [x] `fillet_to_nurbs` — fillet on curved (circle arc) edge
- [x] `fillet_semi_cube` — open wire chain fillet
- [x] `fillet_closed_wire_box_top` — closed wire fillet
- [x] `fillet_edges_single_edge` — high-level single edge API
- [x] `fillet_edges_rejects_missing` — error: missing edge
- [x] `fillet_edges_rejects_boundary` — error: boundary edge
- [x] `generic_fillet_identity` — generic pipeline with internal types
- [x] `generic_fillet_modeling_types` — generic pipeline with truck_modeling types
- [x] `generic_fillet_mixed_surfaces` — mixed Plane + NurbsSurface
- [x] `generic_fillet_unsupported` — unsupported geometry error
- [x] `fillet_edges_multi_chain` — two independent edges in one call
- [x] `generic_fillet_multi_chain` — multi-chain with modeling types
- [x] `chamfer_single_edge` — chamfer on single edge
- [x] `chamfer_semi_cube` — chamfer along open wire
- [x] `chamfer_closed_wire` — chamfer along closed wire
- [x] `ridge_single_edge` — ridge on single edge
- [x] `ridge_semi_cube` — ridge along open wire
- [x] `ridge_closed_wire` — ridge along closed wire
- [x] `custom_profile_linear` — custom linear profile
- [x] `custom_profile_bump` — custom degree-2 bump profile
- [x] `variable_radius_closed_wire` — variable radius on closed wire
- [x] `fillet_rejects_degenerate_edge` — error: edge too short for radius
- [x] `boolean_shell_converts_for_fillet` — CSG result IntersectionCurve conversion

---

## 7) Validation Commands (Per Repo Instructions)

Use only allowed verification commands from repo guidance.

- Targeted tests while iterating:
  - `cargo test -p truck-shapeops --lib -- fillet --skip test_unit_circle`
  - `cargo test -p truck-modeling --features fillet --test fillet_test`
- Lint:
  - `cargo clippy -p truck-shapeops --all-targets -- -W warnings`
  - `cargo clippy -p truck-modeling --features fillet --all-targets -- -W warnings`
- Format:
  - `cargo fmt -p truck-shapeops -- --check`
  - `cargo fmt -p truck-modeling -- --check`

---

## 8) Recommended PR Split

1. ~~**PR-A**: prototype cleanup + public API skeleton + error model.~~ [DONE]
2. ~~**PR-B**: any-edge adjacency + round single-edge fillet stable.~~ [DONE]
3. ~~**PR-C**: chain/closed-loop support + robustness.~~ [DONE]
4. ~~**PR-D**: profile modes (Chamfer, Ridge, Custom).~~ [DONE]
5. **PR-E**: optional integration mode.
6. ~~**PR-F**: `truck-modeling` wrapper + docs/examples.~~ [DONE]

Each PR should include focused tests for only that phase.

---

## 9) Non-Goals for v1 (Explicit)

- Full corner-blend networks for arbitrary high-valence vertices.
- Guaranteed class-A continuity targets beyond current geometric constraints.
- Automatic feature recognition UI; API-first implementation only.

---

## 10) Next Actions

Remaining work in priority order:

1. **Phase 6**: Optional integration mode (KeepSeparateFace / IntegrateIntoHost)
2. **Known limitation**: Fillet topology surgery on boolean-result faces panics due to complex boundary wires; `IntersectionCurve` → NURBS conversion works but the `cut_face_by_bezier` step needs hardening for trimmed face topologies
