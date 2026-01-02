pub mod article;
pub mod utils;

use crate::build::article::ArticleSource;
use crate::{Article, EntityPath, Id, Node, Section};
use relative_path::{PathExt, RelativePath};
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

pub fn build_article(
    vault_root: impl AsRef<Path>,
    rel_path: impl AsRef<RelativePath>,
) -> Option<Article> {
    let vault_root = vault_root.as_ref();
    let rel_path = rel_path.as_ref();
    // println!("cargo::warning={}", format!("Building article {rel_path}"));
    let path = rel_path.to_logical_path(vault_root);
    if !path.exists() || !path.starts_with(vault_root) {
        return None;
    }

    if path.is_dir() {
        for ext in ["md", "typ"] {
            if let Some(mut article) =
                build_article(vault_root, rel_path.join(format!("main.{ext}")))
            {
                if !article.entity_path.ids.is_empty() {
                    article.entity_path.ids.pop();
                    article.entity_path.rel_path.pop();
                }
                return Some(article);
            }
        }
        return None;
    }

    let entity_path = EntityPath::new(vault_root.to_owned(), rel_path.to_owned());
    ArticleSource::try_from(Entity::new(path))
        .ok()
        .map(|source| source.to_article(entity_path))
}

/// Build a section from a path inside vault root
pub fn build_section(
    vault_root: impl AsRef<Path>,
    rel_path: impl AsRef<RelativePath>,
) -> Option<Section> {
    let vault_root = vault_root.as_ref();
    let rel_path = rel_path.as_ref();
    let path = rel_path.to_logical_path(vault_root);

    if path.is_file() || !path.exists() || !path.starts_with(vault_root) {
        return None;
    }
    // If this is a Directory Article, return None
    if ["md", "typ"]
        .iter()
        .any(|ext| path.join(format!("main.{ext}")).exists())
    {
        return None;
    }

    let entity_path = EntityPath::new(vault_root.to_owned(), rel_path.to_owned());

    // Children
    // TODO: read_dir should not failed?
    let mut children = std::fs::read_dir(&path)
        .ok()
        .into_iter()
        .flat_map(|entries| {
            entries.flatten().filter_map(|entry| {
                let path = entry.path();
                let rel_path = path.relative_to(vault_root).unwrap();
                if path.is_dir() {
                    build_section(vault_root, &rel_path)
                        .map(Node::Section)
                        .or(build_article(vault_root, &rel_path).map(Node::Article))
                } else {
                    build_article(vault_root, &rel_path).map(Node::Article)
                }
            })
        })
        .collect::<Vec<_>>();

    let index = children
        .extract_if(.., |n| {
            if let Node::Article(article) = n
                && article.entity_path.id().is_index()
            {
                true
            } else {
                false
            }
        })
        .next()
        .map(|n| match n {
            Node::Article(article) => article,
            _ => unreachable!(),
        });

    // TODO: is sorting neccesary?
    Some(Section {
        entity_path,
        children,
        index,
    })
}

pub fn build_vault(root_dir: impl AsRef<Path>) -> crate::Vault {
    let root_dir = root_dir.as_ref();
    let root_section = build_section(root_dir, "").expect("faild to build root section");

    crate::Vault {
        root_dir: root_dir.to_owned(),
        root_section,
    }
}

pub fn export_vault(vault: &crate::Vault, out_dir: impl AsRef<Path>, public_url_prefix: &str) {
    println!(
        "cargo::warning={}",
        format!(
            "exporting vault with public_url_prefix: {}",
            public_url_prefix
        )
    );
    let public_url_prefix = public_url_prefix.trim_end_matches("/");
    let vault_root = &vault.root_dir;
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
        // Process assets and rewrite HTML

        let mut article_clone = article.clone();
        let assets = utils::rewrite_html_links(
            &mut article_clone,
            vault_root,
            &format!("{public_url_prefix}/static/vault/articles"),
        );

        let detail = article_clone.to_detail();
        let json = serde_json::to_string(&detail).unwrap();

        let path = base_articles_dir.join(format!("{}.json", article.entity_path.ids));

        write_if_changed(&path, &json);
        generated_files.insert(path);

        // Copy assets
        for (src_path, ids) in assets {
            let mut dst_path = out_dir.join("articles").join(ids.to_string());
            if let Some(ext) = src_path.extension() {
                dst_path.set_extension(ext);
            }
            if let Some(parent) = dst_path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            println!(
                "cargo::warning=copying from {} to {}",
                src_path.display(),
                dst_path.display()
            );
            std::fs::copy(&src_path, &dst_path).unwrap();
            generated_files.insert(dst_path);
        }
    };

    fn export_section_content(
        section: &crate::Section,
        fn_export_article: &mut impl FnMut(&crate::Article),
    ) {
        if let Some(article) = section.index.as_ref() {
            println!(
                "cargo::warning={}",
                format!(
                    "Exporting index article for section {:?}",
                    section.entity_path
                )
            );
            fn_export_article(article);
        }
        for child in &section.children {
            export_node_content(child, fn_export_article);
        }
    }

    fn export_node_content(node: &crate::Node, export_fn: &mut impl FnMut(&crate::Article)) {
        match node {
            crate::Node::Section(c) => {
                export_section_content(c, export_fn);
            }
            crate::Node::Article(a) => {
                export_fn(a);
            }
        }
    }

    export_section_content(&vault.root_section, &mut export_article);

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
