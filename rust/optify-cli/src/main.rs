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
    /// List all available feature names.
    ListFeatures {
        /// Also list aliases alongside canonical feature names.
        #[arg(long)]
        include_aliases: bool,
    },

    /// Get all options (full merged configuration) for the given features.
    GetAllOptions {
        /// Feature names to apply, in order from lowest to highest priority.
        #[arg(short, long, value_name = "FEATURE", num_args = 0..)]
        features: Vec<String>,

        /// Output compact JSON instead of pretty-printed JSON.
        #[arg(long)]
        compact: bool,
    },

    /// Get options for a specific configuration key with the given features.
    GetOptions {
        /// The configuration key to retrieve (e.g. "myConfig").
        key: String,

        /// Feature names to apply, in order from lowest to highest priority.
        #[arg(short, long, value_name = "FEATURE", num_args = 0..)]
        features: Vec<String>,

        /// Output compact JSON instead of pretty-printed JSON.
        #[arg(long)]
        compact: bool,
    },
}

fn build_provider(dirs: &[PathBuf], schema: Option<&PathBuf>) -> Result<OptionsProvider, String> {
    match schema {
        Some(schema_path) => OptionsProvider::build_from_directories_with_schema(dirs, schema_path),
        None => OptionsProvider::build_from_directories(dirs),
    }
}

fn serialize(value: &serde_json::Value, compact: bool) -> Result<String, String> {
    if compact {
        serde_json::to_string(value)
    } else {
        serde_json::to_string_pretty(value)
    }
    .map_err(|e| format!("Failed to serialize options: {e}"))
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();

    let provider = build_provider(&cli.dirs, cli.schema.as_ref())?;

    match cli.command {
        Commands::ListFeatures { include_aliases } => {
            let mut names = if include_aliases {
                provider.get_features_and_aliases()
            } else {
                provider.get_features()
            };
            names.sort();
            for name in names {
                println!("{name}");
            }
        }

        Commands::GetAllOptions { features, compact } => {
            // No caching or preferences needed for a one-shot CLI invocation.
            let value = provider.get_all_options(&features, None, None)?;
            println!("{}", serialize(&value, compact)?);
        }

        Commands::GetOptions {
            key,
            features,
            compact,
        } => {
            let value = provider.get_options(&key, &features)?;
            println!("{}", serialize(&value, compact)?);
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
