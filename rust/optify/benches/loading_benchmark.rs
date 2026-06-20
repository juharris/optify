use criterion::{black_box, criterion_group, criterion_main, Criterion};
use optify::builder::BuilderOptions;
use optify::builder::OptionsProviderBuilder;
use optify::builder::OptionsRegistryBuilder;
use optify::provider::OptionsRegistry;
use optify::OptionsProvider;
use std::fs;
use std::io::Write;
use std::path::Path;

struct TestDirGuard {
    dir: &'static Path,
}

impl TestDirGuard {
    fn new(dir: &'static Path) -> Self {
        Self { dir }
    }
}

impl Drop for TestDirGuard {
    fn drop(&mut self) {
        cleanup_test_files(self.dir);
    }
}

fn create_test_files(dir: &Path, num_files: usize) {
    cleanup_test_files(dir);
    fs::create_dir_all(dir).unwrap();

    for i in 0..num_files / 2 {
        let file_path = dir.join(format!("test_{i}.json"));
        let mut file = fs::File::create(&file_path).unwrap();

        // Create a test configuration file with some realistic content
        let content = format!(
            r#"{{
    "metadata": {{
        "aliases": ["alias_{i}_1", "alias_{i}_2"]
    }},
    "options": {{
        "setting1": "value1",
        "setting2": {{
            "existing_value": "exists",
            "value{i}": "abc"
        }},
        "setting3": true,
        "nested": {{
            "field1": "nested_value",
            "field2": [1, 2, 3]
        }}
    }}
}}"#,
        );

        file.write_all(content.as_bytes()).unwrap();
    }

    // Create features with imports.
    for i in num_files / 2..num_files {
        let file_path = dir.join(format!("test_{i}.yaml"));
        let mut file = fs::File::create(&file_path).unwrap();
        let imports = if i > (num_files / 2 + 3) {
            format!(
                "imports:
  - test_{}
",
                i - 1
            )
        } else {
            "".to_string()
        };

        let content = format!(
            r#"metadata:
    aliases:
        - alias_y_{i}_1
        - alias_y_{i}_2
{imports}
options:
    setting1: value1
    setting2:
      existing_value: "exists"
      value{i}: "abc"
    setting3: true
    setting{i}:
      value{i}: 32
    nested:
      field1: nested_value
      field2: [1, 2, 3]
"#
        );
        file.write_all(content.as_bytes()).unwrap();
    }

    // Create raw files.
    for i in 0..num_files / 2 {
        let file_path = dir.join(format!("test_{i}.txt"));
        let mut file = fs::File::create(&file_path).unwrap();

        let content = format!(
            r#"This is what goes in the file {i}
            "#
        );
        file.write_all(content.as_bytes()).unwrap();
    }
}

fn cleanup_test_files(dir: &Path) {
    fs::remove_dir_all(dir).unwrap_or_default();
}

fn benchmark_loading(c: &mut Criterion) {
    let test_dir = Path::new("bench_test_files");
    let num_files = 100;
    let _guard = TestDirGuard::new(test_dir);

    create_test_files(test_dir, num_files);

    // Ensure that there are no errors.
    OptionsProvider::build(test_dir).unwrap();

    let mut group = c.benchmark_group(format!("loading-{num_files}"));

    group.bench_function("parallel loading", |b| {
        b.iter(|| {
            let mut builder = black_box(OptionsProviderBuilder::new());
            black_box(
                builder
                    .with_options(BuilderOptions {
                        are_configurable_values_enabled: true,
                        ..Default::default()
                    })
                    .unwrap(),
            );
            builder.add_directory(black_box(test_dir)).unwrap();
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_loading);
criterion_main!(benches);
