pub mod article;
pub mod utils;

use crate::build::article::ArticleSource;
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

fn path_to_ids(path: &Path) -> Vec<crate::Id> {
    path.components()
        .filter_map(|c| match c {
            std::path::Component::Normal(s) => {
                Some(crate::Id::new(s.to_string_lossy().to_string()))
            }
            _ => None,
        })
        .collect()
}

pub fn build_vault(root_dir: impl AsRef<Path>) -> crate::Vault {
    let root = root_dir.as_ref();
    let posts_dir = root.join("posts");
    let notes_dir = root.join("notes");

    // Build Posts
    let mut posts_index = None;
    let mut posts_articles = Vec::new();
    if posts_dir.exists() {
        for entry in WalkDir::new(&posts_dir)
            .into_iter()
            .flatten()
            .filter(|e| e.file_type().is_file())
        {
            let entity = Entity::new(entry.path());
            if !["md", "typ"].contains(&entity.extension().as_str()) {
                continue;
            }

            if let Ok(src) = ArticleSource::try_from(entity) {
                let rel_path = entry.path().strip_prefix(&posts_dir).unwrap();
                let is_index = src.base_name() == "index"
                    && rel_path.parent().map_or(true, |p| p.as_os_str().is_empty());

                if is_index {
                    posts_index = Some(src.to_article(vec![crate::Id::new("index".to_string())]));
                } else {
                    let mut ids = path_to_ids(rel_path.parent().unwrap_or(Path::new("")));
                    ids.push(crate::Id::new(src.base_name()));
                    posts_articles.push(src.to_article(ids));
                }
            }
        }
    }
    posts_articles.sort_by(|a, b| b.created.cmp(&a.created));
    let posts = crate::Posts {
        index: posts_index,
        articles: posts_articles,
    };

    // Build Notes
    let mut notes_index = None;
    let mut notes_list = Vec::new();

    // First check for global notes index in notes_dir root
    if notes_dir.exists() {
        // Find index file in notes_dir
        for entry in std::fs::read_dir(&notes_dir).unwrap().flatten() {
            if entry.file_type().unwrap().is_file() {
                let entity = Entity::new(entry.path());
                if ["md", "typ"].contains(&entity.extension().as_str())
                    && entity.base_name() == "index"
                {
                    if let Ok(src) = ArticleSource::try_from(entity) {
                        notes_index =
                            Some(src.to_article(vec![crate::Id::new("index".to_string())]));
                    }
                }
            }
        }

        // Iterate subdirectories for individual notes
        for entry in std::fs::read_dir(&notes_dir).unwrap().flatten() {
            if entry.path().is_dir() {
                let note_name = entry.file_name().to_string_lossy().to_string();
                let note_root = entry.path();

                let mut index_article = None;
                let mut articles = Vec::new();

                for e in WalkDir::new(&note_root)
                    .into_iter()
                    .flatten()
                    .filter(|e| e.file_type().is_file())
                {
                    let entity = Entity::new(e.path());
                    if !["md", "typ"].contains(&entity.extension().as_str()) {
                        continue;
                    }

                    if let Ok(src) = ArticleSource::try_from(entity) {
                        let rel_path = e.path().strip_prefix(&note_root).unwrap();

                        let is_index = src.base_name() == "index"
                            && rel_path.parent().map_or(true, |p| p.as_os_str().is_empty());

                        if is_index {
                            index_article =
                                Some(src.to_article(vec![crate::Id::new("index".to_string())]));
                        } else {
                            let mut ids = path_to_ids(rel_path.parent().unwrap_or(Path::new("")));
                            ids.push(crate::Id::new(src.base_name()));
                            articles.push(src.to_article(ids));
                        }
                    }
                }

                if let Some(index) = index_article {
                    articles.sort_by(|a, b| b.created.cmp(&a.created));
                    notes_list.push(crate::Note {
                        id: crate::Id::new(note_name),
                        index,
                        articles,
                    });
                }
            }
        }
    }
    // notes_list.sort_by_key(...); // Sort notes if needed

    let notes = crate::Notes {
        index: notes_index,
        notes: notes_list,
    };

    crate::Vault { posts, notes }
}

pub fn export_vault(vault: &crate::Vault, out_dir: impl AsRef<Path>) {
    let out_dir = out_dir.as_ref();
    if !out_dir.exists() {
        std::fs::create_dir_all(out_dir).unwrap();
    }

    let mut generated_files = std::collections::HashSet::new();

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
    let mut manifest_vault = vault.clone();

    // Clear content for manifest
    if let Some(idx) = &mut manifest_vault.posts.index {
        idx.content_html.clear();
    }
    for post in &mut manifest_vault.posts.articles {
        post.content_html.clear();
    }

    if let Some(idx) = &mut manifest_vault.notes.index {
        idx.content_html.clear();
    }
    for note in &mut manifest_vault.notes.notes {
        note.index.content_html.clear();
        for article in &mut note.articles {
            article.content_html.clear();
        }
    }

    let vault_json = serde_json::to_string(&manifest_vault).unwrap();
    let vault_path = out_dir.join("vault.json");
    write_if_changed(&vault_path, &vault_json);
    generated_files.insert(vault_path);

    // Full Content
    let base_articles_dir = out_dir.join("articles");

    // Posts
    let posts_dir = base_articles_dir.join("posts");

    if let Some(idx) = &vault.posts.index {
        let json = serde_json::to_string(idx).unwrap();
        let path = posts_dir.join("index.json");
        write_if_changed(&path, &json);
        generated_files.insert(path);
    }

    for post in &vault.posts.articles {
        let json = serde_json::to_string(post).unwrap();
        let relative_path: PathBuf = post.path.iter().filter_map(|id| Some(&id.slug)).collect();
        let target_path = posts_dir.join(relative_path).with_extension("json");
        write_if_changed(&target_path, &json);
        generated_files.insert(target_path);
    }

    // Notes
    // Global Notes Index
    if let Some(idx) = &vault.notes.index {
        let notes_root_dir = base_articles_dir.join("notes");
        let json = serde_json::to_string(idx).unwrap();
        let path = notes_root_dir.join("index.json");
        write_if_changed(&path, &json);
        generated_files.insert(path);
    }

    for note in &vault.notes.notes {
        // note.id.slug is the directory name
        let note_dir = base_articles_dir.join(&note.id.slug);

        let index_json = serde_json::to_string(&note.index).unwrap();
        let path = note_dir.join("index.json");
        write_if_changed(&path, &index_json);
        generated_files.insert(path);

        for article in &note.articles {
            let relative_path: PathBuf = article
                .path
                .iter()
                .filter_map(|id| Some(&id.slug))
                .collect();
            let target_path = note_dir.join(relative_path).with_extension("json");
            let json = serde_json::to_string(article).unwrap();
            write_if_changed(&target_path, &json);
            generated_files.insert(target_path);
        }
    }

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
