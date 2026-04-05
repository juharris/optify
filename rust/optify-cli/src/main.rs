use clap::{Parser, Subcommand};
use optify::provider::{GetOptionsPreferences, OptionsProvider, OptionsRegistry};
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

        /// JSON object of preferences forwarded to the provider. Supported keys:
        /// `are_configurable_strings_enabled` (bool), `skip_feature_name_conversion` (bool),
        /// `constraints` (JSON value), `overrides` (JSON value).
        #[arg(long = "preferences", visible_alias = "prefs", value_name = "JSON")]
        preferences: Option<String>,
    },

    /// Get options for a specific configuration key with the given features.
    GetOptions {
        /// The configuration key to retrieve (e.g. "myConfig").
        key: String,

        /// Feature names to apply, in order from lowest to highest priority.
        #[arg(short, long, value_name = "FEATURE", num_args = 0..)]
        features: Vec<String>,

        /// JSON object of preferences forwarded to the provider. Supported keys:
        /// `are_configurable_strings_enabled` (bool), `skip_feature_name_conversion` (bool),
        /// `constraints` (JSON value), `overrides` (JSON value).
        #[arg(long = "preferences", visible_alias = "prefs", value_name = "JSON")]
        preferences: Option<String>,
    },
}

/// Parse a JSON string into a `GetOptionsPreferences`.
///
/// Recognised object keys match the fields on `GetOptionsPreferences`. Unknown
/// keys return an error to catch typos at the CLI layer.
fn parse_preferences(json: &str) -> Result<GetOptionsPreferences, String> {
    let value: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| format!("Invalid --preferences JSON: {e}"))?;

    let object = value
        .as_object()
        .ok_or_else(|| "--preferences must be a JSON object".to_string())?;

    let mut preferences = GetOptionsPreferences::new();

    for (key, val) in object {
        match key.as_str() {
            "are_configurable_strings_enabled" => {
                preferences.are_configurable_strings_enabled = val.as_bool().ok_or_else(|| {
                    "preferences.are_configurable_strings_enabled must be a boolean".to_string()
                })?;
            }
            "skip_feature_name_conversion" => {
                preferences.skip_feature_name_conversion = val.as_bool().ok_or_else(|| {
                    "preferences.skip_feature_name_conversion must be a boolean".to_string()
                })?;
            }
            "constraints" => {
                preferences.set_constraints(Some(val.clone()));
            }
            "overrides" => {
                preferences.overrides = Some(val.clone());
            }
            other => {
                return Err(format!("Unknown preferences key: {other}"));
            }
        }
    }

    Ok(preferences)
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

        Commands::GetAllOptions {
            features,
            preferences,
        } => {
            let preferences = preferences.as_deref().map(parse_preferences).transpose()?;
            // No caching needed for a one-shot CLI invocation.
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
            let preferences = preferences.as_deref().map(parse_preferences).transpose()?;
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
