use std::fs;

use optify::builder::OptionsProviderBuilder;

fn test_suite(path: &std::path::Path) {
    let provider = OptionsProviderBuilder::new()
        .add_directory(&path.join("configs"))
        .unwrap()
        .build();

    // TODO Get names automatically.
    let features = vec!["feature_A".to_string(), "feature_B/initial".to_string()];
    let config = provider.get_options("myConfig", &features);
    println!("{:?}", config);
}

#[test]
fn test_suites() {
    let test_suites_dir = "./tests/test_suites";
    let entries = fs::read_dir(test_suites_dir).unwrap();

    // TODO Split into the equivalent of different `#[test]` functions automatically.
    entries.for_each(|entry| {
        let entry = entry.unwrap();
        let p = entry.path();
        if p.is_dir() {
            test_suite(p.as_path());
        }
    });
}
