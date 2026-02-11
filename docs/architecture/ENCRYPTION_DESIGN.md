# WebTags Encryption Design

## Overview

Optional encryption for bookmarks.json with macOS Touch ID/fingerprint authentication.

## Architecture

```
User Action (read/write bookmarks)
    ↓
Extension requests data
    ↓
Native Host checks if encryption enabled
    ↓
[IF ENCRYPTED]
    ↓
Request Touch ID authentication (macOS)
    ↓
Retrieve encryption key from Keychain
    ↓
Encrypt/Decrypt bookmarks.json with AES-256-GCM
    ↓
Return data to extension
```

## Encryption Details

### Algorithm
- **Cipher**: AES-256-GCM (Authenticated Encryption)
- **Key Size**: 256 bits (32 bytes)
- **Nonce/IV**: 96 bits (12 bytes), randomly generated per encryption
- **Authentication Tag**: 128 bits (16 bytes)

### File Format (Encrypted)
```json
{
  "version": "1",
  "encrypted": true,
  "algorithm": "AES-256-GCM",
  "nonce": "base64-encoded-nonce",
  "ciphertext": "base64-encoded-encrypted-data",
  "tag": "base64-encoded-auth-tag"
}
```

### Key Management

#### Key Storage
- **Location**: macOS Keychain
- **Service**: `com.webtags.encryption`
- **Account**: `master-key`
- **Access Control**: Biometry (Touch ID) required
- **Key Derivation**: PBKDF2-HMAC-SHA256 (100,000 rounds) from user's master key

#### Key Generation
1. Generate 256-bit random key on first enable
2. Derive encryption key using PBKDF2 with device-specific salt
3. Store in Keychain with biometric protection
4. Key never leaves Keychain without Touch ID authentication

### Touch ID Integration

#### Implementation
The encryption key is stored in macOS Keychain using the `security` command-line tool with proper access control flags:

```bash
security add-generic-password \
  -a "master-key" \
  -s "com.webtags.encryption" \
  -w "$BASE64_KEY" \
  -T ""  # Require authentication (Touch ID) for all apps
```

**Key Details:**
- `-T ""` flag: Requires Touch ID/biometric authentication for access
- Access control is set at keychain item creation time
- macOS automatically prompts for Touch ID when the key is accessed
- The Security Framework APIs are called via the `security` CLI for reliable Touch ID integration

#### Why Use CLI Instead of Rust APIs?
The `security-framework` and `security-framework-sys` crates don't provide stable, well-documented access to `SecAccessControl` with biometric flags. Using the `security` CLI tool ensures:
- ✅ Reliable Touch ID prompts (no password prompts)
- ✅ Proper access control configuration
- ✅ Standard macOS keychain behavior
- ✅ Future compatibility with macOS updates

#### Fallback
- If Touch ID is unavailable (no hardware), macOS falls back to system password
- Clear error messages for authentication failures
- Graceful degradation on non-Touch ID capable devices

## User Experience

### First-Time Setup
1. User enables encryption in extension settings
2. System prompts for Touch ID to create encryption key
3. Existing bookmarks.json encrypted in place
4. User sees "🔒 Encryption Enabled" indicator

### Daily Usage
1. Extension starts → attempts to read bookmarks
2. Native host detects encrypted file
3. Touch ID prompt appears: "WebTags needs to access your bookmarks"
4. User authenticates with fingerprint
5. Bookmarks decrypted and loaded (seamless)

### Disabling Encryption
1. User disables encryption in settings
2. Touch ID prompt to decrypt data
3. Bookmarks.json written in plain JSON
4. Encryption key optionally removed from Keychain

## Security Considerations

### Threat Model
**Protected Against:**
- ✅ File system access by malicious apps
- ✅ Backup exposure (encrypted backups)
- ✅ Git repository exposure (don't commit .encrypted files)
- ✅ Stolen device access (requires Touch ID)

**Not Protected Against:**
- ⚠️ Memory dumps while decrypted (transient)
- ⚠️ Screen capture of displayed bookmarks
- ⚠️ Compromised system with keylogger

### Best Practices
- Encryption key never written to disk unencrypted
- Decrypted data only in memory during active use
- Zero data on disk after decryption (secure memory)
- Git should ignore `.encrypted` files

## Configuration

### Settings Structure
```rust
struct EncryptionConfig {
    enabled: bool,
    algorithm: String,  // "AES-256-GCM"
    key_id: String,     // Keychain key identifier
}
```

### Storage Location
- Config file: `~/.local/share/webtags/config.json`
- Separate from bookmarks to avoid circular dependency

## Implementation Plan

### Phase 1: Core Encryption (Rust)
- [x] Design complete
- [ ] Create `encryption.rs` module
- [ ] Implement AES-256-GCM encryption/decryption
- [ ] Implement key generation and storage
- [ ] macOS Keychain integration
- [ ] Touch ID authentication

### Phase 2: Storage Integration
- [ ] Modify `storage.rs` for encryption awareness
- [ ] Add `read_encrypted()` and `write_encrypted()` functions
- [ ] Configuration management

### Phase 3: Extension UI
- [ ] Add encryption toggle in settings
- [ ] Show encryption status indicator
- [ ] Handle Touch ID prompts gracefully
- [ ] Error handling and user feedback

### Phase 4: Testing
- [ ] Unit tests for encryption/decryption
- [ ] Integration tests with Keychain (mocked)
- [ ] Test key rotation
- [ ] Test error scenarios

### Phase 5: Documentation
- [ ] User guide for enabling encryption
- [ ] Security documentation
- [ ] Troubleshooting guide

## API Changes

### New Message Types
```rust
// Enable encryption
{
  "type": "enable_encryption"
}

// Disable encryption
{
  "type": "disable_encryption"
}

// Check encryption status
{
  "type": "encryption_status"
}
```

### Response Types
```rust
{
  "type": "success",
  "data": {
    "encryption_enabled": true,
    "requires_auth": true
  }
}
```

## Dependencies

### Rust Crates
- `ring` or `aes-gcm` - AES-256-GCM encryption
- `security-framework` - macOS Keychain/Touch ID
- `base64` - Encoding encrypted data
- `rand` - Secure random number generation

### macOS Frameworks
- `Security.framework` - Keychain and biometric access
- `LocalAuthentication.framework` - Touch ID prompts

## Migration Path

### Existing Users
1. Encryption is opt-in (default: disabled)
2. When enabled:
   - Current bookmarks.json backed up as bookmarks.json.backup
   - Encrypted version created as bookmarks.json.encrypted
   - Original removed
3. Git ignore rules updated automatically

### Future Enhancements
- [ ] Support for Face ID on Macs with camera
- [ ] Key rotation capabilities
- [ ] Export/import encrypted backups
- [ ] Support for other platforms (Linux keyring, Windows DPAPI)
- [ ] Hardware security module (HSM) support

## Performance Impact

- **Encryption**: <10ms for typical bookmarks file
- **Decryption**: <10ms with cached key
- **Touch ID prompt**: 1-3 seconds (user interaction)
- **Memory overhead**: ~2x file size during encryption/decryption

Negligible impact on user experience.

## Compatibility

- **macOS**: Full support (10.12.2+)
- **Linux**: Future support (secret-service API)
- **Windows**: Future support (DPAPI)

---

**Status**: Design Complete ✅
**Ready for Implementation**: Yes
**Estimated Effort**: 2-3 days development + testing
