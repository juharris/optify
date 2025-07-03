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
            assert_eq!(e, "Error loading file 'tests/invalid_file/invalid.yaml': simple key expected at byte 31 line 4 column 1 in tests/invalid_file/invalid.yaml");
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
