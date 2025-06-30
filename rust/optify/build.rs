use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    generate_test_suites();
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
                                "#[test]\nfn test_suite_{}() {{\n    test_suite(std::path::Path::new(\"../../tests/test_suites/{}\")).unwrap();\n}}\n", 
                                suite_name, suite_name
                            ).unwrap();
                }
            }
        }
    }

    println!("cargo:rerun-if-changed=../../tests/test_suites");
    println!("cargo:rerun-if-changed=build.rs");
}
