use optify::configurable_string::configurable_string_impl::LoadedFiles;
use optify::configurable_string::{ConfigurableString, ReplacementObject, ReplacementValue};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_configurable_string_deserialization() {
    let data = json!({
        "root": {"liquid": "Welcome {{ user }} to {{ app }}"},
        "components": {
            "user": "Alice",
            "app": "Optify"
        }
    });

    let config: ConfigurableString = serde_json::from_value(data).unwrap();
    let files = LoadedFiles::new();
    assert_eq!(config.build(&files).unwrap(), "Welcome Alice to Optify");
}

#[test]
fn test_with_object_components_liquid() {
    let data = json!({
        "root": {"liquid": "Rendered: {{ liquid_template }} was {{ name }} and {{ name }}"},
        "components": {
            "name": "world",
            "liquid_template": {
                "liquid": "Hello {{ name | upcase }}"
            }
        }
    });

    let config: ConfigurableString = serde_json::from_value(data).unwrap();
    let files = LoadedFiles::new();
    assert_eq!(
        config.build(&files).unwrap(),
        "Rendered: Hello WORLD was world and world"
    );
}

#[test]
fn test_with_file_containing_liquid_template() {
    let mut files = LoadedFiles::new();
    let file_path = "test_template.liquid".to_string();
    files.insert(file_path.clone(), "Hello {{ name | upcase }}!".into());

    let data = json!({
        "root": {"liquid": "File content: {{ template_file }}"},
        "components": {
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
    let mut components = HashMap::new();
    components.insert(
        "action".to_string(),
        ReplacementValue::String("building".to_string()),
    );
    components.insert(
        "target".to_string(),
        ReplacementValue::String("application".to_string()),
    );

    let config = ConfigurableString {
        root: ReplacementValue::Object(ReplacementObject::Liquid {
            liquid: "Currently {{ action | upcase }} the {{ target | capitalize }}...".to_string(),
        }),
        components: Some(components),
    };

    // Test build() method with liquid filters
    let files = LoadedFiles::new();
    let result: String = config.build(&files).unwrap();
    assert_eq!(result, "Currently BUILDING the Application...");
}

#[test]
fn test_liquid_control_flow() {
    let mut data = json!({
        "root": {"liquid": "{% if show_message != \"\" %}{{ message }}{% else %}No message{% endif %}"},
        "components": {
            "show_message": "true",
            "message": "Hello from Liquid!"
        }
    });

    let config: ConfigurableString = serde_json::from_value(data.clone()).unwrap();
    let files = LoadedFiles::new();
    assert_eq!(config.build(&files).unwrap(), "Hello from Liquid!");

    data["components"]["show_message"] = json!("");

    let config2: ConfigurableString = serde_json::from_value(data).unwrap();
    assert_eq!(config2.build(&files).unwrap(), "No message");
}

#[test]
fn test_nested_liquid_template_access() {
    // Test that liquid templates in components can access other variables
    let data = json!({
        "root": {"liquid": "{{ greeting }}, {{ formatted_name }}!"},
        "components": {
            "name": "alice",
            "greeting": "Hello",
            "formatted_name": {
                "liquid": "dear {{ name | capitalize }}"
            }
        }
    });

    let config: ConfigurableString = serde_json::from_value(data).unwrap();
    let files = LoadedFiles::new();
    assert_eq!(config.build(&files).unwrap(), "Hello, dear Alice!");
}

#[test]
fn test_file_loading_with_root_path() {
    let mut files = LoadedFiles::new();
    files.insert("simple.txt".into(), "simple text not {{ replaced }}".into());

    let data = json!({
        "root": {"liquid": "File content: {{ simple_file }}"},
        "components": {
            "simple_file": {
                "file": "simple.txt"
            }
        }
    });

    let config: ConfigurableString = serde_json::from_value(data).unwrap();

    // Should work with correct root path
    let result = config.build(&files);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "File content: simple text not {{ replaced }}"
    );
}

#[test]
fn test_root_as_liquid_object() {
    // Test root as a liquid object that can access components
    let data = json!({
        "root": {
            "liquid": "Hello {{ name | upcase }}, welcome to {{ place }}!"
        },
        "components": {
            "name": "alice",
            "place": "Wonderland"
        }
    });

    let config: ConfigurableString = serde_json::from_value(data).unwrap();
    let files = LoadedFiles::new();
    assert_eq!(
        config.build(&files).unwrap(),
        "Hello ALICE, welcome to Wonderland!"
    );
}

#[test]
fn test_root_as_file_object() {
    // Test root as a file object
    let mut files = LoadedFiles::new();
    files.insert(
        "main_template.txt".into(),
        "This is the main content from a file.".into(),
    );

    let data = json!({
        "root": {
            "file": "main_template.txt"
        },
        "components": {}
    });

    let config: ConfigurableString = serde_json::from_value(data).unwrap();
    assert_eq!(
        config.build(&files).unwrap(),
        "This is the main content from a file."
    );
}

#[test]
fn test_root_as_liquid_file_object() {
    // Test root as a liquid file object that processes liquid templates
    let mut files = LoadedFiles::new();
    files.insert(
        "main.liquid".into(),
        "Processing: {{ title | upcase }} by {{ author }}".into(),
    );

    let data = json!({
        "root": {
            "file": "main.liquid"
        },
        "components": {
            "title": "the book",
            "author": "Jane Doe"
        }
    });

    let config: ConfigurableString = serde_json::from_value(data).unwrap();
    assert_eq!(
        config.build(&files).unwrap(),
        "Processing: THE BOOK by Jane Doe"
    );
}

#[test]
fn test_root_object_with_nested_liquid_components() {
    // Test root as object with nested liquid components
    let data = json!({
        "root": {
            "liquid": "Report: {{ header }}\n{{ content }}"
        },
        "components": {
            "title": "quarterly report",
            "header": {
                "liquid": "{{ title | upcase }}"
            },
            "content": "This is the content section."
        }
    });

    let config: ConfigurableString = serde_json::from_value(data).unwrap();
    let files = LoadedFiles::new();
    assert_eq!(
        config.build(&files).unwrap(),
        "Report: QUARTERLY REPORT\nThis is the content section."
    );
}

#[test]
fn test_root_as_string() {
    let data = json!({
        "root": "Hello, world! {{ do_not_replace }}",
    });

    let config: ConfigurableString = serde_json::from_value(data).unwrap();
    let files = LoadedFiles::new();
    assert_eq!(
        config.build(&files).unwrap(),
        "Hello, world! {{ do_not_replace }}"
    );
}

#[test]
fn test_root_file_not_found_error() {
    // Test that missing file in root object produces appropriate error
    let data = json!({
        "root": {
            "file": "nonexistent.txt"
        },
        "components": {}
    });

    let config: ConfigurableString = serde_json::from_value(data).unwrap();
    let files = LoadedFiles::new();
    let result = config.build(&files);
    assert!(result.is_err());
    assert_eq!("File 'nonexistent.txt' not found.", result.unwrap_err());
}

/*
FIXME Try to test. Can't catch easily because Liquid panics.
#[test]
fn test_with_object_components_file() {
    let data = json!({
        "root": "Config loaded from {{ config_source }}",
        "components": {
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
 */
