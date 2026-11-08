use optify::builder::{OptionsProviderBuilder, OptionsRegistryBuilder, OptionsWatcherBuilder};
use optify::provider::OptionsRegistry;
use optify::Feature;
use serde_json::json;

fn feature(name: Option<&str>) -> Feature {
    serde_json::from_value(json!({
        "metadata": { "name": name },
        "options": { "value": 1 }
    }))
    .unwrap()
}

#[test]
fn provider_rejects_missing_feature_names() -> Result<(), String> {
    let mut builder = OptionsProviderBuilder::new();
    let error = builder.add_feature(feature(None)).err().unwrap();
    assert_eq!(error, "Feature metadata must have a name");
    assert_eq!(builder.build()?.get_features(), Vec::<String>::new());

    let error = builder.add_features([feature(None)]).err().unwrap();
    assert_eq!(error, "Feature metadata must have a name");
    assert_eq!(builder.build()?.get_features(), Vec::<String>::new());

    let error = builder
        .add_features([feature(Some("valid")), feature(None)])
        .err()
        .unwrap();
    assert_eq!(error, "Feature metadata must have a name");
    assert_eq!(builder.build()?.get_features(), vec!["valid"]);
    Ok(())
}

#[test]
fn watcher_rejects_missing_feature_names() -> Result<(), String> {
    let mut builder = OptionsWatcherBuilder::new();
    builder.add_feature(feature(None))?;
    assert_eq!(
        builder.build().err().unwrap(),
        "Failed to build provider: Feature metadata must have a name"
    );

    let mut builder = OptionsWatcherBuilder::new();
    builder.add_features([feature(Some("valid")), feature(None)])?;
    assert_eq!(
        builder.build().err().unwrap(),
        "Failed to build provider: Feature metadata must have a name"
    );
    Ok(())
}

fn check_empty_feature_name<T: OptionsRegistry>(
    builder: &mut impl OptionsRegistryBuilder<T>,
) -> Result<(), String> {
    builder.add_feature(feature(Some("")))?;
    let registry = builder.build()?;
    assert_eq!(registry.get_features(), vec![""]);
    assert_eq!(registry.get_options("value", &[""])?, json!(1));
    Ok(())
}

#[test]
fn provider_accepts_empty_feature_name() -> Result<(), String> {
    check_empty_feature_name(&mut OptionsProviderBuilder::new())
}

#[test]
fn watcher_accepts_empty_feature_name() -> Result<(), String> {
    check_empty_feature_name(&mut OptionsWatcherBuilder::new())
}
