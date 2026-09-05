use optify::{
    builder::{OptionsProviderBuilder, OptionsRegistryBuilder},
    provider::{GetOptionsPreferences, OptionsProvider, OptionsRegistry},
    schema::policies::RequesterPolicy,
};
use std::{collections::HashSet, fs, sync::OnceLock};

static CONDITIONS_PROVIDER: OnceLock<OptionsProvider> = OnceLock::new();
static CONFIGURABLE_STRINGS_PROVIDER: OnceLock<OptionsProvider> = OnceLock::new();
static INHERITANCE_PROVIDER: OnceLock<OptionsProvider> = OnceLock::new();
static POLICIES_PROVIDER: OnceLock<OptionsProvider> = OnceLock::new();
static PROVIDER: OnceLock<OptionsProvider> = OnceLock::new();

fn get_configurable_values_provider() -> &'static OptionsProvider {
    CONFIGURABLE_STRINGS_PROVIDER.get_or_init(|| {
        let path = std::path::Path::new("../../tests/test_suites/configurable_values/configs");
        let mut builder = OptionsProviderBuilder::new();
        builder.add_directory(path).unwrap();
        builder.build().unwrap()
    })
}

fn get_provider() -> &'static OptionsProvider {
    PROVIDER.get_or_init(|| {
        let path = std::path::Path::new("../../tests/test_suites/simple/configs");
        let mut builder = OptionsProviderBuilder::new();
        builder.add_directory(path).unwrap();
        builder.build().unwrap()
    })
}

fn get_provider_with_conditions() -> &'static OptionsProvider {
    CONDITIONS_PROVIDER.get_or_init(|| {
        let path = std::path::Path::new("../../tests/test_suites/conditions/configs");
        let mut builder = OptionsProviderBuilder::new();
        builder.add_directory(path).unwrap();
        builder.build().unwrap()
    })
}

fn get_policies_provider() -> &'static OptionsProvider {
    POLICIES_PROVIDER.get_or_init(|| {
        let path = std::path::Path::new("../../tests/test_suites/policies/configs");
        OptionsProvider::build(path).unwrap()
    })
}

fn get_inheritance_provider() -> &'static OptionsProvider {
    INHERITANCE_PROVIDER.get_or_init(|| {
        let path = std::path::Path::new("../../tests/test_suites/inheritance/configs");
        let mut builder = OptionsProviderBuilder::new();
        builder.add_directory(path).unwrap();
        builder.build().unwrap()
    })
}

#[test]
fn test_filtered_feature_names() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_provider_with_conditions();
    let filtered_feature_names = provider.get_filtered_feature_names(&["a", "b"], None)?;
    assert_eq!(filtered_feature_names, vec!["A", "B"]);

    let mut preferences = GetOptionsPreferences::new();
    preferences.skip_feature_name_conversion = true;
    let filtered_feature_names =
        provider.get_filtered_feature_names(&["A", "B"], Some(&preferences))?;
    assert_eq!(filtered_feature_names, vec!["A", "B"]);

    preferences.set_constraints(Some(serde_json::json!({"info": 3, "status": "new"})));
    let filtered_feature_names =
        provider.get_filtered_feature_names(&["A", "B"], Some(&preferences))?;
    assert_eq!(filtered_feature_names, vec!["A", "B"]);

    preferences.set_constraints(Some(serde_json::json!({"info": 2, "status": "new"})));
    preferences.skip_feature_name_conversion = false;
    let filtered_feature_names =
        provider.get_filtered_feature_names(&["a", "b"], Some(&preferences))?;
    assert_eq!(filtered_feature_names, vec!["B"]);

    Ok(())
}

