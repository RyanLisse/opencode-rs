#!/bin/sh
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Print colored output
print_info() {
    printf "${GREEN}[INFO]${NC} %s\n" "$1"
}

print_warn() {
    printf "${YELLOW}[WARN]${NC} %s\n" "$1"
}

print_error() {
    printf "${RED}[ERROR]${NC} %s\n" "$1" >&2
}

# Check if running as root
if [ "$(id -u)" = "0" ]; then
    INSTALL_DIR="/usr/local/bin"
    SUDO=""
else
    INSTALL_DIR="$HOME/.local/bin"
    SUDO="sudo"
    # Create local bin directory if it doesn't exist
    mkdir -p "$INSTALL_DIR"
fi

# Determine OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"
TARGET=""

print_info "Detected OS: $OS, Architecture: $ARCH"

case "$OS" in
    Linux)
        if [ "$ARCH" = "x86_64" ]; then
            TARGET="x86_64-linux-musl"
        else
            print_error "Unsupported Linux architecture: $ARCH"
            print_info "Supported architectures: x86_64"
            exit 1
        fi
        ;;
    Darwin) # macOS
        if [ "$ARCH" = "arm64" ]; then
            TARGET="aarch64-apple-darwin"
        elif [ "$ARCH" = "x86_64" ]; then
            TARGET="x86_64-apple-darwin"
        else
            print_error "Unsupported macOS architecture: $ARCH"
            print_info "Supported architectures: arm64, x86_64"
            exit 1
        fi
        ;;
    *)
        print_error "Unsupported operating system: $OS"
        print_info "Supported systems: Linux, macOS"
        exit 1
        ;;
esac

print_info "Target platform: $TARGET"

# Check for required tools
if ! command -v curl >/dev/null 2>&1; then
    print_error "curl is required but not installed."
    exit 1
fi

if ! command -v tar >/dev/null 2>&1; then
    print_error "tar is required but not installed."
    exit 1
fi

# Fetch the latest release version from GitHub API
print_info "Fetching latest release information..."
LATEST_RELEASE=$(curl -s "https://api.github.com/repos/your-org/opencode-rs/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_RELEASE" ]; then
    print_error "Could not find the latest release."
    print_info "Please check your internet connection and try again."
    exit 1
fi

print_info "Latest version: $LATEST_RELEASE"

DOWNLOAD_URL="https://github.com/your-org/opencode-rs/releases/download/${LATEST_RELEASE}/opencode-${TARGET}.tar.gz"

# Create temporary directory
TMP_DIR=$(mktemp -d)
TMP_FILE="$TMP_DIR/opencode-${TARGET}.tar.gz"

# Download and install
print_info "Downloading OpenCode CLI from ${DOWNLOAD_URL}..."
if ! curl -fsSL "${DOWNLOAD_URL}" -o "$TMP_FILE"; then
    print_error "Failed to download OpenCode CLI."
    rm -rf "$TMP_DIR"
    exit 1
fi

print_info "Extracting to temporary directory..."
cd "$TMP_DIR"
tar -xzf "$TMP_FILE"

# Install binary
print_info "Installing to $INSTALL_DIR..."
if [ "$INSTALL_DIR" = "/usr/local/bin" ] && [ "$(id -u)" != "0" ]; then
    $SUDO mv opencode "$INSTALL_DIR/"
    $SUDO chmod +x "$INSTALL_DIR/opencode"
else
    mv opencode "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/opencode"
fi

# Cleanup
rm -rf "$TMP_DIR"

# Verify installation
if command -v opencode >/dev/null 2>&1; then
    print_info "OpenCode CLI installed successfully!"
    print_info "Version: $(opencode --version)"
else
    print_warn "OpenCode CLI installed to $INSTALL_DIR but is not in PATH."
    print_info "Add $INSTALL_DIR to your PATH environment variable."
    if [ "$INSTALL_DIR" = "$HOME/.local/bin" ]; then
        print_info "Add this line to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
        printf "    export PATH=\"\$HOME/.local/bin:\$PATH\"\n"
    fi
fi

print_info "Run 'opencode --help' to get started."

# Check for GUI version
print_info ""
print_info "For the GUI version, visit: https://github.com/your-org/opencode-rs/releases"