use std::collections::HashMap;
use std::path::Path;

use regex::Regex;

/// Process wikilinks in HTML content.
///
/// - `[[target]]` and `[[target|alias]]` → `<a>` tags (resolved via `link_index`)
/// - `![[image.png]]` → `<img>` tags with a vault-root-relative `src` path
///   (resolved via `asset_index`, so export can later rewrite and copy)
pub fn process_wikilinks(
    html: &str,
    link_index: &HashMap<String, String>,
    asset_index: &HashMap<String, String>,
    vault_dir: &Path,
    source_path: &Path,
) -> (String, Vec<String>) {
    let mut outlinks = Vec::new();
    let source_dir = source_path.parent().unwrap_or(vault_dir);

    // Process image embeds: ![[filename.ext]] or ![[filename.ext|alt]]
    let img_re = Regex::new(r"!\[\[([^\]|]+?)(\|([^\]]*))?\]\]").unwrap();
    let result = img_re.replace_all(html, |caps: &regex::Captures| {
        let target = caps.get(1).unwrap().as_str().trim();
        let alt = caps.get(3).map_or(target, |m| m.as_str());

        // 1. Try asset_index (vault-wide filename lookup)
        if let Some(vault_rel) = asset_index.get(target) {
            // Produce a path relative to the source file so that
            // export::rewrite_html_assets can resolve it against source_dir.
            let abs = vault_dir.join(vault_rel);
            if let Ok(rel_to_source) = pathdiff_relative(source_dir, &abs) {
                return format!(r#"<img src="{}" alt="{}">"#, rel_to_source, alt);
            }
        }

        // 2. Fallback: try as a path relative to the source file
        let candidate = source_dir.join(target);
        if candidate.exists() {
            return format!(r#"<img src="{}" alt="{}">"#, target, alt);
        }

        // 3. Unresolved — leave as-is so it's visible
        format!(r#"<img src="{}" alt="{}" class="wikilink-broken">"#, target, alt)
    });

    // Process link wikilinks: [[target]] or [[target|display]]
    let link_re = Regex::new(r"\[\[([^\]|]+?)(\|([^\]]*))?\]\]").unwrap();
    let result = link_re.replace_all(&result, |caps: &regex::Captures| {
        let target = caps.get(1).unwrap().as_str().trim();
        let display = caps.get(3).map_or(target, |m| m.as_str());
        if let Some(ids_path) = link_index.get(target) {
            outlinks.push(ids_path.clone());
            format!(r#"<a href="/notes/{}">{}</a>"#, ids_path, display)
        } else {
            format!(r#"<span class="wikilink-broken">{}</span>"#, display)
        }
    });

    (result.into_owned(), outlinks)
}

/// Compute a relative path from `base` directory to `target` file,
/// using forward slashes (for use in HTML src attributes).
fn pathdiff_relative(base: &Path, target: &Path) -> Result<String, ()> {
    // Normalize both to canonical if possible, fall back to as-is
    let base = base.canonicalize().unwrap_or_else(|_| base.to_path_buf());
    let target = target.canonicalize().unwrap_or_else(|_| target.to_path_buf());

    // Walk up from base, walk down into target
    let base_components: Vec<_> = base.components().collect();
    let target_components: Vec<_> = target.components().collect();

    // Find common prefix length
    let common = base_components
        .iter()
        .zip(target_components.iter())
        .take_while(|(a, b)| a == b)
        .count();

    if common == 0 {
        return Err(());
    }

    let ups = base_components.len() - common;
    let mut parts: Vec<String> = std::iter::repeat_n("..".to_string(), ups).collect();
    for comp in &target_components[common..] {
        parts.push(comp.as_os_str().to_string_lossy().to_string());
    }

    Ok(parts.join("/"))
}
