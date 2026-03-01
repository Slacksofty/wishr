#!/usr/bin/env bash
# Local CI check — mirrors the GitHub Actions CI job exactly.
# Usage:
#   ./ci-check.sh          # all checks except the full build
#   ./ci-check.sh --full   # also run cargo leptos build --release

set -euo pipefail

FULL=false
[[ "${1:-}" == "--full" ]] && FULL=true

# ── colours ────────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'

pass() { echo -e "${GREEN}✓ $1${RESET}"; }
fail() { echo -e "${RED}✗ $1${RESET}"; exit 1; }
step() { echo -e "\n${CYAN}${BOLD}── $1 ──${RESET}"; }

# ── prerequisites ──────────────────────────────────────────────────────────
step "Prerequisites"

# Ensure Rust toolchain components are present
for component in rustfmt clippy; do
    if ! rustup component list --installed | grep -q "^${component}-"; then
        echo "Installing missing component: $component"
        rustup component add "$component"
    fi
done

# Ensure wasm32 target is present
if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
    echo "Installing wasm32-unknown-unknown target"
    rustup target add wasm32-unknown-unknown
fi

pass "Prerequisites"

# ── format check ──────────────────────────────────────────────────────────
step "Format (cargo fmt)"
cargo fmt --all -- --check && pass "Format" || fail "Format — run 'cargo fmt --all' to fix"

# ── clippy SSR ─────────────────────────────────────────────────────────────
step "Clippy SSR (native)"
cargo clippy --features ssr --target x86_64-unknown-linux-gnu -- -D warnings \
    && pass "Clippy SSR" || fail "Clippy SSR"

# ── clippy hydrate ─────────────────────────────────────────────────────────
step "Clippy hydrate (wasm32)"
cargo clippy --features hydrate --target wasm32-unknown-unknown -- -D warnings \
    && pass "Clippy hydrate" || fail "Clippy hydrate"

# ── type-check ─────────────────────────────────────────────────────────────
step "Type-check SSR"
cargo check --features ssr && pass "Type-check SSR" || fail "Type-check SSR"

step "Type-check hydrate"
cargo check --features hydrate --target wasm32-unknown-unknown \
    && pass "Type-check hydrate" || fail "Type-check hydrate"

# ── tests ──────────────────────────────────────────────────────────────────
step "Tests"
cargo test --features ssr && pass "Tests" || fail "Tests"

# ── full build (optional) ─────────────────────────────────────────────────
if $FULL; then
    step "Full release build (cargo leptos build --release)"
    if ! command -v cargo-leptos &>/dev/null; then
        fail "cargo-leptos not found — install it or run 'cargo leptos build --release' manually"
    fi
    cargo leptos build --release && pass "Full release build" || fail "Full release build"
fi

# ── done ───────────────────────────────────────────────────────────────────
echo -e "\n${GREEN}${BOLD}All checks passed!${RESET}"
