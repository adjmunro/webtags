# WebTags: Git-Synced Browser Bookmark Tagging Extension

WebTags transforms your browser bookmarks into a powerful tagging system with automatic Git/GitHub synchronization across devices and browsers. Take control of your bookmarks with hierarchical tags, full version history, and complete data sovereignty.

## ✨ Features

- **🏷️ True Tagging System**: Many-to-many relationships between bookmarks and tags, with hierarchical tag support (e.g., `tech/programming/rust`)
- **🔄 Cross-Device Sync**: Git-based synchronization works across all your devices and browsers
- **🔒 Data Sovereignty**: Your bookmarks live in your own GitHub repository - no third-party services
- **📝 Version Control**: Full Git history of all bookmark and tag changes with meaningful commit messages
- **🌐 Cross-Browser**: Works in Chrome, Firefox, and Safari (via conversion)
- **🔐 Privacy-Focused**: No tracking, no analytics, all data under your control
- **⚡ Native Performance**: Rust-based native messaging host for fast, reliable Git operations

## 🏗️ Architecture

```
┌─────────────────┐
│ Browser         │
│ Extension (TS)  │──┐
└─────────────────┘  │
                     │ Native Messaging
┌─────────────────┐  │ Protocol (JSON)
│ Native Host     │◄─┘
│ (Rust)          │
├─────────────────┤
│ • Git Operations│
│ • GitHub API    │
│ • JSON Storage  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Local Git Repo  │
│ bookmarks.json  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ GitHub Remote   │
│ (Private Repo)  │
└─────────────────┘
```

### Components

1. **Browser Extension (TypeScript)**
   - Manages browser bookmarks via Bookmarks API
   - Provides popup UI for tag management
   - Communicates with native host via Native Messaging Protocol

2. **Native Messaging Host (Rust)**
   - Handles Git operations (commit, push, pull)
   - Manages GitHub authentication (OAuth Device Flow)
   - Reads/writes JSON API v1.1 compliant bookmark data
   - Secure token storage in OS keychain

3. **Data Format (JSON API v1.1)**
   - Standardized bookmark and tag representation
   - Hierarchical tag relationships
   - Human-readable and version-control friendly

## 📦 Installation

### Prerequisites

