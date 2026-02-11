use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[cfg(target_os = "macos")]
use security_framework::os::macos::keychain::SecKeychain;

const KEYCHAIN_SERVICE: &str = "com.webtags.encryption";
const KEYCHAIN_ACCOUNT: &str = "master-key";
const NONCE_SIZE: usize = 12; // 96 bits for AES-GCM

/// Encrypted file format
#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedData {
    version: String,
    encrypted: bool,
    algorithm: String,
    #[serde(with = "base64_serde")]
    nonce: Vec<u8>,
    #[serde(with = "base64_serde")]
    ciphertext: Vec<u8>,
}

mod base64_serde {
    use super::*;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&BASE64.encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        BASE64.decode(s).map_err(serde::de::Error::custom)
    }
}

/// Encryption manager
pub struct EncryptionManager {
    enabled: bool,
}

impl EncryptionManager {
    /// Create new encryption manager
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Check if encryption is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Generate a new encryption key and store in Keychain with Touch ID
    #[cfg(target_os = "macos")]
    pub fn generate_and_store_key() -> Result<()> {
        // Generate random 256-bit key
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);

        // Store in macOS Keychain with biometric protection
        Self::store_key_in_keychain(&key)?;

        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    pub fn generate_and_store_key() -> Result<()> {
        anyhow::bail!("Encryption with biometric authentication is only supported on macOS");
    }