#[test]
fn test_map_feature_names() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_provider_with_conditions();

    // No preferences: all features kept with canonical names.
    let result = provider.map_feature_names(&["a", "b"], None)?;
    assert_eq!(result, vec![Some("A".to_owned()), Some("B".to_owned())]);

    // skip_feature_name_conversion: names kept as-is.
    let mut preferences = GetOptionsPreferences::new();
    preferences.skip_feature_name_conversion = true;
    let result = provider.map_feature_names(&["A", "B"], Some(&preferences))?;
    assert_eq!(result, vec![Some("A".to_owned()), Some("B".to_owned())]);

    // Constraints that match both features.
    preferences.set_constraints(Some(serde_json::json!({"info": 3, "status": "new"})));
    let result = provider.map_feature_names(&["A", "B"], Some(&preferences))?;
    assert_eq!(result, vec![Some("A".to_owned()), Some("B".to_owned())]);

    // Constraints that filter out A but keep B. Order must match input.
    preferences.set_constraints(Some(serde_json::json!({"info": 2, "status": "new"})));
    preferences.skip_feature_name_conversion = false;
    let result = provider.map_feature_names(&["a", "b"], Some(&preferences))?;
    assert_eq!(result, vec![None, Some("B".to_owned())]);

    // Reversed input order: B first, then A filtered out.
    let result = provider.map_feature_names(&["b", "a"], Some(&preferences))?;
    assert_eq!(result, vec![Some("B".to_owned()), None]);

    // Empty input.
    let empty: Vec<&str> = vec![];
    let result = provider.map_feature_names(&empty, Some(&preferences))?;
    assert_eq!(result, Vec::<Option<String>>::new());

    Ok(())
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
fn test_provider_get_options_missing_key() -> Result<(), Box<dyn std::error::Error>> {
    let key = "does not exist";
    let feature_names = vec!["a"];
    let provider = get_provider();
    let opts = provider.get_options(key, &feature_names);
    assert!(opts.is_err());
    assert_eq!(opts.unwrap_err(), "Error getting options with features [\"a\"]: configuration property \"does not exist\" not found");

    let mut preferences = GetOptionsPreferences::new();
    preferences.overrides = Some(serde_json::json!({
        "does not exist": 42
    }));
    let opts = provider.get_options_with_preferences(key, &feature_names, None, Some(&preferences));
    let value = opts.expect("should be able to get options");
    assert_eq!(value, serde_json::json!(42));

    Ok(())
}

#[test]
fn test_provider_get_options_no_features() -> Result<(), Box<dyn std::error::Error>> {
    let key = "wtv";
    let provider = get_provider();
    let feature_names: Vec<&str> = vec![];
    let opts = provider.get_options(key, &feature_names);
    assert!(opts.is_err());
    assert_eq!(
        opts.unwrap_err(),
        "Error getting options with features []: configuration property \"wtv\" not found"
    );

    let mut preferences = GetOptionsPreferences::new();
    preferences.overrides = Some(serde_json::json!({
        key: 42
    }));
    let opts = provider.get_options_with_preferences(key, &feature_names, None, Some(&preferences));
    let value = opts.expect("should be able to get options");
    assert_eq!(value, serde_json::json!(42));
    Ok(())
}

#[test]
fn test_provider_get_options_with_overrides() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_provider();
    let mut preferences = GetOptionsPreferences::new();
    preferences.overrides = Some(serde_json::json!({
        "myConfig": {
            "new key": 33,
            "rootString": "new string",
            "myObject": {
                "one": 1321,
                "something new for test_provider_get_options_with_overrides": "hello"
            }
        }
    }));
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
fn test_provider_get_all_options() -> Result<(), Box<dyn std::error::Error>> {
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
fn test_provider_get_all_options_multiple_features() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_provider();
    let feature_names: Vec<&str> = vec!["a", "b"];
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
fn test_provider_get_all_options_multiple_features_freezes_primitive_paths(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    fs::write(
        temp_dir.path().join("low.json"),
        r#"{
            "options": {
                "rootSettings": {
                    "low": true
                },
                "settings": {
                    "nested": {
                        "low": true
                    },
                    "sibling": "from low"
                }
            }
        }"#,
    )?;
    fs::write(
        temp_dir.path().join("middle.json"),
        r#"{
            "options": {
                "rootSettings": false,
                "settings": {
                    "nested": false,
                    "sibling": "from middle"
                }
            }
        }"#,
    )?;
    fs::write(
        temp_dir.path().join("high.json"),
        r#"{
            "options": {
                "rootSettings": {
                    "high": true
                },
                "settings": {
                    "nested": {
                        "high": true
                    }
                }
            }
        }"#,
    )?;

    let provider = OptionsProvider::build(temp_dir.path())?;
    let feature_names = ["low", "middle", "high"];
    let expected_settings = serde_json::json!({
        "nested": {
            "high": true
        },
        "sibling": "from middle"
    });
    let expected_root_settings = serde_json::json!({
        "high": true
    });

    let root_options = provider.get_options("rootSettings", &feature_names)?;
    let options = provider.get_options("settings", &feature_names)?;
    let all_options = provider.get_all_options(&feature_names, None, None)?;

    assert_eq!(root_options, expected_root_settings);
    assert_eq!(options, expected_settings);
    assert_eq!(
        all_options,
        serde_json::json!({
            "rootSettings": expected_root_settings,
            "settings": expected_settings
        })
    );

    Ok(())
}

