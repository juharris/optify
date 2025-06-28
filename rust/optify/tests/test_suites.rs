use std::fs;

use optify::{
    builder::OptionsProviderBuilder, builder::OptionsRegistryBuilder, provider::OptionsRegistry,
};

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
            .map(|v| v.as_str().unwrap())
            .collect::<Vec<&str>>();
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

// Include the dynamically generated test functions.
include!(concat!(env!("OUT_DIR"), "/test_suites.rs"));
