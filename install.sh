#!/bin/sh
set -e

REPO="theodaguier/cswitch"
INSTALL_DIR="${CSWITCH_INSTALL_DIR:-/usr/local/bin}"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)  os="unknown-linux-gnu" ;;
  Darwin) os="apple-darwin" ;;
  *)      echo "Error: Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
  x86_64)  arch="x86_64" ;;
  aarch64) arch="aarch64" ;;
  arm64)   arch="aarch64" ;;
  *)       echo "Error: Unsupported architecture: $ARCH"; exit 1 ;;
esac

TARGET="${arch}-${os}"

# Get latest release tag
LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST" ]; then
  echo "Error: Could not determine latest release"
  exit 1
fi

URL="https://github.com/${REPO}/releases/download/${LATEST}/cswitch-${TARGET}.tar.gz"

echo "Downloading cswitch ${LATEST} for ${TARGET}..."

TMPDIR=$(mktemp -d)
curl -fsSL "$URL" -o "${TMPDIR}/cswitch.tar.gz"
tar -xzf "${TMPDIR}/cswitch.tar.gz" -C "${TMPDIR}"

echo "Installing to ${INSTALL_DIR}/cswitch..."
sudo install -m 755 "${TMPDIR}/cswitch" "${INSTALL_DIR}/cswitch"

rm -rf "$TMPDIR"

echo "✓ cswitch ${LATEST} installed successfully"
echo "  Run 'cswitch --help' to get started"
