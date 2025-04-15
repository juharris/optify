use std::fs;

use optify::{builder::OptionsProviderBuilder, OptionsProviderBuilderTrait, OptionsProviderTrait};

fn test_suite(path: &std::path::Path) {
    let mut builder = OptionsProviderBuilder::new();
    builder.add_directory(&path.join("configs")).unwrap();
    let provider = builder.build().unwrap();

    let expectations = fs::read_dir(path.join("expectations")).unwrap();
    expectations.for_each(|expectation_entry| {
        let expectation_path = expectation_entry.unwrap().path();
        let expected_json: String = fs::read_to_string(expectation_path.clone()).unwrap();
        let expected_info: serde_json::Value = serde_json::from_str(&expected_json).unwrap();
        let expected_options = expected_info.get("options").unwrap().as_object().unwrap();
        let features = expected_info
            .get("features")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        expected_options.iter().for_each(|(key, expected_value)| {
            let config = provider.get_options(key, &features);
            if let Err(e) = &config {
                panic!(
                    "Error in {:?} with key: {:?}: {:?}",
                    expectation_path, key, e
                );
            }
            assert_eq!(
                config.unwrap(),
                *expected_value,
                "in {:?} with key: {:?}",
                expectation_path,
                key
            );
        });
    });
}

#[test]
fn test_suites() {
    let test_suites_dir = "../../tests/test_suites";
    let entries = fs::read_dir(test_suites_dir).unwrap();

    // TODO Split into the equivalent of different `#[test]` functions automatically, if possible.
    entries.for_each(|entry| {
        let p = entry.unwrap().path();
        if p.is_dir() {
            test_suite(p.as_path());
        }
    });
}
