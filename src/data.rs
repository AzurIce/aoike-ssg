#![warn(missing_docs)]
//! The data definitions of the exported vault.
//!
//! The exported vault has the following structure:
//! ```ignore
//! exported-vault/
//! ├── articles/
//! │   ├── posts/
//! │   │   └── ... # Articles and assets
//! │   ├── notes/
//! │   │   └── ... # Articles and assets
//! │   ├── ...        # Other Articles and Sections
//! │   └── index.json # The index.(md|typ) of the vault (optional)
//! └── vault.json
//! ```
//!
//! During the export process ([`crate::build::export_vault`]), aoike not only simply
//! exports [`crate::Article`] to the corresponding [`ArticleData`], but also gets through
//! the HTML content to collect links that points to local assets. The collected links will
//! be replaced with the corresponding [`EntityPath`] in the vault, and copy the assets to
//! the correct path.
use serde::{Deserialize, Serialize};

/// Represents the path and identifier of an entity within the vault.
///
/// This structure is used to uniquely identify and locate articles and sections.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityPath {
    /// A sequence of identifiers representing the path from the vault root.
    /// For example, `["posts", "tech", "rust"]`.
    pub ids: Vec<String>,
    /// The relative path to the source file or directory from the vault root.
    /// For example, `posts/tech/rust.md`.
    pub rel_path: String,
}

impl EntityPath {
    /// Returns the last identifier in the chain, which serves as the entity's own ID.
    pub fn id(&self) -> Option<&str> {
        self.ids.last().map(|s| s.as_str())
    }
    /// Returns the path string constructed by joining IDs with slashes.
    /// This is typically used for URL generation.
    pub fn ids_path(&self) -> String {
        self.ids.join("/")
    }
}

impl From<crate::EntityPath> for EntityPath {
    fn from(value: crate::EntityPath) -> Self {
        Self {
            ids: value.ids.0.iter().map(|id| id.to_string()).collect(),
            rel_path: value.rel_path.to_string(),
        }
    }
}

/// The root data structure exported to `vault.json`.
///
/// This contains the metadata for the entire vault, including all posts and the notes tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultMeta {
    /// A flat list of all blog posts, typically sorted by creation date.
    pub posts: Vec<ArticleMeta>,
    /// The root sections of the notes tree (e.g., "notes").
    pub notes: Vec<SectionMeta>,
}

/// Metadata for an article (post or note page).
///
/// This structure contains summary information used for listing and navigation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArticleMeta {
    /// The path and identifier information for this article.
    pub entity_path: EntityPath,
    /// The title of the article.
    pub title: String,
    /// A brief summary or excerpt of the article (HTML).
    pub summary: String,
    /// The creation timestamp (Unix timestamp).
    pub created: i64,
    /// The last update timestamp (Unix timestamp).
    pub updated: i64,
}

/// Metadata for a section (directory) in the vault.
///
/// A section can contain child nodes (articles or subsections) and optionally an index article.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionMeta {
    /// The path and identifier information for this section.
    pub entity_path: EntityPath,
    /// The title of the section.
    pub title: String,
    /// The child nodes contained within this section.
    pub children: Vec<NodeMeta>,
    /// The index article for this section, if one exists (e.g., `index.md`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<ArticleMeta>,
}

impl SectionMeta {
    /// Find a node with the given entity path, searching recursively through child nodes.
    pub fn find_recursive(&self, ids_path: &str) -> Option<NodeMeta> {
        if &self.entity_path.ids_path() == ids_path {
            Some(NodeMeta::Section(self.clone()))
        } else {
            self.children.iter().find_map(|child| match child {
                NodeMeta::Article(article) => {
                    if &article.entity_path.ids_path() == ids_path {
                        Some(NodeMeta::Article(article.clone()))
                    } else {
                        None
                    }
                }
                NodeMeta::Section(section) => section.find_recursive(ids_path),
            })
        }
    }
}

/// A node in the notes tree.
///
/// Can represent a container (Section) or a leaf content page (Article).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeMeta {
    /// A container node that may have children.
    Section(SectionMeta),
    /// A leaf node representing a single article.
    Article(ArticleMeta),
}

impl NodeMeta {
    /// Returns the `EntityPath` of the node.
    pub fn entity_path(&self) -> &EntityPath {
        match self {
            NodeMeta::Section(section) => &section.entity_path,
            NodeMeta::Article(article) => &article.entity_path,
        }
    }
    /// Returns the title of the node.
    pub fn title(&self) -> &str {
        match self {
            NodeMeta::Section(section) => &section.title,
            NodeMeta::Article(article) => &article.title,
        }
    }
}

/// Detailed article data exported to individual JSON files.
///
/// This includes the full content of the article in addition to its metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArticleData {
    /// The metadata of the article.
    #[serde(flatten)]
    pub meta: ArticleMeta,
    /// The full HTML content of the article.
    pub content: String,
}
