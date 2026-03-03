#!/usr/bin/env bash
# publish.sh — Publish all monstertruck crates to crates.io in dependency order.
#
# Usage:
#   ./scripts/publish.sh          # Dry-run (default)
#   ./scripts/publish.sh --exec   # Actually publish
#
# Crates are published in topological order so that each crate's
# dependencies are already available on crates.io when it's published.
set -euo pipefail

DRY_RUN=true
if [[ "${1:-}" == "--exec" ]]; then
    DRY_RUN=false
fi

# Topological order (leaves first, dependents later).
CRATES=(
    monstertruck-core
    monstertruck-derive
    monstertruck-assembly
    monstertruck-traits
    monstertruck-gpu
    monstertruck-mesh
    monstertruck-geometry
    monstertruck-topology
    monstertruck-meshing
    monstertruck-modeling
    monstertruck-solid
    monstertruck-step
    monstertruck-render
    monstertruck-wasm
)

# Seconds to wait between publishes for crates.io index propagation.
WAIT=30

for crate in "${CRATES[@]}"; do
    echo "=== Publishing $crate ==="
    if $DRY_RUN; then
        cargo publish -p "$crate" --dry-run
    else
        cargo publish -p "$crate"
        # Wait for crates.io index to propagate before publishing dependents.
        if [[ "$crate" != "${CRATES[-1]}" ]]; then
            echo "  Waiting ${WAIT}s for crates.io index propagation..."
            sleep "$WAIT"
        fi
    fi
    echo ""
done

if $DRY_RUN; then
    echo "=== Dry run complete. Run with --exec to publish for real. ==="
else
    echo "=== All crates published! ==="
fi
