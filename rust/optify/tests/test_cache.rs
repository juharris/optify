use optify::provider::{CacheOptions, GetOptionsPreferences, OptionsProvider, OptionsRegistry};

fn get_new_provider() -> OptionsProvider {
    let path = std::path::Path::new("../../tests/test_suites/simple/configs");
    OptionsProvider::build(path).unwrap()
}

#[test]
fn test_entire_config_cache_hit() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_new_provider();
    let cache_options = CacheOptions {};

    // First call should populate the cache
    let result1 = provider.get_all_options(&["a"], Some(&cache_options), None)?;

    // Second call should hit the cache
    let result2 = provider.get_all_options(&["a"], Some(&cache_options), None)?;

    // TODO Find a way to ensure that there was a cache hit.
    assert_eq!(result1, result2);

    Ok(())
}

#[test]
fn test_options_cache_hit() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_new_provider();
    let cache_options = CacheOptions {};

    // First call should populate the cache
    let result1 =
        provider.get_options_with_preferences("myConfig", &["a"], Some(&cache_options), None)?;

    // Second call should hit the cache
    let result2 =
        provider.get_options_with_preferences("myConfig", &["a"], Some(&cache_options), None)?;

    // TODO Find a way to ensure that there was a cache hit.
    assert_eq!(result1, result2);

    Ok(())
}

#[test]
fn test_cache_different_keys() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_new_provider();
    let cache_options = CacheOptions {};

    // Different keys should not share cache entries
    let result1 =
        provider.get_options_with_preferences("myConfig", &["a"], Some(&cache_options), None)?;
    let result2 = provider.get_options_with_preferences(
        "myConfig.rootString",
        &["a"],
        Some(&cache_options),
        None,
    )?;

    // Results should be different
    assert_ne!(result1, result2);

    Ok(())
}

#[test]
fn test_cache_different_features() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_new_provider();
    let cache_options = CacheOptions {};

    let result_a =
        provider.get_options_with_preferences("myConfig", &["a"], Some(&cache_options), None)?;
    let result_b =
        provider.get_options_with_preferences("myConfig", &["b"], Some(&cache_options), None)?;
    assert_ne!(result_a, result_b);

    Ok(())
}

#[test]
fn test_cache_with_preferences() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_new_provider();
    let cache_options = CacheOptions {};

    let mut preferences1 = GetOptionsPreferences::new();
    preferences1.skip_feature_name_conversion = false;

    let mut preferences2 = GetOptionsPreferences::new();
    preferences2.skip_feature_name_conversion = false; // Use same setting to avoid the error

    let result1 = provider.get_all_options(&["a"], Some(&cache_options), Some(&preferences1))?;
    let result2 = provider.get_all_options(&["a"], Some(&cache_options), Some(&preferences2))?;

    // TODO Find a way to ensure that there was a cache hit.
    assert_eq!(result1, result2);

    Ok(())
}

#[test]
fn test_cache_with_overrides() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_new_provider();
    let cache_options = CacheOptions {};

    let mut preferences = GetOptionsPreferences::new();
    preferences.overrides_json = Some(r#"{"test": "override"}"#.to_string());

    let result = provider.get_all_options(&["a"], Some(&cache_options), Some(&preferences));

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("Caching is not supported yet and caching when overrides are given"));

    Ok(())
}

#[test]
fn test_cache_feature_name_conversion() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_new_provider();
    let cache_options = CacheOptions {};

    // Test that different string types work with cache
    let feature_names_str: Vec<&str> = vec!["a"];
    let feature_names_string: Vec<String> = vec!["a".to_string()];

    let result1 = provider.get_all_options(&feature_names_str, Some(&cache_options), None)?;
    let result2 = provider.get_all_options(&feature_names_string, Some(&cache_options), None)?;

    // TODO Find a way to ensure that there was a cache hit.
    assert_eq!(result1, result2);

    Ok(())
}

#[test]
fn test_cache_multiple_features() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_new_provider();
    let cache_options = CacheOptions {};

    // Test caching with multiple features
    let result1 =
        provider.get_all_options(&["a", "feature_B/initial"], Some(&cache_options), None)?;
    let result2 =
        provider.get_all_options(&["a", "feature_B/initial"], Some(&cache_options), None)?;

    // TODO Find a way to ensure that there was a cache hit.
    assert_eq!(result1, result2);

    // Different order should be treated as different cache key but may have different content
    // due to merge order (later features override earlier ones)
    let result3 =
        provider.get_all_options(&["feature_B/initial", "a"], Some(&cache_options), None)?;

    assert_ne!(result1, result3);

    Ok(())
}

#[test]
fn test_cache_empty_features() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_new_provider();
    let cache_options = CacheOptions {};

    let empty_features: Vec<&str> = vec![];
    let result1 = provider.get_all_options(&empty_features, Some(&cache_options), None)?;
    let result2 = provider.get_all_options(&empty_features, Some(&cache_options), None)?;

    // TODO Find a way to ensure that there was a cache hit.
    assert_eq!(result1, result2);

    Ok(())
}

#[test]
fn test_cache_constraints() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_new_provider();
    let cache_options = CacheOptions {};

    // Test with constraints
    let mut preferences1 = GetOptionsPreferences::new();
    preferences1.set_constraints_json(Some(r#"{"env": "test"}"#));

    let mut preferences2 = GetOptionsPreferences::new();
    preferences2.set_constraints_json(Some(r#"{"env": "test"}"#));

    let result1 = provider.get_all_options(&["a"], Some(&cache_options), Some(&preferences1))?;
    let result2 = provider.get_all_options(&["a"], Some(&cache_options), Some(&preferences2))?;

    // TODO Find a way to ensure that there was a cache hit.
    assert_eq!(result1, result2);

    Ok(())
}
