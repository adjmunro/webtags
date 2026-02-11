use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Message types supported by the native messaging protocol
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Message {
    Init {
        repo_path: Option<String>,
        repo_url: Option<String>,
    },
    Write {
        data: serde_json::Value,
    },
    Read,
    Sync,
    Auth {
        method: AuthMethod,
        token: Option<String>,
    },
    Status,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethod {
    OAuth,
    PAT,
}

/// Response types sent back to the extension
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Response {
    Success {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<serde_json::Value>,
    },
    Error {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<String>,
    },
    AuthFlow {
        user_code: String,
        verification_uri: String,
        device_code: String,
    },
}

/// Read a message from stdin using the native messaging protocol
/// Format: 4-byte length prefix (little-endian) + JSON message
pub fn read_message<R: Read>(mut reader: R) -> Result<Message> {
    // Read 4-byte length prefix
    let mut length_bytes = [0u8; 4];
    reader
        .read_exact(&mut length_bytes)
        .context("Failed to read message length")?;
    let length = u32::from_le_bytes(length_bytes) as usize;

    // Validate length (max 1MB for safety)
    if length > 1_000_000 {
        anyhow::bail!("Message too large: {} bytes", length);
    }

    // Read JSON message
    let mut buffer = vec![0u8; length];
    reader
        .read_exact(&mut buffer)
        .context("Failed to read message body")?;

    // Parse JSON
    let message: Message =
        serde_json::from_slice(&buffer).context("Failed to parse JSON message")?;

    Ok(message)
}

/// Write a response to stdout using the native messaging protocol
/// Format: 4-byte length prefix (little-endian) + JSON message
pub fn write_response<W: Write>(mut writer: W, response: &Response) -> Result<()> {
    // Serialize response to JSON
    let json = serde_json::to_vec(response).context("Failed to serialize response")?;
    let length = json.len() as u32;

    // Write length prefix
    writer
        .write_all(&length.to_le_bytes())
        .context("Failed to write response length")?;

    // Write JSON
    writer
        .write_all(&json)
        .context("Failed to write response body")?;

    writer.flush().context("Failed to flush output")?;

    Ok(())
}

/// Async version of read_message for use in async contexts
pub async fn read_message_async<R: AsyncReadExt + Unpin>(
    mut reader: R,
) -> Result<Message> {
    // Read 4-byte length prefix
    let mut length_bytes = [0u8; 4];
    reader
        .read_exact(&mut length_bytes)
        .await
        .context("Failed to read message length")?;
    let length = u32::from_le_bytes(length_bytes) as usize;

    // Validate length
    if length > 1_000_000 {
        anyhow::bail!("Message too large: {} bytes", length);
    }

    // Read JSON message
    let mut buffer = vec![0u8; length];
    reader
        .read_exact(&mut buffer)
        .await
        .context("Failed to read message body")?;

    // Parse JSON
    let message: Message =
        serde_json::from_slice(&buffer).context("Failed to parse JSON message")?;

    Ok(message)
}

/// Async version of write_response for use in async contexts
pub async fn write_response_async<W: AsyncWriteExt + Unpin>(
    mut writer: W,
    response: &Response,
) -> Result<()> {
    // Serialize response to JSON
    let json = serde_json::to_vec(response).context("Failed to serialize response")?;
    let length = json.len() as u32;

    // Write length prefix
    writer
        .write_all(&length.to_le_bytes())
        .await
        .context("Failed to write response length")?;

    // Write JSON
    writer
        .write_all(&json)
        .await
        .context("Failed to write response body")?;

    writer.flush().await.context("Failed to flush output")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_message_init() {
        let message = Message::Init {
            repo_path: Some("/tmp/test".to_string()),
            repo_url: None,
        };
        let json = serde_json::to_vec(&message).unwrap();
        let length = (json.len() as u32).to_le_bytes();

        let mut input = Vec::new();
        input.extend_from_slice(&length);
        input.extend_from_slice(&json);

        let cursor = Cursor::new(input);
        let result = read_message(cursor).unwrap();

        assert_eq!(result, message);
    }

    #[test]
    fn test_read_message_write() {
        let data = serde_json::json!({"bookmarks": []});
        let message = Message::Write { data: data.clone() };
        let json = serde_json::to_vec(&message).unwrap();
        let length = (json.len() as u32).to_le_bytes();

        let mut input = Vec::new();
        input.extend_from_slice(&length);
        input.extend_from_slice(&json);

        let cursor = Cursor::new(input);
        let result = read_message(cursor).unwrap();

        assert_eq!(result, message);
    }

    #[test]
    fn test_read_message_auth_oauth() {
        let message = Message::Auth {
            method: AuthMethod::OAuth,
            token: None,
        };
        let json = serde_json::to_vec(&message).unwrap();
        let length = (json.len() as u32).to_le_bytes();

        let mut input = Vec::new();
        input.extend_from_slice(&length);
        input.extend_from_slice(&json);

        let cursor = Cursor::new(input);
        let result = read_message(cursor).unwrap();

        assert_eq!(result, message);
    }

    #[test]
    fn test_read_message_auth_pat() {
        let message = Message::Auth {
            method: AuthMethod::PAT,
            token: Some("ghp_test123".to_string()),
        };
        let json = serde_json::to_vec(&message).unwrap();
        let length = (json.len() as u32).to_le_bytes();

        let mut input = Vec::new();
        input.extend_from_slice(&length);
        input.extend_from_slice(&json);

        let cursor = Cursor::new(input);
        let result = read_message(cursor).unwrap();

        assert_eq!(result, message);
    }

    #[test]
    fn test_read_message_too_large() {
        let length = 2_000_000u32.to_le_bytes();
        let input = Vec::from(length);
        let cursor = Cursor::new(input);

        let result = read_message(cursor);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

    #[test]
    fn test_read_message_invalid_json() {
        let invalid_json = b"not valid json";
        let length = (invalid_json.len() as u32).to_le_bytes();

        let mut input = Vec::new();
        input.extend_from_slice(&length);
        input.extend_from_slice(invalid_json);

        let cursor = Cursor::new(input);
        let result = read_message(cursor);

        assert!(result.is_err());
    }

    #[test]
    fn test_write_response_success() {
        let response = Response::Success {
            message: "Operation completed".to_string(),
            data: None,
        };

        let mut output = Vec::new();
        write_response(&mut output, &response).unwrap();

        // Verify length prefix
        let length = u32::from_le_bytes([output[0], output[1], output[2], output[3]]);
        assert_eq!(length as usize, output.len() - 4);

        // Verify JSON
        let json_bytes = &output[4..];
        let parsed: Response = serde_json::from_slice(json_bytes).unwrap();
        assert_eq!(parsed, response);
    }

    #[test]
    fn test_write_response_error() {
        let response = Response::Error {
            message: "Something went wrong".to_string(),
            code: Some("ERR_GIT_PUSH".to_string()),
        };

        let mut output = Vec::new();
        write_response(&mut output, &response).unwrap();

        // Verify JSON can be read back
        let json_bytes = &output[4..];
        let parsed: Response = serde_json::from_slice(json_bytes).unwrap();
        assert_eq!(parsed, response);
    }

    #[test]
    fn test_write_response_auth_flow() {
        let response = Response::AuthFlow {
            user_code: "ABCD-1234".to_string(),
            verification_uri: "https://github.com/login/device".to_string(),
            device_code: "device123".to_string(),
        };

        let mut output = Vec::new();
        write_response(&mut output, &response).unwrap();

        let json_bytes = &output[4..];
        let parsed: Response = serde_json::from_slice(json_bytes).unwrap();
        assert_eq!(parsed, response);
    }

    #[test]
    fn test_round_trip() {
        // Test that we can write a response and read it back as a message
        let original = Message::Status;
        let json = serde_json::to_vec(&original).unwrap();
        let length = (json.len() as u32).to_le_bytes();

        let mut buffer = Vec::new();
        buffer.extend_from_slice(&length);
        buffer.extend_from_slice(&json);

        let cursor = Cursor::new(buffer);
        let parsed = read_message(cursor).unwrap();

        assert_eq!(parsed, original);
    }
}
