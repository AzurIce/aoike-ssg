pub mod article;
pub mod utils;

use crate::build::article::ArticleSource;
use crate::{EntityPath, Id};
use relative_path::RelativePathBuf;
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
}

pub fn build_vault(root_dir: impl AsRef<Path>) -> crate::Vault {
    let root = root_dir
        .as_ref()
        .canonicalize()
        .unwrap_or_else(|_| root_dir.as_ref().to_path_buf());
    let posts_dir = root.join("posts");
    let notes_dir = root.join("notes");

    // Helper to create EntityPath
    let make_path = |ids: Vec<Id>, abs_path: &Path| {
        let rel_path = pathdiff::diff_paths(abs_path, &root).unwrap();
        let rel_path_str = rel_path.to_string_lossy().to_string();
        // Normalize path separators to forward slashes for RelativePathBuf
        let rel_path_str = rel_path_str.replace('\\', "/");
        EntityPath {
            ids,
            vault_root: root.clone(),
            path: RelativePathBuf::from(rel_path_str),
        }
    };

    // --- Process Posts ---
    let posts_id = Id::new("posts");
    let posts_ids = vec![posts_id];
    let posts_ep = make_path(posts_ids.clone(), &posts_dir);

    let mut posts_container = crate::Container {
        entity_path: posts_ep.clone(),
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
                posts_container.index = Some(src.to_article(posts_ep.clone()));
            }
        }

        // 2. Recursive Scan (Flattened)
        scan_posts_recursive(&root, &posts_dir, posts_ids, &mut posts_container.children);
    }

    // Sort posts by created date (descending)
    posts_container.children.sort_by(|a, b| {
        let date_a = match a {
            crate::Node::Article(a) => a.created,
            crate::Node::Container(_) => UtcDateTime::from_unix_timestamp(0).unwrap(),
        };
        let date_b = match b {
            crate::Node::Article(b) => b.created,
            crate::Node::Container(_) => UtcDateTime::from_unix_timestamp(0).unwrap(),
        };
        date_b.cmp(&date_a)
    });

    // --- Process Notes ---
    let notes_id = Id::new("notes");
    let notes_ids = vec![notes_id];
    let notes_ep = make_path(notes_ids.clone(), &notes_dir);

    let mut notes_container = crate::Container {
        entity_path: notes_ep.clone(),
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
                notes_container.index = Some(src.to_article(notes_ep.clone()));
            }
        }

        // 2. Recursive Scan (Tree)
        notes_container.children = scan_notes_content(&root, &notes_dir, notes_ids);
    }

    // Sort notes children by slug (ascending)
    notes_container.children.sort_by(|a, b| {
        let slug_a = &a.id().0;
        let slug_b = &b.id().0;
        slug_a.cmp(slug_b)
    });

    crate::Vault {
        posts: posts_container,
        notes: notes_container,
    }
}

fn make_entity_path(
    vault_root: &Path,
    abs_path: &Path,
    parent_ids: &[Id],
    self_id: Id,
) -> EntityPath {
    let rel_path = pathdiff::diff_paths(abs_path, vault_root).unwrap();
    let rel_path_str = rel_path.to_string_lossy().to_string();
    let rel_path_str = rel_path_str.replace('\\', "/");

    let mut ids = parent_ids.to_vec();
    ids.push(self_id);

    EntityPath {
        ids,
        vault_root: vault_root.to_path_buf(),
        path: RelativePathBuf::from(rel_path_str),
    }
}

fn scan_posts_recursive(
    vault_root: &Path,
    dir: &Path,
    parent_ids: Vec<Id>,
    acc: &mut Vec<crate::Node>,
) {
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
                    if parent_ids.len() > 1 {
                        if let Ok(src) = ArticleSource::try_from(entity) {
                            // For index file in subdir, it represents the subdir itself.
                            // The ID is the subdir's ID (last of parent_ids).
                            // The EntityPath should point to the subdir, not the index file?
                            // Or the index file itself?
                            // In previous logic: "ID is the directory name".
                            // Here, parent_ids includes the directory name as the last element.
                            // So we construct EntityPath for the directory.

                            // Wait, if we are in `posts/tech`, parent_ids is `[posts, tech]`.
                            // The index file is `posts/tech/index.md`.
                            // We want to create an Article node for `tech`.
                            // Its ID is `tech`. Its path is `posts/tech`.
                            // But `scan_posts_recursive` is called for `posts/tech`.
                            // The `parent_ids` passed to it is `[posts, tech]`.

                            // Actually, `scan_posts_recursive` iterates children.
                            // If we find `index.md`, it means the CURRENT directory is a node.
                            // But we are inside the loop iterating children.
                            // The `index.md` is a child file.

                            // In the flattened posts logic:
                            // "Index Article: ... represents the directory node itself".
                            // So we create an Article with the directory's ID.

                            // The `parent_ids` passed to this function corresponds to `dir`.
                            // So `dir`'s ID is `parent_ids.last()`.

                            // We construct EntityPath for the directory `dir`.
                            // But wait, `make_entity_path` takes `self_id` and appends it.
                            // Here `ids` is already complete in `parent_ids`.

                            let rel_path = pathdiff::diff_paths(dir, vault_root).unwrap();
                            let rel_path_str =
                                rel_path.to_string_lossy().to_string().replace('\\', "/");

                            let ep = EntityPath {
                                ids: parent_ids.clone(),
                                vault_root: vault_root.to_path_buf(),
                                path: RelativePathBuf::from(rel_path_str),
                            };

                            acc.push(crate::Node::Article(src.to_article(ep)));
                        }
                    }
                } else if stem != "main" {
                    // Regular article
                    if let Ok(src) = ArticleSource::try_from(entity) {
                        let id = Id::new(&stem);
                        let ep = make_entity_path(vault_root, &path, &parent_ids, id);
                        acc.push(crate::Node::Article(src.to_article(ep)));
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
                let id = Id::new(&dir_name);
                // EntityPath points to the directory, not main.md?
                // "Directory Article ... ID: <dirname> ... Path: path/to"
                // Let's point to the directory.
                let ep = make_entity_path(vault_root, &subdir, &parent_ids, id);
                acc.push(crate::Node::Article(src.to_article(ep)));
            }
            // Do NOT recurse into Directory Article
        } else {
            // It is a Container Directory
            let id = Id::new(&dir_name);
            let mut next_ids = parent_ids.clone();
            next_ids.push(id);
            scan_posts_recursive(vault_root, &subdir, next_ids, acc);
        }
    }
}

