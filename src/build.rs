pub mod article;
pub mod utils;

use crate::Id;
use crate::build::article::ArticleSource;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use time::UtcDateTime;
use walkdir::WalkDir;

pub trait Parser {
    type Output;
    fn try_parse(entity: Entity) -> Result<Self::Output, anyhow::Error>;
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub path: PathBuf,
    pub created: UtcDateTime,
    pub updated: UtcDateTime,
    pub content: Vec<u8>,
}

impl Entity {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        let content = std::fs::read(path).unwrap();
        let created = utils::git_created_datetime(path);
        let updated = utils::git_updated_datetime(path);
        Self {
            path: path.to_path_buf(),
            created,
            updated,
            content,
        }
    }
    pub fn extension(&self) -> String {
        self.path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string()
    }
    pub fn base_name(&self) -> String {
        self.path
            .with_extension("")
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap()
            .to_string()
    }
    pub fn slug(&self) -> String {
        slug::slugify(self.path.file_name().and_then(|s| s.to_str()).unwrap())
    }
}

pub fn build_vault(root_dir: impl AsRef<Path>) -> crate::Vault {
    let root = root_dir.as_ref();
    let posts_dir = root.join("posts");
    let notes_dir = root.join("notes");

    // --- Process Posts ---
    let mut posts_container = crate::Container {
        id: Id::new("posts".to_string()),
        path: vec![],
        index: None,
        children: vec![],
    };

    if posts_dir.exists() {
        // 1. Root Index
        let index_path = ["md", "typ"]
            .iter()
            .map(|ext| posts_dir.join(format!("index.{}", ext)))
            .find(|p| p.exists());

        if let Some(p) = index_path {
            let entity = Entity::new(p);
            if let Ok(src) = ArticleSource::try_from(entity) {
                posts_container.index = Some(src.to_article(Id::new("posts".to_string()), vec![]));
            }
        }

        // 2. Recursive Scan (Flattened)
        scan_posts_recursive(
            &posts_dir,
            posts_container
                .path
                .iter()
                .chain(std::iter::once(&posts_container.id))
                .cloned()
                .collect(),
            &mut posts_container.children,
        );
    }

    // Sort posts by created date (descending)
    posts_container.children.sort_by(|a, b| {
        let date_a = match a {
            crate::Node::Article(a) => a.created,
            crate::Node::Container(_) => UtcDateTime::from_unix_timestamp(0).unwrap(), // Should not happen for posts
        };
        let date_b = match b {
            crate::Node::Article(b) => b.created,
            crate::Node::Container(_) => UtcDateTime::from_unix_timestamp(0).unwrap(),
        };
        date_b.cmp(&date_a)
    });

    // --- Process Notes ---
    let mut notes_container = crate::Container {
        id: crate::Id::new("notes".to_string()),
        path: vec![],
        index: None,
        children: vec![],
    };

    if notes_dir.exists() {
        // 1. Root Index
        let index_path = ["md", "typ"]
            .iter()
            .map(|ext| notes_dir.join(format!("index.{}", ext)))
            .find(|p| p.exists());

        if let Some(p) = index_path {
            let entity = Entity::new(p);
            if let Ok(src) = ArticleSource::try_from(entity) {
                notes_container.index =
                    Some(src.to_article(crate::Id::new("notes".to_string()), vec![]));
            }
        }

        // 2. Recursive Scan (Tree)
        notes_container.children =
            scan_notes_content(&notes_dir, vec![crate::Id::new("notes".to_string())]);
    }

    // Sort notes children by slug (ascending)
    notes_container.children.sort_by(|a, b| {
        let slug_a = match a {
            crate::Node::Article(a) => &a.id.slug,
            crate::Node::Container(c) => &c.id.slug,
        };
        let slug_b = match b {
            crate::Node::Article(b) => &b.id.slug,
            crate::Node::Container(c) => &c.id.slug,
        };
        slug_a.cmp(slug_b)
    });

    crate::Vault {
        posts: posts_container,
        notes: notes_container,
    }
}

fn scan_posts_recursive(dir: &Path, current_path: Vec<crate::Id>, acc: &mut Vec<crate::Node>) {
    let mut subdirs = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let entity = Entity::new(&path);
                if !["md", "typ"].contains(&entity.extension().as_str()) {
                    continue;
                }
                let stem = entity.base_name();

                if stem == "index" {
                    if current_path.len() > 1 {
                        if let Ok(src) = ArticleSource::try_from(entity) {
                            // ID is the directory name (last element of current_path)
                            // Path is parent path
                            if let Some(id) = current_path.last() {
                                let parent_path = current_path[0..current_path.len() - 1].to_vec();
                                acc.push(crate::Node::Article(
                                    src.to_article(id.clone(), parent_path),
                                ));
                            }
                        }
                    }
                } else if stem != "main" {
                    // Regular article
                    if let Ok(src) = ArticleSource::try_from(entity) {
                        let id = crate::Id::new(stem);
                        acc.push(crate::Node::Article(
                            src.to_article(id, current_path.clone()),
                        ));
                    }
                }
            } else if path.is_dir() {
                subdirs.push(path);
            }
        }
    }

    for subdir in subdirs {
        let dir_name = subdir.file_name().unwrap().to_string_lossy().to_string();

        // Check if it is a Directory Article (contains main)
        let main_path = ["md", "typ"]
            .iter()
            .map(|ext| subdir.join(format!("main.{}", ext)))
            .find(|p| p.exists());

        if let Some(p) = main_path {
            // It is a Directory Article
            let entity = Entity::new(p);
            if let Ok(src) = ArticleSource::try_from(entity) {
                let id = crate::Id::new(dir_name);
                acc.push(crate::Node::Article(
                    src.to_article(id, current_path.clone()),
                ));
            }
            // Do NOT recurse into Directory Article
        } else {
            // It is a Container Directory
            let mut next_path = current_path.clone();
            next_path.push(crate::Id::new(dir_name));
            scan_posts_recursive(&subdir, next_path, acc);
        }
    }
}

