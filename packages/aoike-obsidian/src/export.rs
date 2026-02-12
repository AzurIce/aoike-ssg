use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use aoike::data;
use regex::Regex;

use crate::vault::{ParsedArticle, ScannedNode, ScannedSection, ScannedVault};

/// Export a scanned vault to the aoike JSON format.
pub fn export(vault: &ScannedVault, out_dir: &Path, public_url_prefix: &str) -> Result<()> {
    std::fs::create_dir_all(out_dir)?;
    let articles_dir = out_dir.join("articles");
    std::fs::create_dir_all(&articles_dir)?;

    let public_url_prefix = public_url_prefix.trim_end_matches('/');
    let articles_url = format!("{}/static/vault/articles", public_url_prefix);

    // Collect all articles for backlink resolution
    let mut all_articles: Vec<&ParsedArticle> = Vec::new();
    collect_articles_from_posts(&vault.posts, &mut all_articles);
    for section in &vault.notes {
        collect_articles_from_section(section, &mut all_articles);
    }

    // Build backlinks map: ids_path → vec of source ids_paths
    let mut backlinks_map: HashMap<String, Vec<String>> = HashMap::new();
    for article in &all_articles {
        let source_ids_path = article.ids.join("/");
        for target in &article.outlinks {
            backlinks_map
                .entry(target.clone())
                .or_default()
                .push(source_ids_path.clone());
        }
    }

    // Export vault.json
    let vault_meta = build_vault_meta(vault);
    let vault_json = serde_json::to_string(&vault_meta)?;
    std::fs::write(out_dir.join("vault.json"), vault_json)?;

    // Export article JSONs with asset processing
    for article in &all_articles {
        let ids_path = article.ids.join("/");
        let backlinks = backlinks_map
            .get(&ids_path)
            .cloned()
            .unwrap_or_default();

        // Rewrite asset links and collect assets to copy
        let (rewritten_html, assets) = rewrite_html_assets(
            &article.content_html,
            &article.source_path,
            &vault.vault_dir,
            &articles_url,
        );

        let article_data = data::ArticleData {
            meta: article_to_meta(article),
            content: rewritten_html,
            outlinks: article.outlinks.clone(),
            backlinks,
        };
        let json = serde_json::to_string(&article_data)?;
        let json_path = articles_dir.join(format!("{}.json", ids_path));
        if let Some(parent) = json_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(json_path, json)?;

        // Copy assets
        for (src, dst_rel) in &assets {
            let dst = articles_dir.join(dst_rel);
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent)?;
            }
            if src.exists() {
                std::fs::copy(src, &dst)?;
            }
        }
    }

    Ok(())
}

fn collect_articles_from_posts<'a>(
    posts: &'a [ParsedArticle],
    out: &mut Vec<&'a ParsedArticle>,
) {
    out.extend(posts.iter());
}

fn collect_articles_from_section<'a>(
    section: &'a ScannedSection,
    out: &mut Vec<&'a ParsedArticle>,
) {
    if let Some(ref index) = section.index {
        out.push(index);
    }
    for child in &section.children {
        match child {
            ScannedNode::Article(a) => out.push(a),
            ScannedNode::Section(s) => collect_articles_from_section(s, out),
        }
    }
}

fn article_to_meta(article: &ParsedArticle) -> data::ArticleMeta {
    data::ArticleMeta {
        entity_path: data::EntityPath {
            ids: article.ids.clone(),
            rel_path: article.rel_path.clone(),
        },
        title: article.title.clone(),
        summary: article.summary_html.clone(),
        created: article.created,
        updated: article.updated,
        tags: article.tags.clone(),
        extra: article.extra.clone(),
    }
}

