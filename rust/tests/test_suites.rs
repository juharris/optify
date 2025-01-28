use std::fs;

use optify::builder::OptionsProviderBuilder;

fn test_suite(path: &std::path::Path) {
    let provider = OptionsProviderBuilder::new()
        .add_directory(&path.join("configs"))
        .unwrap()
        .build();

    // TODO Get names automatically.
    let features = vec!["feature_A".to_string(), "feature_B/initial".to_string()];
    let expected_json: String = fs::read_to_string(path.join("expected.json")).unwrap();
    let expected_json: serde_json::Value = serde_json::from_str(&expected_json).unwrap();
    let options = expected_json.get("options").unwrap().as_object().unwrap();
    options.keys().for_each(|key| {
        let expected_value = options.get(key).unwrap();
        let config = provider.get_options(key, &features);
        if config.is_err() {
            println!("Error: {:?}", config.err().unwrap());
            return;
        }
        assert_eq!(
            config.unwrap(),
            *expected_value,
            "in {:?} with key: {:?}",
            path,
            key
        );
    });
}

#[test]
fn test_suites() {
    let test_suites_dir = "./tests/test_suites";
    let entries = fs::read_dir(test_suites_dir).unwrap();

    // TODO Split into the equivalent of different `#[test]` functions automatically, if possible.
    entries.for_each(|entry| {
        let entry = entry.unwrap();
        let p = entry.path();
        if p.is_dir() {
            test_suite(p.as_path());
        }
    });
}
