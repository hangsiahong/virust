#!/bin/bash
# Virust Installation Script
# Installs Virust CLI from GitHub repository

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
REPO="hangsiahong/virust"
INSTALL_DIR="${TMPDIR:-/tmp}/virust-install"
BIN_DIR="$HOME/.cargo/bin"
VERSION="${1:-main}"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Virust Installation Script${NC}"
echo -e "${BLUE}========================================${NC}\n"

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}✗${NC} Cargo not found. Please install Rust first:"
    echo "  https://www.rust-lang.org/tools/install"
    exit 1
fi

echo -e "${YELLOW}[1/4] Checking prerequisites...${NC}"
echo -e "${GREEN}✓${NC} Rust and Cargo installed"

# Detect platform
ARCH=$(uname -m)
OS=$(uname -s)

case "$OS" in
    Linux*)     PLATFORM="linux";;
    Darwin*)    PLATFORM="macos";;
    *)          echo -e "${RED}✗${NC} Unsupported OS: $OS"; exit 1;;
esac

case "$ARCH" in
    x86_64)    ARCH="x86_64";;
    aarch64)   ARCH="aarch64";;
    arm64)     ARCH="aarch64";;
    *)         echo -e "${RED}✗${NC} Unsupported architecture: $ARCH"; exit 1;;
esac

echo -e "${GREEN}✓${NC} Platform: $PLATFORM-$ARCH"

echo ""
echo -e "${YELLOW}[2/4] Downloading Virust...${NC}"

# Clean up any previous installation
rm -rf "$INSTALL_DIR"
mkdir -p "$INSTALL_DIR"

# Clone repository
echo -e "${BLUE}Cloning from: https://github.com/${REPO}.git${NC}"
if [ "$VERSION" = "main" ]; then
    git clone --depth 1 "https://github.com/${REPO}.git" "$INSTALL_DIR"
else
    git clone --branch "$VERSION" --depth 1 "https://github.com/${REPO}.git" "$INSTALL_DIR"
fi

echo ""
echo -e "${YELLOW}[3/4] Building Virust CLI...${NC}"
cd "$INSTALL_DIR"

# Build in release mode
cargo build --release --bin virust 2>&1 | while IFS= read -r line; do
    echo "  $line"
done

echo ""
echo -e "${YELLOW}[4/4] Installing Virust CLI...${NC}"

# Ensure bin directory exists
mkdir -p "$BIN_DIR"

# Copy binary
cp "$INSTALL_DIR/target/release/virust" "$BIN_DIR/virust"
chmod +x "$BIN_DIR/virust"

# Clean up
rm -rf "$INSTALL_DIR"

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  ✓ Virust installed successfully!${NC}"
echo -e "${GREEN}========================================${NC}\n"

echo -e "${BLUE}Binary location:${NC} $BIN_DIR/virust"
echo ""
echo -e "${BLUE}Try it out:${NC}"
echo "  virust --version"
echo "  virust init my-project"
echo ""
echo -e "${BLUE}Or create a project with a template:${NC}"
echo "  virust init my-app --template todo"
echo ""

# Verify installation
if command -v virust &> /dev/null; then
    VERSION_OUTPUT=$(virust --version 2>&1 || echo "version info")
    echo -e "${GREEN}✓${NC} Installation verified: $VERSION_OUTPUT"
else
    echo -e "${YELLOW}⚠${NC}  Make sure \$HOME/.cargo/bin is in your PATH"
    echo -e "${YELLOW}  Add this to your ~/.bashrc or ~/.zshrc:${NC}"
    echo "  export PATH=\"\$HOME/.cargo/bin:\$PATH\""
fi
