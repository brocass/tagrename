# Maintainer: Tag:Rename
# Builds and installs Tag:Rename on Arch-based systems (such as Omarchy).
#   cd app && makepkg -f --noconfirm && sudo pacman -U tagrename-1.1.3-1-x86_64.pkg.tar.zst
pkgname=tagrename
pkgver=1.1.3
pkgrel=1
pkgdesc="Tag:Rename - a modern dark-mode music tag editor and file renamer"
arch=('x86_64')
license=('custom')
depends=('libxkbcommon' 'wayland' 'libx11' 'libxcursor' 'libxrandr' 'libxi' 'mesa')
makedepends=('rust')
options=('!debug')
source=()
sha256sums=()

build() {
  cd "$startdir"
  CARGO_TARGET_DIR="$srcdir/target" cargo build --release --locked
}

package() {
  install -Dm755 "$srcdir/target/release/tagrename" "$pkgdir/usr/bin/tagrename"
  install -Dm644 "$startdir/package/tagrename.desktop" "$pkgdir/usr/share/applications/tagrename.desktop"
  for s in 16 32 48 64 128 256 512; do
    install -Dm644 "$startdir/assets/tagrename-$s.png" "$pkgdir/usr/share/icons/hicolor/${s}x${s}/apps/tagrename.png"
  done
  install -Dm644 "$startdir/assets/tagrename.svg" "$pkgdir/usr/share/icons/hicolor/scalable/apps/tagrename.svg"
}
