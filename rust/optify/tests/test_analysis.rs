use std::path::Path;

use optify::builder::BuilderOptions;
use optify::provider::{
    GetOptionsPreferences, OptionAnalysis, OptionOrigin, OptionSource, OptionsProvider,
    OptionsRegistry, OptionsWatcher,
};
use serde_json::{json, Value};

const ANALYSIS_CONFIGS: &str = "tests/analysis";
const INHERITANCE_CONFIGS: &str = "../../tests/test_suites/inheritance/configs";
const TEST_SUITES: &str = "../../tests/test_suites";

fn analysis_options() -> BuilderOptions {
    BuilderOptions {
        is_analysis_enabled: true,
        ..Default::default()
    }
}

fn build(directory: impl AsRef<Path>) -> OptionsProvider {
    OptionsProvider::build_with_options(directory, analysis_options()).unwrap()
}

fn feature(import_chain: &[&str]) -> OptionSource {
    OptionSource::Feature {
        import_chain: import_chain.iter().map(ToString::to_string).collect(),
    }
}

fn origin(path: &str, source: OptionSource, overridden: Vec<OptionSource>) -> OptionOrigin {
    OptionOrigin {
        overridden,
        path: path.to_owned(),
        source,
    }
}

fn found(origins: Vec<OptionOrigin>) -> OptionAnalysis {
    OptionAnalysis {
        blocked_by: None,
        origins,
    }
}

fn blocked(blocked_by: OptionOrigin) -> OptionAnalysis {
    OptionAnalysis {
        blocked_by: Some(blocked_by),
        origins: Vec::new(),
    }
}

/// Collects JSON pointers to every non-object value and to every value in an array.
fn collect_leaf_pointers(value: &Value, path: &str, pointers: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let escaped_key = key.replace('~', "~0").replace('/', "~1");
                collect_leaf_pointers(child, &format!("{path}/{escaped_key}"), pointers);
            }
        }
        _ => pointers.push(path.to_owned()),
    }
}

#[test]
fn test_analyze_alias_uses_canonical_feature_name() {
    let provider = build(ANALYSIS_CONFIGS);
    let analysis = provider.analyze("/k/x", &["the_x"], None).unwrap();
    assert_eq!(
        analysis,
        found(vec![origin("/k/x", feature(&["x"]), vec![])])
    );
}

#[test]
fn test_analyze_array_index_is_attributed_to_the_array() {
    let provider = build(ANALYSIS_CONFIGS);
    let analysis = provider
        .analyze("/k/list/1", &["list_low", "list_high"], None)
        .unwrap();
    assert_eq!(
        analysis,
        found(vec![origin(
            "/k/list/1",
            feature(&["list_high"]),
            vec![feature(&["list_low"])]
        )])
    );
}

#[test]
fn test_analyze_blocked_by_frozen_path() {
    let provider = build(ANALYSIS_CONFIGS);
    let features = ["x", "b", "high"];
    assert_eq!(
        provider.get_all_options(&features, None, None).unwrap(),
        json!({"k": {"h": 1}})
    );

    let analysis = provider.analyze("/k/x", &features, None).unwrap();
    assert_eq!(
        analysis,
        blocked(origin("/k", feature(&["b"]), vec![feature(&["x"])]))
    );
}

#[test]
fn test_analyze_blocked_by_non_object_ancestor() {
    let provider = build(ANALYSIS_CONFIGS);
    let analysis = provider.analyze("/k/x", &["x", "null_k"], None).unwrap();
    assert_eq!(
        analysis,
        blocked(origin("/k", feature(&["null_k"]), vec![feature(&["x"])]))
    );
}

#[test]
fn test_analyze_constraints_filter_conditional_features() {
    let provider = build(ANALYSIS_CONFIGS);
    let features = ["x", "conditional"];

    let mut preferences = GetOptionsPreferences::new();
    preferences.set_constraints(Some(json!({"env": "prod"})));
    let analysis = provider
        .analyze("/k/x", &features, Some(&preferences))
        .unwrap();
    assert_eq!(
        analysis,
        found(vec![origin(
            "/k/x",
            feature(&["conditional"]),
            vec![feature(&["x"])]
        )])
    );

    preferences.set_constraints(Some(json!({"env": "dev"})));
    let analysis = provider
        .analyze("/k/x", &features, Some(&preferences))
        .unwrap();
    assert_eq!(
        analysis,
        found(vec![origin("/k/x", feature(&["x"]), vec![])])
    );
}

#[test]
fn test_analyze_enabled_by_directory_config() {
    let provider = OptionsProvider::build("tests/analysis_enabled_by_config").unwrap();
    let analysis = provider.analyze("/k/x", &["a"], None).unwrap();
    assert_eq!(
        analysis,
        found(vec![origin("/k/x", feature(&["a"]), vec![])])
    );
}