- **Rust**: 1.70 or higher ([install](https://rustup.rs/))
- **Node.js**: 18 or higher ([install](https://nodejs.org/))
- **Git**: For repository management
- **GitHub Account**: For remote sync (free account works)

### Quick Start

#### 1. Clone and Build

```bash
# Clone the repository
git clone https://github.com/yourusername/webtags.git
cd webtags

# Build the native messaging host
cd native-host
cargo build --release
cargo test  # Run tests to verify

# Build the browser extension
cd ../extension
npm install
npm run build
```

#### 2. Install Native Messaging Host

```bash
cd ../native-host
./install.sh
```

This installs:
- Binary to `/usr/local/bin/webtags-host`
- Chrome manifest to `~/.config/google-chrome/NativeMessagingHosts/`
- Firefox manifest to `~/.mozilla/native-messaging-hosts/`

#### 3. Load Browser Extension

**Chrome/Chromium:**
1. Open `chrome://extensions/`
2. Enable "Developer mode"
3. Click "Load unpacked"
4. Select the `extension/dist/` directory
5. Copy the extension ID from the loaded extension

**Firefox:**
1. Open `about:debugging#/runtime/this-firefox`
2. Click "Load Temporary Add-on"
3. Select `extension/dist/manifest.json`

#### 4. Update Native Messaging Manifest

Update the manifest with your extension ID:

**Chrome:** Edit `~/.config/google-chrome/NativeMessagingHosts/com.webtags.host.json`
```json
{
  "allowed_origins": [
    "chrome-extension://YOUR_EXTENSION_ID_HERE/"
  ]
}
```

**Firefox:** Edit `~/.mozilla/native-messaging-hosts/com.webtags.host.json`
```json
{
  "allowed_extensions": [
    "YOUR_EXTENSION_ID_HERE@example.com"
  ]
}
```

#### 5. Initial Setup

1. Click the WebTags extension icon
2. Choose "Create New Repository" or "Clone Existing Repository"
3. Authenticate with GitHub (OAuth Device Flow)
4. Your bookmarks will be synced automatically!

## 🚀 Usage

### Adding Tags to Bookmarks

Use `#tag` syntax in bookmark titles:
```
My Website #programming #rust #web
```

Tags are automatically extracted and organized hierarchically in the extension popup.

### Hierarchical Tags

Create tag hierarchies by using the tag management UI:
- `tech` → `programming` → `rust`
- Breadcrumb display: `tech/programming/rust`
- Organize related tags together

### Syncing

- **Automatic**: Syncs every hour and on bookmark changes
- **Manual**: Click the sync button in the popup
- **Multi-device**: Changes propagate via Git push/pull

### Searching

Use the search bar to filter by:
- Bookmark title
- URL
- Tags

## 📊 Data Format

Bookmarks are stored in `bookmarks.json` following JSON API v1.1:

```json
{
  "jsonapi": { "version": "1.1" },
  "data": [
    {
      "type": "bookmark",
      "id": "abc123",
      "attributes": {
        "url": "https://rust-lang.org",
        "title": "Rust Programming Language",
        "created": "2024-01-15T10:30:00Z"
      },
      "relationships": {
        "tags": {
          "data": [
            { "type": "tag", "id": "tag-rust" }
          ]
        }
      }
    }
  ],
  "included": [
    {
      "type": "tag",
      "id": "tag-rust",
      "attributes": {
        "name": "rust",
        "color": "#3b82f6"
      },
      "relationships": {
        "parent": {
          "data": { "type": "tag", "id": "tag-programming" }
        }
      }
    }
  ]
}
```

## 🧪 Development

### Running Tests

**Rust (Native Host):**
```bash
cd native-host
cargo test
cargo test -- --nocapture  # Show output
```

**TypeScript (Extension):**
```bash
cd extension
npm test
npm run type-check
npm run lint
```

### Project Structure

```
webtags/
├── extension/               # Browser extension (TypeScript)
│   ├── src/
│   │   ├── background.ts   # Service worker
│   │   ├── messaging.ts    # Native messaging client
│   │   ├── bookmarkConverter.ts  # Format conversion
│   │   ├── types.ts        # TypeScript types
│   │   └── popup/          # Popup UI
│   └── dist/               # Built extension
├── native-host/            # Native messaging host (Rust)
│   ├── src/
│   │   ├── main.rs         # Entry point
│   │   ├── messaging.rs    # Native messaging protocol
│   │   ├── git.rs          # Git operations
│   │   ├── github.rs       # GitHub API
│   │   └── storage.rs      # JSON storage
│   ├── manifests/          # Native messaging manifests
│   └── install.sh          # Installation script
└── schemas/                # JSON API v1.1 schema
```

### TDD Approach

This project follows Test-Driven Development:

1. **Write tests first**: Define expected behavior
2. **Implement to pass tests**: Write minimal code
3. **Refactor**: Improve code while keeping tests passing

Run tests frequently during development.

## 🔧 Configuration

### GitHub Authentication

Two methods supported:

1. **OAuth Device Flow** (recommended):
   - Click "Authenticate with GitHub" in setup
   - Enter code on GitHub
   - Token stored securely in OS keychain

2. **Personal Access Token**:
   - Generate at github.com/settings/tokens
   - Permissions: `repo` (full control of private repositories)
   - Paste into extension settings

### Repository Settings

Default repository location: `~/.local/share/webtags/`

Customize via:
```bash
webtags-host --repo-path /custom/path
```

## 🐛 Troubleshooting

### Extension Can't Connect to Native Host

1. Verify binary is installed:
   ```bash
   which webtags-host
   ```

2. Check manifest paths:
   - Chrome: `~/.config/google-chrome/NativeMessagingHosts/com.webtags.host.json`
   - Firefox: `~/.mozilla/native-messaging-hosts/com.webtags.host.json`

3. Verify extension ID in manifest

4. Restart browser completely

### Sync Conflicts

Current strategy: prefer remote changes (will be configurable in future)

To reset local state:
```bash
cd ~/.local/share/webtags
git reset --hard origin/main
```

### Authentication Issues

Re-authenticate:
1. Open extension popup
2. Go to Settings
3. Click "Re-authenticate"
4. Follow OAuth flow

Or manually set token:
```bash
# Using keyring
keyring set com.webtags.github github_token
```

## 🗺️ Roadmap

- [x] Core tagging functionality
- [x] Git synchronization
- [x] GitHub OAuth
- [x] Hierarchical tags
- [x] Chrome support
- [ ] Firefox testing and refinement
- [ ] Safari support (via web extension converter)
- [ ] Conflict resolution UI
- [ ] Full-text search
- [ ] Tag suggestions (AI-powered)
- [ ] Shared repositories (team collaboration)
- [ ] Bookmark import (HTML, CSV)
- [ ] Export formats (Markdown, Notion)
- [ ] Homebrew tap for easy installation

## 🤝 Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Write tests for new features
4. Ensure all tests pass
5. Submit a pull request

## 📄 License

MIT License - see LICENSE file for details

## 🙏 Acknowledgments

- Built with [Rust](https://rust-lang.org/) and [TypeScript](https://www.typescriptlang.org/)
- Uses [git2](https://github.com/rust-lang/git2-rs) for Git operations
- Follows [JSON API v1.1](https://jsonapi.org/) specification
- Inspired by the need for better bookmark management

## 📞 Support

- Issues: [GitHub Issues](https://github.com/yourusername/webtags/issues)
- Discussions: [GitHub Discussions](https://github.com/yourusername/webtags/discussions)

---

**WebTags** - Your bookmarks, your way, under your control.
