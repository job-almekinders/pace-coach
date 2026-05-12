#!/bin/sh
set -e

REPO="job-almekinders/pace-coach"
INSTALL_DIR="/usr/local/bin"

echo "==> Checking system requirements..."

if [ "$(uname)" != "Darwin" ]; then
    echo "Error: pace-coach only supports macOS." >&2
    exit 1
fi
echo "    OS: macOS $(sw_vers -productVersion)"

if [ "$(uname -m)" != "arm64" ]; then
    echo "Error: pace-coach only supports Apple Silicon (arm64)." >&2
    exit 1
fi
echo "    Arch: arm64 (Apple Silicon)"

echo "==> Fetching latest release..."
VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" |
    grep '"tag_name"' | sed 's/.*"tag_name": *"v\([^"]*\)".*/\1/')

if [ -z "$VERSION" ]; then
    echo "Error: could not determine latest version." >&2
    exit 1
fi
echo "    Latest version: v${VERSION}"

TARBALL="pace-coach-${VERSION}-aarch64-apple-darwin.tar.gz"
URL="https://github.com/${REPO}/releases/download/v${VERSION}/${TARBALL}"
TMP=$(mktemp -d)
echo "    Temp dir: ${TMP}"

echo "==> Downloading ${TARBALL}..."
curl -fsSL --progress-bar "$URL" -o "${TMP}/${TARBALL}"
echo "==> Extracting..."
tar -xzf "${TMP}/${TARBALL}" -C "$TMP"

echo "==> Installing binaries to ${INSTALL_DIR} (may require sudo)..."
sudo install -m 755 "${TMP}/pace-coach" "${INSTALL_DIR}/pace-coach"
echo "    Installed: ${INSTALL_DIR}/pace-coach"
sudo install -m 755 "${TMP}/pace-coach-menubar" "${INSTALL_DIR}/pace-coach-menubar"
echo "    Installed: ${INSTALL_DIR}/pace-coach-menubar"

echo "==> Cleaning up..."
rm -rf "$TMP"

echo ""
echo "Done! pace-coach v${VERSION} installed."
echo "Run 'pace-coach start' to begin."
