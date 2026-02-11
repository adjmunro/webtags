mod git;
mod github;
mod messaging;
mod storage;

use anyhow::{Context, Result};
use log::{error, info};
use messaging::{Message, Response};
use std::io::{stdin, stdout};
use std::path::PathBuf;

/// Configuration for the native host
struct HostConfig {
    repo_path: Option<PathBuf>,
}

impl HostConfig {
    fn new() -> Self {
        Self { repo_path: None }
    }

    fn get_repo_path(&self) -> Result<PathBuf> {
        self.repo_path
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Repository not initialized"))
    }
}

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    info!("WebTags native messaging host started");

    let mut config = HostConfig::new();

    // Main message loop
    loop {
        match messaging::read_message(stdin()) {
            Ok(message) => {
                info!("Received message: {:?}", message);

                let response = handle_message(message, &mut config).await;

                if let Err(e) = messaging::write_response(stdout(), &response) {
                    error!("Failed to write response: {}", e);
                    break;
                }
            }
            Err(e) => {
                error!("Failed to read message: {}", e);

                let error_response = Response::Error {
                    message: format!("Failed to read message: {}", e),
                    code: Some("ERR_READ_MESSAGE".to_string()),
                };

                if let Err(e) = messaging::write_response(stdout(), &error_response) {
                    error!("Failed to write error response: {}", e);
                }
                break;
            }
        }
    }

    info!("WebTags native messaging host stopped");
}

async fn handle_message(message: Message, config: &mut HostConfig) -> Response {
    match message {
        Message::Init { repo_path, repo_url } => handle_init(config, repo_path, repo_url).await,
        Message::Write { data } => handle_write(config, data).await,
        Message::Read => handle_read(config).await,
        Message::Sync => handle_sync(config).await,
        Message::Auth { method, token } => handle_auth(method, token).await,
        Message::Status => handle_status(config).await,
    }
}

async fn handle_init(
    config: &mut HostConfig,
    repo_path: Option<String>,
    repo_url: Option<String>,
) -> Response {
    info!("Initializing repository");

    // Determine repo path (use provided or default)
    let path = repo_path
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("webtags")
        });

    // Clone or init repository
    let repo = if let Some(url) = repo_url {
        info!("Cloning repository from {}", url);
        match git::GitRepo::clone(&url, &path) {
            Ok(repo) => repo,
            Err(e) => {
                return Response::Error {
                    message: format!("Failed to clone repository: {}", e),
                    code: Some("ERR_CLONE".to_string()),
                }
            }
        }
    } else {
        info!("Initializing local repository at {:?}", path);
        match git::GitRepo::init(&path) {
            Ok(repo) => repo,
            Err(e) => {
                return Response::Error {
                    message: format!("Failed to initialize repository: {}", e),
                    code: Some("ERR_INIT".to_string()),
                }
            }
        }
    };

    config.repo_path = Some(repo.path().to_path_buf());

    Response::Success {
        message: format!("Repository initialized at {:?}", repo.path()),
        data: None,
    }
}

async fn handle_write(config: &mut HostConfig, data: serde_json::Value) -> Response {
    info!("Writing bookmarks data");

    let repo_path = match config.get_repo_path() {
        Ok(path) => path,
        Err(e) => {
            return Response::Error {
                message: e.to_string(),
                code: Some("ERR_NOT_INITIALIZED".to_string()),
            }
        }
    };

    // Parse bookmarks data
    let bookmarks_data: storage::BookmarksData = match serde_json::from_value(data) {
        Ok(data) => data,
        Err(e) => {
            return Response::Error {
                message: format!("Failed to parse bookmarks data: {}", e),
                code: Some("ERR_PARSE".to_string()),
            }
        }
    };

    // Validate data
    if let Err(e) = bookmarks_data.validate() {
        return Response::Error {
            message: format!("Invalid bookmarks data: {}", e),
            code: Some("ERR_VALIDATE".to_string()),
        };
    }

    // Write to file
    let bookmarks_file = repo_path.join("bookmarks.json");
    if let Err(e) = storage::write_to_file(&bookmarks_file, &bookmarks_data) {
        return Response::Error {
            message: format!("Failed to write bookmarks file: {}", e),
            code: Some("ERR_WRITE_FILE".to_string()),
        };
    }

    // Git operations
    let repo = match git::GitRepo::init(&repo_path) {
        Ok(repo) => repo,
        Err(e) => {
            return Response::Error {
                message: format!("Failed to open repository: {}", e),
                code: Some("ERR_OPEN_REPO".to_string()),
            }
        }
    };

    // Add and commit
    if let Err(e) = repo.add_file("bookmarks.json") {
        return Response::Error {
            message: format!("Failed to stage file: {}", e),
            code: Some("ERR_GIT_ADD".to_string()),
        };
    }

    let commit_message = format!(
        "Update bookmarks: {} bookmarks, {} tags",
        bookmarks_data.get_bookmarks().len(),
        bookmarks_data.get_tags().len()
    );

    if let Err(e) = repo.commit(&commit_message) {
        return Response::Error {
            message: format!("Failed to commit: {}", e),
            code: Some("ERR_GIT_COMMIT".to_string()),
        };
    }

    // Push to remote (if configured)
    if repo.has_remote("origin") {
        if let Err(e) = repo.push("origin", "main") {
            return Response::Error {
                message: format!("Failed to push: {}", e),
                code: Some("ERR_GIT_PUSH".to_string()),
            };
        }
    }

    Response::Success {
        message: "Bookmarks saved and synced".to_string(),
        data: None,
    }
}