#[test]
fn test_provider_get_all_options_multiple_features_with_overrides(
) -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_provider();
    let feature_names: Vec<&str> = vec!["a", "b"];
    let mut preferences = GetOptionsPreferences::new();
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let value = format!("from the overrides {time}");
    preferences.overrides = Some(serde_json::json!({
        "myConfig": {
            "rootString": value
        }
    }));
    let entire_config = provider.get_all_options(&feature_names, None, Some(&preferences))?;
    let key = "myConfig";
    let opts =
        provider.get_options_with_preferences(key, &feature_names, None, Some(&preferences))?;
    let expected = serde_json::json!({
        key: opts
    });
    assert_eq!(entire_config, expected);
    assert_eq!(opts["rootString"], value);
    Ok(())
}

#[test]
fn test_provider_get_dependents() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_inheritance_provider();
    let grandparent_metadata = provider.get_feature_metadata("grandparent").unwrap();
    assert_eq!(grandparent_metadata.dependents, None);

    let parent_metadata = provider.get_feature_metadata("parent1").unwrap();
    assert_eq!(
        parent_metadata.dependents,
        Some(
            ["grandparent", "grandparent_too"]
                .map(String::from)
                .to_vec()
        )
    );

    let base1_metadata = provider.get_feature_metadata("base1").unwrap();
    assert_eq!(
        base1_metadata.dependents,
        Some(
            ["parent1", "super", "super_with_options"]
                .map(String::from)
                .to_vec()
        )
    );

    let base2_metadata = provider.get_feature_metadata("base2").unwrap();
    assert_eq!(
        base2_metadata.dependents,
        Some(
            ["parent2", "super", "super_with_options"]
                .map(String::from)
                .to_vec()
        )
    );
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
    assert_eq!(a_metadata.dependents, None);
    let expected_path =
        dunce::canonicalize("../../tests/test_suites/simple/configs/feature_A.json")
            .unwrap()
            .to_string_lossy()
            .to_string();
    assert_eq!(expected_path, a_metadata.path.as_ref().unwrap().to_string());

    Ok(())
}

#[test]
fn test_provider_has_conditions() -> Result<(), Box<dyn std::error::Error>> {
    let conditions_provider = get_provider_with_conditions();
    assert!(conditions_provider.has_conditions("A"));
    assert!(!conditions_provider.has_conditions("B"));
    Ok(())
}

#[test]
fn test_configurable_values_get_all_options_with_overrides(
) -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_configurable_values_provider();
    let mut preferences = GetOptionsPreferences::new();
    preferences.are_configurable_strings_enabled = true;
    preferences.overrides = Some(serde_json::json!({
        "message": {
            "$type": "Optify.ConfigurableString",
            "base": {
                "liquid": "Hello {{ name }}!"
            },
            "arguments": {
                "name": "from the test"
            }
        }
    }));

    let features: Vec<&str> = vec![];
    let opts = provider.get_all_options(&features, None, Some(&preferences))?;

    assert_eq!(opts["message"], "Hello from the test!");

    Ok(())
}

