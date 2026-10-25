use optify::provider::GetOptionsPreferences;
use serde_json::json;

#[test]
fn with_overrides_replaces_only_the_overrides() {
    let mut original = GetOptionsPreferences {
        are_configurable_strings_enabled: true,
        are_configurable_values_enabled: true,
        requester: Some("service".to_owned()),
        raise_if_policy_denied: true,
        skip_feature_name_conversion: true,
        ..GetOptionsPreferences::default()
    };
    original.set_constraints(Some(json!({"region": "us"})));
    original.overrides = Some(json!({"old": true}));

    let preferences = original.with_overrides(Some(json!({"new": true})));

    assert!(preferences.are_configurable_strings_enabled);
    assert!(preferences.are_configurable_values_enabled);
    assert_eq!(
        preferences.constraints.as_ref().unwrap().constraints,
        json!({"region": "us"})
    );
    assert_eq!(preferences.overrides, Some(json!({"new": true})));
    assert_eq!(preferences.requester.as_deref(), Some("service"));
    assert!(preferences.raise_if_policy_denied);
    assert!(preferences.skip_feature_name_conversion);
    assert_eq!(original.overrides, Some(json!({"old": true})));
}
