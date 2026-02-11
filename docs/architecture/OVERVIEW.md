# Architecture Overview

## System Design

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

## Components

### 1. Browser Extension (TypeScript)

**Purpose**: Manage browser bookmarks and provide UI

**Key Modules**:
- `messaging.ts`: Native messaging client with auto-reconnect
- `bookmarkConverter.ts`: Convert Chrome ↔ JSON API v1.1
- `background.ts`: Service worker, event listeners
- `popup.ts`: UI controller for popup

**Responsibilities**:
- Listen for bookmark changes
- Extract tags from titles (#tag syntax)
- Communicate with native host
- Provide user interface

### 2. Native Messaging Host (Rust)

**Purpose**: Handle Git operations and GitHub integration

**Key Modules**:
- `messaging.rs`: Native messaging protocol (4-byte length prefix)
- `storage.rs`: JSON API v1.1 bookmark storage
- `git.rs`: Git operations (using git2 crate)
- `github.rs`: GitHub API and OAuth Device Flow

**Responsibilities**:
- Git commit, push, pull
- GitHub authentication
- Secure token storage (OS keychain)
- JSON file management

### 3. Data Format (JSON API v1.1)

**Purpose**: Standardized bookmark representation

**Structure**:
```json
{
  "jsonapi": {"version": "1.1"},
  "data": [
    {
      "type": "bookmark",
      "id": "abc123",
      "attributes": {...},
      "relationships": {
        "tags": {"data": [...]}
      }
    }
  ],
  "included": [
    {
      "type": "tag",
      "id": "tag-rust",
      "attributes": {...},
      "relationships": {...}
    }
  ]
}
```

## Communication Flow

### Message Protocol

**Format**: 4-byte length prefix + JSON message

```
[uint32 length][JSON payload]
```

**Message Types**:
1. `init` - Initialize repository
2. `write` - Save bookmarks
3. `read` - Load bookmarks
4. `sync` - Push/pull from remote
5. `auth` - Authenticate with GitHub
6. `status` - Get repository status

### Example Flow: Create Bookmark

```
1. User creates bookmark in browser
   ↓
2. Extension detects change event
   ↓
3. Convert to JSON API format
   ↓
4. Send "write" message to native host
   ↓
5. Native host:
   - Write to bookmarks.json
   - Git add and commit
   ↓
6. Return success response
   ↓
7. Extension updates UI
```

## Design Decisions

### Why Rust for Native Host?

- **Performance**: Fast Git operations, JSON parsing
- **Safety**: Memory safety without GC
- **Cross-platform**: Works on macOS, Linux, Windows
- **Reliability**: Strong type system catches bugs at compile time

### Why JSON API v1.1?

- **Standardized**: Well-defined specification
- **Relationships**: First-class many-to-many support
- **Extensible**: Easy to add new resource types
- **Human-readable**: Git-friendly format

### Why Git for Sync?

- **Version Control**: Full history of changes
- **Distributed**: Works offline, sync when online
- **Conflict Resolution**: Battle-tested merge strategies
- **Universal**: Git already installed on most systems

## Module Dependencies

### Native Host (Rust)

```rust
Dependencies {
    git2: "Git operations",
    serde: "JSON serialization",
    keyring: "Secure token storage",
    reqwest: "HTTP client for GitHub API",
    tokio: "Async runtime",
}
```

### Extension (TypeScript)

```typescript
Dependencies {
    webpack: "Build tool",
    typescript: "Type safety",
    jest: "Testing framework",
}
```

## Security Architecture

### Token Storage

```
GitHub Token
    ↓
OS Keychain (macOS: Keychain, Linux: Secret Service, Windows: Credential Manager)
    ↓
Never stored in:
- Filesystem
- Git repository
- Browser storage
```

### Data Flow Security

1. **Extension → Native Host**: Native messaging protocol (Chrome/Firefox sandboxed)
2. **Native Host → GitHub**: HTTPS with OAuth tokens
3. **Git Operations**: SSH keys or HTTPS with tokens
4. **Local Storage**: Atomic writes, proper permissions

## Error Handling

### Strategy

1. **Extension**: Try/catch with user-friendly messages
2. **Native Host**: Result<T, Error> with context
3. **Git Operations**: Graceful degradation (offline mode)
4. **Network Errors**: Retry with exponential backoff

### Error Propagation

```rust
// Native host
fn operation() -> Result<Response, Box<dyn Error>> {
    let data = read_file()?; // Propagate error
    let result = process(data)?;
    Ok(Response::success(result))
}
```

```typescript
// Extension
try {
    await nativeHost.sync();
    showSuccess("Synced successfully");
} catch (error) {
    showError(`Sync failed: ${error.message}`);
    logError(error);
}
```

## Performance Considerations

- **JSON Parsing**: Streaming for large files
- **Git Operations**: Batch commits when possible
- **Tag Hierarchy**: Cache computed hierarchies
- **Search**: Consider indexing for 1000+ bookmarks

## Extensibility

### Adding New Message Types

1. Define in `messaging.rs` enum
2. Add handler in `main.rs`
3. Update TypeScript types
4. Add client method

### Adding New Resource Types

1. Update JSON schema
2. Add Rust/TypeScript types
3. Update converter logic
4. Write tests

## Testing Architecture

- **Unit Tests**: Test individual functions/modules
- **Integration Tests**: Test message flows
- **Manual Tests**: End-to-end user flows

See [TESTING.md](../workflows/TESTING.md) for details.
