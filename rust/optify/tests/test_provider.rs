use optify::builder::OptionsProviderBuilder;

use std::sync::OnceLock;
static PROVIDER: OnceLock<optify::provider::OptionsProvider> = OnceLock::new();

fn get_provider() -> &'static optify::provider::OptionsProvider {
    PROVIDER.get_or_init(|| {
        let path = std::path::Path::new("../../tests/test_suites/simple/configs");
        let mut builder = OptionsProviderBuilder::new();
        builder.add_directory(path).unwrap();
        builder.build().unwrap()
    })
}

#[test]
fn test_provider_get_entire_config() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_provider();
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

#[test]
fn test_provider_get_features() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_provider();
    let mut features = provider.get_features();
    features.sort_unstable();
    assert_eq!(
        features,
        vec!["A_with_comments", "feature_A", "feature_B/initial"]
    );
    Ok(())
}

#[test]
fn test_provider_get_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_provider();
    let mut features = provider.get_features();
    features.sort_unstable();
    let metadata = provider.get_features_with_metadata();
    let mut metadata_keys: Vec<String> = metadata.keys().cloned().collect();
    metadata_keys.sort_unstable();
    assert_eq!(metadata_keys, features);

    let key = provider.get_canonical_feature_name("a")?;
    let a_metadata = &metadata[key];
    assert_eq!("feature_A", a_metadata.name.as_ref().unwrap());

    Ok(())
}