async fn handle_read(config: &mut HostConfig) -> Response {
    info!("Reading bookmarks data");

    let repo_path = match config.get_repo_path() {
        Ok(path) => path,
        Err(e) => {
            return Response::Error {
                message: e.to_string(),
                code: Some("ERR_NOT_INITIALIZED".to_string()),
            }
        }
    };

    let bookmarks_file = repo_path.join("bookmarks.json");

    // Check if file exists
    if !bookmarks_file.exists() {
        // Return empty bookmarks data
        let empty_data = storage::BookmarksData::new();
        return Response::Success {
            message: "No bookmarks file found, returning empty data".to_string(),
            data: Some(serde_json::to_value(empty_data).unwrap()),
        };
    }

    // Read from file
    let bookmarks_data = match storage::read_from_file(&bookmarks_file) {
        Ok(data) => data,
        Err(e) => {
            return Response::Error {
                message: format!("Failed to read bookmarks file: {}", e),
                code: Some("ERR_READ_FILE".to_string()),
            }
        }
    };

    Response::Success {
        message: "Bookmarks loaded".to_string(),
        data: Some(serde_json::to_value(bookmarks_data).unwrap()),
    }
}

async fn handle_sync(config: &mut HostConfig) -> Response {
    info!("Syncing with remote");

    let repo_path = match config.get_repo_path() {
        Ok(path) => path,
        Err(e) => {
            return Response::Error {
                message: e.to_string(),
                code: Some("ERR_NOT_INITIALIZED".to_string()),
            }
        }
    };

    let repo = match git::GitRepo::init(&repo_path) {
        Ok(repo) => repo,
        Err(e) => {
            return Response::Error {
                message: format!("Failed to open repository: {}", e),
                code: Some("ERR_OPEN_REPO".to_string()),
            }
        }
    };

    if !repo.has_remote("origin") {
        return Response::Error {
            message: "No remote configured".to_string(),
            code: Some("ERR_NO_REMOTE".to_string()),
        };
    }

    // Pull from remote
    if let Err(e) = repo.pull("origin", "main") {
        return Response::Error {
            message: format!("Failed to pull: {}", e),
            code: Some("ERR_GIT_PULL".to_string()),
        };
    }

    Response::Success {
        message: "Synced with remote".to_string(),
        data: None,
    }
}

async fn handle_auth(method: messaging::AuthMethod, token: Option<String>) -> Response {
    info!("Handling authentication: {:?}", method);

    match method {
        messaging::AuthMethod::OAuth => {
            // Start OAuth device flow
            let client = github::GitHubClient::new();

            let device_code_response = match client.start_device_flow().await {
                Ok(response) => response,
                Err(e) => {
                    return Response::Error {
                        message: format!("Failed to start OAuth flow: {}", e),
                        code: Some("ERR_OAUTH_START".to_string()),
                    }
                }
            };

            // Return device code info to extension (which will show to user)
            Response::AuthFlow {
                user_code: device_code_response.user_code,
                verification_uri: device_code_response.verification_uri,
                device_code: device_code_response.device_code,
            }
        }
        messaging::AuthMethod::PAT => {
            // Store provided PAT
            let token = match token {
                Some(t) => t,
                None => {
                    return Response::Error {
                        message: "No token provided".to_string(),
                        code: Some("ERR_NO_TOKEN".to_string()),
                    }
                }
            };

            // Validate token
            let client = github::GitHubClient::new();
            match client.validate_token(&token).await {
                Ok(true) => {
                    // Store in keychain
                    if let Err(e) = github::store_token(&token) {
                        return Response::Error {
                            message: format!("Failed to store token: {}", e),
                            code: Some("ERR_STORE_TOKEN".to_string()),
                        };
                    }

                    Response::Success {
                        message: "Token validated and stored".to_string(),
                        data: None,
                    }
                }
                Ok(false) => Response::Error {
                    message: "Invalid token".to_string(),
                    code: Some("ERR_INVALID_TOKEN".to_string()),
                },
                Err(e) => Response::Error {
                    message: format!("Failed to validate token: {}", e),
                    code: Some("ERR_VALIDATE_TOKEN".to_string()),
                },
            }
        }
    }
}

async fn handle_status(config: &HostConfig) -> Response {
    info!("Getting status");

    let repo_path = match config.repo_path.as_ref() {
        Some(path) => path,
        None => {
            return Response::Success {
                message: "Not initialized".to_string(),
                data: Some(serde_json::json!({
                    "initialized": false,
                })),
            }
        }
    };

    let repo = match git::GitRepo::init(repo_path) {
        Ok(repo) => repo,
        Err(e) => {
            return Response::Error {
                message: format!("Failed to open repository: {}", e),
                code: Some("ERR_OPEN_REPO".to_string()),
            }
        }
    };

    let is_clean = repo.is_clean().unwrap_or(false);
    let has_remote = repo.has_remote("origin");

    let last_commit = repo.get_last_commit_message().ok();

    Response::Success {
        message: "Status retrieved".to_string(),
        data: Some(serde_json::json!({
            "initialized": true,
            "repo_path": repo_path,
            "is_clean": is_clean,
            "has_remote": has_remote,
            "last_commit": last_commit,
        })),
    }
}
