use optify::{
    builder::{OptionsRegistryBuilder, OptionsWatcherBuilder},
    provider::{OptionsRegistry, OptionsWatcher},
};

#[test]
fn test_simple_configs_adhere_to_schema() -> Result<(), String> {
    let configs_dir = "../../tests/test_suites/simple/configs";
    let schema_path = "../../schemas/feature_file.json";
    let result = OptionsWatcher::build_with_schema(configs_dir, schema_path);

    assert!(
        result.is_ok(),
        "Schema validation failed: {:?}",
        result.err()
    );

    Ok(())
}

#[test]
#[ignore]
fn test_schema_with_urls() -> Result<(), String> {
    let configs_dir = "../../tests/test_suites/inheritance/configs";
    let schema_path = "../../tests/test_suites/inheritance/configs/.optify/schema.json";
    let result = OptionsWatcher::build_with_schema(configs_dir, schema_path);

    assert!(
        result.is_ok(),
        "Schema validation failed: {:?}",
        result.err()
    );

    Ok(())
}

#[test]
fn test_invalid_file_fails_schema_validation() -> Result<(), String> {
    use std::fs;
    use tempfile::TempDir;

    let mut builder = OptionsWatcherBuilder::new();

    builder.with_schema("../../schemas/feature_file.json")?;

    // Create a temporary directory with an invalid config file
    let temp_dir = TempDir::new().expect("temp dir to be created");
    let invalid_file_path = temp_dir.path().join("invalid.json");

    // Write an invalid config (missing required properties based on schema)
    let invalid_config = r#"{
            "invalidProperty": "this property is not allowed by the schema"
        }"#;
    fs::write(&invalid_file_path, invalid_config).expect("invalid file to be written");

    // Try to load the directory - this should fail schema validation
    builder.add_directory(temp_dir.path())?;

    let result = builder.build();

    assert!(
        result.is_err(),
        "Expected schema validation to fail for invalid file"
    );

    let error_message = result.err().unwrap();
    assert!(
        error_message.contains("Schema validation failed"),
        "Expected error message to mention schema validation, got: {error_message}"
    );

    Ok(())
}