#[test]
fn test_analyze_import_chain() {
    let provider = build(INHERITANCE_CONFIGS);

    let analysis = provider
        .analyze("/myConfig/key2", &["grandparent"], None)
        .unwrap();
    assert_eq!(
        analysis,
        found(vec![origin(
            "/myConfig/key2",
            feature(&["grandparent", "parent2", "base2"]),
            vec![feature(&["grandparent", "parent1"])]
        )])
    );

    let analysis = provider
        .analyze("/myConfig/key1", &["grandparent"], None)
        .unwrap();
    assert_eq!(
        analysis,
        found(vec![origin(
            "/myConfig/key1",
            feature(&["grandparent", "parent1", "base1"]),
            vec![]
        )])
    );
}

#[test]
fn test_analyze_invalid_key_path() {
    let provider = build(ANALYSIS_CONFIGS);
    for key_path in ["", "k/x", "/k~2", "/k~"] {
        let error = provider.analyze(key_path, &["x"], None).unwrap_err();
        assert!(error.contains("Invalid key path"), "{key_path:?}: {error}");
    }
}

#[test]
fn test_analyze_multiple_features_with_imports() {
    let provider = build(INHERITANCE_CONFIGS);
    let features = ["grandparent", "feature_C", "super"];

    let analysis = provider.analyze("/myConfig/key2", &features, None).unwrap();
    assert_eq!(
        analysis,
        found(vec![origin(
            "/myConfig/key2",
            feature(&["super", "base2"]),
            vec![
                feature(&["grandparent", "parent2", "base2"]),
                feature(&["grandparent", "parent1"]),
            ]
        )])
    );

    let analysis = provider
        .analyze("/myConfig/myObject", &features, None)
        .unwrap();
    assert_eq!(
        analysis,
        found(vec![
            origin(
                "/myConfig/myObject/one",
                feature(&["feature_C", "feature_B"]),
                vec![feature(&["feature_C", "feature_A"])]
            ),
            origin(
                "/myConfig/myObject/three",
                feature(&["feature_C"]),
                vec![feature(&["feature_C", "feature_B"])]
            ),
            origin(
                "/myConfig/myObject/two",
                feature(&["feature_C", "feature_A"]),
                vec![]
            ),
        ])
    );
}

#[test]
fn test_analyze_nested_imports_are_not_flattened() {
    // `f` imports `b`, which sets `k` to a non-object.
    // Within `f`, `b` loses to `f`'s object, but that does not stop `x` from adding to `k` for the request.
    let provider = build(ANALYSIS_CONFIGS);
    let features = ["x", "f"];
    assert_eq!(
        provider.get_all_options(&features, None, None).unwrap(),
        json!({"k": {"f": 1, "x": 1}})
    );

    let analysis = provider.analyze("/k/x", &features, None).unwrap();
    assert_eq!(
        analysis,
        found(vec![origin(
            "/k/x",
            feature(&["x"]),
            vec![feature(&["f", "b"])]
        )])
    );
}

#[test]
fn test_analyze_not_enabled() {
    let provider = OptionsProvider::build(ANALYSIS_CONFIGS).unwrap();
    let error = provider.analyze("/k/x", &["x"], None).unwrap_err();
    assert!(error.contains("Analysis is not enabled"), "{error}");
}

#[test]
fn test_analyze_not_found() {
    let provider = build(ANALYSIS_CONFIGS);

    let error = provider.analyze("/missing", &["x"], None).unwrap_err();
    assert_eq!(
        error,
        "Error analyzing options with features [\"x\"]: configuration property at \"/missing\" not found."
    );

    let error = provider.analyze("/k/x/deeper", &["x"], None).unwrap_err();
    assert_eq!(
        error,
        "Error analyzing options with features [\"x\"]: configuration property at \"/k/x/deeper\" not found. The value at \"/k/x\" is not an object."
    );

    let error = provider.analyze("/k/x", &[] as &[&str], None).unwrap_err();
    assert!(error.contains("not found"), "{error}");
}

#[test]
fn test_analyze_object_with_several_origins() {
    let provider = build(INHERITANCE_CONFIGS);
    let analysis = provider
        .analyze("/myConfig", &["grandparent"], None)
        .unwrap();
    assert_eq!(
        analysis,
        found(vec![
            origin(
                "/myConfig/key1",
                feature(&["grandparent", "parent1", "base1"]),
                vec![]
            ),
            origin(
                "/myConfig/key2",
                feature(&["grandparent", "parent2", "base2"]),
                vec![feature(&["grandparent", "parent1"])]
            ),
        ])
    );
}

#[test]
fn test_analyze_object_with_one_origin() {
    let provider = build(INHERITANCE_CONFIGS);
    let analysis = provider.analyze("/myConfig", &["feature_A"], None).unwrap();
    assert_eq!(
        analysis,
        found(vec![origin("/myConfig", feature(&["feature_A"]), vec![])])
    );
}