#[test]
fn test_get_policies_no_policy() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_policies_provider();
    // A feature that doesn't exist returns None.
    assert!(provider.get_policies("nonexistent_feature").is_none());
    Ok(())
}

#[test]
fn test_get_policies_allowed() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_policies_provider();
    let policies = provider
        .get_policies("feature_allowed")
        .expect("feature_allowed should have policies");

    assert!(policies.is_requester_permitted("service_a"));
    assert!(policies.is_requester_permitted("service_b"));
    // Requester not in the allow list is not permitted.
    assert!(!policies.is_requester_permitted("service_c"));
    assert!(!policies.is_requester_permitted("untrusted_service"));

    // Verify the policy variant and set contents.
    let expected: HashSet<String> = ["service_a", "service_b", "service_d"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    match policies.requester.expect("requester policy should be set") {
        RequesterPolicy::Allow { allow } => assert_eq!(allow, expected),
        other => panic!("expected Allow, got {other:?}"),
    }

    Ok(())
}

#[test]
fn test_get_policies_blocked() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_policies_provider();
    let policies = provider
        .get_policies("feature_blocked")
        .expect("feature_blocked should have policies");

    // Blocked requester is not permitted.
    assert!(!policies.is_requester_permitted("untrusted_service"));
    // Any other requester is permitted.
    assert!(policies.is_requester_permitted("service_a"));
    assert!(policies.is_requester_permitted("any_other_service"));

    // Verify the policy variant and set contents.
    let expected: HashSet<String> = ["untrusted_service", "service_f"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    match policies.requester.expect("requester policy should be set") {
        RequesterPolicy::Block { block } => assert_eq!(block, expected),
        other => panic!("expected Block, got {other:?}"),
    }

    Ok(())
}

#[test]
fn test_policy_filtering_silently_filters_denied() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_policies_provider();

    // Without requester: both features pass through.
    let result =
        provider.get_filtered_feature_names(&["feature_allowed", "feature_blocked"], None)?;
    assert_eq!(result, vec!["feature_allowed", "feature_blocked"]);

    // Requester not in the allow list: feature_allowed is silently filtered out,
    // but feature_blocked is kept (requester is not in the block list).
    let mut preferences = GetOptionsPreferences::new();
    preferences.requester = Some("unknown_service".to_owned());
    let result = provider
        .get_filtered_feature_names(&["feature_allowed", "feature_blocked"], Some(&preferences))?;
    assert_eq!(result, vec!["feature_blocked"]);

    // Requester in the block list: feature_blocked is also silently filtered out.
    preferences.requester = Some("untrusted_service".to_owned());
    let result = provider.get_filtered_feature_names(&["feature_blocked"], Some(&preferences))?;
    assert!(result.is_empty());

    // Allowed requester: feature_allowed is kept.
    preferences.requester = Some("service_a".to_owned());
    let result = provider
        .get_filtered_feature_names(&["feature_allowed", "feature_blocked"], Some(&preferences))?;
    assert_eq!(result, vec!["feature_allowed", "feature_blocked"]);

    Ok(())
}

#[test]
fn test_policy_filtering_raises_when_requested() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_policies_provider();
    let mut preferences = GetOptionsPreferences::new();
    preferences.requester = Some("untrusted_service".to_owned());
    preferences.raise_if_policy_denied = true;

    // Denied feature with raise_if_policy_denied=true returns an error.
    let result = provider.get_filtered_feature_names(&["feature_allowed"], Some(&preferences));
    assert_eq!(
        result.unwrap_err(),
        "Requester \"untrusted_service\" is not permitted to use feature \"feature_allowed\"."
    );

    Ok(())
}

