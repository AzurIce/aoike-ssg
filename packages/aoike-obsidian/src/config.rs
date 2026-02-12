use std::path::Path;

use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub mapping: Mapping,
    #[serde(default)]
    pub ignore: Ignore,
}

#[derive(Debug, Deserialize)]
pub struct Mapping {
    #[serde(default = "default_posts")]
    pub posts: String,
    #[serde(default = "default_notes")]
    pub notes: Vec<String>,
}

impl Default for Mapping {
    fn default() -> Self {
        Self {
            posts: default_posts(),
            notes: default_notes(),
        }
    }
}

fn default_posts() -> String {
    "posts".to_string()
}

fn default_notes() -> Vec<String> {
    vec!["notes".to_string()]
}

#[derive(Debug, Deserialize)]
pub struct Ignore {
    #[serde(default = "default_ignore_patterns")]
    pub patterns: Vec<String>,
}

impl Default for Ignore {
    fn default() -> Self {
        Self {
            patterns: default_ignore_patterns(),
        }
    }
}

fn default_ignore_patterns() -> Vec<String> {
    vec![
        ".obsidian".to_string(),
        "templates".to_string(),
        ".trash".to_string(),
    ]
}

impl Config {
    pub fn load(vault_dir: &Path) -> Result<Self> {
        let config_path = vault_dir.join("aoike.toml");
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Config {
                mapping: Mapping::default(),
                ignore: Ignore::default(),
            })
        }
    }
}
