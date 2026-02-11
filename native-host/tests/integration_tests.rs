use std::fs;
use std::io::Cursor;
use tempfile::TempDir;

mod test_helpers {
    use super::*;
    use std::path::Path;

    pub fn create_test_bookmark_data() -> String {
        r##"{
            "jsonapi": {"version": "1.1"},
            "data": [
                {
                    "type": "bookmark",
                    "id": "bookmark-1",
                    "attributes": {
                        "url": "https://rust-lang.org",
                        "title": "Rust Programming Language",
                        "created": "2024-01-15T10:30:00Z"
                    },
                    "relationships": {
                        "tags": {
                            "data": [
                                {"type": "tag", "id": "tag-rust"}
                            ]
                        }
                    }
                }
            ],
            "included": [
                {
                    "type": "tag",
                    "id": "tag-rust",
                    "attributes": {
                        "name": "rust",
                        "color": "#3b82f6"
                    }
                }
            ]
        }"##
        .to_string()
    }

    pub fn setup_test_repo(temp_dir: &Path) -> std::path::PathBuf {
        let repo_path = temp_dir.join("test-repo");
        fs::create_dir_all(&repo_path).unwrap();
        repo_path
    }
}

#[test]
fn test_end_to_end_bookmark_storage() {
    use webtags_host::storage::{read_from_file, write_to_file, BookmarksData};

    let temp_dir = TempDir::new().unwrap();
    let bookmarks_file = temp_dir.path().join("bookmarks.json");

    // Parse test data
    let json_data = test_helpers::create_test_bookmark_data();
    let bookmarks_data: BookmarksData = serde_json::from_str(&json_data).unwrap();

    // Write to file
    write_to_file(&bookmarks_file, &bookmarks_data).unwrap();

    // Verify file exists
    assert!(bookmarks_file.exists());

    // Read back
    let read_data = read_from_file(&bookmarks_file).unwrap();

    // Verify data matches
    assert_eq!(read_data.jsonapi.version, "1.1");
    assert_eq!(read_data.data.len(), 1);
    assert_eq!(read_data.included.as_ref().unwrap().len(), 1);

    // Verify bookmarks
    let bookmarks = read_data.get_bookmarks();
    assert_eq!(bookmarks.len(), 1);

    // Verify tags
    let tags = read_data.get_tags();
    assert_eq!(tags.len(), 1);
}

#[test]
fn test_git_workflow_integration() {
    use webtags_host::git::GitRepo;

    let temp_dir = TempDir::new().unwrap();
    let repo_path = test_helpers::setup_test_repo(temp_dir.path());

    // Initialize repository
    let repo = GitRepo::init(&repo_path).unwrap();
    assert_eq!(repo.path(), repo_path);

    // Create a test file
    let test_file = repo_path.join("bookmarks.json");
    fs::write(&test_file, test_helpers::create_test_bookmark_data()).unwrap();

    // Add file
    repo.add_file("bookmarks.json").unwrap();

    // Commit
    let commit_id = repo.commit("Add initial bookmarks").unwrap();
    assert!(!commit_id.is_zero());

    // Verify commit message
    let message = repo.get_last_commit_message().unwrap();
    assert_eq!(message, "Add initial bookmarks");

    // Verify repository is clean
    assert!(repo.is_clean().unwrap());

    // Modify file
    fs::write(&test_file, r##"{"jsonapi":{"version":"1.1"},"data":[]}"##).unwrap();

    // Verify repository is dirty
    assert!(!repo.is_clean().unwrap());

    // Add and commit again
    repo.add_file("bookmarks.json").unwrap();
    repo.commit("Update bookmarks").unwrap();

    // Verify new commit message
    let message = repo.get_last_commit_message().unwrap();
    assert_eq!(message, "Update bookmarks");
}

#[test]
fn test_hierarchical_tags_integration() {
    use webtags_host::storage::{create_tag, BookmarksData};

    let mut data = BookmarksData::new();

    // Create hierarchy: tech -> programming -> rust
    let tech_tag = create_tag("tech".to_string(), Some("#10b981".to_string()), None);
    let tech_id = if let webtags_host::storage::Resource::Tag { id, .. } = &tech_tag {
        id.clone()
    } else {
        panic!("Expected tag");
    };
    data.add_tag(tech_tag).unwrap();

    let prog_tag = create_tag(
        "programming".to_string(),
        Some("#3b82f6".to_string()),
        Some(tech_id.clone()),
    );
    let prog_id = if let webtags_host::storage::Resource::Tag { id, .. } = &prog_tag {
        id.clone()
    } else {
        panic!("Expected tag");
    };
    data.add_tag(prog_tag).unwrap();

    let rust_tag = create_tag(
        "rust".to_string(),
        Some("#f97316".to_string()),
        Some(prog_id.clone()),
    );
    let rust_id = if let webtags_host::storage::Resource::Tag { id, .. } = &rust_tag {
        id.clone()
    } else {
        panic!("Expected tag");
    };
    data.add_tag(rust_tag).unwrap();

    // Test hierarchy
    let hierarchy = data.get_tag_hierarchy();
    assert!(hierarchy.contains_key(&tech_id));
    assert!(hierarchy.contains_key(&prog_id));

    let tech_children = hierarchy.get(&tech_id).unwrap();
    assert_eq!(tech_children.len(), 1);
    assert_eq!(tech_children[0], prog_id);

    let prog_children = hierarchy.get(&prog_id).unwrap();
    assert_eq!(prog_children.len(), 1);
    assert_eq!(prog_children[0], rust_id);

    // Test breadcrumb
    let breadcrumb = data.get_tag_breadcrumb(&rust_id);
    assert_eq!(breadcrumb, vec!["tech", "programming", "rust"]);

    // Test serialization roundtrip
    let json = serde_json::to_string_pretty(&data).unwrap();
    let parsed: BookmarksData = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.get_tags().len(), 3);

    let parsed_breadcrumb = parsed.get_tag_breadcrumb(&rust_id);
    assert_eq!(parsed_breadcrumb, breadcrumb);
}

