#!/bin/bash

# WebTags Native Messaging Host Installer
# This script installs the native messaging host for Chrome and Firefox

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINARY_NAME="webtags-host"
INSTALL_DIR="/usr/local/bin"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "WebTags Native Messaging Host Installer"
echo "========================================"
echo ""

# Check if binary exists
if [ ! -f "$SCRIPT_DIR/target/release/$BINARY_NAME" ]; then
    echo -e "${RED}Error: Binary not found. Please build the project first:${NC}"
    echo "  cargo build --release"
    exit 1
fi

# Install binary
echo "Installing binary to $INSTALL_DIR..."
sudo cp "$SCRIPT_DIR/target/release/$BINARY_NAME" "$INSTALL_DIR/"
sudo chmod +x "$INSTALL_DIR/$BINARY_NAME"
echo -e "${GREEN}✓ Binary installed${NC}"

# Install Chrome manifest
CHROME_MANIFEST_DIR="$HOME/.config/google-chrome/NativeMessagingHosts"
CHROMIUM_MANIFEST_DIR="$HOME/.config/chromium/NativeMessagingHosts"

echo ""
echo "Installing Chrome/Chromium manifest..."

# Chrome
if [ -d "$HOME/.config/google-chrome" ]; then
    mkdir -p "$CHROME_MANIFEST_DIR"
    cp "$SCRIPT_DIR/manifests/chrome-manifest.json" "$CHROME_MANIFEST_DIR/com.webtags.host.json"
    echo -e "${GREEN}✓ Chrome manifest installed${NC}"
fi

# Chromium
if [ -d "$HOME/.config/chromium" ]; then
    mkdir -p "$CHROMIUM_MANIFEST_DIR"
    cp "$SCRIPT_DIR/manifests/chrome-manifest.json" "$CHROMIUM_MANIFEST_DIR/com.webtags.host.json"
    echo -e "${GREEN}✓ Chromium manifest installed${NC}"
fi

# Install Firefox manifest
FIREFOX_MANIFEST_DIR="$HOME/.mozilla/native-messaging-hosts"

echo ""
echo "Installing Firefox manifest..."

if [ -d "$HOME/.mozilla" ]; then
    mkdir -p "$FIREFOX_MANIFEST_DIR"
    cp "$SCRIPT_DIR/manifests/firefox-manifest.json" "$FIREFOX_MANIFEST_DIR/com.webtags.host.json"
    echo -e "${GREEN}✓ Firefox manifest installed${NC}"
fi

echo ""
echo -e "${GREEN}Installation complete!${NC}"
echo ""
echo "Next steps:"
echo "1. Load the WebTags extension in your browser:"
echo "   Chrome: chrome://extensions/ (enable Developer mode, Load unpacked)"
echo "   Firefox: about:debugging#/runtime/this-firefox (Load Temporary Add-on)"
echo ""
echo "2. Update the manifest with your extension ID:"
echo "   Chrome: $CHROME_MANIFEST_DIR/com.webtags.host.json"
echo "   Firefox: $FIREFOX_MANIFEST_DIR/com.webtags.host.json"
echo ""
echo "3. Restart your browser and open the extension"
echo ""
echo -e "${YELLOW}Note: You'll need to authenticate with GitHub on first use${NC}"