#[test]
fn test_check_policies() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_policies_provider();
    let cache_options = None;

    // Allowed requester returns Ok(()).
    let check = provider.check_policies(
        "service_a",
        &["feature_allowed", "feature_blocked"],
        cache_options,
    );
    assert_eq!(check, Ok(()));

    // With alias
    let check = provider.check_policies("service_a", &["feat_allow"], cache_options);
    assert_eq!(check, Ok(()));

    let check = provider.check_policies("untrusted service", &["feat_allow"], cache_options);
    assert_eq!(
        check,
        Err(
            "Requester \"untrusted service\" is not permitted to use feature \"feature_allowed\"."
                .to_owned()
        )
    );

    let check = provider.check_policies("untrusted service", &["not a feature"], cache_options);
    assert_eq!(
        check,
        Err("Feature name \"not a feature\" is not a known feature.".to_owned())
    );

    // Disallowed requester on feature_allowed returns error string.
    let check = provider.check_policies("untrusted_service", &["feature_allowed"], cache_options);
    assert_eq!(
        check,
        Err(
            "Requester \"untrusted_service\" is not permitted to use feature \"feature_allowed\"."
                .to_owned()
        )
    );

    // Disallowed requester on feature_blocked returns error string.
    let check = provider.check_policies("untrusted_service", &["feature_blocked"], cache_options);
    assert_eq!(
        check,
        Err(
            "Requester \"untrusted_service\" is not permitted to use feature \"feature_blocked\"."
                .to_owned()
        )
    );

    // Multiple features: returns error for the first disallowed feature.
    let check = provider.check_policies(
        "untrusted_service",
        &["feature_allowed", "feature_blocked"],
        cache_options,
    );
    assert_eq!(
        check,
        Err(
            "Requester \"untrusted_service\" is not permitted to use feature \"feature_allowed\"."
                .to_owned()
        )
    );

    Ok(())
}

#[test]
fn test_requester_feature_policy_allow_from_policies_json() -> Result<(), Box<dyn std::error::Error>>
{
    let provider = get_policies_provider();
    let mut preferences = GetOptionsPreferences::new();
    preferences.raise_if_policy_denied = true;

    // `requester_x` is only allowed to use `feature_neutral` per `.optify/policies.json`.
    preferences.requester = Some("requester_x".to_owned());
    let result = provider.get_filtered_feature_names(&["feature_neutral"], Some(&preferences))?;
    assert_eq!(result, vec!["feature_neutral"]);

    let result = provider.get_filtered_feature_names(&["feature_blocked"], Some(&preferences));
    assert_eq!(
        result.unwrap_err(),
        "Requester \"requester_x\" is not permitted to use feature \"feature_blocked\"."
    );

    Ok(())
}

#[test]
fn test_requester_feature_policy_block_from_policies_json() -> Result<(), Box<dyn std::error::Error>>
{
    let provider = get_policies_provider();
    let mut preferences = GetOptionsPreferences::new();
    preferences.raise_if_policy_denied = true;

    // `requester_y` may not use `feature_neutral` per `.optify/policies.json`, but any other
    // feature is allowed by the requester-feature policy (feature-level policies still apply
    // separately).
    preferences.requester = Some("requester_y".to_owned());
    let result = provider.get_filtered_feature_names(&["feature_blocked"], Some(&preferences))?;
    assert_eq!(result, vec!["feature_blocked"]);

    let result = provider.get_filtered_feature_names(&["feature_neutral"], Some(&preferences));
    assert_eq!(
        result.unwrap_err(),
        "Requester \"requester_y\" is not permitted to use feature \"feature_neutral\"."
    );

    Ok(())
}

#[test]
fn test_requester_feature_policy_combines_with_feature_policy(
) -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_policies_provider();
    let cache_options = None;

    // `requester_z`'s `.optify/policies.json` entry only blocks `feature_blocked`, so the file
    // implicitly permits `requester_z` to use `feature_allowed`. However, `feature_allowed`'s own
    // `policies.requester` only allows `service_a`/`service_b`, so the request is still denied:
    // both policies must permit the requester.
    let check = provider.check_policies("requester_z", &["feature_allowed"], cache_options);
    assert_eq!(
        check,
        Err(
            "Requester \"requester_z\" is not permitted to use feature \"feature_allowed\"."
                .to_owned()
        )
    );

    // `feature_blocked` has no `policies.requester` restriction on `requester_z`, but the file
    // explicitly blocks it, so the request is still denied.
    let check = provider.check_policies("requester_z", &["feature_blocked"], cache_options);
    assert_eq!(
        check,
        Err(
            "Requester \"requester_z\" is not permitted to use feature \"feature_blocked\"."
                .to_owned()
        )
    );

    Ok(())
}

