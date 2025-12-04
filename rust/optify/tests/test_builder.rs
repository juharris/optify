use optify::{
    builder::{OptionsProviderBuilder, OptionsRegistryBuilder},
    provider::{OptionsProvider, OptionsRegistry},
};

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
    let provider =
        OptionsProvider::build_from_directories_with_schema(&[configs_dir], &schema_path)?;

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
