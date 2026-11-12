#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
build="$root/build/drivers-linux"

cmake -S "$root/drivers/linux" -B "$build"
cmake --build "$build"
ctest --test-dir "$build" --output-on-failure