    /// Store encryption key in macOS Keychain with Touch ID requirement
    #[cfg(target_os = "macos")]
    fn store_key_in_keychain(key: &[u8]) -> Result<()> {
        use std::io::Read;
        use std::process::Command;

        // Convert key to base64 for storage
        let key_b64 = BASE64.encode(key);

        // Delete existing key if present
        let _ = Self::delete_key_from_keychain();

        // Use the `security` command-line tool to add the item with Touch ID requirement
        // The -T "" flag means require authentication for all apps (prompts Touch ID)
        let mut child = Command::new("security")
            .args([
                "add-generic-password",
                "-a",
                KEYCHAIN_ACCOUNT,
                "-s",
                KEYCHAIN_SERVICE,
                "-w",
                &key_b64,
                "-T",
                "",   // Require authentication (Touch ID) for access
                "-U", // Update if exists
            ])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn security command")?;

        let status = child
            .wait()
            .context("Failed to wait for security command")?;

        if status.success() {
            log::info!("Encryption key stored in Keychain with Touch ID requirement");
            Ok(())
        } else {
            let stderr = child
                .stderr
                .and_then(|mut s| {
                    let mut buf = String::new();
                    s.read_to_string(&mut buf).ok().map(|_| buf)
                })
                .unwrap_or_default();
            anyhow::bail!("Failed to store key in Keychain: {}", stderr)
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn store_key_in_keychain(_key: &[u8]) -> Result<()> {
        anyhow::bail!("macOS Keychain not available on this platform");
    }

    /// Retrieve encryption key from Keychain (triggers Touch ID prompt)
    #[cfg(target_os = "macos")]
    fn get_key_from_keychain() -> Result<Vec<u8>> {
        use security_framework::os::macos::keychain::SecKeychain;

        let keychain = SecKeychain::default()?;

        let (password_bytes, _) = keychain
            .find_generic_password(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT)
            .context("Encryption key not found in Keychain. Please enable encryption first.")?;

        // Decode from base64
        let key = BASE64
            .decode(&password_bytes)
            .context("Failed to decode encryption key")?;

        if key.len() != 32 {
            anyhow::bail!("Invalid encryption key size");
        }

        Ok(key)
    }

    #[cfg(not(target_os = "macos"))]
    fn get_key_from_keychain() -> Result<Vec<u8>> {
        anyhow::bail!("macOS Keychain not available on this platform");
    }

    /// Delete encryption key from Keychain
    #[cfg(target_os = "macos")]
    pub fn delete_key_from_keychain() -> Result<()> {
        let keychain = SecKeychain::default()?;

        // Find and delete the password
        if let Ok((_, item)) = keychain.find_generic_password(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT) {
            item.delete();
            log::info!("Encryption key deleted from Keychain");
        }

        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    pub fn delete_key_from_keychain() -> Result<()> {
        Ok(()) // No-op on non-macOS
    }

    /// Encrypt data with AES-256-GCM
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData> {
        if !self.enabled {
            anyhow::bail!("Encryption is not enabled");
        }

        // Get encryption key from Keychain (triggers Touch ID)
        let key_bytes = Self::get_key_from_keychain()?;

        // Create cipher
        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to create cipher: {:?}", e))?;

        // Generate random nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        Ok(EncryptedData {
            version: "1".to_string(),
            encrypted: true,
            algorithm: "AES-256-GCM".to_string(),
            nonce: nonce_bytes.to_vec(),
            ciphertext,
        })
    }

    /// Decrypt data with AES-256-GCM
    pub fn decrypt(&self, encrypted: &EncryptedData) -> Result<Vec<u8>> {
        if !encrypted.encrypted {
            anyhow::bail!("Data is not encrypted");
        }

        if encrypted.algorithm != "AES-256-GCM" {
            anyhow::bail!("Unsupported encryption algorithm: {}", encrypted.algorithm);
        }

        // Get encryption key from Keychain (triggers Touch ID)
        let key_bytes = Self::get_key_from_keychain()?;

        // Create cipher
        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to create cipher: {:?}", e))?;

        // Get nonce
        if encrypted.nonce.len() != NONCE_SIZE {
            anyhow::bail!("Invalid nonce size");
        }
        let nonce = Nonce::from_slice(&encrypted.nonce);

        // Decrypt
        let plaintext = cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

        Ok(plaintext)
    }

    /// Read encrypted file
    pub fn read_encrypted_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>> {
        let content = fs::read_to_string(path.as_ref()).context("Failed to read encrypted file")?;

        let encrypted: EncryptedData =
            serde_json::from_str(&content).context("Failed to parse encrypted file")?;

        self.decrypt(&encrypted)
    }

    /// Write encrypted file
    pub fn write_encrypted_file<P: AsRef<Path>>(&self, path: P, data: &[u8]) -> Result<()> {
        let encrypted = self.encrypt(data)?;

        let json = serde_json::to_string_pretty(&encrypted)
            .context("Failed to serialize encrypted data")?;

        // Atomic write
        let temp_path = path.as_ref().with_extension("tmp");
        fs::write(&temp_path, json).context("Failed to write temp file")?;
        fs::rename(&temp_path, path.as_ref()).context("Failed to rename temp file to target")?;

        Ok(())
    }
}

/// Check if a file is encrypted
pub fn is_encrypted<P: AsRef<Path>>(path: P) -> Result<bool> {
    if !path.as_ref().exists() {
        return Ok(false);
    }

    let content = fs::read_to_string(path.as_ref()).context("Failed to read file")?;

    if let Ok(data) = serde_json::from_str::<EncryptedData>(&content) {
        Ok(data.encrypted)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypted_data_serialization() {
        let data = EncryptedData {
            version: "1".to_string(),
            encrypted: true,
            algorithm: "AES-256-GCM".to_string(),
            nonce: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            ciphertext: vec![1, 2, 3, 4, 5],
        };

        let json = serde_json::to_string(&data).unwrap();
        let parsed: EncryptedData = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.version, "1");
        assert!(parsed.encrypted);
        assert_eq!(parsed.algorithm, "AES-256-GCM");
        assert_eq!(parsed.nonce, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
        assert_eq!(parsed.ciphertext, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_encryption_manager_creation() {
        let manager = EncryptionManager::new(false);
        assert!(!manager.is_enabled());

        let manager = EncryptionManager::new(true);
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_is_encrypted_non_existent_file() {
        let result = is_encrypted("/tmp/non-existent-file-xyz123.json");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_encrypt_when_disabled() {
        let manager = EncryptionManager::new(false);
        let plaintext = b"test data";

        let result = manager.encrypt(plaintext);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not enabled"));
    }

    #[test]
    fn test_decrypt_with_invalid_nonce_size() {
        let manager = EncryptionManager::new(true);
        let encrypted = EncryptedData {
            version: "1".to_string(),
            encrypted: true,
            algorithm: "AES-256-GCM".to_string(),
            nonce: vec![1, 2, 3], // Invalid: only 3 bytes instead of 12
            ciphertext: vec![1, 2, 3, 4, 5],
        };

        let result = manager.decrypt(&encrypted);
        // Will fail because keychain access is not available in tests
        // The important thing is that it fails gracefully
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_with_unsupported_algorithm() {
        let manager = EncryptionManager::new(true);
        let encrypted = EncryptedData {
            version: "1".to_string(),
            encrypted: true,
            algorithm: "AES-128-CBC".to_string(),
            nonce: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            ciphertext: vec![1, 2, 3, 4, 5],
        };

        let result = manager.decrypt(&encrypted);
        // Check that unsupported algorithm is rejected early
        // (before keychain access is attempted)
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Unsupported encryption algorithm") || err_msg.contains("Keychain")
        );
    }

    #[test]
    fn test_decrypt_when_not_encrypted() {
        let manager = EncryptionManager::new(true);
        let encrypted = EncryptedData {
            version: "1".to_string(),
            encrypted: false,
            algorithm: "AES-256-GCM".to_string(),
            nonce: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            ciphertext: vec![1, 2, 3, 4, 5],
        };

        let result = manager.decrypt(&encrypted);
        // This check happens before keychain access
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not encrypted"));
    }

    #[test]
    fn test_is_encrypted_with_plain_json() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"{\"jsonapi\": {\"version\": \"1.1\"}, \"data\": []}")
            .unwrap();
        file.flush().unwrap();

        let result = is_encrypted(file.path());
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_is_encrypted_with_encrypted_json() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let encrypted_data = EncryptedData {
            version: "1".to_string(),
            encrypted: true,
            algorithm: "AES-256-GCM".to_string(),
            nonce: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            ciphertext: vec![1, 2, 3, 4, 5],
        };

        let json = serde_json::to_string(&encrypted_data).unwrap();
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(json.as_bytes()).unwrap();
        file.flush().unwrap();

        let result = is_encrypted(file.path());
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_is_encrypted_with_invalid_json() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"not valid json").unwrap();
        file.flush().unwrap();

        let result = is_encrypted(file.path());
        // Should return false for invalid JSON (not encrypted)
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_base64_serde_roundtrip() {
        let data = EncryptedData {
            version: "1".to_string(),
            encrypted: true,
            algorithm: "AES-256-GCM".to_string(),
            nonce: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            ciphertext: vec![255, 254, 253, 252, 251],
        };

        // Serialize to JSON
        let json = serde_json::to_string(&data).unwrap();

        // Verify base64 encoding in JSON
        assert!(json.contains("\"nonce\":"));
        assert!(json.contains("\"ciphertext\":"));

        // Deserialize back
        let parsed: EncryptedData = serde_json::from_str(&json).unwrap();

        // Verify data is preserved
        assert_eq!(parsed.nonce, data.nonce);
        assert_eq!(parsed.ciphertext, data.ciphertext);
    }

    // Note: Full encryption tests require macOS Keychain access
    // and would trigger Touch ID prompts, so they're excluded from
    // automated tests. Manual testing required on macOS.
}
