#!/bin/sh
set -e

VERSION="v1.0.0"
REPO="juniorsir/vessel"

# 1. Detect if running inside Android Termux environment
if [ -d "/data/data/com.termux/files/usr" ] || [ -n "$TERMUX_VERSION" ]; then
    echo "\x1b[1m\x1b[33m[vessel-installer]\x1b[0m Android Termux environment detected!"
    echo "\x1b[1m\x1b[36mWe are actively building native PRoot compatibility for Termux soon. Please stay tuned!\x1b[0m\n"
    exit 0
fi

# 2. Native Linux installation path (for standard hosts)
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)
INSTALL_DIR="/usr/local/bin"

if [ "$OS" != "linux" ]; then
    echo "Error: vessel currently only supports Linux architectures."
    exit 1
fi

case "$ARCH" in
    x86_64)  ASSET="vessel-linux-amd64" ;;
    aarch64) ASSET="vessel-linux-arm64" ;;
    *)
        echo "Error: Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

URL="https://github.com/$REPO/releases/download/$VERSION/$ASSET"

echo "\x1b[1m\x1b[36m[vessel-installer]\x1b[0m Downloading pre-compiled, stripped binary..."
TEMP_FILE=$(mktemp)
curl -L -o "$TEMP_FILE" "$URL"

echo "\x1b[1m\x1b[36m[vessel-installer]\x1b[0m Installing binary to $INSTALL_DIR/vessel (requires sudo)..."
sudo mv "$TEMP_FILE" "$INSTALL_DIR/vessel"
sudo chmod +x "$INSTALL_DIR/vessel"

echo "\x1b[1m\x1b[32m✔ vessel installed successfully to $INSTALL_DIR/vessel\x1b[0m\n"
