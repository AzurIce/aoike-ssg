#[cfg(feature = "build")]
pub mod build;
pub mod data;

use std::fmt::Display;
use std::path::PathBuf;

pub use time;
use time::UtcDateTime;

pub use relative_path;
use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};

// MARK: Id
/// Identifier of an entity (slugified)
#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Id(pub String);

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Id {
    pub fn new(original: &str) -> Self {
        Self(slug::slugify(original))
    }
}

// MARK: EntityPath
/// Path information for an entity
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityPath {
    /// The chain of IDs from the vault root to this entity
    pub ids: Vec<Id>,
    /// The root directory of the vault (absolute path)
    pub vault_root: PathBuf,
    /// The relative path from vault root to the source file/directory
    pub path: RelativePathBuf,
}

impl EntityPath {
    pub fn id(&self) -> &Id {
        self.ids
            .last()
            .expect("EntityPath must have at least one ID")
    }
}

/// An article.
///
/// Article is the basic unit of content in a vault.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Article {
    pub entity_path: EntityPath,
    pub title: String,
    pub summary_html: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub content_html: String,
    pub created: UtcDateTime,
    pub updated: UtcDateTime,
}

impl Article {
    pub fn to_meta(&self) -> data::ArticleMeta {
        data::ArticleMeta {
            id: self.entity_path.id().0.clone(),
            ids: self.entity_path.ids.iter().map(|id| id.0.clone()).collect(),
            path: self.entity_path.path.as_str().to_string(),
            title: self.title.clone(),
            summary: self.summary_html.clone(),
            created: self.created.unix_timestamp(),
            updated: self.updated.unix_timestamp(),
        }
    }

    pub fn to_detail(&self) -> data::ArticleDetail {
        data::ArticleDetail {
            meta: self.to_meta(),
            content: self.content_html.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Container {
    pub entity_path: EntityPath,
    pub index: Option<Article>,
    pub children: Vec<Node>,
}

impl Container {
    pub fn articles(&self) -> impl Iterator<Item = &Article> {
        self.children.iter().filter_map(|n| n.as_article())
    }

    pub fn sub_containers(&self) -> impl Iterator<Item = &Container> {
        self.children.iter().filter_map(|n| n.as_container())
    }
}

/// A node in the content tree.
///
/// Represents a directory that may contain an index article, other articles,
/// and sub-nodes (subdirectories).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Node {
    Container(Container),
    Article(Article),
}

impl Node {
    pub fn entity_path(&self) -> &EntityPath {
        match self {
            Node::Container(c) => &c.entity_path,
            Node::Article(a) => &a.entity_path,
        }
    }

    pub fn id(&self) -> &Id {
        self.entity_path().id()
    }

    pub fn as_container(&self) -> Option<&Container> {
        match self {
            Node::Container(c) => Some(c),
            _ => None,
        }
    }

    pub fn as_article(&self) -> Option<&Article> {
        match self {
            Node::Article(a) => Some(a),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vault {
    pub root_dir: PathBuf,
    pub posts: Container,
    pub notes: Container,
}

impl Vault {
    pub fn export(&self) -> data::VaultData {
        // 1. Flatten posts
        let mut posts: Vec<data::ArticleMeta> =
            self.posts.articles().map(|a| a.to_meta()).collect();

        posts.sort_by(|a, b| b.created.cmp(&a.created));

        // 2. Convert notes tree
        let notes = self
            .notes
            .children
            .iter()
            .map(|node| convert_node_to_note(node))
            .collect();

        data::VaultData { posts, notes }
    }
}

fn convert_node_to_note(node: &Node) -> data::NodeMeta {
    match node {
        Node::Article(article) => data::NodeMeta {
            id: article.entity_path.id().0.clone(),
            ids: article
                .entity_path
                .ids
                .iter()
                .map(|id| id.0.clone())
                .collect(),
            path: article.entity_path.path.as_str().to_string(),
            title: article.title.clone(),
            summary: Some(article.summary_html.clone()),
            created: article.created.unix_timestamp(),
            updated: article.updated.unix_timestamp(),
            children: vec![],
        },
        Node::Container(container) => {
            let (title, summary, created, updated) = if let Some(index) = &container.index {
                (
                    index.title.clone(),
                    Some(index.summary_html.clone()),
                    index.created.unix_timestamp(),
                    index.updated.unix_timestamp(),
                )
            } else {
                (container.entity_path.id().0.clone(), None, 0, 0)
            };

            let children = container
                .children
                .iter()
                .map(|child| convert_node_to_note(child))
                .collect();

            data::NodeMeta {
                id: container.entity_path.id().0.clone(),
                ids: container
                    .entity_path
                    .ids
                    .iter()
                    .map(|id| id.0.clone())
                    .collect(),
                path: container.entity_path.path.as_str().to_string(),
                title,
                summary,
                created,
                updated,
                children,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_identifier_new() {
        let id = Id::new("Test Identifier");
        assert_eq!(id.0, "test-identifier");
        let id_a = Id::new("牛🐮逼");
        assert_eq!(id_a.0, "niu-cow-bi");
        let id_b = Id::new("?牛!🐮#逼$");
        assert_eq!(id_b.0, "niu-cow-bi");
        assert_eq!(id_a, id_b)
    }
}
