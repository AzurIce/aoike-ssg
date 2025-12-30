use std::{ops::Deref, path::Path};

use anyhow::Context;

use crate::build::{Entity, Parser, utils};

#[derive(Debug, Clone)]
pub struct ArticleSource {
    pub entity: Entity,
    pub ref_paths: Vec<String>,
    pub title: String,
    pub summary_html: String,
    pub content_html: String,
}

impl Deref for ArticleSource {
    type Target = Entity;
    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl ArticleSource {
    pub fn from_html_entity(content_html: String, entity: Entity) -> Self {
        let title =
            utils::get_tag_content(&content_html, "h1").unwrap_or(entity.base_name().clone());
        let filtered_html = utils::remove_html_tag(&content_html, &["h1"]);
        let summary_html = utils::extract_html_summary(&filtered_html, 200);

        Self {
            entity,
            ref_paths: utils::get_ref_paths(&content_html),
            title,
            summary_html,
            content_html: filtered_html,
        }
    }

    pub fn to_article(&self, entity_path: crate::EntityPath) -> crate::Article {
        crate::Article {
            title: self.title.clone(),
            entity_path,
            summary_html: self.summary_html.clone(),
            content_html: self.content_html.clone(),
            created: self.entity.created,
            updated: self.entity.updated,
        }
    }
}

impl TryFrom<Entity> for ArticleSource {
    type Error = anyhow::Error;
    fn try_from(entity: Entity) -> Result<Self, Self::Error> {
        match entity.extension().as_str() {
            "md" => MarkdownArticleParser::try_parse(entity),
            "typ" => TypstArticleParser::try_parse(entity),
            _ => anyhow::bail!("unsupported file extension: {}", entity.extension()),
        }
    }
}

pub struct TypstArticleParser;

impl Parser for TypstArticleParser {
    type Output = ArticleSource;
    fn try_parse(entity: Entity) -> Result<Self::Output, anyhow::Error> {
        let content_html = compile_typst_to_html(&entity.path)?;
        let content_html = utils::get_tag_content(&content_html, "body").unwrap_or_default();
        Ok(ArticleSource::from_html_entity(content_html, entity))
    }
}

fn compile_typst_to_html(path: impl AsRef<Path>) -> Result<String, anyhow::Error> {
    let child = std::process::Command::new("typst")
        .arg("compile")
        .arg(path.as_ref())
        .arg("-")
        .arg("-fhtml")
        .args(["--features", "html"])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .context("failed to spawn typst")?;

    let output = child
        .wait_with_output()
        .context("failed to wait for typst")?
        .stdout;
    String::from_utf8(output).context("contains invalid utf-8 content")
}

pub struct MarkdownArticleParser;

impl Parser for MarkdownArticleParser {
    type Output = ArticleSource;
    fn try_parse(entity: Entity) -> Result<Self::Output, anyhow::Error> {
        let content = std::str::from_utf8(&entity.content)?;

        let parser = pulldown_cmark::Parser::new(&content);
        let mut content_html = String::new();
        pulldown_cmark::html::push_html(&mut content_html, parser);

        Ok(ArticleSource::from_html_entity(content_html, entity))
    }
}
