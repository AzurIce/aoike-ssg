#[cfg(feature = "build")]
pub mod build;

use std::fmt::Display;

pub use time;
use time::UtcDateTime;

use serde::{Deserialize, Serialize};

// MARK: Id
/// Identifier of an entity
///
/// The original identifier may contains non-ASCII characters, which is not
/// friendly to urls. So the slugified version is used for comparison.
#[derive(Eq, Debug, Clone, Serialize, Deserialize)]
pub struct Id {
    pub original: String,
    pub slug: String,
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.slug)
    }
}

impl PartialEq for Id {
    fn eq(&self, other: &Self) -> bool {
        self.slug.eq(&other.slug)
    }
}

impl Id {
    pub fn new(original: String) -> Self {
        let slug = slug::slugify(&original);
        Self { original, slug }
    }
}

/// An article.
///
/// Article is the basic unit of content in a vault.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Article {
    pub id: Id,
    pub path: Vec<Id>,
    pub title: String,
    pub summary_html: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub content_html: String,
    pub created: UtcDateTime,
    pub updated: UtcDateTime,
}

impl Article {
    pub fn strip_content(&self) -> Self {
        let mut clone = self.clone();
        clone.content_html.clear();
        clone
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Container {
    pub id: Id,
    pub path: Vec<Id>,
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

    pub fn strip_content(&self) -> Self {
        let mut clone = self.clone();
        if let Some(idx) = &mut clone.index {
            idx.content_html.clear();
        }
        clone.children = clone.children.iter().map(|n| n.strip_content()).collect();
        clone
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
    pub fn id(&self) -> &Id {
        match self {
            Node::Container(c) => &c.id,
            Node::Article(a) => &a.id,
        }
    }

    pub fn path(&self) -> &[Id] {
        match self {
            Node::Container(c) => &c.path,
            Node::Article(a) => &a.path,
        }
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

    pub fn strip_content(&self) -> Self {
        match self {
            Node::Container(c) => Node::Container(c.strip_content()),
            Node::Article(a) => Node::Article(a.strip_content()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vault {
    pub posts: Container,
    pub notes: Container,
}

impl Vault {
    pub fn export(&self) -> ExportedVault {
        // Handle Posts
        let exported_posts = ExportedPosts {
            index: self.posts.index.as_ref().map(Article::strip_content),
            articles: self.posts.articles().map(Article::strip_content).collect(),
        };

        // Handle Notes
        let exported_notes = ExportedNotes {
            index: self.notes.index.as_ref().map(Article::strip_content),
            // For notes, we export the sub-containers (notebooks)
            // The recursion in Container::strip_content will handle the tree structure inside them
            notes: self
                .notes
                .sub_containers()
                .map(Container::strip_content)
                .collect(),
        };

        ExportedVault {
            posts: exported_posts,
            notes: exported_notes,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExportedPosts {
    pub index: Option<Article>,
    pub articles: Vec<Article>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExportedNotes {
    pub index: Option<Article>,
    pub notes: Vec<Container>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExportedVault {
    pub posts: ExportedPosts,
    pub notes: ExportedNotes,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_identifier_new() {
        let id = Id::new("Test Identifier".to_string());
        assert_eq!(id.slug, "test-identifier");
        let id_a = Id::new("牛🐮逼".to_string());
        assert_eq!(id_a.slug, "niu-cow-bi");
        let id_b = Id::new("?牛!🐮#逼$".to_string());
        assert_eq!(id_b.slug, "niu-cow-bi");
        assert_eq!(id_a, id_b)
    }
}
