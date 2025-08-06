use optify::provider::{OptionsProvider, OptionsRegistry};
use std::sync::OnceLock;

static PROVIDER: OnceLock<OptionsProvider> = OnceLock::new();

fn get_provider() -> &'static OptionsProvider {
    PROVIDER.get_or_init(|| {
        let path = std::path::Path::new("../../tests/test_suites/simple/configs");
        OptionsProvider::build(path).unwrap()
    })
}

#[test]
fn test_get_possible_keys_root() {
    let provider = get_provider();
    let keys = provider.get_possible_keys("", None);
    assert_eq!(keys, vec!["myConfig"]);
}

#[test]
fn test_get_possible_keys_invalid_root_slash() {
    let provider = get_provider();
    let keys = provider.get_possible_keys("/", None);
    assert_eq!(keys, Vec::<String>::new());
}

#[test]
fn test_get_possible_keys_my_config() {
    let provider = get_provider();
    let keys = provider.get_possible_keys("/myConfig", None);
    assert_eq!(
        keys,
        vec!["myArray", "myObject", "rootString", "rootString2"]
    );
}

#[test]
fn test_get_possible_keys_nested_object() {
    let provider = get_provider();
    let keys = provider.get_possible_keys("/myConfig/myObject", None);
    assert_eq!(keys, vec!["deeper", "one", "string", "three", "two"]);
}

#[test]
fn test_get_possible_keys_deeper_nested() {
    let provider = get_provider();
    let keys = provider.get_possible_keys("/myConfig/myObject/deeper", None);
    assert_eq!(keys, vec!["list", "new", "wtv"]);
}

#[test]
fn test_get_possible_keys_array_path() {
    let provider = get_provider();
    let keys = provider.get_possible_keys("/myConfig/myArray", None);
    assert_eq!(keys, Vec::<String>::new());
}

#[test]
fn test_get_possible_keys_my_array_element() {
    let provider = get_provider();
    let keys = provider.get_possible_keys("/myConfig/myArray/0", None);
    assert_eq!(keys, Vec::<String>::new());
}

#[test]
fn test_get_possible_keys_nonexistent_path() {
    let provider = get_provider();
    let keys = provider.get_possible_keys("/nonexistent/path", None);
    assert_eq!(keys, Vec::<String>::new());
}

#[test]
fn test_get_possible_keys_scalar_value() {
    let provider = get_provider();
    let keys = provider.get_possible_keys("/myConfig/rootString", None);
    assert_eq!(keys, Vec::<String>::new());
}

#[test]
fn test_get_possible_keys_integer_value() {
    let provider = get_provider();
    let keys = provider.get_possible_keys("/myConfig/myObject/one", None);
    assert_eq!(keys, Vec::<String>::new());
}
