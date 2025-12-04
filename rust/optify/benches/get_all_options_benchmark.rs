use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use optify::provider::{GetOptionsPreferences, OptionsProvider, OptionsRegistry};

const CONFIGS_DIR: &str = "../../tests/test_suites/configurable_values/configs";

fn get_provider() -> OptionsProvider {
    OptionsProvider::build(CONFIGS_DIR).unwrap()
}

fn benchmark_get_all_options_breakdown(c: &mut Criterion) {
    let provider = get_provider();

    let mut group = c.benchmark_group("get_all_options");

    // Feature names are based on file names (without extensions)
    // Keys are the top-level keys inside the "options" object in each file
    let feature_sets: Vec<(&str, Vec<&str>)> = vec![
        ("single_complex_deep", vec!["complex_deep_merge"]),
        ("single_complex_nested", vec!["complex_nested_objects"]),
        ("single_complex_wide", vec!["complex_wide_structure"]),
        (
            "multiple_complex",
            vec![
                "simple",
                "complex_wide_structure",
                "complex_deep_merge",
                "complex_nested_objects",
                "type_other_string",
                "simple",
                "with_files",
            ],
        ),
    ];

    for (name, feature_names) in &feature_sets {
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
    }

    group.finish();
}

fn benchmark_configurable_strings(c: &mut Criterion) {
    let provider = get_provider();

    let mut group = c.benchmark_group("get_all_options/configurable_strings");

    let features_with_files = vec![
        "simple",
        "complex_wide_structure",
        "complex_deep_merge",
        "complex_nested_objects",
        "type_other_string",
        "simple",
        "with_files",
    ];

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

fn benchmark_with_overrides(c: &mut Criterion) {
    let provider = get_provider();

    let mut group = c.benchmark_group("get_all_options/with_overrides");

    let features = vec!["complex_deep_merge"];

    // Without overrides
    group.bench_function("no_overrides", |b| {
        b.iter(|| {
            provider
                .get_all_options(black_box(&features), None, None)
                .unwrap()
        })
    });

    // With small override
    group.bench_function("small_override", |b| {
        let mut preferences = GetOptionsPreferences::new();
        preferences.overrides = Some(serde_json::json!({"config1": {"extra": "value"}}));

        b.iter(|| {
            provider
                .get_all_options(black_box(&features), None, Some(&preferences))
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
                .get_all_options(black_box(&features), None, Some(&preferences))
                .unwrap()
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_get_all_options_breakdown,
    benchmark_configurable_strings,
    benchmark_with_overrides,
);
criterion_main!(benches);
