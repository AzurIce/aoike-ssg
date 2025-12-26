use serde::{Deserialize, Serialize};

/// The root data structure exported to `vault.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultData {
    pub posts: Vec<ArticleMeta>,
    pub notes: Vec<NodeMeta>,
}

/// Metadata for a post, used in lists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleMeta {
    pub id: String,
    pub ids: Vec<String>,
    pub path: String,
    pub title: String,
    pub summary: String,
    pub created: i64,
    pub updated: i64,
}

/// A node in the notes tree.
///
/// Can represent a container (directory) or a leaf article.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMeta {
    pub id: String,
    pub ids: Vec<String>,
    pub path: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub created: i64,
    pub updated: i64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<NodeMeta>,
}

/// Detailed article data exported to individual JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleDetail {
    #[serde(flatten)]
    pub meta: ArticleMeta,
    pub content: String,
}