fn scan_notes_content(dir: &Path, current_path: Vec<crate::Id>) -> Vec<crate::Node> {
    let mut children = Vec::new();
    let mut subdirs = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let entity = Entity::new(&path);
                if !["md", "typ"].contains(&entity.extension().as_str()) {
                    continue;
                }
                let stem = entity.base_name();

                if stem != "index" && stem != "main" {
                    if let Ok(src) = ArticleSource::try_from(entity) {
                        let id = crate::Id::new(stem);
                        children.push(crate::Node::Article(
                            src.to_article(id, current_path.clone()),
                        ));
                    }
                }
            } else if path.is_dir() {
                subdirs.push(path);
            }
        }
    }

    for subdir in subdirs {
        let dir_name = subdir.file_name().unwrap().to_string_lossy().to_string();

        // Check if Directory Article
        let main_path = ["md", "typ"]
            .iter()
            .map(|ext| subdir.join(format!("main.{}", ext)))
            .find(|p| p.exists());

        if let Some(p) = main_path {
            // Directory Article -> Add to children as Article
            let entity = Entity::new(p);
            if let Ok(src) = ArticleSource::try_from(entity) {
                let id = crate::Id::new(dir_name);
                children.push(crate::Node::Article(
                    src.to_article(id, current_path.clone()),
                ));
            }
        } else {
            // Container Directory -> Create Child Node (Container)
            let mut next_path = current_path.clone();
            next_path.push(crate::Id::new(dir_name.clone()));

            let child_node = create_node_from_dir(&subdir, next_path);
            children.push(child_node);
        }
    }

    // Sort children: Articles by created date, Containers by slug?
    // Or just sort everything by slug?
    // Usually notes are sorted by name/slug.
    children.sort_by(|a, b| {
        let slug_a = match a {
            crate::Node::Article(a) => &a.id.slug,
            crate::Node::Container(c) => &c.id.slug,
        };
        let slug_b = match b {
            crate::Node::Article(b) => &b.id.slug,
            crate::Node::Container(c) => &c.id.slug,
        };
        slug_a.cmp(slug_b)
    });

    children
}

fn create_node_from_dir(dir: &Path, current_path: Vec<crate::Id>) -> crate::Node {
    let dir_name = dir.file_name().unwrap().to_string_lossy().to_string();
    let id = crate::Id::new(dir_name);

    let node_path = current_path[0..current_path.len() - 1].to_vec();

    // Find Index
    let index_path = ["md", "typ"]
        .iter()
        .map(|ext| dir.join(format!("index.{}", ext)))
        .find(|p| p.exists());

    let index = if let Some(p) = index_path {
        let entity = Entity::new(p);
        if let Ok(src) = ArticleSource::try_from(entity) {
            Some(src.to_article(id.clone(), node_path.clone()))
        } else {
            None
        }
    } else {
        None
    };

    let children = scan_notes_content(dir, current_path.clone());

    crate::Node::Container(crate::Container {
        id,
        path: node_path,
        index,
        children,
    })
}

pub fn export_vault(vault: &crate::Vault, out_dir: impl AsRef<Path>) {
    let out_dir = out_dir.as_ref();
    if !out_dir.exists() {
        std::fs::create_dir_all(out_dir).unwrap();
    }

    let mut generated_files = HashSet::new();

    let write_if_changed = |path: &Path, content: &str| {
        if path.exists() {
            if let Ok(old_content) = std::fs::read_to_string(path) {
                if old_content == content {
                    return;
                }
            }
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    };

    // Manifest
    let exported_vault = vault.export();
    let vault_json = serde_json::to_string(&exported_vault).unwrap();
    let vault_path = out_dir.join("vault.json");
    write_if_changed(&vault_path, &vault_json);
    generated_files.insert(vault_path);

    // Full Content
    let base_articles_dir = out_dir.join("articles");

    // Helper to export an article
    let mut export_article = |article: &crate::Article| {
        let json = serde_json::to_string(article).unwrap();

        let mut path = base_articles_dir.clone();
        for id in &article.path {
            path.push(id.to_string());
        }
        path.push(format!("{}.json", article.id.slug));

        write_if_changed(&path, &json);
        generated_files.insert(path);
    };

    fn export_node_content(node: &crate::Node, export_fn: &mut impl FnMut(&crate::Article)) {
        match node {
            crate::Node::Container(c) => {
                if let Some(idx) = &c.index {
                    export_fn(idx);
                }
                for child in &c.children {
                    export_node_content(child, export_fn);
                }
            }
            crate::Node::Article(a) => {
                export_fn(a);
            }
        }
    }

    export_node_content(
        &crate::Node::Container(vault.posts.clone()),
        &mut export_article,
    );
    export_node_content(
        &crate::Node::Container(vault.notes.clone()),
        &mut export_article,
    );

    // Cleanup stale files
    for entry in WalkDir::new(out_dir).into_iter().flatten() {
        if entry.file_type().is_file() {
            let path = entry.path();
            if !generated_files.contains(path) && path.extension().map_or(false, |e| e == "json") {
                std::fs::remove_file(path).unwrap();
            }
        }
    }
}
