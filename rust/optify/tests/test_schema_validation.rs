use optify::{
    builder::{OptionsRegistryBuilder, OptionsWatcherBuilder},
    provider::{OptionsProvider, OptionsRegistry, OptionsWatcher},
};

const CONFIGURABLE_VALUES_CONFIGS_DIR: &str = "../../tests/test_suites/configurable_values/configs";
const CONFIGURABLE_VALUES_SCHEMA_PATH: &str =
    "../../tests/test_suites/configurable_values/configs/.optify/schema.json";

fn build_configurable_values_config(config: &str) -> Result<OptionsProvider, String> {
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("temp dir to be created");
    let config_path = temp_dir.path().join("config.json");
    fs::write(&config_path, config).expect("config file to be written");

    OptionsProvider::build_with_schema(temp_dir.path(), CONFIGURABLE_VALUES_SCHEMA_PATH)
}

#[test]
fn test_configurable_list_item_type_is_validated() -> Result<(), String> {
    let invalid_configs = [
        (
            "direct array",
            r#"{
                "options": {
                    "tools": [42]
                }
            }"#,
        ),
        (
            "item property",
            r#"{
                "options": {
                    "tools": [{ "description": 42 }]
                }
            }"#,
        ),
        (
            "configurable string property",
            r#"{
                "options": {
                    "tools": [{ "description": { "invalid": true } }]
                }
            }"#,
        ),
        (
            "configurable item",
            r#"{
                "options": {
                    "tools": {
                        "$type": "Optify.ConfigurableList",
                        "invalid": {
                            "$value": 42
                        }
                    }
                }
            }"#,
        ),
    ];

    for (case_name, invalid_config) in invalid_configs {
        let result = build_configurable_values_config(invalid_config);
        let error_message = match result {
            Ok(_) => {
                return Err(format!(
                    "Expected {case_name} ConfigurableList item schema validation to fail"
                ));
            }
            Err(error) => error,
        };

        assert!(
            error_message.contains("Schema validation failed"),
            "Expected {case_name} schema validation error, got: {error_message}"
        );
    }

    Ok(())
}

#[test]
fn test_configurable_list_partial_items_are_valid() -> Result<(), String> {
    let partial_configs = [
        r#"{
            "options": {
                "tools": [
                    { "description": "Only a description" },
                    { "name": "only_a_name" }
                ]
            }
        }"#,
        r#"{
            "options": {
                "tools": {
                    "$type": "Optify.ConfigurableList",
                    "description_only": {
                        "$value": { "description": "Only a description" }
                    },
                    "name_only": {
                        "$value": { "name": "only_a_name" }
                    },
                    "order_only": {
                        "$order": 1
                    }
                }
            }
        }"#,
    ];

    for partial_config in partial_configs {
        build_configurable_values_config(partial_config)?;
    }

    Ok(())
}

#[test]
fn test_configurable_values_configs_adhere_to_schema() -> Result<(), String> {
    OptionsProvider::build_with_schema(
        CONFIGURABLE_VALUES_CONFIGS_DIR,
        CONFIGURABLE_VALUES_SCHEMA_PATH,
    )
    .map(|_| ())
}

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
fn test_schema_with_urns() -> Result<(), String> {
    let configs_dir = "../../tests/test_suites/inheritance/configs";
    let schema_path = "../../tests/test_suites/inheritance/configs/.optify/schema.json";
    let result = OptionsProvider::build_with_schema(configs_dir, schema_path);

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
        error_message.contains("Failed to build provider: Schema validation failed for \""),
        "Expected error message to mention schema validation, got: {error_message}"
    );
    assert!(
        error_message
            .contains("Additional properties are not allowed ('invalidProperty' was unexpected)"),
        "Expected error message to mention banned properties, got: {error_message}"
    );

    Ok(())
}
