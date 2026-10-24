use std::borrow::Cow;

use optify::{
    provider::{GetOptionsPreferences, OptionsProvider, OptionsRegistry},
    Scope,
};
use serde_json::json;

fn features(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}

#[test]
fn defaults_enable_configurable_values_and_skip_feature_name_conversion() {
    let scope = Scope::new(features(&["feature_A"]), None);
    let preferences = scope.preferences(None);
    assert!(preferences.are_configurable_values_enabled);
    assert!(preferences.skip_feature_name_conversion);
    assert_eq!(preferences.overrides, None);
    assert_eq!(scope.features(), ["feature_A"]);
}

#[test]
fn borrows_preferences_unless_there_are_overrides() {
    let caller = GetOptionsPreferences::default();

    let without_overrides = Scope::new(features(&["feature_A"]), None);
    assert!(matches!(
        without_overrides.preferences(None),
        Cow::Borrowed(_)
    ));
    assert!(matches!(
        without_overrides.preferences(Some(&caller)),
        Cow::Borrowed(preferences) if std::ptr::eq(preferences, &raw const caller)
    ));

    let overrides = json!({"myConfig": {"rootString": "scope override"}});
    let with_overrides = Scope::new(features(&["feature_A"]), Some(overrides.clone()));
    let defaults = with_overrides.preferences(None);
    assert!(matches!(defaults, Cow::Borrowed(_)));
    assert_eq!(defaults.overrides, Some(overrides.clone()));

    let copy = with_overrides.preferences(Some(&caller));
    assert!(matches!(copy, Cow::Owned(_)));
    assert_eq!(copy.overrides, Some(overrides));
    assert!(!copy.are_configurable_values_enabled);
    assert_eq!(caller.overrides, None);
}

#[test]
fn sets_overrides_on_mutable_preferences_only_when_there_are_overrides() {
    let registry = OptionsProvider::build("../../tests/test_suites/simple/configs").unwrap();
    let caller_overrides = json!({"myConfig": {"rootString": "caller"}});
    let mut preferences = GetOptionsPreferences {
        overrides: Some(caller_overrides.clone()),
        ..GetOptionsPreferences::default()
    };

    let without_overrides = Scope::new(features(&["feature_A"]), None);
    let options = without_overrides
        .get_options_with_mut_preferences(&registry, "myConfig", None, &mut preferences)
        .unwrap();
    assert_eq!(options["rootString"], "caller");
    assert_eq!(preferences.overrides, Some(caller_overrides));

    let scope_overrides = json!({"myConfig": {"rootString": "scope"}});
    let with_overrides = Scope::new(features(&["feature_A"]), Some(scope_overrides.clone()));
    let options = with_overrides
        .get_options_with_mut_preferences(&registry, "myConfig", None, &mut preferences)
        .unwrap();
    assert_eq!(options["rootString"], "scope");
    assert_eq!(preferences.overrides, Some(scope_overrides));
    assert_eq!(
        with_overrides
            .get_options(&registry, "myConfig", None, None)
            .unwrap(),
        options
    );
    assert_eq!(
        with_overrides
            .get_all_options(&registry, None, None)
            .unwrap()["myConfig"],
        options
    );
    assert!(registry.get_options("myConfig", &["feature_A"]).unwrap()["rootString"] != "scope");
}

#[test]
fn serializes_and_restores_without_a_registry() {
    let overrides = json!({"custom": {"value": 42}});
    for overrides in [None, Some(overrides)] {
        let scope = Scope::new(
            features(&["feature_A", "feature_B/initial"]),
            overrides.clone(),
        );
        let serialized = scope.serialize();
        assert_eq!(serialized.get("overrides"), overrides.as_ref());
        let restored = Scope::deserialize(serialized.clone()).unwrap();
        assert_eq!(restored.serialize(), serialized);
        assert_eq!(restored.features(), scope.features());
    }
    assert!(Scope::deserialize(json!({"features": [1]})).is_err());
    assert!(Scope::deserialize(json!({})).is_err());
}
