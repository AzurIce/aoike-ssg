#[cfg(feature = "build")]
pub mod build;
pub mod data;

use std::ops::DerefMut;
use std::path::PathBuf;
use std::{fmt::Display, ops::Deref};

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ids(Vec<Id>);

impl Ids {
    pub fn root() -> Self {
        Ids(vec![])
    }
    pub fn new(ids: &[Id]) -> Self {
        Self(ids.to_vec())
    }
    pub fn parent(&self) -> Option<Ids> {
        if self.len() <= 1 {
            None
        } else {
            Some(Self(self.0[1..].to_vec()))
        }
    }
}

impl<S: AsRef<str>> From<S> for Ids {
    fn from(value: S) -> Self {
        Self(
            value
                .as_ref()
                .trim_start_matches("/")
                .trim_end_matches("/")
                .split("/")
                .map(|s| Id(slug::slugify(s)))
                .collect::<Vec<_>>(),
        )
    }
}

impl Deref for Ids {
    type Target = Vec<Id>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Ids {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Ids {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, id) in self.iter().enumerate() {
            if i > 0 {
                write!(f, "/")?;
            }
            write!(f, "{}", id)?;
        }
        Ok(())
    }
}

// MARK: EntityPath
/// Path information for an entity
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityPath {
    /// The chain of IDs from the vault root to this entity
    pub ids: Ids,
    /// The root directory of the vault (absolute path)
    pub vault_root: PathBuf,
    /// The relative path from vault root to the source file/directory
    pub rel_path: RelativePathBuf,
}

impl EntityPath {
    pub fn new(vault_root: PathBuf, rel_path: RelativePathBuf) -> Self {
        // TODO: Make it safer, now Ids from may failed when rel_path is invalid
        let ids = Ids::from(rel_path.with_extension("").as_str());
        Self {
            ids,
            vault_root,
            rel_path,
        }
    }
    pub fn id(&self) -> &Id {
        self.ids
            .last()
            .expect("EntityPath must have at least one ID")
    }
}

/// An article.
///
/// Article is the basic unit of content in a vault.
#[derive(Clone, Debug)]
pub struct Article {
    pub entity_path: EntityPath,
    pub title: String,
    pub summary_html: String,
    pub content_html: String,
    pub created: UtcDateTime,
    pub updated: UtcDateTime,
}

impl Article {
    pub fn to_meta(&self) -> data::ArticleMeta {
        data::ArticleMeta {
            id: self.entity_path.id().0.clone(),
            ids: self.entity_path.ids.iter().map(|id| id.0.clone()).collect(),
            path: self.entity_path.rel_path.as_str().to_string(),
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

#[derive(Clone, Debug)]
pub struct Section {
    pub entity_path: EntityPath,
    pub children: Vec<Node>,
}

impl Section {
    pub fn children(&self) -> impl Iterator<Item = &Node> {
        self.children.iter()
    }
    pub fn articles(&self) -> impl Iterator<Item = &Article> {
        self.children.iter().filter_map(|n| n.as_article())
    }
    pub fn sub_sections(&self) -> impl Iterator<Item = &Section> {
        self.children.iter().filter_map(|n| n.as_section())
    }
    pub fn index(&self) -> Option<&Article> {
        self.get(&Id::new("index")).and_then(|n| n.as_article())
    }
    pub fn get(&self, id: &Id) -> Option<&Node> {
        self.children.iter().find(|n| n.id() == id)
    }
    pub fn to_meta(&self) -> data::SectionMeta {
        data::SectionMeta {
            id: self.entity_path.id().to_string(),
            ids: self.entity_path.ids.iter().map(|id| id.0.clone()).collect(),
            path: self.entity_path.rel_path.as_str().to_string(),
            title: self
                .index()
                .map(|article| article.title.clone())
                .unwrap_or(self.entity_path.id().to_string()),
            children: self.children().map(Node::to_meta).collect(),
            has_index: self.index().is_some(),
        }
    }
}

/// A node in the content tree.
///
/// Represents a directory that may contain an index article, other articles,
/// and sub-nodes (subdirectories).
#[derive(Clone, Debug)]
pub enum Node {
    Section(Section),
    Article(Article),
}

impl Node {
    pub fn entity_path(&self) -> &EntityPath {
        match self {
            Node::Section(c) => &c.entity_path,
            Node::Article(a) => &a.entity_path,
        }
    }

    pub fn id(&self) -> &Id {
        self.entity_path().id()
    }

    pub fn as_section(&self) -> Option<&Section> {
        match self {
            Node::Section(c) => Some(c),
            _ => None,
        }
    }

    pub fn as_article(&self) -> Option<&Article> {
        match self {
            Node::Article(a) => Some(a),
            _ => None,
        }
    }

    pub fn to_meta(&self) -> data::NodeMeta {
        match self {
            Node::Section(c) => data::NodeMeta::Section(c.to_meta()),
            Node::Article(a) => data::NodeMeta::Article(a.to_meta()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Vault {
    pub root_dir: PathBuf,
    pub root_section: Section,
}

impl Vault {
    fn posts_section(&self) -> Option<&Section> {
        self.root_section
            .get(&Id::new("posts"))
            .and_then(|n| n.as_section())
    }
    fn notes_section(&self) -> Option<&Section> {
        self.root_section
            .get(&Id::new("notes"))
            .and_then(|n| n.as_section())
    }
    pub fn export(&self) -> data::VaultData {
        // 1. Flatten posts
        let mut posts: Vec<data::ArticleMeta> = self
            .posts_section()
            .iter()
            .flat_map(|s| s.articles().map(|a| a.to_meta()))
            .collect();

        posts.sort_by(|a, b| b.created.cmp(&a.created));

        // 2. Convert notes tree
        let notes = self
            .notes_section()
            .iter()
            .flat_map(|s| s.children().filter_map(|n| n.as_section()))
            .map(|s| s.to_meta())
            .collect();

        data::VaultData { posts, notes }
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
        assert_eq!(id_a, id_b);
        let id_b = Id::new("ra as [d s ]e$");
        assert_eq!(id_b.0, "ra-as-d-s-e");
        assert_eq!(id_a, id_b)
    }
}
