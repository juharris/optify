use std::fs;

use optify::provider::{OptionsProvider, OptionsRegistry};

fn test_suite(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let provider = OptionsProvider::build(&path.join("configs"))?;

    let expectations = fs::read_dir(path.join("expectations"))?;
    for expectation_entry in expectations {
        let expectation_path = expectation_entry?.path();
        let expected_json: String = fs::read_to_string(expectation_path.clone())?;
        let expected_info: serde_json::Value = serde_json::from_str(&expected_json)?;
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
                panic!("Error in {expectation_path:?} with key: {key:?}: {e:?}");
            }
            assert_eq!(
                config.unwrap(),
                *expected_value,
                "in {expectation_path:?} with key: {key:?}"
            );
        });
    }

    Ok(())
}

// Include the dynamically generated test functions.
include!(concat!(env!("OUT_DIR"), "/test_suites.rs"));
