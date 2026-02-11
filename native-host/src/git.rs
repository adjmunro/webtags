use anyhow::{Context, Result};
use git2::{Cred, FetchOptions, PushOptions, RemoteCallbacks, Repository, Signature};
use std::path::{Path, PathBuf};

pub struct GitRepo {
    repo: Repository,
    path: PathBuf,
}

impl GitRepo {
    /// Initialize or open a git repository
    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        let repo = if path.join(".git").exists() {
            Repository::open(&path).context("Failed to open existing repository")?
        } else {
            Repository::init(&path).context("Failed to initialize repository")?
        };

        Ok(Self { repo, path })
    }

    /// Clone a repository from a URL
    pub fn clone<P: AsRef<Path>>(url: &str, path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create parent directories")?;
        }

        let repo = Repository::clone(url, &path).context("Failed to clone repository")?;

        Ok(Self { repo, path })
    }

    /// Get the repository path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Check if the repository has a remote configured
    pub fn has_remote(&self, remote_name: &str) -> bool {
        self.repo.find_remote(remote_name).is_ok()
    }

    /// Add a remote to the repository
    pub fn add_remote(&mut self, name: &str, url: &str) -> Result<()> {
        self.repo
            .remote(name, url)
            .context("Failed to add remote")?;
        Ok(())
    }

    /// Stage a file for commit
    pub fn add_file<P: AsRef<Path>>(&self, file_path: P) -> Result<()> {
        let mut index = self
            .repo
            .index()
            .context("Failed to get repository index")?;

        // Convert to relative path from repo root
        let relative_path = if file_path.as_ref().is_absolute() {
            file_path
                .as_ref()
                .strip_prefix(&self.path)
                .context("File path is not within repository")?
        } else {
            file_path.as_ref()
        };

        index
            .add_path(relative_path)
            .context("Failed to add file to index")?;
        index.write().context("Failed to write index")?;

        Ok(())
    }

    /// Commit staged changes
    pub fn commit(&self, message: &str) -> Result<git2::Oid> {
        let mut index = self.repo.index().context("Failed to get index")?;
        let tree_id = index.write_tree().context("Failed to write tree")?;
        let tree = self
            .repo
            .find_tree(tree_id)
            .context("Failed to find tree")?;

        // Get signature (use git config or default)
        let signature = self.get_signature()?;

        // Get parent commit (if any)
        let parent_commit = match self.repo.head() {
            Ok(head) => {
                let commit = head.peel_to_commit().context("Failed to peel to commit")?;
                Some(commit)
            }
            Err(_) => None,
        };

        // Create commit
        let commit_id = if let Some(parent) = parent_commit {
            self.repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                message,
                &tree,
                &[&parent],
            )
        } else {
            // Initial commit (no parent)
            self.repo
                .commit(Some("HEAD"), &signature, &signature, message, &tree, &[])
        }
        .context("Failed to create commit")?;

        Ok(commit_id)
    }

    /// Push to remote
    pub fn push(&self, remote_name: &str, branch: &str) -> Result<()> {
        let mut remote = self
            .repo
            .find_remote(remote_name)
            .context("Failed to find remote")?;

        // Set up callbacks for authentication
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            // Try SSH key first
            if let Some(username) = username_from_url {
                if let Ok(cred) = Cred::ssh_key_from_agent(username) {
                    return Ok(cred);
                }
            }

            // Fallback to default
            Cred::default()
        });

        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        let refspec = format!("refs/heads/{}:refs/heads/{}", branch, branch);
        remote
            .push(&[&refspec], Some(&mut push_options))
            .context("Failed to push to remote")?;

        Ok(())
    }

    /// Pull from remote (with rebase)
    pub fn pull(&self, remote_name: &str, branch: &str) -> Result<()> {
        // Fetch from remote
        let mut remote = self
            .repo
            .find_remote(remote_name)
            .context("Failed to find remote")?;

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            if let Some(username) = username_from_url {
                if let Ok(cred) = Cred::ssh_key_from_agent(username) {
                    return Ok(cred);
                }
            }
            Cred::default()
        });

        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        remote
            .fetch(&[branch], Some(&mut fetch_options), None)
            .context("Failed to fetch from remote")?;

        // Get fetch head
        let fetch_head = self.repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = self.repo.reference_to_annotated_commit(&fetch_head)?;

        // Perform merge analysis
        let analysis = self.repo.merge_analysis(&[&fetch_commit])?;

        if analysis.0.is_up_to_date() {
            // Already up to date
            return Ok(());
        } else if analysis.0.is_fast_forward() {
            // Fast-forward merge
            let refname = format!("refs/heads/{}", branch);
            let mut reference = self.repo.find_reference(&refname)?;
            reference.set_target(fetch_commit.id(), "Fast-forward")?;
            self.repo.set_head(&refname)?;
            self.repo
                .checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        } else {
            // Need to merge - for now, prefer remote (simple strategy)
            // In a real implementation, we'd want conflict resolution UI
            self.repo.merge(
                &[&fetch_commit],
                None,
                Some(
                    git2::build::CheckoutBuilder::default()
                        .force()
                        .use_theirs(true),
                ),
            )?;

            // Check if merge resulted in conflicts
            let mut index = self.repo.index()?;
            if index.has_conflicts() {
                // For now, just use "theirs" strategy
                // TODO: Implement conflict resolution UI
                let conflicts: Vec<_> = index.conflicts()?.flatten().collect();
                for conflict in conflicts {
                    if let Some(their) = conflict.their {
                        index.add(&their)?;
                    }
                }
                index.write()?;
            }

            // Complete the merge with a commit
            let signature = self.get_signature()?;
            let tree_id = self.repo.index()?.write_tree()?;
            let tree = self.repo.find_tree(tree_id)?;
            let head_commit = self.repo.head()?.peel_to_commit()?;
            let fetch_commit_obj = self.repo.find_commit(fetch_commit.id())?;

            self.repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                &format!("Merge from {}/{}", remote_name, branch),
                &tree,
                &[&head_commit, &fetch_commit_obj],
            )?;

            // Clean up merge state
            self.repo.cleanup_state()?;
        }

        Ok(())
    }

    /// Get the current commit message
    pub fn get_last_commit_message(&self) -> Result<String> {
        let head = self.repo.head().context("Failed to get HEAD")?;
        let commit = head.peel_to_commit().context("Failed to peel to commit")?;
        Ok(commit.message().unwrap_or("(no message)").to_string())
    }

    /// Check if working directory is clean
    pub fn is_clean(&self) -> Result<bool> {
        let statuses = self
            .repo
            .statuses(None)
            .context("Failed to get repository status")?;
        Ok(statuses.is_empty())
    }

    /// Get signature from git config or use default
    fn get_signature(&self) -> Result<Signature<'_>> {
        let config = self.repo.config().context("Failed to get git config")?;

        let name = config
            .get_string("user.name")
            .unwrap_or_else(|_| "WebTags User".to_string());
        let email = config
            .get_string("user.email")
            .unwrap_or_else(|_| "webtags@localhost".to_string());

        Signature::now(&name, &email).context("Failed to create signature")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let file_path = dir.join(name);
        fs::write(&file_path, content).unwrap();
        file_path
    }

    #[test]
    fn test_init_new_repo() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let repo = GitRepo::init(repo_path).unwrap();
        assert_eq!(repo.path(), repo_path);
        assert!(repo_path.join(".git").exists());
    }

    #[test]
    fn test_init_existing_repo() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Initialize once
        GitRepo::init(repo_path).unwrap();

        // Initialize again (should open existing)
        let repo = GitRepo::init(repo_path).unwrap();
        assert_eq!(repo.path(), repo_path);
    }

    #[test]
    fn test_add_and_commit() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        let repo = GitRepo::init(repo_path).unwrap();

        // Create a test file
        let file_path = create_test_file(repo_path, "test.txt", "Hello, World!");

        // Add and commit
        repo.add_file("test.txt").unwrap();
        let commit_id = repo.commit("Initial commit").unwrap();

        assert!(!commit_id.is_zero());
    }

    #[test]
    fn test_is_clean() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        let repo = GitRepo::init(repo_path).unwrap();

        // Should be clean initially
        assert!(repo.is_clean().unwrap());

        // Create a file
        create_test_file(repo_path, "test.txt", "content");

        // Should not be clean
        assert!(!repo.is_clean().unwrap());

        // Add and commit
        repo.add_file("test.txt").unwrap();
        repo.commit("Add test file").unwrap();

        // Should be clean again
        assert!(repo.is_clean().unwrap());
    }

    #[test]
    fn test_add_remote() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        let mut repo = GitRepo::init(repo_path).unwrap();

        assert!(!repo.has_remote("origin"));

        repo.add_remote("origin", "https://github.com/test/repo.git")
            .unwrap();

        assert!(repo.has_remote("origin"));
    }

    #[test]
    fn test_get_last_commit_message() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        let repo = GitRepo::init(repo_path).unwrap();

        // Create and commit a file
        create_test_file(repo_path, "test.txt", "content");
        repo.add_file("test.txt").unwrap();
        repo.commit("Test commit message").unwrap();

        let message = repo.get_last_commit_message().unwrap();
        assert_eq!(message, "Test commit message");
    }

    #[test]
    fn test_multiple_commits() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        let repo = GitRepo::init(repo_path).unwrap();

        // First commit
        create_test_file(repo_path, "file1.txt", "content1");
        repo.add_file("file1.txt").unwrap();
        repo.commit("First commit").unwrap();

        // Second commit
        create_test_file(repo_path, "file2.txt", "content2");
        repo.add_file("file2.txt").unwrap();
        repo.commit("Second commit").unwrap();

        let message = repo.get_last_commit_message().unwrap();
        assert_eq!(message, "Second commit");
    }

    #[test]
    fn test_add_file_with_absolute_path() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        let repo = GitRepo::init(repo_path).unwrap();

        let file_path = create_test_file(repo_path, "test.txt", "content");

        // Add with absolute path
        repo.add_file(&file_path).unwrap();
        repo.commit("Test commit").unwrap();

        assert!(repo.is_clean().unwrap());
    }

    #[test]
    fn test_commit_without_staged_changes() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        let repo = GitRepo::init(repo_path).unwrap();

        // Create initial commit
        create_test_file(repo_path, "test.txt", "content");
        repo.add_file("test.txt").unwrap();
        repo.commit("Initial commit").unwrap();

        // Try to commit without staging anything
        // This should create an empty commit (same tree as parent)
        let result = repo.commit("Empty commit");
        // git2 doesn't prevent empty commits by default, so this should succeed
        assert!(result.is_ok());
    }

    // Note: Testing clone, push, pull requires a real git server or complex mocking
    // These would be covered in integration tests with a local git server
}
