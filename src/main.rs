use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the vault directory (source)
    #[arg(value_name = "VAULT_DIR", default_value = ".")]
    vault: PathBuf,

    /// Path to the output directory
    #[arg(short, long, value_name = "OUTPUT_DIR", default_value = "static/vault")]
    output: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    #[cfg(feature = "build")]
    {
        println!("Building vault from: {:?}", cli.vault);
        let vault = aoike::build::build_vault(&cli.vault);
        
        println!("Exporting vault to: {:?}", cli.output);
        aoike::build::export_vault(&vault, &cli.output);
        
        println!("Done!");
    }

    #[cfg(not(feature = "build"))]
    {
        let _ = cli; // suppress unused warning
        eprintln!("Error: The 'build' feature must be enabled to use this CLI.");
        eprintln!("Try running with: cargo run --features build");
        std::process::exit(1);
    }
}
