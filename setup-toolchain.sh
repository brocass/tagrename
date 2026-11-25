# Installs (if missing) the private toolchain this sandbox needs to build Rust:
#  - a persistent copy under Craft Anything's own data folder
#    (~/.var/app/ai.storyteller.craft-anything/toolchain), which is writable
#    from inside the sandbox
#  - or, as a fallback, a session copy under /tmp/opencode (the home
#    directory is read-only and the sandbox ships no gcc)
# Sourcing this file exports the environment variables cargo needs.
#
# HOW CRAFT ANYTHING'S CHECK FINDS CARGO (settled 2026-11-22): the checker
# spawns `cargo build --quiet` with a PATH it builds itself, and that PATH
# includes $HOME/.cargo/bin (the standard rustup location). The sandbox home
# is read-only, so the previous plan (a Flatpak PATH override) could never
# work: ~/.local/share/flatpak is a private tmpfs inside the sandbox, so the
# override never reached the host. The working fix uses the app's own
# host-spawn permission instead: `flatpak-spawn --host` created two small
# launcher scripts on the host at ~/.cargo/bin/cargo and ~/.cargo/bin/cc that
# point back at the persistent toolchain (cargo, rustc and the zig linker).
# Every process that builds this project - the checker, the MCP server and
# build.sh - now resolves cargo through them. The shims also log each spawn
# to toolchain/invocations.log. If they are ever removed, recreate them with
# app/make-cargo-shims.sh.
set -e

PERSIST=/home/steven/.var/app/ai.storyteller.craft-anything/toolchain
BASE=/tmp/opencode
export RUSTUP_HOME="$BASE/rustup"
export CARGO_HOME="$BASE/cargo"
export PATH="$CARGO_HOME/bin:$BASE/bin:$PATH"

# Prefer the persistent toolchain when it is complete.
if [ -x "$PERSIST/bin/cargo" ] && [ -d "$PERSIST/rustup/toolchains" ] && [ -d "$PERSIST/zig-dir" ]; then
    export RUSTUP_HOME="$PERSIST/rustup"
    export CARGO_HOME="$PERSIST/cargo"
    export PATH="$PERSIST/bin:$PATH"
    # The linker is resolved through PATH (the zig "cc" wrapper in
    # $PERSIST/bin), exactly like Craft Anything's own check build does.
    export ZIG_GLOBAL_CACHE_DIR="$PERSIST/zig-cache"
    return 0 2>/dev/null || true
fi

if [ ! -x "$CARGO_HOME/bin/cargo" ]; then
    mkdir -p "$BASE"
    curl -sSfL https://sh.rustup.rs -o "$BASE/rustup-init.sh"
    sh "$BASE/rustup-init.sh" -y --profile minimal --default-toolchain stable --no-modify-path
fi

if [ ! -d "$BASE/zig-dir" ]; then
    curl -s -m 20 https://ziglang.org/download/index.json -o "$BASE/zig-index.json"
    URL=$(python3 -c "import json;print(json.load(open('$BASE/zig-index.json'))['0.17.1']['x86_64-linux']['tarball'])")
    curl -sL -o "$BASE/zig.tar.xz" "$URL"
    tar xf "$BASE/zig.tar.xz" -C "$BASE"
    mv "$BASE"/zig-x86_64-linux-* "$BASE/zig-dir"
fi

mkdir -p "$BASE/bin" "$BASE/zig-cache"
printf '#!/bin/sh\nexport ZIG_GLOBAL_CACHE_DIR=/tmp/opencode/zig-cache\nexec /tmp/opencode/zig-dir/zig cc -target x86_64-linux-gnu.2.41 "$@"\n' > "$BASE/bin/cc"
chmod +x "$BASE/bin/cc"

export RUSTFLAGS="-C linker=$BASE/bin/cc -C linker-flavor=gcc"
export CARGO_NET_GIT_FETCH_WITH_CLI=true
