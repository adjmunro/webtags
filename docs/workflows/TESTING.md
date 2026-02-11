# Testing Strategy

## Test-Driven Development (TDD)

We follow strict TDD:

1. **Red**: Write a failing test
2. **Green**: Write minimal code to pass
3. **Refactor**: Improve code while keeping tests passing

## Rust Tests

### Unit Tests (34 passing)

#### messaging.rs (10 tests)
- Message parsing for all types
- Response serialization
- Error handling (invalid JSON, oversized messages)
- Edge cases

#### storage.rs (15 tests)
- JSON API v1.1 serialization
- Hierarchical tag structures
- Tag breadcrumb generation
- Circular reference detection
- Atomic file writes
- Validation

#### git.rs (9 tests)
- Repository initialization
- Commit workflows
- Status checking
- Conflict scenarios

### Running Tests

```bash
cd native-host

# All tests
cargo test

# Specific module
cargo test storage::tests

# With output
cargo test -- --nocapture

# Release mode
cargo test --release
```

### Test Helpers

Use `tempfile` for filesystem tests:
```rust
use tempfile::TempDir;

#[test]
fn test_file_operation() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.json");
    // ... test code
}
```

## TypeScript Tests

### Setup (Jest)

```bash
cd extension
npm test
```

### Test Structure

```typescript
import { describe, it, expect } from '@jest/globals';

describe('BookmarkConverter', () => {
  it('should convert Chrome bookmarks to JSON API format', () => {
    const chromeBookmark = { /* ... */ };
    const result = chromeToJsonApi(chromeBookmark);
    expect(result.data).toHaveLength(1);
  });
});
```

## Integration Tests

### Current Status
- Framework configured (Jest for TS, Rust test framework)
- Ready for end-to-end flow tests
- Pending implementation

### Planned Coverage
1. Full message flow (extension → host → Git)
2. GitHub API mocking with `wiremock`
3. Multi-device sync scenarios
4. Conflict resolution

## Manual Testing Checklist

Before each release:

### Installation
- [ ] Fresh install on clean system
- [ ] Manifest installation (Chrome, Firefox)
- [ ] Extension loading

### Setup
- [ ] GitHub OAuth flow
- [ ] Repository creation
- [ ] Repository cloning
- [ ] Token storage in keychain

### Bookmarks
- [ ] Create bookmark with tags
- [ ] Edit bookmark
- [ ] Delete bookmark
- [ ] Move bookmark to folder
- [ ] Tag extraction from title

### Sync
- [ ] Auto-sync on change
- [ ] Manual sync button
- [ ] Cross-device sync
- [ ] Conflict handling
- [ ] Offline behavior

### Tags
- [ ] Create tag
- [ ] Create hierarchical tags (parent/child)
- [ ] Tag breadcrumb display
- [ ] Tag filtering in popup
- [ ] Tag color assignment

### Error Scenarios
- [ ] Network offline
- [ ] Invalid GitHub token
- [ ] Merge conflicts
- [ ] Corrupted JSON
- [ ] Permission errors

## Coverage Targets

- **Critical paths**: >90% coverage
- **Overall**: >80% coverage
- **Integration**: End-to-end flows tested

## Test Data

Use consistent test data:
```rust
fn create_test_bookmark_data() -> String {
    r#"{
        "jsonapi": {"version": "1.1"},
        "data": [{"type": "bookmark", ...}]
    }"#.to_string()
}
```

## Performance Tests

Not currently implemented, but consider:
- Large bookmark sets (1000+ bookmarks)
- Deep tag hierarchies
- Git performance with large repos
- Memory usage profiling
