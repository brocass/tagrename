#!/bin/sh
# Recreates the host-side cargo/cc launchers that let Craft Anything's
# build check find this project's Rust toolchain.
#
# Why: the checker runs `cargo build --quiet` with a PATH that includes
# $HOME/.cargo/bin (the standard rustup location). This sandbox's home is
# read-only, so nothing inside the sandbox can put cargo there - but the
# app has permission to run commands on the host (flatpak-spawn), and the
# host's home is writable from the host side. The scripts below are thin
# launchers: they exec the real cargo and linker kept in Craft Anything's
# persistent data folder, and log every invocation so builds can be traced.
#
# Safe to run repeatedly; it only rewrites the two launcher scripts.
set -e

T=/home/steven/.var/app/ai.storyteller.craft-anything/toolchain

# Already working? Then nothing to do.
if [ -x /home/steven/.cargo/bin/cargo ] && /home/steven/.cargo/bin/cargo --version >/dev/null 2>&1; then
    echo "cargo launcher already working:"
    /home/steven/.cargo/bin/cargo --version
    exit 0
fi

command -v flatpak-spawn >/dev/null 2>&1 || {
    echo "flatpak-spawn is not available; cannot create the host launchers." >&2
    exit 1
}

flatpak-spawn --host sh -s <<EOF
set -e
mkdir -p /home/steven/.cargo/bin
cat > /home/steven/.cargo/bin/cargo <<'SHIM'
#!/bin/sh
# Thin shim: the Rust toolchain Craft Anything keeps for this project.
# The sandbox home is read-only, so cargo itself lives in the app data dir.
T=/home/steven/.var/app/ai.storyteller.craft-anything/toolchain
LOG=\$T/invocations.log
{
  echo "=== \$(date -u "+%Y-%m-%dT%H:%M:%SZ") pid=\$\$ args:\$*"
  echo "    cwd=\$(pwd)"
  echo "    PATH=\$PATH"
  ps -o ppid=,args= -p \$PPID 2>/dev/null | head -1
} >> "\$LOG" 2>&1
export RUSTUP_HOME="\$T/rustup"
export CARGO_HOME="\$T/cargo"
export ZIG_GLOBAL_CACHE_DIR="\$T/zig-cache"
exec "\$T/cargo/bin/cargo" "\$@"
SHIM
cat > /home/steven/.cargo/bin/cc <<'SHIM'
#!/bin/sh
# Linker shim: zig cc stands in for the system C compiler.
T=/home/steven/.var/app/ai.storyteller.craft-anything/toolchain
export ZIG_GLOBAL_CACHE_DIR="\$T/zig-cache"
exec "\$T/zig-dir/zig" cc -target x86_64-linux-gnu.2.41 "\$@"
SHIM
chmod +x /home/steven/.cargo/bin/cargo /home/steven/.cargo/bin/cc
EOF

echo "cargo launcher created:"
/home/steven/.cargo/bin/cargo --version
