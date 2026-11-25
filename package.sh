#!/bin/sh
# Builds the release program and packs it for deployment:
#   dist/tagrename-<version>-linux-x86_64.tar.gz   (any Linux, via install.sh)
#   (Arch-based systems: run makepkg in this folder, see PKGBUILD)
# The archive holds the program, its icons and menu entry, and install.sh.
set -e

HERE="$(cd "$(dirname "$0")" && pwd)"
cd "$HERE"

VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
NAME="tagrename-${VERSION}-linux-x86_64"
OUT="$HERE/dist"
STAGE="$OUT/$NAME"

cargo build --release

rm -rf "$STAGE"
mkdir -p "$STAGE/icons"
cp target/release/tagrename "$STAGE/tagrename"
cp assets/tagrename-48.png assets/tagrename-128.png assets/tagrename-256.png assets/tagrename-512.png "$STAGE/icons/"
cp package/install.sh package/tagrename.desktop package/README.txt "$STAGE/"
chmod 755 "$STAGE/install.sh" "$STAGE/tagrename"

tar -C "$OUT" -czf "$OUT/$NAME.tar.gz" "$NAME"
rm -rf "$STAGE"
(cd "$OUT" && sha256sum "$NAME.tar.gz" > "$NAME.tar.gz.sha256")

echo "Package written to $OUT/$NAME.tar.gz"
