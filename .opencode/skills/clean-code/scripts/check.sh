#!/usr/bin/env bash
set -euo pipefail

if ! command -v cargo >/dev/null 2>&1; then
    if [[ -f "$HOME/.cargo/env" ]]; then
        # shellcheck disable=SC1091
        source "$HOME/.cargo/env"
    fi
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "error: cargo not found; install Rust or source ~/.cargo/env" >&2
    exit 1
fi

if [[ ! -f Cargo.toml ]]; then
    echo "error: run this script from the workspace root (Cargo.toml not found)" >&2
    exit 1
fi

echo "== cargo fmt =="
cargo fmt --all -- --check

echo "== cargo clippy =="
cargo clippy --workspace --all-targets -- -D warnings

echo "== cargo test =="
cargo test --workspace
