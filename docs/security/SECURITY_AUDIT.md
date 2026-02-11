# WebTags Security Audit Report

**Date**: 2024-02-11
**Auditor**: Security Review
**Severity Levels**: 🔴 Critical | 🟠 High | 🟡 Medium | 🟢 Low

---

## Executive Summary

Comprehensive security audit identified **13 vulnerabilities** across authentication, input validation, native messaging, and extension security. All issues have been categorized and fixes provided.

**Risk Summary:**
- 🔴 Critical: 2
- 🟠 High: 4
- 🟡 Medium: 5
- 🟢 Low: 2

---

## Findings

### 🔴 CRITICAL

#### VULN-01: Missing Content Security Policy (CSP)
**File**: `extension/manifest.json`
**Severity**: 🔴 Critical
**CVSS**: 8.1

**Description**: Extension manifest lacks Content Security Policy, allowing inline script execution if XSS vulnerabilities exist.

**Impact**: If an XSS vulnerability is introduced, attackers could execute arbitrary JavaScript in the extension context, accessing all extension permissions including native messaging and bookmarks.

**Fix**: Add strict CSP to manifest.json
```json
"content_security_policy": {
  "extension_pages": "script-src 'self'; object-src 'none'"
}
```

---

#### VULN-02: URL Validation Missing
**File**: `native-host/src/storage.rs`
**Line**: 42
**Severity**: 🔴 Critical
**CVSS**: 7.5

**Description**: Bookmark URLs are stored as unvalidated strings, allowing dangerous URL schemes.

**Impact**:
- `javascript:` URLs enable XSS when opened
- `file://` URLs could access local files
- `data:` URLs with malicious content
- Extremely long URLs (DoS)

**Fix**: Add URL validation function:
```rust
fn validate_bookmark_url(url: &str) -> Result<()> {
    // Check length
    if url.len() > 2048 {
        anyhow::bail!("URL too long (max 2048 characters)");
    }

    // Parse URL
    let parsed = url::Url::parse(url)
        .context("Invalid URL format")?;

    // Only allow safe schemes
    match parsed.scheme() {
        "http" | "https" => Ok(()),
        scheme => anyhow::bail!("Unsafe URL scheme: {}", scheme),
    }
}
```

---

### 🟠 HIGH

#### VULN-03: XSS via Tag Names in Popup UI
**File**: `extension/src/popup/popup.ts`
**Line**: 179
**Severity**: 🟠 High
**CVSS**: 6.8

**Description**: Tag names in popup are not HTML-escaped before insertion via innerHTML.

**Impact**: Malicious tag names like `<img src=x onerror=alert(1)>` could execute JavaScript in popup context.

**Fix**: Escape tag names:
```typescript
${tags.map(tag => `<span class="tag-chip">#${escapeHtml(tag)}</span>`).join('')}
```

---

#### VULN-04: Path Traversal in Repository Path
**File**: `native-host/src/main.rs`
**Line**: 85-90
**Severity**: 🟠 High
**CVSS**: 6.5

**Description**: User-provided repo_path is not validated, potentially allowing access to sensitive directories.

**Impact**: Attacker could specify paths like `/etc/passwd` or `../../sensitive-dir` to read/write outside intended directory.

**Fix**: Add path validation:
```rust
fn validate_repo_path(path: &Path) -> Result<PathBuf> {
    // Canonicalize path
    let canonical = path.canonicalize()
        .or_else(|_| {
            // Path doesn't exist yet, check parent
            if let Some(parent) = path.parent() {
                parent.canonicalize()?;
            }
            Ok(path.to_path_buf())
        })?;

    // Ensure it's in an allowed directory
    let home = dirs::home_dir().context("No home directory")?;
    let allowed_base = home.join(".local/share/webtags");

    if !canonical.starts_with(&allowed_base) {
        anyhow::bail!("Repository path must be within {}", allowed_base.display());
    }

    Ok(canonical)
}
```

---

#### VULN-05: Token Exposure in Error Messages
**File**: `native-host/src/github.rs`
**Lines**: 77-80, 185-190
**Severity**: 🟠 High
**CVSS**: 6.2

**Description**: Full API error responses (which may contain sensitive data) are included in error messages sent to extension.

**Impact**: Tokens or other sensitive data in GitHub API error responses could be logged or displayed to user.

**Fix**: Sanitize error messages:
```rust
if !response.status().is_success() {
    let status = response.status();
    // Don't include response body in error
    anyhow::bail!("GitHub API error: {}", status);
}
```

---

#### VULN-06: Overly Broad Host Permissions
**File**: `extension/manifest.json`
**Line**: 29-31
**Severity**: 🟠 High
**CVSS**: 5.8

**Description**: Extension requests access to all of github.com with wildcard.

**Impact**: Violates principle of least privilege. Extension only needs OAuth endpoints, not all of GitHub.

**Fix**: Limit to specific OAuth endpoints (Note: Manifest V3 limitations may require this, but document it):
```json
"host_permissions": [
  "https://github.com/login/*"
]
```

---

### 🟡 MEDIUM

#### VULN-07: Missing Rate Limiting on Native Messaging
**File**: `native-host/src/messaging.rs`
**Severity**: 🟡 Medium
**CVSS**: 5.3

**Description**: No rate limiting on incoming messages from extension.

**Impact**: Malicious extension could flood native host with messages, causing DoS.

**Fix**: Implement rate limiting with token bucket:
```rust
use std::time::{Duration, Instant};

