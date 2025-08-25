use criterion::{black_box, criterion_group, criterion_main, Criterion};
use optify::provider::{CacheOptions, GetOptionsPreferences, OptionsProvider, OptionsRegistry};

fn get_simple_provider() -> OptionsProvider {
    OptionsProvider::build("../../tests/test_suites/simple/configs").unwrap()
}

fn benchmark_cache_vs_no_cache(c: &mut Criterion) {
    let provider = get_simple_provider();
    let cache_options = CacheOptions {};

    let mut group = c.benchmark_group("cache_performance");
    let features_a = ["a"];

    // Benchmark without caching
    group.bench_function("no_cache", |b| {
        b.iter(|| {
            let _ = provider
                .get_all_options(black_box(&features_a), None, None)
                .unwrap();
        })
    });

    // Benchmark with caching (should be faster after first call)
    group.bench_function("with_cache", |b| {
        // Pre-populate cache
        let _ = provider
            .get_all_options(&features_a, Some(&cache_options), None)
            .unwrap();

        b.iter(|| {
            let _ = provider
                .get_all_options(black_box(&features_a), Some(&cache_options), None)
                .unwrap();
        })
    });

    // Benchmark specific config retrieval without cache
    group.bench_function("no_cache_specific_config", |b| {
        b.iter(|| {
            let _ = provider
                .get_options_with_preferences(
                    black_box("myConfig"),
                    black_box(&features_a),
                    None,
                    None,
                )
                .unwrap();
        })
    });

    // Benchmark specific config retrieval with cache
    group.bench_function("with_cache_specific_config", |b| {
        // Pre-populate cache
        let _ = provider
            .get_options_with_preferences("myConfig", &features_a, Some(&cache_options), None)
            .unwrap();

        b.iter(|| {
            let _ = provider
                .get_options_with_preferences(
                    black_box("myConfig"),
                    black_box(&features_a),
                    Some(&cache_options),
                    None,
                )
                .unwrap();
        })
    });

    group.finish();
}

fn benchmark_cache_with_preferences(c: &mut Criterion) {
    let features_a = ["a"];
    let provider = get_simple_provider();
    let cache_options = CacheOptions {};

    let mut group = c.benchmark_group("cache_with_preferences");

    // Benchmark without caching
    group.bench_function("no_cache_with_preferences", |b| {
        let mut preferences = GetOptionsPreferences::new();
        preferences.skip_feature_name_conversion = false;

        b.iter(|| {
            let _ = provider
                .get_all_options(black_box(&features_a), None, Some(&preferences))
                .unwrap();
        })
    });

    // Benchmark with caching and preferences (cache hit)
    group.bench_function("cache_hit_with_preferences", |b| {
        let mut preferences = GetOptionsPreferences::new();
        preferences.skip_feature_name_conversion = false;

        // Pre-populate cache
        let _ = provider
            .get_all_options(&features_a, Some(&cache_options), Some(&preferences))
            .unwrap();

        b.iter(|| {
            let _ = provider
                .get_all_options(
                    black_box(&features_a),
                    Some(&cache_options),
                    Some(&preferences),
                )
                .unwrap();
        })
    });

    group.finish();
}

fn benchmark_cache_multiple_features(c: &mut Criterion) {
    let features_a_b = ["a", "b"];
    let provider = get_simple_provider();
    let cache_options = CacheOptions {};

    let mut group = c.benchmark_group("cache_multiple_features");

    // Benchmark without caching for multiple features
    group.bench_function("no_cache_multiple_features", |b| {
        b.iter(|| {
            let _ = provider
                .get_all_options(black_box(&features_a_b), None, None)
                .unwrap();
        })
    });

    // Benchmark with caching for multiple features (cache hit)
    group.bench_function("cache_hit_multiple_features", |b| {
        // Pre-populate cache
        let _ = provider
            .get_all_options(&features_a_b, Some(&cache_options), None)
            .unwrap();

        b.iter(|| {
            let _ = provider
                .get_all_options(black_box(&features_a_b), Some(&cache_options), None)
                .unwrap();
        })
    });

    group.finish();
}

fn benchmark_cache_performance_scaling(c: &mut Criterion) {
    let features_a = ["a"];
    let provider = get_simple_provider();
    let cache_options = CacheOptions {};

    let mut group = c.benchmark_group("cache_performance_scaling");

    // Benchmark repeated calls without caching
    group.bench_function("repeated_calls_no_cache", |b| {
        b.iter(|| {
            for _ in 0..10 {
                let _ = provider
                    .get_all_options(black_box(&features_a), None, None)
                    .unwrap();
            }
        })
    });

    // Benchmark repeated calls with caching (should show significant improvement)
    group.bench_function("repeated_calls_with_cache", |b| {
        // Pre-populate cache
        let _ = provider
            .get_all_options(&features_a, Some(&cache_options), None)
            .unwrap();

        b.iter(|| {
            for _ in 0..10 {
                let _ = provider
                    .get_all_options(black_box(&features_a), Some(&cache_options), None)
                    .unwrap();
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_cache_vs_no_cache,
    benchmark_cache_with_preferences,
    benchmark_cache_multiple_features,
    benchmark_cache_performance_scaling
);
criterion_main!(benches);
