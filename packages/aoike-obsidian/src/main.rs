use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "aoike-obsidian", about = "Export an Obsidian vault to aoike JSON format")]
struct Cli {
    /// Path to the Obsidian vault directory
    vault_dir: PathBuf,

    /// Output directory for exported JSON
    #[arg(short, long, default_value = "output")]
    output: PathBuf,

    /// Public URL prefix for asset links
    #[arg(long, default_value = "")]
    public_url_prefix: String,

    /// Skip files with `publish: false` in frontmatter
    #[arg(long)]
    respect_publish: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = aoike_obsidian::config::Config::load(&cli.vault_dir)?;
    let vault = aoike_obsidian::vault::scan_vault(&cli.vault_dir, &config, cli.respect_publish)?;
    aoike_obsidian::export::export(&vault, &cli.output, &cli.public_url_prefix)?;

    eprintln!(
        "Exported {} posts and {} note sections to {}",
        vault.posts.len(),
        vault.notes.len(),
        cli.output.display()
    );
    Ok(())
}