#[test]
fn test_bookmark_with_tags_integration() {
    use webtags_host::storage::{create_bookmark, create_tag, write_to_file, BookmarksData};

    let temp_dir = TempDir::new().unwrap();
    let bookmarks_file = temp_dir.path().join("bookmarks.json");

    let mut data = BookmarksData::new();

    // Create tags
    let tag1 = create_tag("rust".to_string(), Some("#3b82f6".to_string()), None);
    let tag1_id = if let webtags_host::storage::Resource::Tag { id, .. } = &tag1 {
        id.clone()
    } else {
        panic!("Expected tag");
    };
    data.add_tag(tag1).unwrap();

    let tag2 = create_tag("programming".to_string(), Some("#10b981".to_string()), None);
    let tag2_id = if let webtags_host::storage::Resource::Tag { id, .. } = &tag2 {
        id.clone()
    } else {
        panic!("Expected tag");
    };
    data.add_tag(tag2).unwrap();

    // Create bookmark with tags
    let bookmark = create_bookmark(
        "https://rust-lang.org".to_string(),
        "Rust Programming Language".to_string(),
        vec![tag1_id.clone(), tag2_id.clone()],
    );
    data.add_bookmark(bookmark).unwrap();

    // Validate data
    data.validate().unwrap();

    // Write to file
    write_to_file(&bookmarks_file, &data).unwrap();

    // Read back
    let content = fs::read_to_string(&bookmarks_file).unwrap();
    let read_data: BookmarksData = serde_json::from_str(&content).unwrap();

    // Verify structure
    assert_eq!(read_data.data.len(), 1);
    assert_eq!(read_data.included.as_ref().unwrap().len(), 2);

    // Verify bookmark has relationships
    if let webtags_host::storage::Resource::Bookmark {
        relationships, ..
    } = &read_data.data[0]
    {
        assert!(relationships.is_some());
        let tags = relationships.as_ref().unwrap().tags.as_ref().unwrap();
        assert_eq!(tags.data.len(), 2);
    } else {
        panic!("Expected bookmark");
    }
}

#[test]
fn test_native_messaging_protocol_integration() {
    use webtags_host::messaging::{read_message, write_response, Message, Response};

    // Test init message
    let init_msg = Message::Init {
        repo_path: Some("/tmp/test".to_string()),
        repo_url: None,
    };
    let json = serde_json::to_vec(&init_msg).unwrap();
    let length = (json.len() as u32).to_le_bytes();

    let mut input = Vec::new();
    input.extend_from_slice(&length);
    input.extend_from_slice(&json);

    let cursor = Cursor::new(input);
    let parsed = read_message(cursor).unwrap();
    assert_eq!(parsed, init_msg);

    // Test response writing
    let response = Response::Success {
        message: "Test success".to_string(),
        data: None,
    };

    let mut output = Vec::new();
    write_response(&mut output, &response).unwrap();

    // Verify response format
    assert!(output.len() > 4);
    let response_length = u32::from_le_bytes([output[0], output[1], output[2], output[3]]);
    assert_eq!(response_length as usize, output.len() - 4);

    // Parse response back
    let json_bytes = &output[4..];
    let parsed_response: Response = serde_json::from_slice(json_bytes).unwrap();
    assert_eq!(parsed_response, response);
}

