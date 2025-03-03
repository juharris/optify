use optify::builder::OptionsProviderBuilder;

#[test]
fn test_provider_get_features() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new("../../tests/test_suites/simple/configs");
    let mut builder = OptionsProviderBuilder::new();
    builder.add_directory(path)?;
    let provider = builder.build()?;
    let mut features = provider.get_features();
    features.sort_unstable();
    assert_eq!(
        features,
        vec!["A_with_comments", "feature_A", "feature_B/initial"]
    );
    Ok(())
}

#[test]
fn test_provider_get_entire_config() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new("../../tests/test_suites/simple/configs");
    let mut builder = OptionsProviderBuilder::new();
    builder.add_directory(path)?;
    let provider = builder.build()?;
    let feature_names: Vec<String> = vec!["a".to_string()];
    let entire_config = provider.get_all_options(&feature_names, &None, &None)?;
    let key = "myConfig";
    let opts = provider.get_options(key, &feature_names)?;
    let expected = serde_json::json!({
        key: opts
    });
    assert_eq!(entire_config, expected);
    Ok(())
}
