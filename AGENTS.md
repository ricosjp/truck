# Repository Guidelines

## Build, Test, and Development Commands

**Only use `cargo test` and `cargo run`. NEVER use `cargo check` or `cargo build` for verification.**

**NEVER run `cargo clean` without asking the user first.**

```bash
# Run tests (this also builds everything)
cargo test -p truck-geometry --lib

# Run all core tests
cargo make cpu-test

# Run a specific test
cargo test -p truck-geometry test_name

# Format code
cargo fmt --all

# Run clippy linter
cargo clippy --all-targets -- -W warnings

# IMPORTANT: NEVER use --release flag unless the user EXPLICITLY requests it.
```

## Coding Style & Naming Conventions

- Follow standard Rust style: four-space indentation, `snake_case` for modules/functions, `CamelCase` for types.
- Write idiomatic and canonical Rust code. Avoid patterns common in imperative languages like C/C++/JS/TS that can be expressed more elegantly in Rust.

### Functional Style (CRITICAL)

- PREFER functional style over imperative style. Use `for_each` or `map` instead of for loops, use `collect` instead of pre-allocating a `Vec` and using `push`.
- PREFER direct initialization of collections. Use `BTreeMap::from([...])` or `vec![...]` instead of `new()` followed by `insert()`/`push()`.
- PREFER iterator chains over indexing into pre-allocated vectors. If you find yourself writing `vec![0.0; n]` followed by `for i in 0..n { v[i] = expr }`, rewrite as `(0..n).map(|i| expr).collect()`.

### Allocation Discipline

- AVOID unnecessary allocations, conversions, copies.
- When you need a small, fixed-upper-bound collection, prefer `SmallVec` (or similar stack-backed types) and build it with iterator/`collect` patterns rather than `new()+push` loops. This keeps things alloc-free.
- Prefer using the stack. Use `SmallVec` whenever it makes sense.

### Parallelism

- USE rayon to parallelize whenever larger amounts of data are being processed.
- Rayon is disabled on WASM targets.

### Safety

- AVOID using `unsafe` code unless absolutely necessary.
- AVOID return statements; structure functions with `if ... else if ... else` blocks instead. Early-return guard clauses (`if bad { return Err(...) }`) and `return` inside `match` arms in loops are acceptable when the alternative would deeply nest the entire function body.
- Prefer `?` operator and `Result` return types over `.unwrap()`.
- `.unwrap()` is permitted ONLY when an invariant guarantees the value will never be `None`/`Err`, with a `// SAFETY:` comment explaining why.

### Naming Conventions (Rust API Guidelines)

- **Casing**: `UpperCamelCase` for types/traits/variants; `snake_case` for functions/methods/modules/variables; `SCREAMING_SNAKE_CASE` for constants/statics.
- **Conversions**: `as_` for cheap borrowed-to-borrowed; `to_` for expensive conversions; `into_` for ownership-consuming conversions.
- **Getters**: No `get_` prefix (use `width()` not `get_width()`), except for unsafe variants like `get_unchecked()`.
- **Iterators**: `iter()` for `&T`, `iter_mut()` for `&mut T`, `into_iter()` for `T` by value.

### Imports

- Always import types at the top with `use` statements; never use inline paths in function bodies.
- External crate imports come before internal crate imports.

## Testing Guidelines

- **CRITICAL: ALWAYS run `cargo test` and `cargo clippy --all-targets -- -W warnings` before committing!**
- **CRITICAL: Address ALL warnings before EVERY commit!** This includes unused imports, dead code, deprecated API usage, and clippy warnings.
- Never use `#[allow(warnings)]` or similar suppressions without explicit user approval.
- **CRITICAL: Never modify test files** -- tests encode human intent.

## Documentation

- All code comments MUST end with a period.
- All comments must be on their own line. Never put comments at the end of a line of code.
- All references to types, keywords, symbols etc. MUST be enclosed in backticks: `struct`, `Foo`.
- For each part of the docs, every first reference to a type, keyword, symbol etc. that is NOT the item itself being described MUST be linked: [`Foo`].

## Commit & Pull Request Guidelines

- Use conventional commit prefixes: `feat:`, `fix:`, `chore:`.
- Keep messages concise and describe the user-facing effect.

## Error Handling

### Fallible Operations

- Prefer `?` operator and `Result` return types over `.unwrap()`.
- Use `.ok_or()` or `.ok_or_else()` to convert `Option` to `Result` with meaningful errors.

### Safe Unwrap Pattern

`.unwrap()` is permitted ONLY when an invariant guarantees the value will never be `None`/`Err`. This requires a **safety comment**:

```rust
// SAFETY: We just inserted `key` into `map` on the previous line,
// so it must exist.
let value = map.get(&key).unwrap();
```

## Writing Instructions

- Be concise.
- Use simple sentences. Technical jargon is fine.
- Do NOT overexplain basic concepts. Assume the user is technically proficient.
- AVOID flattering, corporate-ish or marketing language.
- AVOID vague and/or generic claims not substantiated by the context.
- AVOID weasel words.
