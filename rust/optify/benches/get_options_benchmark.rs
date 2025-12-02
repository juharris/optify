use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use optify::provider::{CacheOptions, GetOptionsPreferences, OptionsProvider, OptionsRegistry};

const CONFIGS_DIR: &str = "../../tests/test_suites/configurable_values/configs";

fn get_provider() -> OptionsProvider {
    OptionsProvider::build(CONFIGS_DIR).unwrap()
}

fn benchmark_get_options_with_preferences_breakdown(c: &mut Criterion) {
    let provider = get_provider();

    let mut group = c.benchmark_group("get_options_with_preferences");

    // Feature names are based on file names (without extensions)
    // Keys are the top-level keys inside the "options" object in each file
    let feature_sets: Vec<(&str, Vec<&str>, &str)> = vec![
        ("single_complex_deep", vec!["complex_deep_merge"], "config1"),
        (
            "single_complex_nested",
            vec!["complex_nested_objects"],
            "nestedConfig1",
        ),
        (
            "single_complex_wide",
            vec!["complex_wide_structure"],
            "wideConfig1",
        ),
        (
            "multiple_complex",
            vec![
                "complex_deep_merge",
                "complex_nested_objects",
                "complex_wide_structure",
            ],
            "config1",
        ),
    ];

    for (name, feature_names, key) in &feature_sets {
        // Benchmark: get_options call (no cache, no preferences)
        group.bench_with_input(
            BenchmarkId::new("get_options", name),
            &(feature_names, key),
            |b, (features, key)| {
                b.iter(|| {
                    provider
                        .get_options(black_box(*key), black_box(*features))
                        .unwrap()
                })
            },
        );

        // Benchmark: Full get_options_with_preferences (no cache)
        group.bench_with_input(
            BenchmarkId::new("get_options_with_preferences no cache", name),
            &(feature_names, key),
            |b, (features, key)| {
                let preferences = GetOptionsPreferences::new();
                b.iter(|| {
                    provider
                        .get_options_with_preferences(
                            black_box(*key),
                            black_box(*features),
                            None,
                            Some(&preferences),
                        )
                        .unwrap()
                })
            },
        );

        // Benchmark: With cache (cache miss then hits)
        group.bench_with_input(
            BenchmarkId::new("get_options_with_preferences with cache", name),
            &(feature_names, key),
            |b, (features, key)| {
                let cache_options = CacheOptions {};
                // Pre-populate cache
                let _ = provider
                    .get_options_with_preferences(*key, *features, Some(&cache_options), None)
                    .unwrap();

                b.iter(|| {
                    provider
                        .get_options_with_preferences(
                            black_box(*key),
                            black_box(*features),
                            Some(&cache_options),
                            None,
                        )
                        .unwrap()
                })
            },
        );

        // Benchmark: get_filtered_feature_names
        group.bench_with_input(
            BenchmarkId::new("get_filtered_feature_names", name),
            feature_names,
            |b, features| {
                b.iter(|| {
                    provider
                        .get_filtered_feature_names(black_box(features), None)
                        .unwrap()
                })
            },
        );

        // Benchmark: get_canonical_feature_names
        group.bench_with_input(
            BenchmarkId::new("get_canonical_feature_names", name),
            feature_names,
            |b, features| {
                b.iter(|| {
                    provider
                        .get_canonical_feature_names(black_box(features))
                        .unwrap()
                })
            },
        );
    }

    group.finish();
}

fn benchmark_with_overrides(c: &mut Criterion) {
    let provider = get_provider();

    let mut group = c.benchmark_group("get_options/with_overrides");

    let features = vec!["complex_deep_merge"];

    // Without overrides
    group.bench_function("no_overrides", |b| {
        b.iter(|| {
            provider
                .get_options_with_preferences(
                    black_box("config1"),
                    black_box(&features),
                    None,
                    None,
                )
                .unwrap()
        })
    });

    // With small override
    group.bench_function("small_override", |b| {
        let mut preferences = GetOptionsPreferences::new();
        preferences.overrides = Some(serde_json::json!({"config1": {"extra": "value"}}));

        b.iter(|| {
            provider
                .get_options_with_preferences(
                    black_box("config1"),
                    black_box(&features),
                    None,
                    Some(&preferences),
                )
                .unwrap()
        })
    });

    // With larger override
    group.bench_function("larger_override", |b| {
        let mut preferences = GetOptionsPreferences::new();
        preferences.overrides = Some(serde_json::json!({
            "config1": {
                "level1": {
                    "level2": {
                        "key1": "overridden value 1",
                        "key2": "overridden value 2",
                        "newKey": "brand new value"
                    }
                }
            }
        }));

        b.iter(|| {
            provider
                .get_options_with_preferences(
                    black_box("config1"),
                    black_box(&features),
                    None,
                    Some(&preferences),
                )
                .unwrap()
        })
    });

    group.finish();
}

fn benchmark_skip_feature_name_conversion(c: &mut Criterion) {
    let provider = get_provider();

    let mut group = c.benchmark_group("get_options/skip_feature_name_conversion");

    let features = vec!["complex_deep_merge", "complex_nested_objects"];

    // With feature name conversion (default)
    group.bench_function("with_conversion", |b| {
        let preferences = GetOptionsPreferences::new();
        b.iter(|| {
            provider
                .get_options_with_preferences(
                    black_box("config1"),
                    black_box(&features),
                    None,
                    Some(&preferences),
                )
                .unwrap()
        })
    });

    // Without feature name conversion
    group.bench_function("skip_conversion", |b| {
        let mut preferences = GetOptionsPreferences::new();
        preferences.skip_feature_name_conversion = true;

        b.iter(|| {
            provider
                .get_options_with_preferences(
                    black_box("config1"),
                    black_box(&features),
                    None,
                    Some(&preferences),
                )
                .unwrap()
        })
    });

    group.finish();
}

fn benchmark_many_features(c: &mut Criterion) {
    let provider = get_provider();

    let mut group = c.benchmark_group("get_options/many features");

    let keys = vec![
        "config1",
        "config2",
        "config3",
        "config6",
        "config10",
        "config11",
        "config15",
        "greeting",
        "wideConfig3",
    ];
    let features = vec![
        "simple",
        "complex_wide_structure",
        "complex_deep_merge",
        "complex_nested_objects",
        "type_other_string",
        "simple",
        "with_files",
    ];

    for key in keys {
        group.bench_function(key, |b| {
            let mut preferences = GetOptionsPreferences::new();
            preferences.are_configurable_strings_enabled = true;
            let preferences = Some(&preferences);
            b.iter(|| {
                provider
                    .get_options_with_preferences(
                        black_box(key),
                        black_box(&features),
                        None,
                        preferences,
                    )
                    .unwrap()
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_get_options_with_preferences_breakdown,
    benchmark_many_features,
    benchmark_with_overrides,
    benchmark_skip_feature_name_conversion,
);
criterion_main!(benches);
