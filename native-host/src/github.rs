use anyhow::{Context, Result};
use keyring::Entry;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

const GITHUB_CLIENT_ID: &str = "Ov23liYifB4i3sUooRaE"; // WebTags OAuth app
const KEYRING_SERVICE: &str = "com.webtags.github";
const KEYRING_USERNAME: &str = "github_token";

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPollResponse {
    pub access_token: Option<String>,
    pub token_type: Option<String>,
    pub scope: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRepoRequest {
    pub name: String,
    pub description: Option<String>,
    pub private: bool,
    pub auto_init: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Repository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub clone_url: String,
    pub ssh_url: String,
    pub private: bool,
}

pub struct GitHubClient {
    client: Client,
}

impl GitHubClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Start OAuth device flow
    pub async fn start_device_flow(&self) -> Result<DeviceCodeResponse> {
        let response = self
            .client
            .post("https://github.com/login/device/code")
            .header("Accept", "application/json")
            .form(&[("client_id", GITHUB_CLIENT_ID)])
            .send()
            .await
            .context("Failed to start device flow")?;

        if !response.status().is_success() {
            let status = response.status();
            // Don't include response body in error (may contain sensitive data)
            anyhow::bail!("GitHub API error: {status}");
        }

        let device_code: DeviceCodeResponse = response
            .json()
            .await
            .context("Failed to parse device code response")?;

        Ok(device_code)
    }

    /// Poll for OAuth access token
    pub async fn poll_for_token(
        &self,
        device_code: &str,
        interval: u64,
    ) -> Result<AccessTokenResponse> {
        let mut attempts = 0;
        let max_attempts = 100; // 100 * interval seconds timeout

        loop {
            if attempts >= max_attempts {
                anyhow::bail!("Timeout waiting for user authorization");
            }

            sleep(Duration::from_secs(interval)).await;

            let response = self
                .client
                .post("https://github.com/login/oauth/access_token")
                .header("Accept", "application/json")
                .form(&[
                    ("client_id", GITHUB_CLIENT_ID),
                    ("device_code", device_code),
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ])
                .send()
                .await
                .context("Failed to poll for token")?;

            let poll_response: TokenPollResponse = response
                .json()
                .await
                .context("Failed to parse poll response")?;

            if let Some(access_token) = poll_response.access_token {
                return Ok(AccessTokenResponse {
                    access_token,
                    token_type: poll_response.token_type.unwrap_or_default(),
                    scope: poll_response.scope.unwrap_or_default(),
                });
            }

            match poll_response.error.as_deref() {
                Some("authorization_pending") => {
                    // Continue polling
                    attempts += 1;
                }
                Some("slow_down") => {
                    // Increase interval
                    sleep(Duration::from_secs(interval)).await;
                    attempts += 1;
                }
                Some("expired_token") => {
                    anyhow::bail!("Device code expired");
                }
                Some("access_denied") => {
                    anyhow::bail!("User denied access");
                }
                Some(other) => {
                    anyhow::bail!("OAuth error: {other}");
                }
                None => {
                    anyhow::bail!("Unexpected response from GitHub");
                }
            }
        }
    }

    /// Create a new private repository
    pub async fn create_repository(
        &self,
        token: &str,
        name: &str,
        description: Option<String>,
    ) -> Result<Repository> {
        let request = CreateRepoRequest {
            name: name.to_string(),
            description,
            private: true,
            auto_init: true, // Initialize with README
        };

        let response = self
            .client
            .post("https://api.github.com/user/repos")
            .header("Accept", "application/vnd.github+json")
            .header("Authorization", format!("Bearer {token}"))
            .header("User-Agent", "WebTags")
            .json(&request)
            .send()
            .await
            .context("Failed to create repository")?;

        if !response.status().is_success() {
            let status = response.status();
            // Don't include response body in error (may contain sensitive data)
            anyhow::bail!("Failed to create repository: {status}");
        }

        let repo: Repository = response
            .json()
            .await
            .context("Failed to parse repository response")?;

        Ok(repo)
    }

    /// Validate a token by making a test API call
    pub async fn validate_token(&self, token: &str) -> Result<bool> {
        let response = self
            .client
            .get("https://api.github.com/user")
            .header("Accept", "application/vnd.github+json")
            .header("Authorization", format!("Bearer {token}"))
            .header("User-Agent", "WebTags")
            .send()
            .await
            .context("Failed to validate token")?;

        Ok(response.status().is_success())
    }
}

impl Default for GitHubClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Store GitHub token in OS keychain
pub fn store_token(token: &str) -> Result<()> {
    let entry =
        Entry::new(KEYRING_SERVICE, KEYRING_USERNAME).context("Failed to create keyring entry")?;
    entry
        .set_password(token)
        .context("Failed to store token in keychain")?;
    Ok(())
}

/// Retrieve GitHub token from OS keychain
pub fn get_token() -> Result<String> {
    let entry =
        Entry::new(KEYRING_SERVICE, KEYRING_USERNAME).context("Failed to create keyring entry")?;
    entry
        .get_password()
        .context("Failed to retrieve token from keychain")
}

/// Delete GitHub token from OS keychain
pub fn delete_token() -> Result<()> {
    let entry =
        Entry::new(KEYRING_SERVICE, KEYRING_USERNAME).context("Failed to create keyring entry")?;
    entry
        .delete_password()
        .context("Failed to delete token from keychain")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_github_client() {
        let client = GitHubClient::new();
        assert!(std::mem::size_of_val(&client) > 0);
    }

    // Note: Most tests require mocking the GitHub API
    // These would use wiremock in integration tests

    #[tokio::test]
    async fn test_device_code_response_deserialization() {
        let json = r#"{
            "device_code": "test_device_code",
            "user_code": "ABCD-1234",
            "verification_uri": "https://github.com/login/device",
            "expires_in": 900,
            "interval": 5
        }"#;

        let response: DeviceCodeResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.user_code, "ABCD-1234");
        assert_eq!(response.interval, 5);
    }

    #[tokio::test]
    async fn test_access_token_response_deserialization() {
        let json = r#"{
            "access_token": "gho_test123",
            "token_type": "bearer",
            "scope": "repo"
        }"#;

        let response: AccessTokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.access_token, "gho_test123");
        assert_eq!(response.token_type, "bearer");
    }

    #[tokio::test]
    async fn test_repository_deserialization() {
        let json = r#"{
            "id": 12345,
            "name": "webtags-bookmarks",
            "full_name": "user/webtags-bookmarks",
            "clone_url": "https://github.com/user/webtags-bookmarks.git",
            "ssh_url": "git@github.com:user/webtags-bookmarks.git",
            "private": true
        }"#;

        let repo: Repository = serde_json::from_str(json).unwrap();
        assert_eq!(repo.name, "webtags-bookmarks");
        assert!(repo.private);
    }

    // Keyring tests are platform-specific and may require mocking
    // Skip them in CI environments
}
