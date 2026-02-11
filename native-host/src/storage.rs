use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use url::Url;
use uuid::Uuid;

/// Validate bookmark URL for security
fn validate_bookmark_url(url_str: &str) -> Result<()> {
    // Check length
    if url_str.is_empty() {
        anyhow::bail!("URL cannot be empty");
    }
    if url_str.len() > 2048 {
        anyhow::bail!("URL too long (max 2048 characters)");
    }

    // Parse URL
    let parsed = Url::parse(url_str).context("Invalid URL format")?;

    // Only allow safe schemes
    match parsed.scheme() {
        "http" | "https" => Ok(()),
        scheme => anyhow::bail!("Unsafe URL scheme '{scheme}'. Only http and https are allowed."),
    }
}

/// JSON API v1.1 compliant data structure
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BookmarksData {
    pub jsonapi: JsonApiVersion,
    pub data: Vec<Resource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub included: Option<Vec<Resource>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct JsonApiVersion {
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Resource {
    Bookmark {
        id: String,
        attributes: BookmarkAttributes,
        #[serde(skip_serializing_if = "Option::is_none")]
        relationships: Option<BookmarkRelationships>,
    },
    Tag {
        id: String,
        attributes: TagAttributes,
        #[serde(skip_serializing_if = "Option::is_none")]
        relationships: Option<TagRelationships>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BookmarkAttributes {
    pub url: String,
    pub title: String,
    pub created: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BookmarkRelationships {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<RelationshipData>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RelationshipData {
    pub data: Vec<ResourceIdentifier>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ResourceIdentifier {
    #[serde(rename = "type")]
    pub resource_type: String,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TagAttributes {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TagRelationships {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<ParentRelationship>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ParentRelationship {
    pub data: Option<ResourceIdentifier>,
}

impl BookmarksData {
    /// Create a new empty `BookmarksData` structure
    pub fn new() -> Self {
        Self {
            jsonapi: JsonApiVersion {
                version: "1.1".to_string(),
            },
            data: Vec::new(),
            included: None,
        }
    }

    /// Add a bookmark to the data
    pub fn add_bookmark(&mut self, bookmark: Resource) -> Result<()> {
        match bookmark {
            Resource::Bookmark { .. } => {
                self.data.push(bookmark);
                Ok(())
            }
            Resource::Tag { .. } => anyhow::bail!("Expected bookmark resource"),
        }
    }

    /// Add a tag to the included section
    pub fn add_tag(&mut self, tag: Resource) -> Result<()> {
        match tag {
            Resource::Tag { .. } => {
                if self.included.is_none() {
                    self.included = Some(Vec::new());
                }
                if let Some(included) = &mut self.included {
                    included.push(tag);
                }
                Ok(())
            }
            Resource::Bookmark { .. } => anyhow::bail!("Expected tag resource"),
        }
    }

    /// Get all bookmarks
    pub fn get_bookmarks(&self) -> Vec<&Resource> {
        self.data
            .iter()
            .filter(|r| matches!(r, Resource::Bookmark { .. }))
            .collect()
    }

    /// Get all tags (from both data and included)
    pub fn get_tags(&self) -> Vec<&Resource> {
        let mut tags = Vec::new();

        // Tags from data section
        tags.extend(
            self.data
                .iter()
                .filter(|r| matches!(r, Resource::Tag { .. })),
        );

        // Tags from included section
        if let Some(included) = &self.included {
            tags.extend(
                included
                    .iter()
                    .filter(|r| matches!(r, Resource::Tag { .. })),
            );
        }

        tags
    }

    /// Get tag hierarchy (parent-child relationships)
    pub fn get_tag_hierarchy(&self) -> HashMap<String, Vec<String>> {
        let mut hierarchy: HashMap<String, Vec<String>> = HashMap::new();

        for tag in self.get_tags() {
            if let Resource::Tag {
                id,
                relationships: Some(rels),
                ..
            } = tag
            {
                if let Some(parent_rel) = &rels.parent {
                    if let Some(parent_id) = &parent_rel.data {
                        hierarchy
                            .entry(parent_id.id.clone())
                            .or_default()
                            .push(id.clone());
                    }
                }
            }
        }

        hierarchy
    }

    /// Get breadcrumb path for a tag (e.g., `["tech", "programming", "rust"]`)
    pub fn get_tag_breadcrumb(&self, tag_id: &str) -> Vec<String> {
        let mut breadcrumb = Vec::new();
        let tags_by_id: HashMap<String, &Resource> = self
            .get_tags()
            .into_iter()
            .filter_map(|t| {
                if let Resource::Tag { id, .. } = t {
                    Some((id.clone(), t))
                } else {
                    None
                }
            })
            .collect();

        let mut current_id = tag_id.to_string();
        let mut visited = std::collections::HashSet::new();

        // Traverse up the hierarchy
        loop {
            if visited.contains(&current_id) {
                // Circular reference detected
                break;
            }
            visited.insert(current_id.clone());

            if let Some(Resource::Tag {
                attributes,
                relationships,
                ..
            }) = tags_by_id.get(&current_id)
            {
                breadcrumb.insert(0, attributes.name.clone());

                // Check for parent
                if let Some(rels) = relationships {
                    if let Some(parent_rel) = &rels.parent {
                        if let Some(parent_id) = &parent_rel.data {
                            current_id = parent_id.id.clone();
                            continue;
                        }
                    }
                }
            }
            break;
        }

        breadcrumb
    }

    /// Validate the data structure against JSON API v1.1 spec
    pub fn validate(&self) -> Result<()> {
        // Check version
        if self.jsonapi.version != "1.1" {
            anyhow::bail!("Invalid JSON API version: {}", self.jsonapi.version);
        }

        // Validate all resources have unique IDs and valid data
        let mut ids = std::collections::HashSet::new();
        for resource in &self.data {
            let id = match resource {
                Resource::Bookmark { id, attributes, .. } => {
                    // Validate bookmark URL
                    validate_bookmark_url(&attributes.url)?;
                    // Validate title length
                    if attributes.title.len() > 500 {
                        anyhow::bail!("Bookmark title too long (max 500 characters)");
                    }
                    id
                }
                Resource::Tag { id, attributes, .. } => {
                    // Validate tag name
                    if attributes.name.is_empty() || attributes.name.len() > 100 {
                        anyhow::bail!("Tag name must be between 1-100 characters");
                    }
                    // Validate tag name doesn't contain HTML
                    if attributes.name.contains('<') || attributes.name.contains('>') {
                        anyhow::bail!("Tag name cannot contain HTML characters");
                    }
                    id
                }
            };
            if !ids.insert(id) {
                anyhow::bail!("Duplicate resource ID: {id}");
            }
        }

        if let Some(included) = &self.included {
            for resource in included {
                let id = match resource {
                    Resource::Bookmark { id, .. } | Resource::Tag { id, .. } => id,
                };
                if !ids.insert(id) {
                    anyhow::bail!("Duplicate resource ID: {id}");
                }
            }
        }

        Ok(())
    }
}

impl Default for BookmarksData {
    fn default() -> Self {
        Self::new()
    }
}

/// Read bookmarks data from a file (handles both plain and encrypted)
pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<BookmarksData> {
    read_from_file_with_encryption(path, false)
}

/// Read bookmarks data from a file with optional encryption support
pub fn read_from_file_with_encryption<P: AsRef<Path>>(
    path: P,
    encryption_enabled: bool,
) -> Result<BookmarksData> {
    use crate::encryption::{is_encrypted, EncryptionManager};

    let path_ref = path.as_ref();

    // Check if file is encrypted
    let file_encrypted = is_encrypted(path_ref).unwrap_or(false);

    let content = if file_encrypted {
        // File is encrypted, decrypt it
        if !encryption_enabled {
            anyhow::bail!(
                "Bookmarks file is encrypted but encryption is not enabled. \
                 Enable encryption to access your bookmarks."
            );
        }

        let manager = EncryptionManager::new(true);
        let decrypted_bytes = manager.read_encrypted_file(path_ref).context(
            "Failed to decrypt bookmarks file. Touch ID authentication may be required.",
        )?;

        String::from_utf8(decrypted_bytes).context("Decrypted data is not valid UTF-8")?
    } else {
        // File is plain text
        fs::read_to_string(path_ref).context("Failed to read bookmarks file")?
    };

    let data: BookmarksData =
        serde_json::from_str(&content).context("Failed to parse bookmarks JSON")?;
    data.validate()?;
    Ok(data)
}

/// Write bookmarks data to a file atomically (plain text)
pub fn write_to_file<P: AsRef<Path>>(path: P, data: &BookmarksData) -> Result<()> {
    write_to_file_with_encryption(path, data, false)
}

/// Write bookmarks data to a file with optional encryption
pub fn write_to_file_with_encryption<P: AsRef<Path>>(
    path: P,
    data: &BookmarksData,
    encryption_enabled: bool,
) -> Result<()> {
    use crate::encryption::EncryptionManager;

    data.validate()?;

    let path_ref = path.as_ref();

    if encryption_enabled {
        // Encrypt the data
        let manager = EncryptionManager::new(true);

        // Serialize to JSON first
        let json =
            serde_json::to_string_pretty(data).context("Failed to serialize bookmarks data")?;

        // Encrypt and write
        manager
            .write_encrypted_file(path_ref, json.as_bytes())
            .context(
                "Failed to write encrypted bookmarks. Touch ID authentication may be required.",
            )?;

        log::info!("Bookmarks written (encrypted)");
    } else {
        // Write as plain text
        let json =
            serde_json::to_string_pretty(data).context("Failed to serialize bookmarks data")?;

        // Atomic write: write to temp file, then rename
        let temp_path = path_ref.with_extension("tmp");
        fs::write(&temp_path, json).context("Failed to write temp file")?;
        fs::rename(&temp_path, path_ref).context("Failed to rename temp file to target")?;

        log::info!("Bookmarks written (plain text)");
    }

    Ok(())
}

/// Helper to create a new bookmark resource
pub fn create_bookmark(url: String, title: String, tag_ids: Vec<String>) -> Resource {
    let now = Utc::now();
    Resource::Bookmark {
        id: Uuid::new_v4().to_string(),
        attributes: BookmarkAttributes {
            url,
            title,
            created: now,
            modified: None,
            notes: None,
        },
        relationships: if tag_ids.is_empty() {
            None
        } else {
            Some(BookmarkRelationships {
                tags: Some(RelationshipData {
                    data: tag_ids
                        .into_iter()
                        .map(|id| ResourceIdentifier {
                            resource_type: "tag".to_string(),
                            id,
                        })
                        .collect(),
                }),
            })
        },
    }
}

/// Helper to create a new tag resource
pub fn create_tag(name: String, color: Option<String>, parent_id: Option<String>) -> Resource {
    Resource::Tag {
        id: Uuid::new_v4().to_string(),
        attributes: TagAttributes {
            name,
            color,
            description: None,
        },
        relationships: parent_id.map(|pid| TagRelationships {
            parent: Some(ParentRelationship {
                data: Some(ResourceIdentifier {
                    resource_type: "tag".to_string(),
                    id: pid,
                }),
            }),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_new_bookmarks_data() {
        let data = BookmarksData::new();
        assert_eq!(data.jsonapi.version, "1.1");
        assert!(data.data.is_empty());
        assert!(data.included.is_none());
    }

    #[test]
    fn test_add_bookmark() {
        let mut data = BookmarksData::new();
        let bookmark = create_bookmark(
            "https://example.com".to_string(),
            "Example".to_string(),
            vec![],
        );
        data.add_bookmark(bookmark).unwrap();
        assert_eq!(data.data.len(), 1);
    }

    #[test]
    fn test_add_tag() {
        let mut data = BookmarksData::new();
        let tag = create_tag("rust".to_string(), Some("#3b82f6".to_string()), None);
        data.add_tag(tag).unwrap();
        assert!(data.included.is_some());
        assert_eq!(data.included.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_hierarchical_tags() {
        let mut data = BookmarksData::new();

        // Create parent tag
        let parent_tag = create_tag("programming".to_string(), None, None);
        let parent_id = if let Resource::Tag { id, .. } = &parent_tag {
            id.clone()
        } else {
            panic!("Expected tag");
        };
        data.add_tag(parent_tag).unwrap();

        // Create child tag
        let child_tag = create_tag("rust".to_string(), None, Some(parent_id.clone()));
        data.add_tag(child_tag).unwrap();

        let hierarchy = data.get_tag_hierarchy();
        assert!(hierarchy.contains_key(&parent_id));
        assert_eq!(hierarchy.get(&parent_id).unwrap().len(), 1);
    }

    #[test]
    fn test_tag_breadcrumb() {
        let mut data = BookmarksData::new();

        // Create hierarchy: tech -> programming -> rust
        let tech_tag = create_tag("tech".to_string(), None, None);
        let tech_id = if let Resource::Tag { id, .. } = &tech_tag {
            id.clone()
        } else {
            panic!("Expected tag");
        };
        data.add_tag(tech_tag).unwrap();

        let prog_tag = create_tag("programming".to_string(), None, Some(tech_id.clone()));
        let prog_id = if let Resource::Tag { id, .. } = &prog_tag {
            id.clone()
        } else {
            panic!("Expected tag");
        };
        data.add_tag(prog_tag).unwrap();

        let rust_tag = create_tag("rust".to_string(), None, Some(prog_id.clone()));
        let rust_id = if let Resource::Tag { id, .. } = &rust_tag {
            id.clone()
        } else {
            panic!("Expected tag");
        };
        data.add_tag(rust_tag).unwrap();

        let breadcrumb = data.get_tag_breadcrumb(&rust_id);
        assert_eq!(breadcrumb, vec!["tech", "programming", "rust"]);
    }

    #[test]
    fn test_validate_duplicate_ids() {
        let mut data = BookmarksData::new();
        let bookmark1 = Resource::Bookmark {
            id: "same-id".to_string(),
            attributes: BookmarkAttributes {
                url: "https://example.com".to_string(),
                title: "Example".to_string(),
                created: Utc::now(),
                modified: None,
                notes: None,
            },
            relationships: None,
        };
        let bookmark2 = Resource::Bookmark {
            id: "same-id".to_string(),
            attributes: BookmarkAttributes {
                url: "https://example2.com".to_string(),
                title: "Example 2".to_string(),
                created: Utc::now(),
                modified: None,
                notes: None,
            },
            relationships: None,
        };

        data.data.push(bookmark1);
        data.data.push(bookmark2);

        assert!(data.validate().is_err());
    }

    #[test]
    fn test_json_serialization() {
        let mut data = BookmarksData::new();
        let bookmark = create_bookmark(
            "https://rust-lang.org".to_string(),
            "Rust Programming Language".to_string(),
            vec!["tag-1".to_string()],
        );
        data.add_bookmark(bookmark).unwrap();

        let tag = create_tag("rust".to_string(), Some("#3b82f6".to_string()), None);
        data.add_tag(tag).unwrap();

        // Serialize
        let json = serde_json::to_string_pretty(&data).unwrap();

        // Deserialize
        let parsed: BookmarksData = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.data.len(), 1);
        assert_eq!(parsed.included.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_read_write_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let mut data = BookmarksData::new();
        let bookmark = create_bookmark(
            "https://example.com".to_string(),
            "Example".to_string(),
            vec![],
        );
        data.add_bookmark(bookmark).unwrap();

        // Write
        write_to_file(path, &data).unwrap();

        // Read
        let read_data = read_from_file(path).unwrap();
        assert_eq!(read_data.data.len(), 1);
    }

    #[test]
    fn test_atomic_write() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let data = BookmarksData::new();

        // First write
        write_to_file(path, &data).unwrap();

        // Verify temp file is cleaned up
        let temp_path = path.with_extension("tmp");
        assert!(!temp_path.exists());

        // Verify target file exists
        assert!(path.exists());
    }

    #[test]
    fn test_get_bookmarks_only() {
        let mut data = BookmarksData::new();
        let bookmark = create_bookmark(
            "https://example.com".to_string(),
            "Example".to_string(),
            vec![],
        );
        data.add_bookmark(bookmark).unwrap();

        let tag = create_tag("test".to_string(), None, None);
        data.add_tag(tag).unwrap();

        let bookmarks = data.get_bookmarks();
        assert_eq!(bookmarks.len(), 1);
    }

    #[test]
    fn test_circular_reference_in_breadcrumb() {
        let mut data = BookmarksData::new();

        // Create circular reference: tag1 -> tag2 -> tag1
        let tag1 = Resource::Tag {
            id: "tag1".to_string(),
            attributes: TagAttributes {
                name: "Tag 1".to_string(),
                color: None,
                description: None,
            },
            relationships: Some(TagRelationships {
                parent: Some(ParentRelationship {
                    data: Some(ResourceIdentifier {
                        resource_type: "tag".to_string(),
                        id: "tag2".to_string(),
                    }),
                }),
            }),
        };

        let tag2 = Resource::Tag {
            id: "tag2".to_string(),
            attributes: TagAttributes {
                name: "Tag 2".to_string(),
                color: None,
                description: None,
            },
            relationships: Some(TagRelationships {
                parent: Some(ParentRelationship {
                    data: Some(ResourceIdentifier {
                        resource_type: "tag".to_string(),
                        id: "tag1".to_string(),
                    }),
                }),
            }),
        };

        data.add_tag(tag1).unwrap();
        data.add_tag(tag2).unwrap();

        // Should not infinite loop
        let breadcrumb = data.get_tag_breadcrumb("tag1");
        assert!(!breadcrumb.is_empty());
    }
}
