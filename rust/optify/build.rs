use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    copy_schema_file().unwrap();
    generate_test_suites();

    println!("cargo:rerun-if-changed=build.rs");
}

/// Copy the schema file to the output directory so it can be used during the build to embed it in the crate.
fn copy_schema_file() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("schemas/feature_file.json");
    let dest_dir = dest_path.parent().unwrap();
    fs::create_dir_all(dest_dir)?;

    // Try multiple locations for the schema file.
    let original_path = Path::new("../../schemas/feature_file.json");
    let copied_path = Path::new("schemas/feature_file.json");
    let possible_paths = [
        // Original location in repo, used when running tests.
        original_path,
        // A copy for `cargo publish`.
        copied_path,
    ];

    let mut source_path = None;
    for path in &possible_paths {
        if path.exists() {
            source_path = Some(path);
            break;
        }
    }

    if let Some(path) = source_path {
        fs::copy(path, &dest_path)
            .unwrap_or_else(|e| panic!("Failed to copy schema file from {path:?}: {e}"));
        println!("cargo:rerun-if-changed={}", path.display());
    } else {
        panic!(
            "Schema file not found at any of the expected locations: {possible_paths:?}. If you are running `cargo publish`, then the schema needs to be copied from {original_path:?} to {copied_path:?}.",
        );
    }

    Ok(())
}

/// Dynamically generate the tests for each folder in tests/test_suites.
fn generate_test_suites() {
    let test_suites_path = Path::new("../../tests/test_suites");
    if !test_suites_path.exists() {
        // Not found most likely because we're running `cargo publish`.
        return;
    }

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("test_suites.rs");
    let mut f = fs::File::create(&dest_path).unwrap();
    for entry in fs::read_dir(test_suites_path).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() && !path.starts_with(".") {
            if let Some(file_name) = path.file_name() {
                if let Some(suite_name) = file_name.to_str() {
                    writeln!(f,
                                    "#[test]\nfn test_suite_{suite_name}() {{\n    test_suite(std::path::Path::new(\"../../tests/test_suites/{suite_name}\")).unwrap();\n}}\n"
                                ).unwrap();
                }
            }
        }
    }

    println!("cargo:rerun-if-changed=../../tests/test_suites");
}
