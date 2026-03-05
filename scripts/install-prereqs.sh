#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
LOCAL_DIR="$PROJECT_DIR/.local"
BIN_DIR="$LOCAL_DIR/bin"

mkdir -p "$BIN_DIR"

OS="$(uname -s)"
ARCH="$(uname -m)"

ok()   { printf '  \033[32m✓\033[0m %s\n' "$1"; }
info() { printf '  \033[34m→\033[0m %s\n' "$1"; }
warn() { printf '  \033[33m!\033[0m %s\n' "$1"; }

command_available() {
    command -v "$1" &>/dev/null || [[ -x "$BIN_DIR/$1" ]]
}

path_has_local() {
    echo "$PATH" | tr ':' '\n' | grep -q "^$BIN_DIR$"
}

echo "=== surrealdb.c prerequisites installer ==="
echo "    Install prefix: $LOCAL_DIR"
echo ""

# --- Rust toolchain ---
if command -v rustc &>/dev/null && command -v cargo &>/dev/null; then
    ok "Rust toolchain: $(rustc --version)"
else
    info "Installing Rust toolchain via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
    export PATH="$HOME/.cargo/bin:$PATH"
    ok "Rust installed: $(rustc --version)"
fi

# --- cmake ---
if command_available cmake; then
    ok "cmake: $(cmake --version 2>/dev/null | head -1)"
else
    info "Installing cmake..."
    case "$OS" in
        Darwin)
            if command -v brew &>/dev/null; then
                brew install cmake
            else
                warn "Homebrew not found. Please install cmake manually."
                exit 1
            fi
            ;;
        Linux)
            CMAKE_VER="3.31.6"
            case "$ARCH" in
                x86_64)  CMAKE_ARCH="x86_64" ;;
                aarch64) CMAKE_ARCH="aarch64" ;;
                *)       warn "Unsupported arch $ARCH for cmake"; exit 1 ;;
            esac
            curl -fsSL "https://github.com/Kitware/CMake/releases/download/v${CMAKE_VER}/cmake-${CMAKE_VER}-linux-${CMAKE_ARCH}.tar.gz" \
                | tar xz -C "$LOCAL_DIR" --strip-components=1
            ;;
    esac
    ok "cmake installed"
fi

# --- make ---
if command_available make; then
    ok "make: $(make --version 2>/dev/null | head -1)"
else
    info "Installing make..."
    case "$OS" in
        Darwin)
            xcode-select --install 2>/dev/null || true
            warn "Xcode Command Line Tools install triggered. Re-run this script after install completes."
            exit 1
            ;;
        Linux)
            if command -v apt-get &>/dev/null; then
                sudo apt-get update && sudo apt-get install -y build-essential
            elif command -v dnf &>/dev/null; then
                sudo dnf install -y make gcc
            else
                warn "Cannot auto-install make. Please install it manually."
                exit 1
            fi
            ;;
    esac
    ok "make installed"
fi

# --- C compiler ---
if command -v cc &>/dev/null || command -v gcc &>/dev/null || command -v clang &>/dev/null; then
    CC_BIN="$(command -v cc || command -v clang || command -v gcc)"
    ok "C compiler: $CC_BIN"
else
    warn "No C compiler found. Install Xcode CLT (macOS) or build-essential (Linux)."
    exit 1
fi

# --- SurrealDB CLI ---
if command_available surreal; then
    ok "surreal CLI: $(surreal version 2>/dev/null || echo 'available')"
else
    info "Installing SurrealDB CLI into $BIN_DIR..."
    case "$OS" in
        Darwin)
            case "$ARCH" in
                arm64)   SDB_ARCH="arm64" ;;
                x86_64)  SDB_ARCH="x86_64" ;;
                *)       warn "Unsupported arch $ARCH"; exit 1 ;;
            esac
            curl -fsSL "https://download.surrealdb.com/v3.0.1/surreal-v3.0.1.darwin-${SDB_ARCH}.tgz" \
                | tar xz -C "$BIN_DIR"
            ;;
        Linux)
            case "$ARCH" in
                x86_64)  SDB_ARCH="x86_64" ;;
                aarch64) SDB_ARCH="aarch64" ;;
                *)       warn "Unsupported arch $ARCH"; exit 1 ;;
            esac
            curl -fsSL "https://download.surrealdb.com/v3.0.1/surreal-v3.0.1.linux-${SDB_ARCH}.tgz" \
                | tar xz -C "$BIN_DIR"
            ;;
    esac
    chmod +x "$BIN_DIR/surreal"
    ok "surreal CLI installed to $BIN_DIR/surreal"
fi

echo ""
echo "=== All prerequisites satisfied ==="
echo ""
if ! path_has_local; then
    echo "Add this to your shell profile to use local tools:"
    echo "  export PATH=\"$BIN_DIR:\$PATH\""
    echo ""
fi
echo "Verify with:"
echo "  rustc --version"
echo "  cargo --version"
echo "  cmake --version"
echo "  make --version"
echo "  $BIN_DIR/surreal version"
