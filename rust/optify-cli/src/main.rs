use clap::{Parser, Subcommand, ValueEnum};
use optify::builder::BuilderOptions;
use optify::provider::{GetOptionsPreferences, OptionsProvider, OptionsRegistry};
use std::path::PathBuf;
use std::process;

mod format_analysis;

use format_analysis::format_analysis_text;

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
    /// Explain which feature (and import chain) set the value at a path in the options.
    Analyze {
        /// JSON pointer to the value to explain, such as "/myConfig/myKey".
        #[arg(short, long, value_name = "JSON_POINTER")]
        path: String,

        /// Feature names to apply, in order from lowest to highest priority.
        #[arg(short, long, value_name = "FEATURE", num_args = 0..)]
        features: Vec<String>,

        /// JSON preferences for configuring how options are resolved.
        #[arg(long = "preferences", visible_alias = "prefs", value_name = "JSON")]
        preferences: Option<String>,

        /// The output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },

    /// List all features with their metadata as a JSON object.
    ListFeatures,

    /// Get all options (full merged configuration) for the given features.
    GetAllOptions {
        /// Feature names to apply, in order from lowest to highest priority.
        #[arg(short, long, value_name = "FEATURE", num_args = 0..)]
        features: Vec<String>,

        /// JSON preferences for configuring how options are resolved.
        #[arg(long = "preferences", visible_alias = "prefs", value_name = "JSON")]
        preferences: Option<String>,
    },

    /// Get options for a specific configuration key with the given features.
    GetOptions {
        /// The configuration key to retrieve (e.g. "myConfig").
        #[arg(short, long)]
        key: String,

        /// Feature names to apply, in order from lowest to highest priority.
        #[arg(short, long, value_name = "FEATURE", num_args = 0..)]
        features: Vec<String>,

        /// JSON preferences for configuring how options are resolved.
        #[arg(long = "preferences", visible_alias = "prefs", value_name = "JSON")]
        preferences: Option<String>,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    /// Compact single-line JSON.
    Json,
    /// Human-readable lines.
    Text,
}

fn parse_preferences(json: Option<&str>) -> Result<Option<GetOptionsPreferences>, String> {
    json.map(|s| serde_json::from_str(s).map_err(|e| format!("Failed to parse preferences: {e}")))
        .transpose()
}

fn build_provider(
    dirs: &[PathBuf],
    schema: Option<&PathBuf>,
    is_analysis_enabled: bool,
) -> Result<OptionsProvider, String> {
    let options = BuilderOptions {
        is_analysis_enabled,
        schema_path: schema.cloned(),
        ..BuilderOptions::default()
    };
    OptionsProvider::build_from_directories_with_options(dirs, options)
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();

    // Analysis retains more data, so only enable it when it is needed.
    let is_analysis_enabled = matches!(cli.command, Commands::Analyze { .. });
    let provider = build_provider(&cli.dirs, cli.schema.as_ref(), is_analysis_enabled)?;

    match cli.command {
        Commands::Analyze {
            path,
            features,
            preferences,
            format,
        } => {
            let preferences = parse_preferences(preferences.as_deref())?;
            let analysis = provider.analyze(&path, &features, preferences.as_ref())?;
            match format {
                OutputFormat::Json => println!(
                    "{}",
                    serde_json::to_string(&analysis)
                        .map_err(|e| format!("Failed to serialize analysis: {e}"))?
                ),
                OutputFormat::Text => println!("{}", format_analysis_text(&path, &analysis)),
            }
        }

        Commands::ListFeatures => {
            let features = provider.get_features_with_metadata();
            let list: Vec<_> = features.into_values().collect();
            println!(
                "{}",
                serde_json::to_string(&list)
                    .map_err(|e| format!("Failed to serialize features: {e}"))?
            );
        }

        Commands::GetAllOptions {
            features,
            preferences,
        } => {
            let preferences = parse_preferences(preferences.as_deref())?;
            let value = provider.get_all_options(&features, None, preferences.as_ref())?;
            println!(
                "{}",
                serde_json::to_string(&value)
                    .map_err(|e| format!("Failed to serialize options: {e}"))?
            );
        }

        Commands::GetOptions {
            key,
            features,
            preferences,
        } => {
            let preferences = parse_preferences(preferences.as_deref())?;
            let value = provider.get_options_with_preferences(
                &key,
                &features,
                None,
                preferences.as_ref(),
            )?;
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
