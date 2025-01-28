use std::fs;

use config::{File, Value};
use optify::builder::OptionsProviderBuilder;

fn test_suite(path: &std::path::Path) {
    let provider = OptionsProviderBuilder::new()
        .add_directory(&path.join("configs"))
        .unwrap()
        .build();

    // TODO Get names automatically.
    let features = vec!["feature_A".to_string(), "feature_B/initial".to_string()];
    let config = provider.get_options("myConfig", &features);
    if config.is_err() {
        println!("Error: {:?}", config.err().unwrap());
        return;
    }
    println!("{:?}", config);
}

#[test]
fn test_suites() {
    let test_suites_dir = "./tests/test_suites";
    let entries = fs::read_dir(test_suites_dir).unwrap();

    println!("Running test suites in {}", test_suites_dir);

    // TODO Split into the equivalent of different `#[test]` functions automatically.
    entries.for_each(|entry| {
        let entry = entry.unwrap();
        let p = entry.path();
        if p.is_dir() {
            test_suite(p.as_path());
        }
    });
}

#[test]
fn example() {
    let conf = config::Config::builder()
        .add_source(File::with_name(
            "./tests/test_suites/simple/configs/feature_A.json",
        ))
        .add_source(File::with_name(
            "./tests/test_suites/simple/configs/feature_B/initial.yaml",
        ))
        .build();
    let conf = conf.unwrap();
    let my_config: serde_json::Value = conf.get("options").unwrap();
    let expected_json: serde_json::Value = serde_json::from_reader(
        fs::File::open("./tests/test_suites/simple/expected.json").unwrap(),
    )
    .unwrap();
    assert_eq!(my_config, expected_json["options"]);
}
