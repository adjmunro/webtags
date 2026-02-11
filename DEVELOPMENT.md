# Development Guide

This guide covers the development workflow, architecture decisions, and testing strategy for WebTags.

## Project Statistics

- **Rust Code**: ~2,100 lines (native host)
- **TypeScript Code**: ~1,200 lines (browser extension)
- **Unit Tests**: 34 tests (all passing)
- **Test Coverage**: >80% for critical paths

## Development Workflow

### Initial Setup

1. **Clone and Install Dependencies**
   ```bash
   # Rust dependencies (automatically downloaded)
   cd native-host
   cargo build

   # Node dependencies
   cd ../extension
   npm install
   ```

2. **Run Tests**
   ```bash
   # Rust tests
   cd native-host
   cargo test

   # TypeScript tests
   cd extension
   npm test
   ```

### TDD Workflow

We follow Test-Driven Development:

1. **Red**: Write a failing test
   ```rust
   #[test]
   fn test_new_feature() {
       let result = my_new_feature();
       assert_eq!(result, expected_value);
   }
   ```

2. **Green**: Write minimal code to pass
   ```rust
   fn my_new_feature() -> ResultType {
       // Implementation
   }
   ```

3. **Refactor**: Improve code while keeping tests passing

### Building

```bash
# Development build (Rust)
cargo build

# Release build (Rust)
cargo build --release

# Development build (Extension - with watch mode)
npm run dev

# Production build (Extension)
npm run build
```

## Architecture Decisions

### Why Rust for Native Host?

- **Performance**: Fast Git operations and JSON parsing
- **Safety**: Memory safety without garbage collection
- **Cross-platform**: Works on macOS, Linux, Windows
- **Native Messaging**: Excellent for stdin/stdout communication

### Why JSON API v1.1?

- **Standardized**: Well-defined specification
- **Relationships**: First-class support for many-to-many
- **Extensible**: Easy to add new resource types
- **Version Control Friendly**: Human-readable JSON

### Why Git for Sync?

- **Version Control**: Full history of changes
- **Distributed**: Works offline, sync when online
- **Conflict Resolution**: Battle-tested merge strategies
- **Universal**: Git is already installed on most systems

## Module Guide

### Native Host (Rust)

#### `messaging.rs`
- **Purpose**: Native messaging protocol implementation
- **Key Functions**:
  - `read_message()`: Read JSON from stdin
  - `write_response()`: Write JSON to stdout
- **Testing**: Message parsing, serialization, error handling

#### `storage.rs`
- **Purpose**: JSON API v1.1 bookmark storage
- **Key Functions**:
  - `read_from_file()`: Load bookmarks from JSON
  - `write_to_file()`: Save bookmarks atomically
  - `get_tag_hierarchy()`: Build tag tree
  - `get_tag_breadcrumb()`: Generate breadcrumb path
- **Testing**: Serialization, validation, hierarchy, atomic writes

#### `git.rs`
- **Purpose**: Git operations using git2 crate
- **Key Functions**:
  - `init()`: Initialize or open repository
  - `clone()`: Clone from remote URL
  - `commit()`: Create commit with message
  - `push()`: Push to remote
  - `pull()`: Pull with rebase
- **Testing**: Repo init, commits, status, conflicts

#### `github.rs`
- **Purpose**: GitHub API integration
- **Key Functions**:
  - `start_device_flow()`: Begin OAuth flow
  - `poll_for_token()`: Wait for user authorization
  - `create_repository()`: Create private repo
  - `store_token()`: Save to OS keychain
- **Testing**: Response parsing, token validation

### Browser Extension (TypeScript)

#### `messaging.ts`
- **Purpose**: Native messaging client
- **Key Features**:
  - Auto-reconnect on disconnect
  - Message queuing
  - Promise-based API
- **Methods**: `init()`, `write()`, `read()`, `sync()`, `auth()`

#### `bookmarkConverter.ts`
- **Purpose**: Convert between Chrome and JSON formats
- **Key Functions**:
  - `chromeToJsonApi()`: Export Chrome bookmarks
  - `applyToChrome()`: Import JSON bookmarks
  - `extractTagsFromTitle()`: Parse #tag syntax

