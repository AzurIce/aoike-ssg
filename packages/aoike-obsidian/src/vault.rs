use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::config::Config;
use crate::parser;
use crate::wikilink;

/// A scanned and parsed vault ready for export.
#[derive(Debug)]
pub struct ScannedVault {
    pub posts: Vec<ParsedArticle>,
    pub notes: Vec<ScannedSection>,
    pub vault_dir: PathBuf,
}

/// A parsed article with resolved metadata.
#[derive(Debug)]
pub struct ParsedArticle {
    pub ids: Vec<String>,
    pub rel_path: String,
    /// Absolute path to the source .md file (for resolving relative asset paths).
    pub source_path: PathBuf,
    pub title: String,
    pub summary_html: String,
    pub content_html: String,
    pub created: i64,
    pub updated: i64,
    pub tags: Vec<String>,
    pub extra: Option<serde_json::Value>,
    pub outlinks: Vec<String>,
}

/// A section in the notes tree.
#[derive(Debug)]
pub struct ScannedSection {
    pub ids: Vec<String>,
    pub rel_path: String,
    pub title: String,
    pub description: Option<String>,
    pub index: Option<ParsedArticle>,
    pub children: Vec<ScannedNode>,
}

/// A node: either a section or an article.
#[derive(Debug)]
pub enum ScannedNode {
    Section(ScannedSection),
    Article(ParsedArticle),
}

/// Scan an Obsidian vault and build the internal representation.
pub fn scan_vault(vault_dir: &Path, config: &Config, respect_publish: bool) -> Result<ScannedVault> {
    let vault_dir = vault_dir.canonicalize()?;

    // Build filename → ids_path index for wikilink resolution
    let index = build_file_index(&vault_dir, config)?;

    // Scan posts
    let posts_dir = vault_dir.join(&config.mapping.posts);
    let mut posts = Vec::new();
    if posts_dir.exists() {
        scan_posts(
            &vault_dir,
            &posts_dir,
            &[slug::slugify(&config.mapping.posts)],
            &index,
            respect_publish,
            &mut posts,
        )?;
    }
    posts.sort_by(|a, b| b.created.cmp(&a.created));

    // Scan notes
    let mut notes = Vec::new();
    for note_dir_name in &config.mapping.notes {
        let note_dir = vault_dir.join(note_dir_name);
        if note_dir.exists() {
            if let Some(section) = scan_section(
                &vault_dir,
                &note_dir,
                &[slug::slugify(note_dir_name)],
                &index,
                respect_publish,
            )? {
                // Expose children of the top-level note dir as root sections
                for child in section.children {
                    if let ScannedNode::Section(s) = child {
                        notes.push(s);
                    }
                }
            }
        }
    }

    Ok(ScannedVault {
        posts,
        notes,
        vault_dir,
    })
}

/// Indexes built from the vault for wikilink resolution.
pub struct VaultIndex {
    /// filename (without .md extension) → slugified ids_path, for `[[link]]` resolution.
    pub link_index: HashMap<String, String>,
    /// filename (with extension, e.g. "image.png") → vault-relative path (e.g. "attachments/image.png"),
    /// for `![[embed]]` resolution.
    pub asset_index: HashMap<String, String>,
}

/// Build a mapping from filename (without extension) → ids_path.
fn build_file_index(vault_dir: &Path, config: &Config) -> Result<VaultIndex> {
    let mut link_index = HashMap::new();
    let mut asset_index = HashMap::new();

    for entry in walkdir::WalkDir::new(vault_dir)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !config.ignore.patterns.iter().any(|p| name.as_ref() == p.as_str())
        })
        .flatten()
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let rel = path.strip_prefix(vault_dir)?;
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if path.extension().map_or(false, |e| e == "md") {
            // Markdown → link_index (stem → ids_path)
            let ids: Vec<String> = rel
                .with_extension("")
                .components()
                .map(|c| slug::slugify(c.as_os_str().to_string_lossy()))
                .collect();
            let ids_path = ids.join("/");
            let stem = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            link_index.insert(stem, ids_path);
        } else {
            // Non-markdown → asset_index (filename with ext → vault-relative path)
            asset_index.insert(filename, rel_str);
        }
    }

    Ok(VaultIndex { link_index, asset_index })
}

/// Scan a flat posts directory.
fn scan_posts(
    vault_dir: &Path,
    dir: &Path,
    parent_ids: &[String],
    index: &VaultIndex,
    respect_publish: bool,
    out: &mut Vec<ParsedArticle>,
) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |e| e == "md") {
            if let Some(article) = parse_article_file(
                vault_dir,
                &path,
                parent_ids,
                index,
                respect_publish,
            )? {
                out.push(article);
            }
        }
    }
    Ok(())
}

