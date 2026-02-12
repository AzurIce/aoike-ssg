use anyhow::Result;
use pulldown_cmark::Options;
use serde::Deserialize;

/// YAML frontmatter extracted from a markdown file.
#[derive(Debug, Default, Deserialize)]
pub struct Frontmatter {
    pub title: Option<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub tags: Option<Vec<String>>,
    pub publish: Option<bool>,
    pub description: Option<String>,
    #[serde(flatten)]
    pub extra: Option<serde_json::Value>,
}

/// Parsed markdown file.
#[derive(Debug)]
pub struct ParsedMarkdown {
    pub frontmatter: Frontmatter,
    pub content_html: String,
    pub raw_content: String,
}

/// Parse a markdown file, extracting YAML frontmatter and converting body to HTML.
pub fn parse_markdown(source: &str) -> Result<ParsedMarkdown> {
    let (frontmatter, body) = extract_frontmatter(source);

    let mut options = Options::empty();
    options.extend([
        Options::ENABLE_TABLES,
        Options::ENABLE_FOOTNOTES,
        Options::ENABLE_STRIKETHROUGH,
        Options::ENABLE_TASKLISTS,
        Options::ENABLE_HEADING_ATTRIBUTES,
        Options::ENABLE_MATH,
        Options::ENABLE_GFM,
        Options::ENABLE_SMART_PUNCTUATION,
    ]);

    let parser = pulldown_cmark::Parser::new_ext(body, options);
    let mut content_html = String::new();
    pulldown_cmark::html::push_html(&mut content_html, parser);

    Ok(ParsedMarkdown {
        frontmatter,
        content_html,
        raw_content: body.to_string(),
    })
}

/// Extract YAML frontmatter from markdown source.
/// Returns (Frontmatter, body_after_frontmatter).
fn extract_frontmatter(source: &str) -> (Frontmatter, &str) {
    let trimmed = source.trim_start();
    if !trimmed.starts_with("---") {
        return (Frontmatter::default(), source);
    }

    // Find the closing ---
    let after_open = &trimmed[3..];
    if let Some(end) = after_open.find("\n---") {
        let yaml_str = &after_open[..end];
        let body_start = end + 4; // skip \n---
        let body = after_open[body_start..].trim_start_matches(['\n', '\r']);

        match serde_yaml::from_str::<Frontmatter>(yaml_str) {
            Ok(fm) => (fm, body),
            Err(_) => (Frontmatter::default(), source),
        }
    } else {
        (Frontmatter::default(), source)
    }
}
