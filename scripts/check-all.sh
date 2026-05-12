#!/usr/bin/env bash
# check-all.sh — local reproduction of the CI quality gate.
#
# Runs the same checks as .github/workflows/check.yml against every
# dev-* platform in this repo, in sequence. Designed as the "one command
# to validate everything" before opening a PR.
#
# Checks per platform (platform/dev-*):
#   1. cargo fmt --check
#   2. cargo build --locked
#   3. cargo clippy --locked -- -D warnings
#   4. cargo deny --locked check          (if cargo-deny is installed)
#
# fmt is additionally run against `platform-common` (library-only crate,
# no build target) so every tracked Rust file is formatted.
#
# Requirements:
#   - rustup / cargo (honours ./rust-toolchain.toml — all targets preinstalled)
#   - cargo-deny (optional; skipped with a warning if absent)
#   - flip-link  (required by dev-imxrt + dev-npcx linker; warned-about if absent)
#
# Exits non-zero on the first failure. `set -euo pipefail`.

set -euo pipefail

REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT"

PLATFORMS=(dev-imxrt dev-mcxa dev-npcx dev-qemu)
FMT_CRATES=(platform-common dev-imxrt dev-mcxa dev-npcx dev-qemu)

banner() { printf '\n=== %s ===\n' "$*"; }

have() { command -v "$1" >/dev/null 2>&1; }

if ! have flip-link; then
    echo "warning: flip-link not installed — dev-imxrt / dev-mcxa / dev-npcx builds will fail." >&2
    echo "         Install with: cargo install flip-link --locked" >&2
fi

if ! have cargo-deny; then
    echo "warning: cargo-deny not installed — skipping deny checks. Install with:" >&2
    echo "         cargo install --locked cargo-deny" >&2
fi

for c in "${FMT_CRATES[@]}"; do
    banner "$c :: cargo fmt --check"
    (cd "platform/$c" && cargo fmt --check)
done

for p in "${PLATFORMS[@]}"; do
    dir="platform/$p"
    banner "$p :: cargo build --locked"
    (cd "$dir" && cargo build --locked)

    banner "$p :: cargo clippy --locked -- -D warnings"
    (cd "$dir" && cargo clippy --locked -- -D warnings)

    if have cargo-deny; then
        banner "$p :: cargo deny --locked check"
        (cd "$dir" && cargo deny --locked check)
    fi
done

banner "ALL CHECKS PASSED"
