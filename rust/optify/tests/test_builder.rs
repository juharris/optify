use optify::{
    builder::{BuilderOptions, OptionsProviderBuilder, OptionsRegistryBuilder},
    provider::{GetOptionsPreferences, OptionsProvider, OptionsRegistry},
};

use serde_json::json;

#[test]
fn test_builder_circular_imports() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new("tests/circular_imports");
    match OptionsProvider::build(path) {
        Ok(_) => panic!("Expected an error."),
        Err(e) => {
            // Just use a big regex instead of being consistent slowing down the builder to keep ordered maps or sets.
            let pattern = r#"Error when resolving imports for 'a': Cycle detected with import 'b'. The features in the path \(not in order\): \{"a", "b"}|Error when resolving imports for 'a': Cycle detected with import 'b'. The features in the path \(not in order\): \{"b", "a"}|Error when resolving imports for 'b': Cycle detected with import 'a'. The features in the path \(not in order\): \{"a", "b"}|Error when resolving imports for 'b': Cycle detected with import 'a'. The features in the path \(not in order\): \{"b", "a"}"#;
            assert!(
                regex::Regex::new(pattern)?.is_match(&e),
                "Got: {e}\nExpected pattern: {pattern}",
            );
            Ok(())
        }
    }
}

#[test]
fn test_builder_cycle_in_imports() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new("tests/cycle_in_imports");
    match OptionsProvider::build_from_directories(&[path]) {
        Ok(_) => panic!("Expected an error."),
        Err(e) => {
            let pattern = r#"Error when resolving imports for '[abc]': Cycle detected with import '[abc]'. The features in the path \(not in order\): \{("([abc]|start)", ){2,3}"([abc]|start)"}"#;
            assert!(
                regex::Regex::new(pattern)?.is_match(&e),
                "Got: {e}\nExpected pattern: {pattern}",
            );
            Ok(())
        }
    }
}

#[test]
fn test_builder_duplicate_alias() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new("tests/duplicate_alias");
    let mut builder = OptionsProviderBuilder::new();
    match builder.add_directory(path) {
        Ok(_) => panic!("Expected an error."),
        Err(e) => {
            let pattern = r#"The alias 'b' for canonical feature name 'a' is already mapped to 'b'\.|The alias 'b' for canonical feature name 'b' is already mapped to 'a'\."#;
            assert!(
                regex::Regex::new(pattern)?.is_match(&e),
                "Got: {e}\nExpected pattern: {pattern}",
            );
            Ok(())
        }
    }
}

#[test]
fn test_builder_invalid_file() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new("tests/invalid_file");
    match OptionsProvider::build(path) {
        Ok(_) => panic!("Expected an error."),
        Err(e) => {
            let expected_path = dunce::canonicalize("tests/invalid_file/invalid.yaml")
                .unwrap()
                .to_string_lossy()
                .to_string();
            assert_eq!(e, format!("Error loading file '{expected_path}': simple key expected at byte 31 line 4 column 1 in tests/invalid_file/invalid.yaml"));
            Ok(())
        }
    }
}

#[test]
fn test_builder_conditions_in_imported_feature() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new("../../tests/invalid_suites/conditions_in_import/configs");
    match OptionsProvider::build(path) {
        Ok(_) => panic!("Expected an error."),
        Err(e) => {
            assert!(e.starts_with(
                "Error when resolving imports for 'parent': The import 'invalid' has conditions. Conditions cannot be used in imported features."
            ));
            Ok(())
        }
    }
}

#[test]
fn test_builder_invalid_condition_pattern() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new("../../tests/invalid_suites/invalid_condition_pattern/configs");
    let feature_file = path.join("invalid.yaml");
    match OptionsProvider::build(path) {
        Ok(_) => panic!("Expected an error."),
        Err(e) => {
            let expected_path = dunce::canonicalize(feature_file)
                .unwrap()
                .to_string_lossy()
                .to_string();
            let expected = format!(
                "Error deserializing configuration for file '{expected_path}': regex parse error:\n    {{invalid}}\n    ^\nerror: repetition operator missing expression"
            );
            assert_eq!(e, expected);
            Ok(())
        }
    }
}

#[test]
fn test_builder_name_with_no_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new("tests/no_metadata");
    let mut builder = OptionsProviderBuilder::new();
    builder.add_directory(path)?;
    let provider = builder.build()?;
    let metadata = provider
        .get_feature_metadata("subdir/a")
        .expect("metadata should be found");
    assert_eq!(metadata.name, Some("subdir/a".to_string()));
    assert_eq!(metadata.dependents, None);
    let opts = provider.get_options("wtv", &["subdir/a"])?;
    assert_eq!(opts.as_i64(), Some(3));
    Ok(())
}

