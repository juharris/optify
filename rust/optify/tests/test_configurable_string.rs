use optify::configurable_string::{ConfigurableString, ReplacementValue};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_configurable_string_from_external_module() {
    // Test direct construction
    let mut replacements = HashMap::new();
    replacements.insert(
        "{{name}}".to_string(),
        ReplacementValue::String("Bob".to_string()),
    );
    replacements.insert(
        "{{greeting}}".to_string(),
        ReplacementValue::String("Hello".to_string()),
    );

    let config = ConfigurableString {
        template: "{{greeting}}, {{name}}!".to_string(),
        replacements,
    };

    assert_eq!(config.build(), "Hello, Bob!");
}

#[test]
fn test_configurable_string_deserialization() {
    let data = json!({
        "template": "Welcome {{user}} to {{app}}",
        "replacements": {
            "{{user}}": "Alice",
            "{{app}}": "Optify"
        }
    });

    let config: ConfigurableString = serde_json::from_value(data).unwrap();
    assert_eq!(config.build(), "Welcome Alice to Optify");
}

#[test]
fn test_with_object_replacements() {
    let data = json!({
        "template": "Config: {{settings}}",
        "replacements": {
            "{{settings}}": {
                "theme": "dark",
                "language": "en",
                "notifications": true
            }
        }
    });

    let config: ConfigurableString = serde_json::from_value(data).unwrap();
    let result = config.build();

    // The object should be serialized as JSON
    assert!(result.contains("theme"));
    assert!(result.contains("dark"));
    assert!(result.contains("language"));
}

#[test]
fn test_build_and_to_string_methods() {
    let mut replacements = HashMap::new();
    replacements.insert(
        "{{action}}".to_string(),
        ReplacementValue::String("building".to_string()),
    );
    replacements.insert(
        "{{target}}".to_string(),
        ReplacementValue::String("application".to_string()),
    );

    let config = ConfigurableString {
        template: "Currently {{action}} the {{target}}...".to_string(),
        replacements,
    };

    // Test build() method returns a String
    let result: String = config.build();
    assert_eq!(result, "Currently building the application...");

    // Test to_string() method (provided by Display trait implementation)
    let result2: String = config.build();
    assert_eq!(result2, "Currently building the application...");

    // Verify both methods produce the same result
    assert_eq!(config.build(), config.build());
}
