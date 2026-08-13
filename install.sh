#!/bin/sh
# Installs the latest sklint binary for your platform.
# Usage: curl -fsSL https://raw.githubusercontent.com/markus-azer/sklint/main/install.sh | sh
set -eu

REPO="markus-azer/sklint"
BIN="sklint"
INSTALL_DIR="${SKLINT_INSTALL_DIR:-$HOME/.local/bin}"

os=$(uname -s)
case "$os" in
  Linux) os=linux ;;
  Darwin) os=darwin ;;
  *) echo "sklint: unsupported OS '$os'. Use npm or a prebuilt binary from the releases page." >&2; exit 1 ;;
esac

arch=$(uname -m)
case "$arch" in
  x86_64 | amd64) arch=x64 ;;
  arm64 | aarch64) arch=arm64 ;;
  *) echo "sklint: unsupported architecture '$arch'." >&2; exit 1 ;;
esac

asset="${BIN}-${os}-${arch}"
base="https://github.com/${REPO}/releases/latest/download"

download() { # url dest
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$1" -o "$2"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "$2" "$1"
  else
    echo "sklint: need curl or wget to download." >&2; exit 1
  fi
}

echo "Downloading ${asset}..."
tmp=$(mktemp)
sums=$(mktemp)
trap 'rm -f "$tmp" "$sums"' EXIT
download "${base}/${asset}" "$tmp"
download "${base}/SHA256SUMS" "$sums"

expected=$(grep " ${asset}\$" "$sums" | awk '{print $1}')
[ -n "$expected" ] || { echo "sklint: no checksum found for ${asset}" >&2; exit 1; }
if command -v sha256sum >/dev/null 2>&1; then
  actual=$(sha256sum "$tmp" | awk '{print $1}')
else
  actual=$(shasum -a 256 "$tmp" | awk '{print $1}')
fi
[ "$expected" = "$actual" ] || { echo "sklint: checksum mismatch for ${asset}" >&2; exit 1; }

chmod +x "$tmp"
mkdir -p "$INSTALL_DIR"
mv "$tmp" "$INSTALL_DIR/$BIN"
echo "Installed sklint to $INSTALL_DIR/$BIN"

case ":$PATH:" in
  *":$INSTALL_DIR:"*) echo "Run: sklint --help" ;;
  *) echo "Add it to your PATH:"; echo "  export PATH=\"$INSTALL_DIR:\$PATH\"" ;;
esac
