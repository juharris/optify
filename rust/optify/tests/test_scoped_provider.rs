use std::sync::OnceLock;

use optify::{
    provider::{GetOptionsPreferences, OptionsRegistry},
    OptionsProvider, ScopedProvider,
};
use serde_json::json;

fn registry() -> &'static OptionsProvider {
    static REGISTRY: OnceLock<OptionsProvider> = OnceLock::new();
    REGISTRY
        .get_or_init(|| OptionsProvider::build("../../tests/test_suites/simple/configs").unwrap())
}

#[test]
fn gets_options_with_features_and_overrides() {
    let registry = registry();
    for features in [vec!["a"], vec!["a", "b"], vec!["b", "a"]] {
        for overrides in [None, Some(json!({"myConfig": {"rootString": "scoped"}}))] {
            let scoped = ScopedProvider::new(registry, &features, overrides.clone());
            let preferences = GetOptionsPreferences {
                overrides,
                ..GetOptionsPreferences::default()
            };
            assert_eq!(
                scoped.get_options("myConfig").unwrap(),
                registry
                    .get_options_with_preferences("myConfig", &features, None, Some(&preferences),)
                    .unwrap()
            );
            assert_eq!(
                scoped.get_all_options().unwrap(),
                registry
                    .get_all_options(&features, None, Some(&preferences))
                    .unwrap()
            );
        }
    }
}

#[test]
fn scopes_do_not_affect_each_other_or_the_registry() {
    let registry = registry();
    let original = registry.get_options("myConfig", &["a"]).unwrap();
    let scoped = ScopedProvider::new(
        registry,
        &["a"],
        Some(json!({"myConfig": {"rootString": "scoped"}})),
    );
    let other = ScopedProvider::new(registry, &["a"], None);
    assert_eq!(
        scoped.get_options("myConfig").unwrap()["rootString"],
        "scoped"
    );
    assert_eq!(other.get_options("myConfig").unwrap(), original);
    assert_eq!(registry.get_options("myConfig", &["a"]).unwrap(), original);
}

#[test]
fn serializes_and_restores_scope() {
    let registry = registry();
    for overrides in [None, Some(json!({"myConfig": {"rootString": "scoped"}}))] {
        let scoped = ScopedProvider::new(registry, &["a", "b"], overrides.clone());
        let value = scoped.serialize();
        assert_eq!(
            value,
            json!({"features": ["a", "b"], "overrides": overrides})
        );
        let restored = ScopedProvider::deserialize(registry, value.clone()).unwrap();
        assert_eq!(restored.serialize(), value);
        assert_eq!(
            restored.get_options("myConfig").unwrap(),
            scoped.get_options("myConfig").unwrap()
        );
        assert_eq!(
            restored.get_all_options().unwrap(),
            scoped.get_all_options().unwrap()
        );
    }
}

#[test]
fn supports_empty_features_and_overrides_only() {
    let features: [&str; 0] = [];
    let empty = ScopedProvider::new(registry(), &features, None);
    assert!(empty.get_options("missing").is_err());
    assert_eq!(
        empty.get_all_options(),
        registry().get_all_options(&features, None, None)
    );

    let scoped = ScopedProvider::new(
        registry(),
        &features,
        Some(json!({"custom": {"value": 42}})),
    );
    assert_eq!(scoped.get_options("custom").unwrap(), json!({"value": 42}));
    assert_eq!(
        scoped.get_all_options().unwrap(),
        json!({"custom": {"value": 42}})
    );
    let restored = ScopedProvider::deserialize(registry(), scoped.serialize()).unwrap();
    assert_eq!(restored.get_all_options(), scoped.get_all_options());
}

#[test]
fn propagates_registry_errors() {
    let registry = registry();
    let scoped = ScopedProvider::new(registry, &["a"], None);
    assert_eq!(
        scoped.get_options("missing"),
        registry.get_options("missing", &["a"])
    );
    let invalid = ScopedProvider::new(registry, &["unknown feature"], None);
    assert_eq!(
        invalid.get_all_options(),
        registry.get_all_options(&["unknown feature"], None, None)
    );
    assert!(invalid.get_all_options().is_err());
}

#[test]
fn rejects_invalid_serialized_scopes() {
    for value in [
        json!(null),
        json!([]),
        json!({}),
        json!({"features": "a"}),
        json!({"features": [1]}),
        json!({"features": null}),
    ] {
        assert!(ScopedProvider::deserialize(registry(), value).is_err());
    }
}

#[test]
fn deserializes_scope_without_overrides() {
    let scoped = ScopedProvider::deserialize(registry(), json!({"features": ["a"]})).unwrap();
    assert_eq!(
        scoped.get_options("myConfig").unwrap(),
        registry().get_options("myConfig", &["a"]).unwrap()
    );
    assert_eq!(
        scoped.serialize(),
        json!({"features": ["a"], "overrides": null})
    );
}
