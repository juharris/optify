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

    let cached_result =
        provider.get_options_from_cache("myConfig", &["a"], Some(&cache_options), None)?;
    assert!(cached_result.is_some());
    assert_eq!(cached_result.unwrap(), result1);

    let result2 =
        provider.get_options_with_preferences("myConfig", &["a"], Some(&cache_options), None)?;

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
        .contains("Caching when overrides are given is not supported"));

    Ok(())
}

#[test]
fn test_cache_feature_name_conversion() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_new_provider();
    let cache_options = CacheOptions {};

    // Test that different string types work with cache
    let feature_names_str: Vec<&str> = vec!["a"];
    let feature_names_string: Vec<String> = vec!["A".to_string()];

    let result1 = provider.get_options_with_preferences(
        "myConfig",
        &feature_names_str,
        Some(&cache_options),
        None,
    )?;
    let cached_result = provider.get_options_from_cache(
        "myConfig",
        &feature_names_string,
        Some(&cache_options),
        None,
    )?;
    assert!(cached_result.is_some());
    assert_eq!(cached_result.unwrap(), result1);

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

    assert_eq!(result1, result2);

    // Different order should be treated as different cache key but may have different content
    // due to merge order (later features override earlier ones)
    let result3 =
        provider.get_all_options(&["feature_B/initial", "a"], Some(&cache_options), None)?;

    assert_ne!(result1, result3);

    Ok(())
}

#[test]
fn test_cache_multiple_features_get_options() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_new_provider();
    let cache_options = CacheOptions {};

    // Test caching with multiple features
    let result1 = provider.get_options_with_preferences(
        "myConfig",
        &["a", "feature_B/initial"],
        Some(&cache_options),
        None,
    )?;
    let cached_result = provider.get_options_from_cache(
        "myConfig",
        &["A", "FEature_B/initial"],
        Some(&cache_options),
        None,
    )?;
    assert!(cached_result.is_some());
    assert_eq!(cached_result.unwrap(), result1);

    let result2 = provider.get_options_with_preferences(
        "myConfig",
        &["A", "feature_B/iniTial"],
        Some(&cache_options),
        None,
    )?;
    assert_eq!(result1, result2);

    Ok(())
}

#[test]
fn test_cache_multiple_features_get_options_with_preferences(
) -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_new_provider();
    let cache_options = CacheOptions {};
    let mut preferences = GetOptionsPreferences::new();
    preferences.skip_feature_name_conversion = true;
    let feature_names = &["feature_A", "feature_B/initial"];

    // Test caching with multiple features
    let result1 = provider.get_options_with_preferences(
        "myConfig",
        feature_names,
        Some(&cache_options),
        Some(&preferences),
    )?;

    let cached_miss_result = provider.get_options_from_cache(
        "myConfig",
        &["a", "FEature_B/initial"],
        Some(&cache_options),
        Some(&preferences),
    )?;
    assert!(cached_miss_result.is_none());

    let mut preferences = GetOptionsPreferences::new();
    preferences.skip_feature_name_conversion = true;
    let feature_names = &["feature_A", "feature_B/initial"];
    let cached_hit_result = provider.get_options_from_cache(
        "myConfig",
        feature_names,
        Some(&cache_options),
        Some(&preferences),
    )?;
    assert!(cached_hit_result.is_some());
    assert_eq!(cached_hit_result.unwrap(), result1);

    Ok(())
}

#[test]
fn test_cache_empty_features() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_new_provider();
    let cache_options = CacheOptions {};

    let empty_features: Vec<&str> = vec![];
    let result1 = provider.get_all_options(&empty_features, Some(&cache_options), None)?;
    let result2 = provider.get_all_options(&empty_features, Some(&cache_options), None)?;

    assert_eq!(result1, result2);

    Ok(())
}