/// Rewrite relative asset references in HTML and return (new_html, assets_to_copy).
///
/// Each asset is (absolute_source_path, relative_destination_path_under_articles_dir).
/// HTML `src` and `href` attributes pointing to local files are rewritten to the public URL.
fn rewrite_html_assets(
    html: &str,
    source_path: &Path,
    vault_dir: &Path,
    articles_url: &str,
) -> (String, Vec<(PathBuf, String)>) {
    let mut assets: Vec<(PathBuf, String)> = Vec::new();
    let re = Regex::new(r#"(src|href)="([^"]+)""#).unwrap();

    let source_dir = source_path.parent().unwrap_or(vault_dir);

    let new_html = re
        .replace_all(html, |caps: &regex::Captures| {
            let attr = &caps[1];
            let val = &caps[2];

            // Skip absolute URLs, data URIs, anchors, mailto
            if val.starts_with("http")
                || val.starts_with('/')
                || val.starts_with("mailto:")
                || val.starts_with("data:")
                || val.starts_with('#')
            {
                return format!(r#"{}="{}""#, attr, val);
            }

            // Skip markdown files — those are article links, not assets
            if val.ends_with(".md") || val.contains(".md#") {
                return format!(r#"{}="{}""#, attr, val);
            }

            // Resolve relative path against the source file's directory
            let decoded = percent_decode(val);
            let abs_path = source_dir.join(&decoded);

            if !abs_path.exists() || abs_path.is_dir() {
                return format!(r#"{}="{}""#, attr, val);
            }

            // Build destination path: use vault-relative path, slugified
            if let Ok(rel_in_vault) = abs_path.strip_prefix(vault_dir) {
                let components: Vec<String> = rel_in_vault
                    .parent()
                    .unwrap_or(Path::new(""))
                    .components()
                    .map(|c| slug::slugify(c.as_os_str().to_string_lossy()))
                    .collect();

                let stem = abs_path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy();
                let slug_stem = slug::slugify(&stem);

                let ext = abs_path
                    .extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_default();

                let dst_rel = if components.is_empty() {
                    if ext.is_empty() {
                        slug_stem
                    } else {
                        format!("{}.{}", slug_stem, ext)
                    }
                } else {
                    let dir = components.join("/");
                    if ext.is_empty() {
                        format!("{}/{}", dir, slug_stem)
                    } else {
                        format!("{}/{}.{}", dir, slug_stem, ext)
                    }
                };

                let new_url = format!(
                    "{}/{}",
                    articles_url.trim_end_matches('/'),
                    dst_rel
                );

                assets.push((abs_path, dst_rel));
                format!(r#"{}="{}""#, attr, new_url)
            } else {
                format!(r#"{}="{}""#, attr, val)
            }
        })
        .to_string();

    (new_html, assets)
}

/// Simple percent-decoding for URL-encoded paths.
fn percent_decode(s: &str) -> String {
    let mut result = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(
                &s[i + 1..i + 3],
                16,
            ) {
                result.push(byte);
                i += 3;
                continue;
            }
        }
        result.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&result).to_string()
}

fn section_to_meta(section: &ScannedSection) -> data::SectionMeta {
    data::SectionMeta {
        entity_path: data::EntityPath {
            ids: section.ids.clone(),
            rel_path: section.rel_path.clone(),
        },
        title: section.title.clone(),
        children: section
            .children
            .iter()
            .map(|child| match child {
                ScannedNode::Article(a) => data::NodeMeta::Article(article_to_meta(a)),
                ScannedNode::Section(s) => data::NodeMeta::Section(section_to_meta(s)),
            })
            .collect(),
        index: section.index.as_ref().map(article_to_meta),
        description: section.description.clone(),
    }
}

fn build_vault_meta(vault: &ScannedVault) -> data::VaultMeta {
    let posts = vault.posts.iter().map(article_to_meta).collect();
    let notes = vault.notes.iter().map(|s| section_to_meta(s)).collect();
    data::VaultMeta {
        version: env!("CARGO_PKG_VERSION").to_string(),
        posts,
        notes,
    }
}
