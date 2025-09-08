use optify::configurable_string::{ConfigurableString, ReplacementObject, ReplacementValue};
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;

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

    assert_eq!(config.build(None).unwrap(), "Hello, Bob!");
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
    assert_eq!(config.build(None).unwrap(), "Welcome Alice to Optify");
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

    // Verify the object was parsed correctly
    if let Some(ReplacementValue::Object(obj)) = config.replacements.get("config_source") {
        match obj {
            ReplacementObject::File { file } => {
                assert_eq!(file, "/path/to/nonexistent/config.json");
            }
            _ => panic!("Expected File variant"),
        }
    } else {
        panic!("Expected Object variant with file property");
    }

    // Test build output - should show file error since file doesn't exist
    let result = config.build(None);
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

    // Verify the object was parsed correctly
    if let Some(ReplacementValue::Object(obj)) = config.replacements.get("liquid_template") {
        match obj {
            ReplacementObject::Liquid { liquid } => {
                assert_eq!(liquid, "Hello {{ name | upcase }}");
            }
            _ => panic!("Expected Liquid variant"),
        }
    } else {
        panic!("Expected Object variant with liquid property");
    }

    // Test build output - liquid template should be rendered with available variables
    assert_eq!(config.build(None).unwrap(), "Rendered: Hello WORLD");
}

#[test]
fn test_with_file_containing_liquid_template() {
    // Create a temp file with liquid template content
    use std::fs::File;
    use std::io::Write;
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("test_template.liquid");
    let mut file = File::create(&temp_file).unwrap();
    writeln!(file, "Hello {{{{ name | upcase }}}}!").unwrap();

    let data = json!({
        "template": "File content: {{ template_file }}",
        "replacements": {
            "template_file": {
                "file": temp_file.to_str().unwrap()
            }
        }
    });

    let config: ConfigurableString = serde_json::from_value(data).unwrap();

    // Verify the object was parsed correctly
    if let Some(ReplacementValue::Object(obj)) = config.replacements.get("template_file") {
        match obj {
            ReplacementObject::File { file } => {
                assert_eq!(file, temp_file.to_str().unwrap());
            }
            _ => panic!("Expected File variant"),
        }
    } else {
        panic!("Expected Object variant with file property");
    }

    // Test build output - file content should be read (the liquid syntax within is just text)
    assert_eq!(
        config.build(None).unwrap(),
        "File content: Hello {{ name | upcase }}!\n"
    );

    // Clean up
    std::fs::remove_file(temp_file).ok();
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
    let result: String = config.build(None).unwrap();
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
    assert_eq!(config.build(None).unwrap(), "Hello from Liquid!");

    // Test with show_message as false (empty string is falsy in liquid)
    let data2 = json!({
        "template": "{% if show_message == \"\" %}No message{% else %}{{ message }}{% endif %}",
        "replacements": {
            "show_message": "",
            "message": "Hello from Liquid!"
        }
    });

    let config2: ConfigurableString = serde_json::from_value(data2).unwrap();
    assert_eq!(config2.build(None).unwrap(), "No message");
}

#[test]
fn test_liquid_string_filters() {
    // Test with various string filters
    let mut replacements = HashMap::new();
    replacements.insert(
        "text".to_string(),
        ReplacementValue::String("hello world".to_string()),
    );

    let config = ConfigurableString {
        template:
            "Original: {{ text }}, Upper: {{ text | upcase }}, Capitalize: {{ text | capitalize }}"
                .to_string(),
        replacements,
    };

    let result = config.build(None).unwrap();
    assert_eq!(
        result,
        "Original: hello world, Upper: HELLO WORLD, Capitalize: Hello world"
    );
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
    assert_eq!(config.build(None).unwrap(), "Hello, Alice!");
}

#[test]
fn test_cannot_have_both_file_and_liquid() {
    // Test that having both file and liquid in the same object picks the first match (file)
    let data = json!({
        "template": "Test: {{ test }}",
        "replacements": {
            "test": {
                "file": "some_file.txt",
                "liquid": "{{ some_template }}"
            }
        }
    });

    // With untagged enum, serde will match the first variant that fits
    // Since File comes first in our enum, it will match as File and ignore liquid
    let config: ConfigurableString = serde_json::from_value(data).unwrap();

    // Verify it was parsed as File variant, ignoring the liquid field
    if let Some(ReplacementValue::Object(obj)) = config.replacements.get("test") {
        match obj {
            ReplacementObject::File { file } => {
                assert_eq!(file, "some_file.txt");
            }
            _ => panic!("Expected File variant when both are present"),
        }
    }
}

#[test]
fn test_file_loading_with_root_path() {
    // Test loading files relative to a root path
    let test_files_dir = Path::new("tests/configurable_string_resources");

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
    let result = config.build(Some(test_files_dir));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "File content: Hello from file!");

    // Should fail without root path (file not found)
    let result = config.build(None);
    assert!(result.is_err());
}

#[test]
fn test_liquid_template_file_with_root_path() {
    let test_files_dir = Path::new("tests/configurable_string_resources");

    let data = json!({
        "template": "{{ greeting }}",
        "replacements": {
            "greeting": {
                "file": "template.liquid"
            }
        }
    });

    let config: ConfigurableString = serde_json::from_value(data).unwrap();
    let result = config.build(Some(test_files_dir));
    assert!(result.is_ok());
    // The file contains a liquid template but it's treated as plain text
    assert_eq!(result.unwrap(), "Welcome {{ name | capitalize }}!");
}

#[test]
fn test_json_file_loading() {
    let test_files_dir = Path::new("tests/configurable_string_resources");

    let data = json!({
        "template": "Config: {{ config_file }}",
        "replacements": {
            "config_file": {
                "file": "config.json"
            }
        }
    });

    let config: ConfigurableString = serde_json::from_value(data).unwrap();
    let result = config.build(Some(test_files_dir));
    assert!(result.is_ok());

    // Check the exact content of the loaded JSON file
    let expected = "Config: {\n  \"version\": \"1.0.0\",\n  \"features\": [\"auth\", \"api\", \"database\"]\n}";
    assert_eq!(result.unwrap(), expected);
}