#[test]
fn test_analyze_overrides() {
    let provider = build(ANALYSIS_CONFIGS);
    let mut preferences = GetOptionsPreferences::new();
    preferences.overrides = Some(json!({"k": {"x": 99}}));

    let analysis = provider
        .analyze("/k/x", &["x"], Some(&preferences))
        .unwrap();
    assert_eq!(
        analysis,
        found(vec![origin(
            "/k/x",
            OptionSource::Overrides,
            vec![feature(&["x"])]
        )])
    );

    let analysis = provider
        .analyze("/k/x", &[] as &[&str], Some(&preferences))
        .unwrap();
    assert_eq!(
        analysis,
        found(vec![origin("/k/x", OptionSource::Overrides, vec![])])
    );
}

#[test]
fn test_analyze_serialization() {
    let provider = build(INHERITANCE_CONFIGS);
    let analysis = provider
        .analyze("/myConfig/key2", &["grandparent"], None)
        .unwrap();
    assert_eq!(
        serde_json::to_value(&analysis).unwrap(),
        json!({
            "origins": [{
                "overridden": [{"kind": "feature", "importChain": ["grandparent", "parent1"]}],
                "path": "/myConfig/key2",
                "source": {"kind": "feature", "importChain": ["grandparent", "parent2", "base2"]},
            }]
        })
    );

    let provider = build(ANALYSIS_CONFIGS);
    let analysis = provider.analyze("/k/x", &["x", "null_k"], None).unwrap();
    assert_eq!(
        serde_json::to_value(&analysis).unwrap(),
        json!({
            "blockedBy": {
                "overridden": [{"kind": "feature", "importChain": ["x"]}],
                "path": "/k",
                "source": {"kind": "feature", "importChain": ["null_k"]},
            },
            "origins": [],
        })
    );
}

#[test]
fn test_analyze_special_characters_in_keys() {
    let provider = build(ANALYSIS_CONFIGS);
    let analysis = provider.analyze("/a~1b/c~0d", &["special"], None).unwrap();
    assert_eq!(
        analysis,
        found(vec![origin("/a~1b/c~0d", feature(&["special"]), vec![])])
    );
}

/// For every value in every test suite expectation, the feature that analysis reports as defining the value must have that value.
#[test]
fn test_analyze_test_suites_match_options() {
    let mut analyzed_count = 0;
    for suite in std::fs::read_dir(TEST_SUITES).unwrap() {
        let suite_path = suite.unwrap().path();
        let provider = build(suite_path.join("configs"));
        for expectation in std::fs::read_dir(suite_path.join("expectations")).unwrap() {
            let expectation_path = expectation.unwrap().path();
            let expectation: Value =
                serde_json::from_str(&std::fs::read_to_string(&expectation_path).unwrap()).unwrap();
            let features: Vec<&str> = expectation["features"]
                .as_array()
                .unwrap()
                .iter()
                .map(|feature| feature.as_str().unwrap())
                .collect();
            let mut preferences = GetOptionsPreferences::new();
            preferences.set_constraints(expectation.get("constraints").cloned());

            let options = provider
                .get_all_options(&features, None, Some(&preferences))
                .unwrap();
            let mut pointers = Vec::new();
            collect_leaf_pointers(&options, "", &mut pointers);
            for pointer in pointers {
                let context = format!("{expectation_path:?} at {pointer:?}");
                let analysis = provider
                    .analyze(&pointer, &features, Some(&preferences))
                    .unwrap_or_else(|error| panic!("{context}: {error}"));
                assert!(analysis.blocked_by.is_none(), "{context}");
                assert_eq!(analysis.origins.len(), 1, "{context}");
                let origin = &analysis.origins[0];
                assert_eq!(origin.path, pointer, "{context}");

                let defining_feature = origin.source.defining_feature().unwrap();
                let defining_options = provider
                    .get_all_options(&[defining_feature], None, None)
                    .unwrap();
                assert_eq!(
                    defining_options.pointer(&pointer),
                    options.pointer(&pointer),
                    "{context}"
                );
                analyzed_count += 1;
            }
        }
    }
    assert!(analyzed_count > 100, "analyzed {analyzed_count} values");
}

#[test]
fn test_analyze_watcher() {
    // `OptionsWatcher` has an inherent `build_with_options` that takes `WatcherOptions`.
    let watcher = <OptionsWatcher as OptionsRegistry>::build_with_options(
        ANALYSIS_CONFIGS,
        analysis_options(),
    )
    .unwrap();
    let analysis = watcher.analyze("/k/x", &["x"], None).unwrap();
    assert_eq!(
        analysis,
        found(vec![origin("/k/x", feature(&["x"]), vec![])])
    );
}
