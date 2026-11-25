#!/bin/sh
# Build script for the Tag:Rename app.
#
# This sandbox has no system C toolchain, so the build uses a private
# Rust toolchain plus `zig cc` as the linker (both installed under
# /tmp/opencode). setup-toolchain.sh (re)creates them if they are missing.
set -e
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/setup-toolchain.sh"

cd "$HERE"
cargo build --release "$@"
