use optify::builder::OptionsProviderBuilder;

#[test]
fn test_builder_circular_imports() {
    let path = std::path::Path::new("tests/circular_imports");
    let mut builder = OptionsProviderBuilder::new();
    builder.add_directory(path).unwrap();
    match builder.build() {
        Ok(_) => panic!("Expected an error."),
        Err(e) => {
            // Just use a big regex instead of being consistent slowing down the builder to keep ordered maps or sets.
            let pattern = r#"Error when resolving imports for 'a': Cycle detected with import 'b'. The features in the path \(not in order\): \{"a", "b"}|Error when resolving imports for 'a': Cycle detected with import 'b'. The features in the path \(not in order\): \{"b", "a"}|Error when resolving imports for 'b': Cycle detected with import 'a'. The features in the path \(not in order\): \{"a", "b"}|Error when resolving imports for 'b': Cycle detected with import 'a'. The features in the path \(not in order\): \{"b", "a"}"#;
            assert!(
                regex::Regex::new(pattern).unwrap().is_match(&e),
                "Got: {e}\nExpected pattern: {pattern}",
            );
        }
    }
}

#[test]
fn test_builder_cycle_in_imports() {
    let path = std::path::Path::new("tests/cycle_in_imports");
    let mut builder = OptionsProviderBuilder::new();
    builder.add_directory(path).unwrap();
    match builder.build() {
        Ok(_) => panic!("Expected an error."),
        Err(e) => {
            let pattern = r#"Error when resolving imports for '[abc]': Cycle detected with import '[abc]'. The features in the path \(not in order\): \{("([abc]|start)", ){2,3}"([abc]|start)"}"#;
            assert!(
                regex::Regex::new(pattern).unwrap().is_match(&e),
                "Got: {e}\nExpected pattern: {pattern}",
            );
        }
    }
}

#[test]
fn test_builder_duplicate_alias() {
    let path = std::path::Path::new("tests/duplicate_alias");
    let mut builder = OptionsProviderBuilder::new();
    match builder.add_directory(path) {
        Ok(_) => panic!("Expected an error."),
        Err(e) => {
            let pattern = r#"The alias 'b' for canonical feature name 'a' is already mapped to 'b'\.|The alias 'b' for canonical feature name 'b' is already mapped to 'a'\."#;
            assert!(
                regex::Regex::new(pattern).unwrap().is_match(&e),
                "Got: {e}\nExpected pattern: {pattern}",
            );
        }
    }
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
