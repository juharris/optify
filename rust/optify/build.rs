use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    copy_schema_file().unwrap();
    generate_test_suites();

    println!("cargo:rerun-if-changed=build.rs");
}

/// Copy the schema file to the output directory so it can be included in the crate
fn copy_schema_file() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("schemas/feature_file.json");
    let dest_dir = dest_path.parent().unwrap();
    fs::create_dir_all(dest_dir)?;

    let source_path = Path::new("../../schemas/feature_file.json");
    if source_path.exists() {
        fs::copy(source_path, &dest_path)
            .unwrap_or_else(|e| panic!("Failed to copy schema file from {:?}: {}", source_path, e));
        println!("cargo:rerun-if-changed={}", source_path.display());
    } else {
        panic!("Schema file not found at {:?}", source_path);
    }

    Ok(())
}

/// Dynamically generate the tests for each folder in tests/test_suites.
fn generate_test_suites() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("test_suites.rs");
    let mut f = fs::File::create(&dest_path).unwrap();

    for entry in fs::read_dir("../../tests/test_suites").unwrap().flatten() {
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