/// Parse a single markdown file into a ParsedArticle.
fn parse_article_file(
    vault_dir: &Path,
    path: &Path,
    parent_ids: &[String],
    index: &VaultIndex,
    respect_publish: bool,
) -> Result<Option<ParsedArticle>> {
    let source = std::fs::read_to_string(path)?;
    let parsed = parser::parse_markdown(&source)?;

    // Respect publish flag
    if respect_publish && parsed.frontmatter.publish == Some(false) {
        return Ok(None);
    }

    let stem = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let id = slug::slugify(&stem);

    let mut ids = parent_ids.to_vec();
    ids.push(id);

    let rel_path = path
        .strip_prefix(vault_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");

    let title = parsed
        .frontmatter
        .title
        .unwrap_or_else(|| stem.clone());

    // Process wikilinks
    let (content_html, outlinks) =
        wikilink::process_wikilinks(&parsed.content_html, &index.link_index, &index.asset_index, vault_dir, path);

    // Extract summary (first 200 chars of text)
    let summary_html = extract_text_summary(&content_html, 200);

    // Timestamps
    let meta = std::fs::metadata(path)?;
    let created = parse_timestamp(&parsed.frontmatter.created)
        .unwrap_or_else(|| file_created_timestamp(&meta));
    let updated = parse_timestamp(&parsed.frontmatter.updated)
        .unwrap_or_else(|| file_modified_timestamp(&meta));

    let tags = parsed.frontmatter.tags.unwrap_or_default();

    // Clean extra: remove known keys
    let extra = parsed.frontmatter.extra.and_then(|v| {
        if let serde_json::Value::Object(mut map) = v {
            for key in &["title", "created", "updated", "tags", "publish", "description"] {
                map.remove(*key);
            }
            if map.is_empty() {
                None
            } else {
                Some(serde_json::Value::Object(map))
            }
        } else {
            None
        }
    });

    Ok(Some(ParsedArticle {
        ids,
        rel_path,
        source_path: path.to_path_buf(),
        title,
        summary_html,
        content_html,
        created,
        updated,
        tags,
        extra,
        outlinks,
    }))
}

/// Scan a directory as a notes section (recursive).
fn scan_section(
    vault_dir: &Path,
    dir: &Path,
    parent_ids: &[String],
    index: &VaultIndex,
    respect_publish: bool,
) -> Result<Option<ScannedSection>> {
    let dir_name = dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let id = slug::slugify(&dir_name);

    let mut ids = parent_ids.to_vec();
    // Don't duplicate if parent_ids already ends with this id
    if parent_ids.last().map_or(true, |last| last != &id) {
        ids.push(id);
    }

    let rel_path = dir
        .strip_prefix(vault_dir)
        .unwrap_or(dir)
        .to_string_lossy()
        .replace('\\', "/");

    let mut index_article = None;
    let mut children = Vec::new();
    let mut description = None;

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Skip ignored
        if name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            if let Some(sub) = scan_section(
                vault_dir,
                &path,
                &ids,
                index,
                respect_publish,
            )? {
                children.push(ScannedNode::Section(sub));
            }
        } else if path.extension().map_or(false, |e| e == "md") {
            let stem = path.file_stem().unwrap_or_default().to_string_lossy();
            if stem.eq_ignore_ascii_case("index") {
                // This is the section's index article
                if let Some(article) = parse_article_file(
                    vault_dir,
                    &path,
                    // Use parent ids for index (it represents the section itself)
                    &ids,
                    index,
                    respect_publish,
                )? {
                    description = article.tags.first().cloned(); // Use frontmatter description if available
                    index_article = Some(article);
                }
            } else {
                if let Some(article) = parse_article_file(
                    vault_dir,
                    &path,
                    &ids,
                    index,
                    respect_publish,
                )? {
                    children.push(ScannedNode::Article(article));
                }
            }
        }
    }

    let title = index_article
        .as_ref()
        .map(|a| a.title.clone())
        .unwrap_or_else(|| dir_name.clone());

    Ok(Some(ScannedSection {
        ids,
        rel_path,
        title,
        description,
        index: index_article,
        children,
    }))
}

/// Extract a plain-text summary from HTML, truncated to max_len characters.
fn extract_text_summary(html: &str, max_len: usize) -> String {
    let mut text = String::new();
    let mut char_count = 0;
    let mut in_tag = false;
    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            text.push(ch);
            char_count += 1;
            if char_count >= max_len {
                break;
            }
        }
    }
    text
}

/// Try to parse a date string to Unix timestamp.
fn parse_timestamp(s: &Option<String>) -> Option<i64> {
    let s = s.as_ref()?;
    // Try "YYYY-MM-DD"
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() == 3 {
        if let (Ok(y), Ok(m), Ok(d)) = (
            parts[0].parse::<i32>(),
            parts[1].parse::<u8>(),
            parts[2].parse::<u8>(),
        ) {
            if let Ok(month) = time::Month::try_from(m) {
                if let Ok(date) = time::Date::from_calendar_date(y, month, d) {
                    return Some(date.midnight().assume_utc().unix_timestamp());
                }
            }
        }
    }
    None
}

fn file_created_timestamp(meta: &std::fs::Metadata) -> i64 {
    meta.created()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn file_modified_timestamp(meta: &std::fs::Metadata) -> i64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