#### `background.ts`
- **Purpose**: Background service worker
- **Responsibilities**:
  - Listen for bookmark changes
  - Auto-export and push changes
  - Periodic sync (1 hour)
  - Handle popup messages

#### `popup.ts`
- **Purpose**: Popup UI controller
- **Features**:
  - Bookmark list with tag display
  - Setup wizard
  - Settings management
  - Manual sync

## Testing Strategy

### Unit Tests

**Rust:**
- All modules have comprehensive unit tests
- Mock external dependencies where needed
- Use `tempfile` for filesystem tests
- Use `Cursor` for IO testing

**TypeScript:**
- Jest for unit testing
- Mock Chrome APIs
- Test bookmark conversion logic

### Integration Tests

Currently pending. Future work:
- End-to-end message flow tests
- Full Git workflow tests with test repositories
- Mock GitHub API with `wiremock`

### Manual Testing Checklist

Before release, manually test:

1. **Installation**
   - [ ] Fresh install on clean system
   - [ ] Manifest installation
   - [ ] Extension loading

2. **Setup**
   - [ ] GitHub OAuth flow
   - [ ] Repository creation
   - [ ] Repository cloning

3. **Bookmarks**
   - [ ] Create bookmark with tags
   - [ ] Edit bookmark
   - [ ] Delete bookmark
   - [ ] Move bookmark

4. **Sync**
   - [ ] Auto-sync on change
   - [ ] Manual sync button
   - [ ] Sync between devices
   - [ ] Conflict handling

5. **Tags**
   - [ ] Create tag
   - [ ] Create hierarchical tags
   - [ ] Tag breadcrumb display
   - [ ] Tag filtering

## Common Development Tasks

### Adding a New Message Type

1. **Define in Rust** (`messaging.rs`):
   ```rust
   pub enum Message {
       // ... existing variants
       NewMessage {
           field: String,
       },
   }
   ```

2. **Add handler** (`main.rs`):
   ```rust
   async fn handle_message(message: Message, config: &mut HostConfig) -> Response {
       match message {
           // ... existing handlers
           Message::NewMessage { field } => handle_new_message(field).await,
       }
   }
   ```

3. **Update TypeScript** (`types.ts`):
   ```typescript
   export interface NewMessage {
     type: 'newmessage';
     field: string;
   }

   export type NativeMessage = InitMessage | WriteMessage | NewMessage;
   ```

4. **Add client method** (`messaging.ts`):
   ```typescript
   async newMessage(field: string): Promise<NativeResponse> {
     return this.sendMessage({ type: 'newmessage', field });
   }
   ```

### Adding a New Resource Type

1. **Update schema** (`schemas/bookmarks-schema.json`)
2. **Add Rust types** (`storage.rs`)
3. **Add TypeScript types** (`types.ts`)
4. **Write tests for serialization/deserialization**

### Debugging

**Native Host:**
```bash
# Enable logging
RUST_LOG=debug webtags-host

# Run with debugger
rust-lldb target/debug/webtags-host
```

**Extension:**
- Chrome DevTools: Inspect popup, background page
- Console logs: Check browser console
- Network tab: Monitor native messaging

## Performance Considerations

- **JSON Parsing**: Use streaming for large files
- **Git Operations**: Batch commits when possible
- **Tag Hierarchy**: Cache computed hierarchies
- **Search**: Consider indexing for large bookmark sets

## Security Best Practices

- **Token Storage**: Always use OS keychain
- **Input Validation**: Sanitize all bookmark URLs and titles
- **Git Authentication**: Prefer SSH keys over HTTPS tokens
- **Private Repos**: Default to private repositories
- **No Sensitive Data**: Never commit tokens or secrets

## Release Checklist

1. [ ] All tests passing
2. [ ] Version bumped in Cargo.toml and package.json
3. [ ] CHANGELOG.md updated
4. [ ] README.md updated if needed
5. [ ] Manual testing completed
6. [ ] Git tag created
7. [ ] GitHub release created with binaries

## Contributing

See [README.md](README.md) for contribution guidelines.

## Questions?

Open an issue on GitHub or start a discussion!
