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
        // Benchmark: Full get_options call (no cache, no preferences)
        group.bench_with_input(
            BenchmarkId::new("full_no_cache", name),
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
            BenchmarkId::new("full_with_prefs_no_cache", name),
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
            BenchmarkId::new("with_cache_hit", name),
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

        // Benchmark: get_all_options (entire config)
        group.bench_with_input(
            BenchmarkId::new("get_all_options", name),
            feature_names,
            |b, features| {
                b.iter(|| {
                    provider
                        .get_all_options(black_box(features), None, None)
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

fn benchmark_configurable_strings(c: &mut Criterion) {
    let provider = get_provider();

    let mut group = c.benchmark_group("configurable_strings");

    let features_with_files = vec!["with_files"];

    // Without configurable strings enabled (default)
    group.bench_function("disabled", |b| {
        b.iter(|| {
            provider
                .get_all_options(black_box(&features_with_files), None, None)
                .unwrap()
        })
    });

    // With configurable strings enabled
    group.bench_function("enabled", |b| {
        let mut preferences = GetOptionsPreferences::new();
        preferences.are_configurable_strings_enabled = true;

        b.iter(|| {
            provider
                .get_all_options(black_box(&features_with_files), None, Some(&preferences))
                .unwrap()
        })
    });

    group.finish();
}

fn benchmark_different_keys(c: &mut Criterion) {
    let provider = get_provider();

    let mut group = c.benchmark_group("different_keys");

    let features = vec!["complex_deep_merge"];

    // Getting top-level config key
    group.bench_function("top_level_config1", |b| {
        b.iter(|| {
            provider
                .get_options(black_box("config1"), black_box(&features))
                .unwrap()
        })
    });

    // Getting all options (entire merged config)
    group.bench_function("get_all_options", |b| {
        b.iter(|| {
            provider
                .get_all_options(black_box(&features), None, None)
                .unwrap()
        })
    });

    group.finish();
}

fn benchmark_with_overrides(c: &mut Criterion) {
    let provider = get_provider();

    let mut group = c.benchmark_group("with_overrides");

    let features = vec!["complex_deep_merge"];

    // Without overrides
    group.bench_function("no_overrides", |b| {
        b.iter(|| {
            provider
                .get_options(black_box("config1"), black_box(&features))
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

    let mut group = c.benchmark_group("skip_feature_name_conversion");

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

criterion_group!(
    benches,
    benchmark_get_options_with_preferences_breakdown,
    benchmark_configurable_strings,
    benchmark_different_keys,
    benchmark_with_overrides,
    benchmark_skip_feature_name_conversion,
);
criterion_main!(benches);
