#!/usr/bin/env bash
set -euo pipefail

REPO="${REPO:-besingamkb/grit-msg}"
BINARY_NAME="grit-msg"
BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"
VERSION="${VERSION:-latest}"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux) platform="unknown-linux-gnu" ;;
  Darwin) platform="apple-darwin" ;;
  *)
    echo "Unsupported OS: $OS"
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64|amd64) target_arch="x86_64" ;;
  arm64|aarch64) target_arch="aarch64" ;;
  *)
    echo "Unsupported architecture: $ARCH"
    exit 1
    ;;
esac

TARGET="${target_arch}-${platform}"
ARCHIVE="${BINARY_NAME}-${VERSION}-${TARGET}.tar.gz"
CHECKSUM="${ARCHIVE}.sha256"

if [[ "$VERSION" == "latest" ]]; then
  BASE_URL="https://github.com/${REPO}/releases/latest/download"
else
  BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

echo "Downloading ${ARCHIVE}..."
curl -fsSL "${BASE_URL}/${ARCHIVE}" -o "${TMP_DIR}/${ARCHIVE}"
curl -fsSL "${BASE_URL}/${CHECKSUM}" -o "${TMP_DIR}/${CHECKSUM}"

echo "Verifying checksum..."
if command -v sha256sum >/dev/null 2>&1; then
  (cd "$TMP_DIR" && sha256sum -c "$CHECKSUM")
elif command -v shasum >/dev/null 2>&1; then
  expected="$(awk '{print $1}' "${TMP_DIR}/${CHECKSUM}")"
  actual="$(shasum -a 256 "${TMP_DIR}/${ARCHIVE}" | awk '{print $1}')"
  if [[ "$expected" != "$actual" ]]; then
    echo "Checksum mismatch."
    exit 1
  fi
else
  echo "No SHA256 tool found (need sha256sum or shasum)."
  exit 1
fi

mkdir -p "$BIN_DIR"
tar -xzf "${TMP_DIR}/${ARCHIVE}" -C "$TMP_DIR"
install -m 0755 "${TMP_DIR}/${BINARY_NAME}" "${BIN_DIR}/${BINARY_NAME}"

echo "Installed ${BINARY_NAME} to ${BIN_DIR}/${BINARY_NAME}"
echo "Run: ${BINARY_NAME} --help"
