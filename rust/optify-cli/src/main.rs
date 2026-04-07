use clap::{Parser, Subcommand};
use optify::provider::{OptionsProvider, OptionsRegistry};
use std::path::PathBuf;
use std::process;

/// Inspect and query Optify configuration options from config directories.
#[derive(Parser)]
#[command(name = "optify", version, about, long_about = None)]
struct Cli {
    /// Path to a configuration directory.
    /// Can be specified multiple times to load from multiple directories.
    #[arg(short = 'd', long = "dir", value_name = "DIR", required = true)]
    dirs: Vec<PathBuf>,

    /// Optional path to a JSON schema file for validating configurations.
    #[arg(long, value_name = "PATH")]
    schema: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all features with their metadata as a JSON object.
    ListFeatures,

    /// Get all options (full merged configuration) for the given features.
    GetAllOptions {
        /// Feature names to apply, in order from lowest to highest priority.
        #[arg(short, long, value_name = "FEATURE", num_args = 0..)]
        features: Vec<String>,
    },

    /// Get options for a specific configuration key with the given features.
    GetOptions {
        /// The configuration key to retrieve (e.g. "myConfig").
        #[arg(short, long)]
        key: String,

        /// Feature names to apply, in order from lowest to highest priority.
        #[arg(short, long, value_name = "FEATURE", num_args = 0..)]
        features: Vec<String>,
    },
}

fn build_provider(dirs: &[PathBuf], schema: Option<&PathBuf>) -> Result<OptionsProvider, String> {
    match schema {
        Some(schema_path) => OptionsProvider::build_from_directories_with_schema(dirs, schema_path),
        None => OptionsProvider::build_from_directories(dirs),
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();

    let provider = build_provider(&cli.dirs, cli.schema.as_ref())?;

    match cli.command {
        Commands::ListFeatures => {
            let features = provider.get_features_with_metadata();
            let list: Vec<_> = features.into_values().collect();
            println!(
                "{}",
                serde_json::to_string(&list)
                    .map_err(|e| format!("Failed to serialize features: {e}"))?
            );
        }

        Commands::GetAllOptions { features } => {
            // No caching or preferences needed for a one-shot CLI invocation.
            let value = provider.get_all_options(&features, None, None)?;
            println!(
                "{}",
                serde_json::to_string(&value)
                    .map_err(|e| format!("Failed to serialize options: {e}"))?
            );
        }

        Commands::GetOptions { key, features } => {
            let value = provider.get_options(&key, &features)?;
            println!(
                "{}",
                serde_json::to_string(&value)
                    .map_err(|e| format!("Failed to serialize options: {e}"))?
            );
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}
