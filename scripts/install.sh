#!/usr/bin/env bash
set -euo pipefail

# Standalone zero-Rust installer for Linux and macOS.
# Downloads the matching GitHub Release executable into ~/.local/bin/brahm.

VERSION="${BRAHM_VERSION:-latest}"
INSTALL_DIR="${BRAHM_INSTALL_DIR:-$HOME/.local/bin}"
mkdir -p "$INSTALL_DIR"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}_${ARCH}" in
  Linux_x86_64|Linux_amd64)   ASSET="brahm-linux-x86_64" ;;
  Darwin_x86_64)              ASSET="brahm-macos-x86_64" ;;
  Darwin_arm64|Darwin_aarch64) ASSET="brahm-macos-aarch64" ;;
  *)
    echo "Unsupported platform: ${OS} ${ARCH}. Build from source or use Docker." >&2
    exit 1
    ;;
esac

if [ "$VERSION" = "latest" ]; then
  BASE_URL="https://github.com/DDharm007/brahm/releases/latest/download"
else
  BASE_URL="https://github.com/DDharm007/brahm/releases/download/${VERSION}"
fi

TMP_BIN="$(mktemp)"
curl -fsSL "${BASE_URL}/${ASSET}" -o "$TMP_BIN"

if curl -fsSL "${BASE_URL}/SHA256SUMS.txt" -o "${TMP_BIN}.sums" 2>/dev/null; then
  EXPECTED="$(grep " ${ASSET}\$" "${TMP_BIN}.sums" | awk '{print $1}' | head -n1)"
  if [ -n "$EXPECTED" ]; then
    if command -v sha256sum >/dev/null 2>&1; then
      ACTUAL="$(sha256sum "$TMP_BIN" | awk '{print $1}')"
    else
      ACTUAL="$(shasum -a 256 "$TMP_BIN" | awk '{print $1}')"
    fi
    if [ "$EXPECTED" != "$ACTUAL" ]; then
      rm -f "$TMP_BIN" "${TMP_BIN}.sums"
      echo "SHA-256 verification failed: expected $EXPECTED, got $ACTUAL" >&2
      exit 1
    fi
    echo "[BRAHM] Verified SHA-256: $ACTUAL"
  fi
  rm -f "${TMP_BIN}.sums"
fi

chmod +x "$TMP_BIN"
mv "$TMP_BIN" "${INSTALL_DIR}/brahm"
echo "[BRAHM] Installed to ${INSTALL_DIR}/brahm"
"${INSTALL_DIR}/brahm" --version
echo "[BRAHM] Add ${INSTALL_DIR} to PATH if needed, then run: brahm apply /path/to/your-project --shield"
