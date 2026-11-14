#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

cargo build --workspace --release

target="$(rustc -vV | sed -n 's/^host: //p')"
name="emucad-$target"
stage="dist/$name"

rm -rf "$stage"
mkdir -p "$stage/docs" "$stage/share/applications"
cp target/release/emucad "$stage/"
cp target/release/emucad-gui "$stage/"
cp packaging/emucad-gui.desktop "$stage/share/applications/"
cp README.md LICENSE "$stage/"
cp docs/*.md "$stage/docs/"
cp -r examples "$stage/examples"

tar -czf "dist/$name.tar.gz" -C dist "$name"
rm -rf "$stage"

echo "dist/$name.tar.gz"
