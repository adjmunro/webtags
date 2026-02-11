# Browser Setup Guide

## Quick Setup (Recommended)

After installing with Homebrew, run the automated setup script:

```bash
brew install webtags
curl -sSL https://raw.githubusercontent.com/adjmunro/webtags/master/scripts/setup-manifests.sh | bash
```

This automatically detects and configures all installed browsers!

## Supported Browsers

### ✅ Chromium-Based
- **Google Chrome**
- **Microsoft Edge**
- **Brave**
- **Chromium** (including ungoogled-chromium variants like Helium)
- **Arc Browser**
- Any other Chromium-based browser

### ✅ Firefox-Based
- **Firefox**
- **Zen Browser**
- **Waterfox**
- **LibreWolf**
- Any other Firefox-based browser

### ✅ Safari
- **Safari** (macOS only)

## Manual Setup

If you prefer manual setup or need to customize:

### Chromium-Based Browsers

1. Find your browser's manifest directory:

   ```bash
   # Google Chrome
   ~/Library/Application Support/Google/Chrome/NativeMessagingHosts/

   # Microsoft Edge
   ~/Library/Application Support/Microsoft Edge/NativeMessagingHosts/

   # Chromium / Helium
   ~/Library/Application Support/Chromium/NativeMessagingHosts/

   # Brave
   ~/Library/Application Support/BraveSoftware/Brave-Browser/NativeMessagingHosts/
   ```

2. Create `com.webtags.host.json` in that directory:

   ```json
   {
     "name": "com.webtags.host",
     "description": "WebTags Native Messaging Host",
     "path": "/opt/homebrew/bin/webtags-host",
     "type": "stdio",
     "allowed_origins": [
       "chrome-extension://YOUR_EXTENSION_ID/"
     ]
   }
   ```

### Firefox-Based Browsers

1. Find your browser's manifest directory:

   ```bash
   # Firefox
   ~/.mozilla/native-messaging-hosts/

   # Zen Browser
   ~/Library/Application Support/zen/NativeMessagingHosts/
   ```

2. Create `com.webtags.host.json` in that directory:

   ```json
   {
     "name": "com.webtags.host",
     "description": "WebTags Native Messaging Host",
     "path": "/opt/homebrew/bin/webtags-host",
     "type": "stdio",
     "allowed_extensions": [
       "webtags@example.com"
     ]
   }
   ```

### Safari

1. Create manifest directory:

   ```bash
   mkdir -p ~/Library/Application\ Support/Mozilla/NativeMessagingHosts/
   ```

2. Create `com.webtags.host.json` (same format as Chromium):

   ```json
   {
     "name": "com.webtags.host",
     "description": "WebTags Native Messaging Host",
     "path": "/opt/homebrew/bin/webtags-host",
     "type": "stdio",
     "allowed_origins": [
       "YOUR_SAFARI_EXTENSION_ID"
     ]
   }
   ```

## Finding Your Extension ID

After installing the extension in your browser:

### Chrome/Edge/Brave
1. Go to `chrome://extensions`
2. Enable "Developer mode" (top right)
3. Find WebTags and copy the ID

### Firefox/Zen
1. Go to `about:debugging#/runtime/this-firefox`
2. Find WebTags and copy the Extension ID

### Safari
1. Safari → Settings → Extensions
2. Right-click on WebTags → Show Extension ID

## Troubleshooting

### Extension can't connect to native host

1. **Check binary exists:**
   ```bash
   which webtags-host
   # Should show: /opt/homebrew/bin/webtags-host
   ```

2. **Check manifest exists:**
   ```bash
   # Example for Chrome:
   cat ~/Library/Application\ Support/Google/Chrome/NativeMessagingHosts/com.webtags.host.json
   ```

3. **Test native host manually:**
   ```bash
   echo '{"type":"status"}' | webtags-host
   # Should respond with JSON
   ```

4. **Verify extension ID matches:**
   - The manifest must have your actual extension ID
   - Update the manifest if you reinstalled the extension

### Permission denied

The binary must be executable:
```bash
chmod +x /opt/homebrew/bin/webtags-host
```

### Wrong path in manifest

If you installed to a different location:
```bash
# Find the actual path
which webtags-host

# Update manifest with correct path
```

## For Non-Technical Users

**Simplest setup:**

1. Install:
   ```bash
   brew install webtags
   ```

2. Run setup:
   ```bash
   curl -sSL https://raw.githubusercontent.com/adjmunro/webtags/master/scripts/setup-manifests.sh | bash
   ```

3. Install browser extension (link TBD)

4. Done! ✅

The setup script handles everything automatically - it detects which browsers you have and configures them all at once.
