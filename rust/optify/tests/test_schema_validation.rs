use optify::builder::{OptionsProviderBuilder, OptionsRegistryBuilder};
use std::path::Path;

#[test]
fn test_simple_configs_adhere_to_schema() {
    let mut builder = OptionsProviderBuilder::new();

    let schema_path = Path::new("../../schemas/feature_file.json");
    builder
        .with_schema(schema_path)
        .expect("Failed to load schema");

    let configs_dir = Path::new("../../tests/test_suites/simple/configs");
    let result = builder.add_directory(configs_dir);

    assert!(
        result.is_ok(),
        "Schema validation failed: {:?}",
        result.err()
    );

    let provider = builder.build();
    assert!(
        provider.is_ok(),
        "Failed to build provider: {:?}",
        provider.err()
    );
}

#[test]
fn test_invalid_file_fails_schema_validation() {
    use std::fs;
    use tempfile::TempDir;

    let mut builder = OptionsProviderBuilder::new();

    // Load the schema
    let schema_path = Path::new("../../schemas/feature_file.json");
    builder
        .with_schema(schema_path)
        .expect("Failed to load schema");

    // Create a temporary directory with an invalid config file
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let invalid_file_path = temp_dir.path().join("invalid.json");

    // Write an invalid config (missing required properties based on schema)
    let invalid_config = r#"{
            "invalidProperty": "this property is not allowed by the schema"
        }"#;
    fs::write(&invalid_file_path, invalid_config).expect("Failed to write invalid file");

    // Try to load the directory - this should fail schema validation
    let result = builder.add_directory(temp_dir.path());

    assert!(
        result.is_err(),
        "Expected schema validation to fail for invalid file"
    );

    let error_message = result.err().unwrap();
    assert!(
        error_message.contains("Schema validation failed"),
        "Expected error message to mention schema validation, got: {error_message}"
    );
}
