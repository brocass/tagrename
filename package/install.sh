#!/bin/sh
# Installs Tag:Rename for the current user (no administrator rights needed).
#
#   ./install.sh             install into ~/.local
#   ./install.sh --uninstall remove it again
set -e

HERE="$(cd "$(dirname "$0")" && pwd)"
BIN_DIR="$HOME/.local/bin"
APP_DIR="$HOME/.local/share/applications"
ICON_ROOT="$HOME/.local/share/icons/hicolor"
SIZES="48 128 256 512"

if [ "$1" = "--uninstall" ]; then
    rm -f "$BIN_DIR/tagrename" "$APP_DIR/tagrename.desktop"
    for s in $SIZES; do rm -f "$ICON_ROOT/${s}x${s}/apps/tagrename.png"; done
    echo "Tag:Rename was removed. Your settings in ~/.config/tagrename were kept."
    exit 0
fi

mkdir -p "$BIN_DIR" "$APP_DIR"
install -m 755 "$HERE/tagrename" "$BIN_DIR/tagrename"
for s in $SIZES; do
    mkdir -p "$ICON_ROOT/${s}x${s}/apps"
    install -m 644 "$HERE/icons/tagrename-$s.png" "$ICON_ROOT/${s}x${s}/apps/tagrename.png"
done
sed "s|^Exec=.*|Exec=$BIN_DIR/tagrename|" "$HERE/tagrename.desktop" > "$APP_DIR/tagrename.desktop"
chmod 644 "$APP_DIR/tagrename.desktop"

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APP_DIR" >/dev/null 2>&1 || true
fi

echo "Tag:Rename is installed in $BIN_DIR/tagrename."
case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *) echo "Note: $BIN_DIR is not on your PATH yet, so start it from the app menu or with its full path." ;;
esac