#[test]
fn test_builder_tracking() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new("tests/configurable_string_resources_tracked");
    let provider = OptionsProvider::build(path)?;

    let referenced_features = provider
        .get_features_referencing_file("simple.txt")
        .expect("Should have tracking data for simple.txt");
    assert_eq!(referenced_features, vec!["feature_with_cs"]);
    Ok(())
}

#[test]
fn test_builder_used_canonical_alias() {
    let path = std::path::Path::new("tests/used_canonical_name");
    let mut builder = OptionsProviderBuilder::new();
    match builder.add_directory(path) {
        Ok(_) => panic!("Expected an error."),
        Err(e) => {
            assert_eq!(
                e,
                "The alias 'a' for canonical feature name 'a' is already mapped to 'a'.",
            );
        }
    }
}

#[test]
fn test_build_from_directories_with_schema() -> Result<(), Box<dyn std::error::Error>> {
    let configs_dir = std::path::Path::new("../../tests/test_suites/inheritance/configs");
    let schema_path = configs_dir.join(".optify/schema.json");
    let options = BuilderOptions {
        schema_path: Some(schema_path),
        ..BuilderOptions::default()
    };
    let provider = OptionsProvider::build_from_directories_with_options(&[configs_dir], options)?;

    let features = provider.get_features();
    assert!(!features.is_empty());
    let feature_c = "feature_C".to_string();
    assert!(features.contains(&feature_c));

    let options = provider.get_options("myConfig", &[feature_c])?;
    assert_eq!(options["myObject"]["one"], 11);
    assert_eq!(options["myObject"]["two"], 2);
    assert_eq!(options["myObject"]["three"], 3);

    Ok(())
}

#[test]
fn test_with_options_applies_to_add_directory() -> Result<(), Box<dyn std::error::Error>> {
    // When `with_options` is called before `add_directory`, the options should be used
    // for processing entries in that directory (even without a `.optify/config.json`).
    let path = std::path::Path::new("tests/configurable_string_no_config");

    let provider = OptionsProvider::build(path)?;

    let referenced_features = provider.get_features_referencing_file("simple.txt");
    assert_eq!(referenced_features, None);

    let prefs = GetOptionsPreferences {
        are_configurable_strings_enabled: true,
        ..GetOptionsPreferences::default()
    };
    let message_obj = provider.get_options_with_preferences(
        "my_message",
        &["feature_with_cs"],
        None,
        Some(&prefs),
    )?;
    assert_eq!(
        message_obj,
        json!({"$type": "Optify.ConfigurableString", "base": {"file": "simple.txt"}})
    );

    let options = BuilderOptions {
        are_configurable_strings_enabled: true,
        track_file_references: optify::builder::TrackReferenceMode::ConfigurableStrings,
        ..BuilderOptions::default()
    };
    let provider = OptionsProvider::build_with_options(path, options)?;

    let referenced_features = provider
        .get_features_referencing_file("simple.txt")
        .expect("with_options should enable file reference tracking for add_directory");
    assert_eq!(referenced_features, vec!["feature_with_cs"]);

    let message = provider.get_options_with_preferences(
        "my_message",
        &["feature_with_cs"],
        None,
        Some(&prefs),
    )?;
    assert_eq!(message, "Hello from simple file!");
    Ok(())
}

#[test]
fn test_directory_config_enables_configurable_strings_without_builder_options(
) -> Result<(), Box<dyn std::error::Error>> {
    // The directory `.optify/config.json` enables configurable strings even when
    // the builder-level options leave them disabled.
    let path = std::path::Path::new("tests/configurable_string_resources_tracked");
    let provider = OptionsProvider::build(path)?;

    let referenced_features = provider
        .get_features_referencing_file("simple.txt")
        .expect("directory config should enable file reference tracking");
    assert_eq!(referenced_features, vec!["feature_with_cs"]);
    Ok(())
}

#[test]
fn test_directory_config_merges_with_builder_options() -> Result<(), Box<dyn std::error::Error>> {
    // The directory `.optify/config.json` does NOT set `trackFileReferences`.
    let path = std::path::Path::new("tests/configurable_string_cs_only");
    let options = BuilderOptions {
        track_file_references: optify::builder::TrackReferenceMode::ConfigurableStrings,
        ..BuilderOptions::default()
    };

    let provider = OptionsProvider::build(path)?;
    let referenced_features = provider.get_features_referencing_file("simple.txt");
    assert_eq!(referenced_features, None);

    let provider = OptionsProvider::build_with_options(path, options)?;

    let referenced_features = provider
        .get_features_referencing_file("simple.txt")
        .expect("merged options should enable file reference tracking");
    assert_eq!(referenced_features, vec!["feature_with_cs"]);
    Ok(())
}
