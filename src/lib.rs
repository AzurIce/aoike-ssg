#[cfg(feature = "build")]
pub mod build;

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
    original: String,
    slug: String,
}

impl std::fmt::Display for Id {
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
    pub title: String,
    pub slug: String,
    pub path: Vec<Id>,
    pub summary_html: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub content_html: String,
    pub created: UtcDateTime,
    pub updated: UtcDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: Id,
    pub index: Article,
    pub articles: Vec<Article>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Posts {
    pub index: Option<Article>,
    pub articles: Vec<Article>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notes {
    pub index: Option<Article>,
    pub notes: Vec<Note>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vault {
    pub posts: Posts,
    pub notes: Notes,
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