#[test]
fn test_cache_get_all_options_configurable_strings() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_new_provider();
    let cache_options = CacheOptions {};

    // Test with are_configurable_strings_enabled = false (default)
    let mut preferences_false = GetOptionsPreferences::new();
    preferences_false.are_configurable_strings_enabled = false;

    let result_false =
        provider.get_all_options(&["a"], Some(&cache_options), Some(&preferences_false))?;

    // Test with are_configurable_strings_enabled = true - should be a cache miss
    let mut preferences_true = GetOptionsPreferences::new();
    preferences_true.are_configurable_strings_enabled = true;

    let result_true =
        provider.get_all_options(&["a"], Some(&cache_options), Some(&preferences_true))?;

    // Results might be the same content but should have been fetched separately (cache miss)
    // We can verify cache miss by checking that we get a cache hit when using same preferences
    let result_true2 =
        provider.get_all_options(&["a"], Some(&cache_options), Some(&preferences_true))?;
    assert_eq!(result_true, result_true2);

    // And verify another cache hit with false
    let result_false2 =
        provider.get_all_options(&["a"], Some(&cache_options), Some(&preferences_false))?;
    assert_eq!(result_false, result_false2);

    Ok(())
}

#[test]
fn test_cache_configurable_strings_false_equals_no_preferences(
) -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_new_provider();
    let cache_options = CacheOptions {};
    // Test with are_configurable_strings_enabled = false (default)
    let mut preferences_false = GetOptionsPreferences::new();
    preferences_false.are_configurable_strings_enabled = false;
    let feature_names = &["a"];

    // Test with no preferences
    let result_no_preferences = provider.get_options_with_preferences(
        "myConfig",
        feature_names,
        Some(&cache_options),
        None,
    )?;

    // Should hit the cache from the no preferences call since false is the default
    let cached_result = provider.get_options_from_cache(
        "myConfig",
        feature_names,
        Some(&cache_options),
        Some(&preferences_false),
    )?;
    assert!(cached_result.is_some());

    let result_with_false = cached_result.unwrap();
    assert_eq!(result_no_preferences, result_with_false);

    // With no preferences should hit the cache
    let cached_result =
        provider.get_options_from_cache("myConfig", &["a"], Some(&cache_options), None)?;
    assert!(cached_result.is_some());
    assert_eq!(cached_result.unwrap(), result_with_false);

    Ok(())
}

#[test]
fn test_cache_get_options_configurable_strings_enabled() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_new_provider();
    let cache_options = CacheOptions {};

    // First call with are_configurable_strings_enabled = false
    let mut preferences_false = GetOptionsPreferences::new();
    preferences_false.are_configurable_strings_enabled = false;

    let _result_false = provider.get_options_with_preferences(
        "myConfig",
        &["a"],
        Some(&cache_options),
        Some(&preferences_false),
    )?;

    let cached_hit_with_false = provider.get_options_from_cache(
        "myConfig",
        &["a"],
        Some(&cache_options),
        Some(&preferences_false),
    )?;
    assert!(cached_hit_with_false.is_some());

    // Second call with are_configurable_strings_enabled = true - should be cache miss
    let mut preferences_true = GetOptionsPreferences::new();
    preferences_true.are_configurable_strings_enabled = true;

    let cached_miss = provider.get_options_from_cache(
        "myConfig",
        &["a"],
        Some(&cache_options),
        Some(&preferences_true),
    )?;
    assert!(cached_miss.is_none());

    let result_true = provider.get_options_with_preferences(
        "myConfig",
        &["a"],
        Some(&cache_options),
        Some(&preferences_true),
    )?;

    // Now verify cache hit with same preferences
    let cached_hit = provider.get_options_from_cache(
        "myConfig",
        &["a"],
        Some(&cache_options),
        Some(&preferences_true),
    )?;
    assert!(cached_hit.is_some());
    assert_eq!(cached_hit.unwrap(), result_true);

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

    assert_eq!(result1, result2);

    Ok(())
}
