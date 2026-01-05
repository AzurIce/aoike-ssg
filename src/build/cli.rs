use std::path::PathBuf;
use tracing::{debug, info};

use clap::Parser;
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to the vault directory (source)
    #[arg(value_name = "VAULT_DIR", default_value = "vault")]
    vault: PathBuf,

    /// Path to the output directory
    #[arg(short, long, value_name = "OUTPUT_DIR", default_value = "static/vault")]
    output: PathBuf,
}

pub fn run_cli() {
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

    fn build_filter() -> EnvFilter {
        const DEFAULT_DIRECTIVES: &[(&str, LevelFilter)] = &[("aoike", LevelFilter::INFO)];
        let mut filter = EnvFilter::from_default_env();
        let env = std::env::var("RUST_LOG").unwrap_or_default();
        for (name, level) in DEFAULT_DIRECTIVES
            .iter()
            .filter(|(name, _)| !env.contains(name))
        {
            filter = filter.add_directive(format!("{name}={level}").parse().unwrap());
        }
        filter
    }

    let indicatif_layer = tracing_indicatif::IndicatifLayer::new();

    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .without_time()
                .with_writer(indicatif_layer.get_stderr_writer()),
        )
        .with(indicatif_layer)
        .with(build_filter())
        .init();

    let public_url_prefix = option_env!("TRUNK_BUILD_PUBLIC_URL").unwrap_or("/");
    let cli = Cli::parse();

    info!("Building vault from: {:?}", cli.vault);

    let root = cli
        .vault
        .canonicalize()
        .unwrap_or_else(|_| cli.vault.clone());
    debug!("root: {root:?}");
    let vault = crate::build::build_vault(&root);

    info!("Exporting vault to: {:?}", cli.output);
    crate::build::export_vault(&vault, &cli.output, public_url_prefix);

    info!("Done!");
}
