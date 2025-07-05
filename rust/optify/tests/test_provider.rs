use optify::{
    builder::{OptionsProviderBuilder, OptionsRegistryBuilder},
    provider::{GetOptionsPreferences, OptionsProvider, OptionsRegistry},
};
use std::sync::OnceLock;

static PROVIDER: OnceLock<OptionsProvider> = OnceLock::new();

fn get_provider() -> &'static OptionsProvider {
    PROVIDER.get_or_init(|| {
        let path = std::path::Path::new("../../tests/test_suites/simple/configs");
        let mut builder = OptionsProviderBuilder::new();
        builder.add_directory(path).unwrap();
        builder.build().unwrap()
    })
}

#[test]
fn test_provider_get_aliases() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_provider();
    let mut aliases = provider.get_aliases();
    aliases.sort();
    assert_eq!(aliases, vec!["a", "b",]);
    Ok(())
}

#[test]
fn test_provider_get_features_and_aliases() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_provider();
    let mut features_and_aliases = provider.get_features_and_aliases();
    features_and_aliases.sort();
    assert_eq!(
        features_and_aliases,
        vec![
            "A_with_comments",
            "a",
            "b",
            "feature_A",
            "feature_B/initial",
        ]
    );
    Ok(())
}

#[test]
fn test_provider_get_options_with_overrides() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_provider();
    let mut preferences = GetOptionsPreferences::new();
    preferences.overrides_json = Some(
        serde_json::json!({
            "myConfig": {
                "new key": 33,
                "rootString": "new string",
                "myObject": {
                    "one": 1321,
                    "something new for test_provider_get_options_with_overrides": "hello"
                }
            }
        })
        .to_string(),
    );
    let opts = provider.get_options_with_preferences("myConfig", &["a"], None, Some(&preferences));

    let expected = serde_json::json!({
        "new key": 33,
        "rootString": "new string",
        "rootString2": "gets overridden",
        "myArray": [
            "example item 1"
        ],
        "myObject": {
            "one": 1321,
            "two": 2,
            "something new for test_provider_get_options_with_overrides": "hello",
            "string": "string",
            "deeper": {
                "wtv": 3,
                "list": [
                    1,
                    2
                ]
            }
        }
    });

    assert_eq!(opts.unwrap(), expected);

    Ok(())
}

#[test]
fn test_provider_get_canonical_feature_names() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_provider();
    let canonical_feature_names = provider.get_canonical_feature_names(&["a", "b", "feature_A"])?;
    assert_eq!(
        canonical_feature_names,
        vec!["feature_A", "feature_B/initial", "feature_A"]
    );

    Ok(())
}

#[test]
fn test_provider_get_entire_config() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_provider();
    let feature_names: Vec<&str> = vec!["a"];
    let entire_config = provider.get_all_options(&feature_names, None, None)?;
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
    let a_metadata = &metadata[&key];
    let expected_aliases: Vec<String> = vec!["a".to_owned()];
    assert_eq!(expected_aliases, a_metadata.aliases.clone().unwrap());
    let details = a_metadata.details.as_ref().unwrap();
    assert_eq!(serde_json::json!("The file is for testing."), *details);
    assert_eq!("feature_A", a_metadata.name.as_ref().unwrap());
    assert_eq!("a-team@company.com", a_metadata.owners.as_ref().unwrap());

    Ok(())
}
