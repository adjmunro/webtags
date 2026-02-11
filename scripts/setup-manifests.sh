#!/usr/bin/env bash
# WebTags Native Messaging Setup
# Automatically installs manifests for all detected browsers

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Find webtags-host binary
if command -v webtags-host &> /dev/null; then
    BINARY_PATH=$(which webtags-host)
elif [ -f "/opt/homebrew/bin/webtags-host" ]; then
    BINARY_PATH="/opt/homebrew/bin/webtags-host"
elif [ -f "/usr/local/bin/webtags-host" ]; then
    BINARY_PATH="/usr/local/bin/webtags-host"
else
    echo -e "${RED}✗ webtags-host not found. Please install with: brew install webtags${NC}"
    exit 1
fi

echo -e "${BLUE}WebTags Native Messaging Setup${NC}"
echo -e "Binary: ${BINARY_PATH}"
echo ""

# Chrome manifest template
create_chrome_manifest() {
    local path=$1
    cat > "$path" <<EOF
{
  "name": "com.webtags.host",
  "description": "WebTags Native Messaging Host",
  "path": "${BINARY_PATH}",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://EXTENSION_ID_HERE/"
  ]
}
EOF
}

# Firefox manifest template
create_firefox_manifest() {
    local path=$1
    cat > "$path" <<EOF
{
  "name": "com.webtags.host",
  "description": "WebTags Native Messaging Host",
  "path": "${BINARY_PATH}",
  "type": "stdio",
  "allowed_extensions": [
    "webtags@example.com"
  ]
}
EOF
}

install_manifest() {
    local name=$1
    local dir=$2
    local type=$3
    local parent_dir=$(dirname "$dir")

    if [ -d "$parent_dir" ]; then
        mkdir -p "$dir"
        if [ "$type" = "chrome" ]; then
            create_chrome_manifest "$dir/com.webtags.host.json"
        else
            create_firefox_manifest "$dir/com.webtags.host.json"
        fi
        echo -e "${GREEN}✓ Installed for ${name}${NC}"
        echo -e "  → ${dir}/com.webtags.host.json"
        return 0
    fi
    return 1
}

installed_count=0

# Chromium-based browsers
echo -e "${YELLOW}Checking Chromium-based browsers...${NC}"

install_manifest "Google Chrome" "$HOME/Library/Application Support/Google/Chrome/NativeMessagingHosts" "chrome" && ((installed_count++)) || true
install_manifest "Chromium" "$HOME/Library/Application Support/Chromium/NativeMessagingHosts" "chrome" && ((installed_count++)) || true
install_manifest "Microsoft Edge" "$HOME/Library/Application Support/Microsoft Edge/NativeMessagingHosts" "chrome" && ((installed_count++)) || true
install_manifest "Brave" "$HOME/Library/Application Support/BraveSoftware/Brave-Browser/NativeMessagingHosts" "chrome" && ((installed_count++)) || true

# Firefox-based browsers
echo ""
echo -e "${YELLOW}Checking Firefox-based browsers...${NC}"

install_manifest "Firefox" "$HOME/.mozilla/native-messaging-hosts" "firefox" && ((installed_count++)) || true
install_manifest "Zen Browser" "$HOME/Library/Application Support/zen/NativeMessagingHosts" "firefox" && ((installed_count++)) || true

# Safari (uses Mozilla path on macOS)
echo ""
echo -e "${YELLOW}Checking Safari...${NC}"
if [ -d "/Applications/Safari.app" ]; then
    safari_dir="$HOME/Library/Application Support/Mozilla/NativeMessagingHosts"
    mkdir -p "$safari_dir"
    create_chrome_manifest "$safari_dir/com.webtags.host.json"
    echo -e "${GREEN}✓ Installed for Safari${NC}"
    echo -e "  → ${safari_dir}/com.webtags.host.json"
    ((installed_count++))
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ $installed_count -eq 0 ]; then
    echo -e "${RED}✗ No supported browsers found${NC}"
    echo ""
    echo "Supported browsers:"
    echo "  - Chrome, Chromium, Edge, Brave (Chromium-based)"
    echo "  - Firefox, Zen Browser (Firefox-based)"
    echo "  - Safari"
    exit 1
else
    echo -e "${GREEN}✓ Installed manifests for ${installed_count} browser(s)${NC}"
fi

echo ""
echo -e "${BLUE}Next steps:${NC}"
echo "1. Install the WebTags browser extension"
echo "2. Open your browser and load the extension"
echo "3. The extension will automatically connect to the native host"
echo ""
echo -e "${YELLOW}Note:${NC} You'll need to update the manifest with your actual extension ID"
echo "after installing the extension. Check the extension's page for the ID."
