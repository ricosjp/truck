#!/usr/bin/env bash
# rename-truck.sh — Idempotent rename of truck-* crates to monstertruck-*
# Safe to re-run: skips already-renamed directories and content.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

###############################################################################
# 1. Directory renames (git mv if old dir exists and new dir does not)
###############################################################################
declare -A DIR_MAP=(
    ["truck-base"]="monstertruck-core"
    ["truck-derivers"]="monstertruck-derive"
    ["truck-geotrait"]="monstertruck-traits"
    ["truck-geometry"]="monstertruck-geometry"
    ["truck-topology"]="monstertruck-topology"
    ["truck-polymesh"]="monstertruck-mesh"
    ["truck-platform"]="monstertruck-platform"
    ["truck-meshalgo"]="monstertruck-meshing"
    ["truck-shapeops"]="monstertruck-solid"
    ["truck-modeling"]="monstertruck-modeling"
    ["truck-assembly"]="monstertruck-assembly"
    ["truck-stepio"]="monstertruck-step"
    ["truck-rendimpl"]="monstertruck-render"
    ["truck-js"]="monstertruck-wasm"
)

echo "=== Step 1: Directory renames ==="
for old in "${!DIR_MAP[@]}"; do
    new="${DIR_MAP[$old]}"
    if [ -d "$old" ] && [ ! -d "$new" ]; then
        echo "  git mv $old -> $new"
        git mv "$old" "$new"
    else
        echo "  skip $old (already renamed or missing)"
    fi
done

###############################################################################
# 2. Text replacements in file contents
#
# ORDERING IS CRITICAL: Non-standard suffix renames first, then prefix-only.
# This prevents partial matches (e.g. truck-base -> monstertruck-base -> ...).
###############################################################################

# File patterns to process
FILE_GLOBS='*.toml *.yml *.yaml *.md *.rs *.txt'

# Helper: idempotent sed across matching files
do_replace() {
    local old="$1" new="$2"
    shift 2
    local globs=("$@")
    # Build find args
    local find_args=()
    for g in "${globs[@]}"; do
        if [ ${#find_args[@]} -gt 0 ]; then
            find_args+=("-o")
        fi
        find_args+=("-name" "$g")
    done
    # Use find + sed; skip .git, target, resources dirs
    find "$REPO_ROOT" \
        -path '*/.git' -prune -o \
        -path '*/target' -prune -o \
        -path '*/.target-local' -prune -o \
        -path '*/resources' -prune -o \
        \( "${find_args[@]}" \) -type f -print0 \
        | xargs -0 -r sed -i "s|${old}|${new}|g"
}

echo ""
echo "=== Step 2: Hyphenated name replacements ==="

# Phase A: Non-standard renames (suffix changes) — MUST come first
declare -a HYPHEN_PHASE_A=(
    "truck-polymesh:monstertruck-mesh"
    "truck-stepio:monstertruck-step"
    "truck-rendimpl:monstertruck-render"
    "truck-derivers:monstertruck-derive"
    "truck-base:monstertruck-core"
    "truck-js:monstertruck-wasm"
    "truck-geotrait:monstertruck-traits"
    "truck-meshalgo:monstertruck-meshing"
    "truck-shapeops:monstertruck-solid"
)

# Phase B: Standard prefix-only renames
declare -a HYPHEN_PHASE_B=(
    "truck-geometry:monstertruck-geometry"
    "truck-topology:monstertruck-topology"
    "truck-platform:monstertruck-platform"
    "truck-modeling:monstertruck-modeling"
    "truck-assembly:monstertruck-assembly"
)

HYPH_GLOBS=(*.toml *.yml *.yaml *.md *.rs *.txt)

for pair in "${HYPHEN_PHASE_A[@]}"; do
    old="${pair%%:*}"
    new="${pair##*:}"
    echo "  $old -> $new"
    do_replace "$old" "$new" "${HYPH_GLOBS[@]}"
done

for pair in "${HYPHEN_PHASE_B[@]}"; do
    old="${pair%%:*}"
    new="${pair##*:}"
    echo "  $old -> $new"
    do_replace "$old" "$new" "${HYPH_GLOBS[@]}"
done

echo ""
echo "=== Step 3: Underscored name replacements ==="

# Phase A: Non-standard renames (suffix changes) — MUST come first
declare -a USCORE_PHASE_A=(
    "truck_polymesh:monstertruck_mesh"
    "truck_stepio:monstertruck_step"
    "truck_rendimpl:monstertruck_render"
    "truck_derivers:monstertruck_derive"
    "truck_base:monstertruck_core"
    "truck_js:monstertruck_wasm"
    "truck_geotrait:monstertruck_traits"
    "truck_meshalgo:monstertruck_meshing"
    "truck_shapeops:monstertruck_solid"
)

# Phase B: Standard prefix-only renames
declare -a USCORE_PHASE_B=(
    "truck_geometry:monstertruck_geometry"
    "truck_topology:monstertruck_topology"
    "truck_platform:monstertruck_platform"
    "truck_modeling:monstertruck_modeling"
    "truck_assembly:monstertruck_assembly"
)

USCORE_GLOBS=(*.rs *.toml *.md)

for pair in "${USCORE_PHASE_A[@]}"; do
    old="${pair%%:*}"
    new="${pair##*:}"
    echo "  $old -> $new"
    do_replace "$old" "$new" "${USCORE_GLOBS[@]}"
done

for pair in "${USCORE_PHASE_B[@]}"; do
    old="${pair%%:*}"
    new="${pair##*:}"
    echo "  $old -> $new"
    do_replace "$old" "$new" "${USCORE_GLOBS[@]}"
done

###############################################################################
# 4. Manual fixups not covered by mechanical rename
###############################################################################

echo ""
echo "=== Step 4: Manual fixups ==="

# STEP file header strings (prose 'truck' inside string literals)
echo "  STEP header strings"
find "$REPO_ROOT" -name '*.rs' -path '*/monstertruck-step/*' -print0 \
    | xargs -0 -r sed -i \
        -e "s|'Shape Data from truck'|'Shape Data from monstertruck'|g" \
        -e "s|'truck'|'monstertruck'|g"

# Cargo.toml homepage/repository URLs
echo "  Cargo.toml URLs: ricosjp/truck -> virtualritz/truck"
find "$REPO_ROOT" -name 'Cargo.toml' \
    -not -path '*/target/*' -not -path '*/.target-local/*' -print0 \
    | xargs -0 -r sed -i 's|https://github.com/ricosjp/truck|https://github.com/virtualritz/truck|g'

echo ""
echo "=== Done! ==="
echo "Run 'cargo build --workspace' to verify."
