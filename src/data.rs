#![warn(missing_docs)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityPath {
    pub ids: Vec<String>,
    pub rel_path: String,
}

impl EntityPath {
    pub fn id(&self) -> Option<&str> {
        self.ids.last().map(|s| s.as_str())
    }
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultMeta {
    pub posts: Vec<ArticleMeta>,
    pub notes: Vec<SectionMeta>,
}

/// Metadata for a post, used in lists.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArticleMeta {
    pub entity_path: EntityPath,
    pub title: String,
    pub summary: String,
    pub created: i64,
    pub updated: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionMeta {
    pub entity_path: EntityPath,
    pub title: String,
    pub children: Vec<NodeMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<ArticleMeta>,
}

/// A node in the notes tree.
///
/// Can represent a container (directory) or a leaf article.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeMeta {
    Section(SectionMeta),
    Article(ArticleMeta),
}

impl NodeMeta {
    pub fn entity_path(&self) -> &EntityPath {
        match self {
            NodeMeta::Section(section) => &section.entity_path,
            NodeMeta::Article(article) => &article.entity_path,
        }
    }
    pub fn title(&self) -> &str {
        match self {
            NodeMeta::Section(section) => &section.title,
            NodeMeta::Article(article) => &article.title,
        }
    }
}

/// Detailed article data exported to individual JSON files.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArticleData {
    #[serde(flatten)]
    pub meta: ArticleMeta,
    pub content: String,
}
