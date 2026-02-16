use anyhow::{Context, Result};
use regex::Regex;
use std::sync::LazyLock;

// Compile regexes once at startup
// SSH URLs: git@host:path or ssh://git@host/path
static SSH_URL_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:ssh://git@([^/]+)/(.+)|git@([^:]+):(.+?))(?:\.git)?$").unwrap()
});

// HTTPS URLs: https://host/path or http://host/path
static HTTPS_URL_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^https?://([^/]+)/(.+?)(?:\.git)?$").unwrap());

/// Parse a git URL and determine its type
#[derive(Debug, PartialEq)]
pub enum GitUrlType {
    Ssh,
    Https,
}

/// Parse git URL type
pub fn parse_git_url(url: &str) -> Result<GitUrlType> {
    // Check HTTPS first since it's more specific
    if HTTPS_URL_PATTERN.is_match(url) {
        Ok(GitUrlType::Https)
    } else if SSH_URL_PATTERN.is_match(url) {
        Ok(GitUrlType::Ssh)
    } else {
        anyhow::bail!("Invalid git URL format: {url}")
    }
}

/// Convert SSH URL to HTTPS format
/// Examples:
/// - `git@github.com:user/repo.git` → `https://github.com/user/repo.git`
/// - `ssh://git@github.com/user/repo` → `https://github.com/user/repo.git`
pub fn convert_ssh_to_https(url: &str) -> Result<String> {
    let captures = SSH_URL_PATTERN
        .captures(url)
        .context("Invalid SSH URL format")?;

    // Handle both ssh:// format (groups 1,2) and git@ format (groups 3,4)
    let (host, path) = if let Some(host) = captures.get(1) {
        // ssh://git@host/path format
        (
            host.as_str(),
            captures.get(2).context("Missing path")?.as_str(),
        )
    } else {
        // git@host:path format
        (
            captures.get(3).context("Missing host")?.as_str(),
            captures.get(4).context("Missing path")?.as_str(),
        )
    };

    Ok(format!("https://{host}/{path}.git"))
}

/// Convert HTTPS URL to SSH format
/// Examples:
/// - `https://github.com/user/repo.git` → `git@github.com:user/repo.git`
/// - `https://gitlab.com/user/repo` → `git@gitlab.com:user/repo.git`
pub fn convert_https_to_ssh(url: &str) -> Result<String> {
    let captures = HTTPS_URL_PATTERN
        .captures(url)
        .context("Invalid HTTPS URL format")?;

    let host = captures
        .get(1)
        .context("Missing host in HTTPS URL")?
        .as_str();
    let path = captures
        .get(2)
        .context("Missing path in HTTPS URL")?
        .as_str();

    Ok(format!("git@{host}:{path}.git"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssh_urls() {
        assert_eq!(
            parse_git_url("git@github.com:user/repo.git").unwrap(),
            GitUrlType::Ssh
        );
        assert_eq!(
            parse_git_url("git@github.com:user/repo").unwrap(),
            GitUrlType::Ssh
        );
        assert_eq!(
            parse_git_url("ssh://git@github.com/user/repo.git").unwrap(),
            GitUrlType::Ssh
        );
    }

    #[test]
    fn test_parse_https_urls() {
        assert_eq!(
            parse_git_url("https://github.com/user/repo.git").unwrap(),
            GitUrlType::Https
        );
        assert_eq!(
            parse_git_url("https://github.com/user/repo").unwrap(),
            GitUrlType::Https
        );
        assert_eq!(
            parse_git_url("http://github.com/user/repo.git").unwrap(),
            GitUrlType::Https
        );
    }

    #[test]
    fn test_parse_invalid_url() {
        assert!(parse_git_url("not-a-url").is_err());
        assert!(parse_git_url("ftp://example.com/repo").is_err());
    }

    #[test]
    fn test_convert_ssh_to_https_github() {
        let ssh = "git@github.com:user/repo.git";
        let https = convert_ssh_to_https(ssh).unwrap();
        assert_eq!(https, "https://github.com/user/repo.git");
    }

    #[test]
    fn test_convert_ssh_to_https_without_git_extension() {
        let ssh = "git@github.com:user/repo";
        let https = convert_ssh_to_https(ssh).unwrap();
        assert_eq!(https, "https://github.com/user/repo.git");
    }

    #[test]
    fn test_convert_ssh_to_https_gitlab() {
        let ssh = "git@gitlab.com:group/subgroup/repo.git";
        let https = convert_ssh_to_https(ssh).unwrap();
        assert_eq!(https, "https://gitlab.com/group/subgroup/repo.git");
    }

    #[test]
    fn test_convert_ssh_to_https_ssh_protocol() {
        let ssh = "ssh://git@github.com/user/repo";
        let https = convert_ssh_to_https(ssh).unwrap();
        assert_eq!(https, "https://github.com/user/repo.git");
    }

    #[test]
    fn test_convert_https_to_ssh_github() {
        let https = "https://github.com/user/repo.git";
        let ssh = convert_https_to_ssh(https).unwrap();
        assert_eq!(ssh, "git@github.com:user/repo.git");
    }

    #[test]
    fn test_convert_https_to_ssh_without_git_extension() {
        let https = "https://github.com/user/repo";
        let ssh = convert_https_to_ssh(https).unwrap();
        assert_eq!(ssh, "git@github.com:user/repo.git");
    }

    #[test]
    fn test_convert_https_to_ssh_gitlab() {
        let https = "https://gitlab.com/group/subgroup/repo.git";
        let ssh = convert_https_to_ssh(https).unwrap();
        assert_eq!(ssh, "git@gitlab.com:group/subgroup/repo.git");
    }

    #[test]
    fn test_convert_https_to_ssh_bitbucket() {
        let https = "https://bitbucket.org/user/repo.git";
        let ssh = convert_https_to_ssh(https).unwrap();
        assert_eq!(ssh, "git@bitbucket.org:user/repo.git");
    }

    #[test]
    fn test_convert_invalid_ssh() {
        assert!(convert_ssh_to_https("not-a-url").is_err());
        assert!(convert_ssh_to_https("https://github.com/user/repo").is_err());
    }

    #[test]
    fn test_convert_invalid_https() {
        assert!(convert_https_to_ssh("not-a-url").is_err());
        assert!(convert_https_to_ssh("git@github.com:user/repo").is_err());
    }

    #[test]
    fn test_roundtrip_conversion() {
        let original_ssh = "git@github.com:user/repo.git";
        let https = convert_ssh_to_https(original_ssh).unwrap();
        let back_to_ssh = convert_https_to_ssh(&https).unwrap();
        assert_eq!(back_to_ssh, original_ssh);
    }
}
