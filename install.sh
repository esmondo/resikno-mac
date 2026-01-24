#!/bin/bash
set -e

# Resikno Installer
# Usage: curl -sSL https://raw.githubusercontent.com/esmondo/resikno-mac/main/install.sh | bash

REPO="esmondo/resikno-mac"
BINARY_NAME="resikno"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() { echo -e "${GREEN}[INFO]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# Detect OS and architecture
detect_platform() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)

    case "$OS" in
        darwin) OS="apple-darwin" ;;
        linux) OS="unknown-linux-gnu" ;;
        *) error "Unsupported OS: $OS" ;;
    esac

    case "$ARCH" in
        x86_64) ARCH="x86_64" ;;
        arm64|aarch64) ARCH="aarch64" ;;
        *) error "Unsupported architecture: $ARCH" ;;
    esac

    PLATFORM="${ARCH}-${OS}"
    info "Detected platform: $PLATFORM"
}

# Check for required tools
check_requirements() {
    if ! command -v curl &> /dev/null; then
        error "curl is required but not installed"
    fi
}

# Install via cargo (fallback if no prebuilt binary)
install_via_cargo() {
    info "Installing via cargo..."

    if ! command -v cargo &> /dev/null; then
        warn "Cargo not found. Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi

    cargo install --git "https://github.com/${REPO}.git" --branch main
    info "Installed successfully via cargo!"
    echo ""
    echo "Run 'resikno --help' to get started"
}

# Try to download prebuilt binary, fall back to cargo
install_binary() {
    LATEST_RELEASE=$(curl -sSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/' || echo "")

    if [ -z "$LATEST_RELEASE" ]; then
        warn "No prebuilt releases found, building from source..."
        install_via_cargo
        return
    fi

    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_RELEASE}/${BINARY_NAME}-${PLATFORM}.tar.gz"

    info "Downloading ${BINARY_NAME} ${LATEST_RELEASE}..."

    TMP_DIR=$(mktemp -d)
    trap "rm -rf $TMP_DIR" EXIT

    if curl -sSL "$DOWNLOAD_URL" -o "$TMP_DIR/resikno.tar.gz" 2>/dev/null; then
        tar -xzf "$TMP_DIR/resikno.tar.gz" -C "$TMP_DIR"

        info "Installing to $INSTALL_DIR..."
        sudo mkdir -p "$INSTALL_DIR"
        sudo mv "$TMP_DIR/$BINARY_NAME" "$INSTALL_DIR/"
        sudo chmod +x "$INSTALL_DIR/$BINARY_NAME"

        info "Installed successfully!"
        echo ""
        echo "Run 'resikno --help' to get started"
    else
        warn "Prebuilt binary not available for $PLATFORM, building from source..."
        install_via_cargo
    fi
}

main() {
    echo ""
    echo "  ╭─────────────────────────────────╮"
    echo "  │     Resikno Installer           │"
    echo "  │     Disk Cleanup for macOS      │"
    echo "  ╰─────────────────────────────────╯"
    echo ""

    check_requirements
    detect_platform
    install_binary
}

main