/// A requester's `.optify/policies.json` entry (the "global" policy) and a feature's own
/// `policies.requester` (the "feature" policy) are independently configurable and both checked:
/// neither can be used to bypass the other. This covers every combination of global/feature
/// policy shape (none/allow/block) that can coexist without a build-time conflict.
#[test]
fn test_requester_feature_policy_all_combinations() -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_policies_provider();
    let cache_options = None;

    // Global: none. Feature: none. -> permitted.
    assert_eq!(
        provider.check_policies(
            "totally_unknown_requester",
            &["feature_neutral"],
            cache_options
        ),
        Ok(())
    );

    // Global: none. Feature: allow (requester listed). -> permitted. (test_get_policies_allowed
    // and test_get_policies_blocked cover the "requester not listed"/"requester blocked" cases
    // for feature-only policies.)

    // Global: allow (includes the feature). Feature: none. -> permitted.
    assert_eq!(
        provider.check_policies("requester_x", &["feature_neutral"], cache_options),
        Ok(())
    );

    // Global: allow (includes the feature). Feature: allow (requester also listed, consistent).
    // -> permitted.
    assert_eq!(
        provider.check_policies("service_a", &["feature_allowed"], cache_options),
        Ok(())
    );

    // Global: allow (includes the feature). Feature: block (requester not listed, consistent).
    // -> permitted.
    assert_eq!(
        provider.check_policies("service_a", &["feature_blocked"], cache_options),
        Ok(())
    );

    // Global: allow (excludes the feature). Feature: allow (requester is listed). The global
    // list is checked independently and denies any feature it doesn't list, even though the
    // feature's own policy would allow this requester: a feature-level allow cannot bypass a
    // global list that excludes the feature.
    let check = provider.check_policies("service_b", &["feature_allowed"], cache_options);
    assert_eq!(
        check,
        Err(
            "Requester \"service_b\" is not permitted to use feature \"feature_allowed\"."
                .to_owned()
        )
    );

    // Global: allow (excludes the feature). Feature: none. -> denied by the global list alone.
    let check = provider.check_policies("service_b", &["feature_blocked"], cache_options);
    assert_eq!(
        check,
        Err(
            "Requester \"service_b\" is not permitted to use feature \"feature_blocked\"."
                .to_owned()
        )
    );

    // Global: block (excludes the feature, so implicitly permitted by the file). Feature: allow
    // (requester is listed). -> permitted.
    assert_eq!(
        provider.check_policies("service_d", &["feature_allowed"], cache_options),
        Ok(())
    );

    // Global: block (excludes the feature, so implicitly permitted by the file). Feature: block
    // (requester is listed). The feature's own block list is checked independently and denies
    // this requester even though the global file doesn't mention this feature: a global policy
    // that is silent on a feature cannot bypass the feature's own block list.
    let check = provider.check_policies("untrusted_service", &["feature_blocked"], cache_options);
    assert_eq!(
        check,
        Err(
            "Requester \"untrusted_service\" is not permitted to use feature \"feature_blocked\"."
                .to_owned()
        )
    );

    // Global: block (includes the feature). Feature: block (requester also listed, consistent,
    // redundant). -> denied.
    let check = provider.check_policies("service_f", &["feature_blocked"], cache_options);
    assert_eq!(
        check,
        Err(
            "Requester \"service_f\" is not permitted to use feature \"feature_blocked\"."
                .to_owned()
        )
    );

    // Global: block (includes the feature). Feature: none. -> denied by the global list alone.
    let check = provider.check_policies("requester_y", &["feature_neutral"], cache_options);
    assert_eq!(
        check,
        Err(
            "Requester \"requester_y\" is not permitted to use feature \"feature_neutral\"."
                .to_owned()
        )
    );

    Ok(())
}