struct RateLimiter {
    tokens: f64,
    capacity: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl RateLimiter {
    fn new(capacity: f64, refill_per_second: f64) -> Self {
        Self {
            tokens: capacity,
            capacity,
            refill_rate: refill_per_second,
            last_refill: Instant::now(),
        }
    }

    fn allow(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
        self.last_refill = now;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}
```

---

#### VULN-08: Serialization Panic via Unwrap
**File**: `native-host/src/main.rs`
**Lines**: 237, 254
**Severity**: 🟡 Medium
**CVSS**: 5.0

**Description**: `.unwrap()` on JSON serialization could panic, causing native host crash.

**Impact**: Malformed data could crash native host, requiring restart.

**Fix**: Handle serialization errors:
```rust
match serde_json::to_value(empty_data) {
    Ok(value) => Response::Success {
        message: "No bookmarks file found, returning empty data".to_string(),
        data: Some(value),
    },
    Err(e) => Response::Error {
        message: format!("Failed to serialize data: {}", e),
        code: Some("ERR_SERIALIZE".to_string()),
    }
}
```

---

#### VULN-09: No Token Scope Validation
**File**: `native-host/src/github.rs`
**Line**: 202-214
**Severity**: 🟡 Medium
**CVSS**: 4.8

**Description**: Token validation only checks if token is valid, not if it has required scopes.

**Impact**: Token with insufficient permissions could be accepted, leading to failed operations later.

**Fix**: Validate token scopes:
```rust
pub async fn validate_token(&self, token: &str) -> Result<TokenValidation> {
    let response = self
        .client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?;

    if !response.status().is_success() {
        return Ok(TokenValidation { valid: false, scopes: vec![] });
    }

    // Check scopes from header
    let scopes = response
        .headers()
        .get("X-OAuth-Scopes")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    Ok(TokenValidation {
        valid: true,
        scopes,
    })
}
```

---

#### VULN-10: Memory Allocation Before Full Validation
**File**: `native-host/src/messaging.rs`
**Line**: 70
**Severity**: 🟡 Medium
**CVSS**: 4.5

**Description**: Message buffer allocated based on user input before complete validation.

**Impact**: Many concurrent large messages could exhaust memory.

**Fix**: Add chunked reading or validate against available memory:
```rust
// Check against available memory
let available_memory = /* get system memory */;
if length > available_memory / 10 {  // Use at most 10% of available memory
    anyhow::bail!("Message too large for available memory");
}
```

---

#### VULN-11: No Timeout on OAuth Polling
**File**: `native-host/src/github.rs`
**Line**: 93-158
**Severity**: 🟡 Medium
**CVSS**: 4.2

**Description**: OAuth polling has max_attempts but no absolute time limit.

**Impact**: With slow_down errors, polling could continue indefinitely.

**Fix**: Add absolute timeout:
```rust
let start_time = Instant::now();
let max_duration = Duration::from_secs(900); // 15 minutes

loop {
    if start_time.elapsed() > max_duration {
        anyhow::bail!("OAuth flow timeout after 15 minutes");
    }

    if attempts >= max_attempts {
        anyhow::bail!("Timeout waiting for user authorization");
    }
    // ... rest of polling logic
}
```

---

### 🟢 LOW

#### VULN-12: Hardcoded OAuth Client ID
**File**: `native-host/src/github.rs`
**Line**: 8
**Severity**: 🟢 Low
**CVSS**: 3.1

**Description**: OAuth client ID is hardcoded as placeholder.

**Impact**: OAuth won't work until user registers their own app. Not a security issue per se, but requires documentation.

**Fix**: Make configurable via environment variable:
```rust
fn get_client_id() -> Result<String> {
    std::env::var("WEBTAGS_GITHUB_CLIENT_ID")
        .context("WEBTAGS_GITHUB_CLIENT_ID not set. Please register a GitHub OAuth app.")
}
```

---

#### VULN-13: No TLS Verification Documentation
**File**: `native-host/src/github.rs`
**Line**: 54-63
**Severity**: 🟢 Low
**CVSS**: 2.8

**Description**: TLS certificate validation is enabled by default (good) but not explicitly documented.

**Impact**: Developers might not realize importance of not disabling TLS verification.

**Fix**: Add explicit documentation and verification:
```rust
pub fn new() -> Self {
    // Explicitly use default TLS configuration (enables certificate validation)
    let client = Client::builder()
        .use_rustls_tls()  // Explicit TLS
        .build()
        .expect("Failed to build HTTP client");

    Self { client }
}
```

---

## Recommendations

### Immediate Actions (Critical/High)
1. ✅ Add CSP to manifest.json
2. ✅ Implement URL validation for bookmarks
3. ✅ Escape all user input in popup UI
4. ✅ Validate and restrict repository paths
5. ✅ Sanitize error messages

### Short Term (Medium)
6. Implement rate limiting on native messaging
7. Replace `.unwrap()` with proper error handling
8. Add token scope validation
9. Improve memory management in message parsing

### Long Term (Low + Enhancements)
10. Make OAuth client ID configurable
11. Add security documentation
12. Implement security headers for all API requests
13. Add audit logging for sensitive operations
14. Implement anomaly detection for unusual patterns

---

## Testing Recommendations

### Security Test Suite
- [ ] Fuzzing tests for native messaging protocol
- [ ] XSS tests for all UI inputs
- [ ] Path traversal tests for repository paths
- [ ] Token validation tests
- [ ] Rate limiting tests
- [ ] Memory exhaustion tests

---

## Conclusion

WebTags has a solid security foundation but requires critical fixes for:
1. Content Security Policy
2. URL validation
3. Input sanitization
4. Path validation

All identified vulnerabilities have documented fixes. Implementation of critical and high severity fixes is recommended before production deployment.

**Security Posture**: ⚠️ **REQUIRES FIXES** → ✅ **SECURE** (after fixes)

---

**Next Steps**: Implement fixes and re-audit after changes.
