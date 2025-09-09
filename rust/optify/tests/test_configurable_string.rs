use optify::configurable_string::configurable_string_impl::LoadedFiles;
use optify::configurable_string::{ConfigurableString, ReplacementValue};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_configurable_string_from_external_module() {
    // Test direct construction with liquid syntax
    let mut replacements = HashMap::new();
    replacements.insert(
        "name".to_string(),
        ReplacementValue::String("Bob".to_string()),
    );
    replacements.insert(
        "greeting".to_string(),
        ReplacementValue::String("Hello".to_string()),
    );

    let config = ConfigurableString {
        template: "{{ greeting }}, {{ name }}!".to_string(),
        replacements,
    };

    let files = LoadedFiles::new();
    assert_eq!(config.build(&files).unwrap(), "Hello, Bob!");
}

#[test]
fn test_configurable_string_deserialization() {
    let data = json!({
        "template": "Welcome {{ user }} to {{ app }}",
        "replacements": {
            "user": "Alice",
            "app": "Optify"
        }
    });

    let config: ConfigurableString = serde_json::from_value(data).unwrap();
    let files = LoadedFiles::new();
    assert_eq!(config.build(&files).unwrap(), "Welcome Alice to Optify");
}

#[test]
fn test_with_object_replacements_file() {
    let data = json!({
        "template": "Config loaded from {{ config_source }}",
        "replacements": {
            "config_source": {
                "file": "/path/to/nonexistent/config.json"
            }
        }
    });

    let config: ConfigurableString = serde_json::from_value(data).unwrap();
    let files = LoadedFiles::new();
    let result = config.build(&files);
    assert!(result.is_err(), "Should fail when file doesn't exist");
}

#[test]
fn test_with_object_replacements_liquid() {
    let data = json!({
        "template": "Rendered: {{ liquid_template }}",
        "replacements": {
            "name": "world",
            "liquid_template": {
                "liquid": "Hello {{ name | upcase }}"
            }
        }
    });

    let config: ConfigurableString = serde_json::from_value(data).unwrap();
    let files = LoadedFiles::new();
    assert_eq!(config.build(&files).unwrap(), "Rendered: Hello WORLD");
}

#[test]
fn test_with_file_containing_liquid_template() {
    let mut files = LoadedFiles::new();
    let file_path = "test_template.liquid".to_string();
    files.insert(file_path.clone(), "Hello {{ name | upcase }}!".into());

    let data = json!({
        "template": "File content: {{ template_file }}",
        "replacements": {
            "name": "Justin",
            "template_file": {
                "file": file_path,
            }
        }
    });

    let config: ConfigurableString = serde_json::from_value(data).unwrap();

    assert_eq!(config.build(&files).unwrap(), "File content: Hello JUSTIN!");
}

#[test]
fn test_build_with_liquid_filters() {
    let mut replacements = HashMap::new();
    replacements.insert(
        "action".to_string(),
        ReplacementValue::String("building".to_string()),
    );
    replacements.insert(
        "target".to_string(),
        ReplacementValue::String("application".to_string()),
    );

    let config = ConfigurableString {
        template: "Currently {{ action | upcase }} the {{ target | capitalize }}...".to_string(),
        replacements,
    };

    // Test build() method with liquid filters
    let files = LoadedFiles::new();
    let result: String = config.build(&files).unwrap();
    assert_eq!(result, "Currently BUILDING the Application...");
}

#[test]
fn test_liquid_control_flow() {
    let data = json!({
        "template": "{% if show_message %}{{ message }}{% else %}No message{% endif %}",
        "replacements": {
            "show_message": "true",
            "message": "Hello from Liquid!"
        }
    });

    let config: ConfigurableString = serde_json::from_value(data).unwrap();
    let files = LoadedFiles::new();
    assert_eq!(config.build(&files).unwrap(), "Hello from Liquid!");

    // Test with show_message as false (empty string is falsy in liquid)
    let data2 = json!({
        "template": "{% if show_message == \"\" %}No message{% else %}{{ message }}{% endif %}",
        "replacements": {
            "show_message": "",
            "message": "Hello from Liquid!"
        }
    });

    let config2: ConfigurableString = serde_json::from_value(data2).unwrap();
    assert_eq!(config2.build(&files).unwrap(), "No message");
}

#[test]
fn test_nested_liquid_template_access() {
    // Test that liquid templates in replacements can access other variables
    let data = json!({
        "template": "{{ greeting }}, {{ formatted_name }}!",
        "replacements": {
            "name": "alice",
            "greeting": "Hello",
            "formatted_name": {
                "liquid": "{{ name | capitalize }}"
            }
        }
    });

    let config: ConfigurableString = serde_json::from_value(data).unwrap();
    let files = LoadedFiles::new();
    assert_eq!(config.build(&files).unwrap(), "Hello, Alice!");
}

#[test]
fn test_file_loading_with_root_path() {
    let mut files = LoadedFiles::new();
    files.insert("simple.txt".into(), "simple text".into());

    let data = json!({
        "template": "File content: {{ simple_file }}",
        "replacements": {
            "simple_file": {
                "file": "simple.txt"
            }
        }
    });

    let config: ConfigurableString = serde_json::from_value(data).unwrap();

    // Should work with correct root path
    let result = config.build(&files);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "File content: simple text");

    files.remove("simple.txt");
    let result = config.build(&files);
    assert!(result.is_err());
}
