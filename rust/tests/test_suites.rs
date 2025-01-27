use std::fs;

fn test_suite(path: &std::path::Path) {
    
}

#[test]
fn test_suites() {
    let test_suites_dir = "./test_suites";
    let entries = fs::read_dir(test_suites_dir).unwrap();

    // TODO Parallelize this.
    entries.for_each(|entry| {
        let entry = entry.unwrap();
        if entry.path().is_dir() {
            test_suite(entry.path().as_path());
        }
    });
}