fn scan_notes_content(vault_root: &Path, dir: &Path, parent_ids: Vec<Id>) -> Vec<crate::Node> {
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
                        let id = Id::new(&stem);
                        let ep = make_entity_path(vault_root, &path, &parent_ids, id);
                        children.push(crate::Node::Article(src.to_article(ep)));
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
                let id = Id::new(&dir_name);
                let ep = make_entity_path(vault_root, &subdir, &parent_ids, id);
                children.push(crate::Node::Article(src.to_article(ep)));
            }
        } else {
            // Container Directory -> Create Child Node (Container)
            let id = Id::new(&dir_name);
            let mut next_ids = parent_ids.clone();
            next_ids.push(id.clone());

            let child_node = create_node_from_dir(vault_root, &subdir, next_ids);
            children.push(child_node);
        }
    }

    // Sort children
    children.sort_by(|a, b| {
        let slug_a = &a.id().0;
        let slug_b = &b.id().0;
        slug_a.cmp(slug_b)
    });

    children
}

fn create_node_from_dir(vault_root: &Path, dir: &Path, ids: Vec<Id>) -> crate::Node {
    // EntityPath for this container
    let rel_path = pathdiff::diff_paths(dir, vault_root).unwrap();
    let rel_path_str = rel_path.to_string_lossy().to_string().replace('\\', "/");
    let ep = EntityPath {
        ids: ids.clone(),
        vault_root: vault_root.to_path_buf(),
        path: RelativePathBuf::from(rel_path_str),
    };

    // Find Index
    let index_path = ["md", "typ"]
        .iter()
        .map(|ext| dir.join(format!("index.{}", ext)))
        .find(|p| p.exists());

    let index = if let Some(p) = index_path {
        let entity = Entity::new(p);
        if let Ok(src) = ArticleSource::try_from(entity) {
            Some(src.to_article(ep.clone()))
        } else {
            None
        }
    } else {
        None
    };

    let children = scan_notes_content(vault_root, dir, ids);

    crate::Node::Container(crate::Container {
        entity_path: ep,
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
        let detail = article.to_detail();
        let json = serde_json::to_string(&detail).unwrap();

        let mut path = base_articles_dir.clone();
        // Use ids for path structure: articles/posts/tech/rust.json
        // Note: ids includes the article id itself at the end.
        // We want to use all ids except the last one for directory structure?
        // Or just use all ids as path components?
        // If ids = [posts, tech, rust], we want articles/posts/tech/rust.json

        for (i, id) in article.entity_path.ids.iter().enumerate() {
            if i == article.entity_path.ids.len() - 1 {
                path.push(format!("{}.json", id.0));
            } else {
                path.push(&id.0);
            }
        }

        write_if_changed(&path, &json);
        generated_files.insert(path);
    };

    fn export_container_content(
        container: &crate::Container,
        export_fn: &mut impl FnMut(&crate::Article),
    ) {
        if let Some(idx) = &container.index {
            export_fn(idx);
        }
        for child in &container.children {
            export_node_content(child, export_fn);
        }
    }

    fn export_node_content(node: &crate::Node, export_fn: &mut impl FnMut(&crate::Article)) {
        match node {
            crate::Node::Container(c) => {
                export_container_content(c, export_fn);
            }
            crate::Node::Article(a) => {
                export_fn(a);
            }
        }
    }

    export_container_content(&vault.posts, &mut export_article);
    export_container_content(&vault.notes, &mut export_article);

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

// Simple path diff implementation since pathdiff crate might not be available
// or we can use relative-path if it supports it, but relative-path works on strings/abstract paths.
// We need to diff std::path::Path.
mod pathdiff {
    use std::path::{Path, PathBuf};

    pub fn diff_paths<P, B>(path: P, base: B) -> Option<PathBuf>
    where
        P: AsRef<Path>,
        B: AsRef<Path>,
    {
        let path = path.as_ref();
        let base = base.as_ref();

        if path.is_absolute() != base.is_absolute() {
            if path.is_absolute() {
                return Some(PathBuf::from(path));
            } else {
                return None;
            }
        }

        let mut ita = path.components();
        let mut itb = base.components();
        let mut comps: Vec<std::path::Component> = vec![];

        loop {
            match (ita.next(), itb.next()) {
                (None, None) => break,
                (Some(a), None) => {
                    comps.push(a);
                    comps.extend(ita.by_ref());
                    break;
                }
                (None, _) => comps.push(std::path::Component::ParentDir),
                (Some(a), Some(b)) if a == b => (), // same component
                (Some(a), Some(_)) => {
                    comps.push(std::path::Component::ParentDir);
                    for _ in itb {
                        comps.push(std::path::Component::ParentDir);
                    }
                    comps.push(a);
                    comps.extend(ita.by_ref());
                    break;
                }
            }
        }

        Some(comps.iter().map(|c| c.as_os_str()).collect())
    }
}
