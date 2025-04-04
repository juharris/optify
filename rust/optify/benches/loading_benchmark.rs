use criterion::{black_box, criterion_group, criterion_main, Criterion};
use optify::builder::OptionsProviderBuilder;
use std::fs;
use std::io::Write;
use std::path::Path;

fn create_test_files(dir: &Path, num_files: usize) {
    fs::create_dir_all(dir).unwrap();

    for i in 0..num_files {
        let file_path = dir.join(format!("test_{}.json", i));
        let mut file = fs::File::create(&file_path).unwrap();

        // Create a test configuration file with some realistic content
        let content = format!(
            r#"{{
                "metadata": {{
                    "name": "test_{}",
                    "description": "Test feature {}",
                    "aliases": ["alias_{}_1", "alias_{}_2"]
                }},
                "options": {{
                    "setting1": "value1",
                    "setting2": 42,
                    "setting3": true,
                    "nested": {{
                        "field1": "nested_value",
                        "field2": [1, 2, 3]
                    }}
                }}
            }}"#,
            i, i, i, i
        );

        file.write_all(content.as_bytes()).unwrap();
    }
}

fn cleanup_test_files(dir: &Path) {
    fs::remove_dir_all(dir).unwrap();
}

fn benchmark_loading(c: &mut Criterion) {
    let test_dir = Path::new("bench_test_files");
    let num_files = 1000;

    create_test_files(test_dir, num_files);

    let mut group = c.benchmark_group(format!("file_loading-{}", num_files));

    group.bench_function("parallel_loading", |b| {
        b.iter(|| {
            let mut builder = OptionsProviderBuilder::new();
            builder
                .add_directory_with_parallel(black_box(test_dir), true)
                .unwrap();
        })
    });

    group.bench_function("sequential_loading", |b| {
        b.iter(|| {
            let mut builder = OptionsProviderBuilder::new();
            builder
                .add_directory_with_parallel(black_box(test_dir), false)
                .unwrap();
        })
    });

    group.finish();

    cleanup_test_files(test_dir);
}

criterion_group!(benches, benchmark_loading);
criterion_main!(benches);