#[test]
fn test_multiple_bookmarks_and_tags() {
    use webtags_host::storage::{create_bookmark, create_tag, BookmarksData};

    let mut data = BookmarksData::new();

    // Create multiple tags with hierarchy
    let web_tag = create_tag("web".to_string(), None, None);
    let web_id = if let webtags_host::storage::Resource::Tag { id, .. } = &web_tag {
        id.clone()
    } else {
        panic!("Expected tag");
    };
    data.add_tag(web_tag).unwrap();

    let frontend_tag = create_tag("frontend".to_string(), None, Some(web_id.clone()));
    let frontend_id = if let webtags_host::storage::Resource::Tag { id, .. } = &frontend_tag {
        id.clone()
    } else {
        panic!("Expected tag");
    };
    data.add_tag(frontend_tag).unwrap();

    let backend_tag = create_tag("backend".to_string(), None, Some(web_id.clone()));
    let backend_id = if let webtags_host::storage::Resource::Tag { id, .. } = &backend_tag {
        id.clone()
    } else {
        panic!("Expected tag");
    };
    data.add_tag(backend_tag).unwrap();

    // Create multiple bookmarks
    let bookmark1 = create_bookmark(
        "https://react.dev".to_string(),
        "React".to_string(),
        vec![frontend_id.clone()],
    );
    data.add_bookmark(bookmark1).unwrap();

    let bookmark2 = create_bookmark(
        "https://nodejs.org".to_string(),
        "Node.js".to_string(),
        vec![backend_id.clone()],
    );
    data.add_bookmark(bookmark2).unwrap();

    let bookmark3 = create_bookmark(
        "https://developer.mozilla.org".to_string(),
        "MDN Web Docs".to_string(),
        vec![frontend_id.clone(), backend_id.clone()],
    );
    data.add_bookmark(bookmark3).unwrap();

    // Validate
    data.validate().unwrap();

    // Verify counts
    assert_eq!(data.get_bookmarks().len(), 3);
    assert_eq!(data.get_tags().len(), 3);

    // Verify hierarchy
    let hierarchy = data.get_tag_hierarchy();
    assert_eq!(hierarchy.get(&web_id).unwrap().len(), 2); // frontend and backend

    // Verify breadcrumbs
    let frontend_breadcrumb = data.get_tag_breadcrumb(&frontend_id);
    assert_eq!(frontend_breadcrumb, vec!["web", "frontend"]);

    let backend_breadcrumb = data.get_tag_breadcrumb(&backend_id);
    assert_eq!(backend_breadcrumb, vec!["web", "backend"]);
}

#[test]
fn test_git_and_storage_integration() {
    use webtags_host::git::GitRepo;
    use webtags_host::storage::{create_bookmark, write_to_file, BookmarksData};

    let temp_dir = TempDir::new().unwrap();
    let repo_path = test_helpers::setup_test_repo(temp_dir.path());

    // Initialize Git repo
    let repo = GitRepo::init(&repo_path).unwrap();

    // Create bookmark data
    let mut data = BookmarksData::new();
    let bookmark = create_bookmark(
        "https://example.com".to_string(),
        "Example Site".to_string(),
        vec![],
    );
    data.add_bookmark(bookmark).unwrap();

    // Write to file
    let bookmarks_file = repo_path.join("bookmarks.json");
    write_to_file(&bookmarks_file, &data).unwrap();

    // Add and commit
    repo.add_file("bookmarks.json").unwrap();
    let commit_msg = format!("Update bookmarks: {} bookmarks, {} tags",
        data.get_bookmarks().len(),
        data.get_tags().len()
    );
    repo.commit(&commit_msg).unwrap();

    // Verify commit message includes stats
    let last_msg = repo.get_last_commit_message().unwrap();
    assert!(last_msg.contains("1 bookmarks"));
    assert!(last_msg.contains("0 tags"));

    // Add another bookmark
    let bookmark2 = create_bookmark(
        "https://example.org".to_string(),
        "Another Example".to_string(),
        vec![],
    );
    data.add_bookmark(bookmark2).unwrap();

    write_to_file(&bookmarks_file, &data).unwrap();
    repo.add_file("bookmarks.json").unwrap();

    let commit_msg2 = format!("Update bookmarks: {} bookmarks, {} tags",
        data.get_bookmarks().len(),
        data.get_tags().len()
    );
    repo.commit(&commit_msg2).unwrap();

    let last_msg2 = repo.get_last_commit_message().unwrap();
    assert!(last_msg2.contains("2 bookmarks"));
}

#[test]
fn test_error_handling_invalid_json() {
    use webtags_host::storage::read_from_file;

    let temp_dir = TempDir::new().unwrap();
    let invalid_file = temp_dir.path().join("invalid.json");

    // Write invalid JSON
    fs::write(&invalid_file, "not valid json at all").unwrap();

    // Should error
    let result = read_from_file(&invalid_file);
    assert!(result.is_err());
}

#[test]
fn test_error_handling_missing_file() {
    use webtags_host::storage::read_from_file;

    let temp_dir = TempDir::new().unwrap();
    let missing_file = temp_dir.path().join("nonexistent.json");

    // Should error
    let result = read_from_file(&missing_file);
    assert!(result.is_err());
}

#[test]
fn test_atomic_write_safety() {
    use webtags_host::storage::{write_to_file, BookmarksData};

    let temp_dir = TempDir::new().unwrap();
    let bookmarks_file = temp_dir.path().join("bookmarks.json");

    let data = BookmarksData::new();

    // Write file
    write_to_file(&bookmarks_file, &data).unwrap();

    // Verify temp file was cleaned up
    let temp_file = temp_dir.path().join("bookmarks.tmp");
    assert!(!temp_file.exists());

    // Verify target file exists
    assert!(bookmarks_file.exists());

    // Overwrite with new data
    write_to_file(&bookmarks_file, &data).unwrap();

    // Verify still no temp file
    assert!(!temp_file.exists());
}
