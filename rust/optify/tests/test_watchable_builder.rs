use optify::builder::{OptionsRegistryBuilder, OptionsWatcherBuilder};
use optify::provider::OptionsRegistry;
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::Duration;

const SLEEP_TIME: u64 = 50;

#[test]
fn test_watchable_builder_modify_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let test_dir = temp_dir.path();

    let options_file = test_dir.join("modifiable_test.json");
    let mut file = File::create(&options_file)?;
    file.write_all(b"{\"options\":{\"test\":42}}")?;

    let mut builder = OptionsWatcherBuilder::new();
    builder.add_directory(test_dir)?;
    let provider = builder.build()?;
    let created_at = provider.last_modified();

    let options = provider.get_options("test", &["modifiable_test"])?;
    assert_eq!(options.as_i64(), Some(42));

    let mut file = File::create(&options_file)?;
    file.write_all(b"{\"options\":{\"test\":43}}")?;

    let start_time = std::time::Instant::now();
    let max_sleep_time = 3000;
    while provider.last_modified() == created_at {
        thread::sleep(Duration::from_millis(SLEEP_TIME));
        if start_time.elapsed().as_millis() > max_sleep_time {
            panic!("Provider did not update after {}ms.", max_sleep_time);
        }
    }

    let options = provider.get_options("test", &["modifiable_test"])?;
    assert_eq!(options.as_i64(), Some(43));

    Ok(())
}

#[test]
fn test_watchable_builder_multiple_directories() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir1 = tempfile::tempdir()?;
    let subdir1 = temp_dir1.path().join("dir1");
    std::fs::create_dir_all(&subdir1)?;

    let file1 = subdir1.join("test1.json");
    File::create(&file1)?.write_all(b"{\"options\": {\"test1\": 1}}")?;

    let temp_dir2 = tempfile::tempdir()?;
    let subdir2 = temp_dir2.path().join("dir2");
    std::fs::create_dir_all(&subdir2)?;

    let mut builder = OptionsWatcherBuilder::new();
    builder.add_directory(subdir1.as_path())?;
    builder.add_directory(subdir2.as_path())?;
    let provider = builder.build()?;
    let created_at = provider.last_modified();

    let options1 = provider.get_options("test1", &["test1"])?;
    assert_eq!(options1.as_i64(), Some(1));

    assert_eq!(provider.last_modified(), created_at);

    let options2 = provider.get_options("test2", &["test2"]);
    assert!(options2.is_err());

    assert_eq!(provider.last_modified(), created_at);

    let file2 = subdir2.join("test2.json");
    File::create(&file2)?.write_all(b"{\"options\": {\"test2\": 2}}")?;

    let start_time = std::time::Instant::now();
    let max_sleep_time = 3000;
    while provider.last_modified() == created_at {
        thread::sleep(Duration::from_millis(SLEEP_TIME));
        if start_time.elapsed().as_millis() > max_sleep_time {
            panic!("Provider did not update after {}ms.", max_sleep_time);
        }
    }

    assert!(provider.last_modified() > created_at);
    let last_modified = provider.last_modified();

    let options2 = provider.get_options("test2", &["test2"])?;
    assert_eq!(options2.as_i64(), Some(2));

    // Ensure that using the first file still works.
    let options1 = provider.get_options("test1", &["test1"])?;
    assert_eq!(options1.as_i64(), Some(1));

    // Remove the first file.
    std::fs::remove_file(&file1)?;

    // Some operating systems need more time to actually remove the file.
    let start_time = std::time::Instant::now();
    while provider.last_modified() == last_modified {
        thread::sleep(Duration::from_millis(SLEEP_TIME));
        if start_time.elapsed().as_millis() > max_sleep_time {
            panic!(
                "File {} still exists after {}ms.",
                file1.display(),
                max_sleep_time
            );
        }
    }

    assert!(provider.last_modified() > last_modified);

    let options1 = provider.get_options("test1", &["test1"]);
    assert!(
        options1.is_err(),
        "There should be an error because the file was removed."
    );
    assert_eq!(
        options1.err().unwrap(),
        "The given feature \"test1\" was not found."
    );

    Ok(())
}

#[test]
fn test_watchable_builder_error_rebuilding_provider() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let test_dir = temp_dir.path();

    let options_file = test_dir.join("error_rebuilding_test.json");
    let mut file = File::create(&options_file)?;
    file.write_all(b"{\"metadata\":{\"aliases\":[\"test\"]}, \"options\":{\"test\":42}}")?;

    let mut builder = OptionsWatcherBuilder::new();
    builder.add_directory(test_dir)?;
    let provider = builder.build()?;

    let options = provider.get_options("test", &["test"])?;
    let expected_value = Some(42);
    assert_eq!(options.as_i64(), expected_value);

    // Use invalid JSON.
    file.write_all(b"append this to make the JSON invalid")?;
    thread::sleep(Duration::from_millis(SLEEP_TIME));

    let options = provider.get_options("test", &["test"])?;
    assert_eq!(options.as_i64(), expected_value, "Expected the same value as before because we'll give the developer a change to fix the file.");

    let last_modified = provider.last_modified();

    // Rewrite the file.
    let mut file = File::create(&options_file)?;
    file.write_all(b"{\"metadata\":{\"aliases\":[\"test\"]}, \"options\":{\"test\":43}}")?;

    let start_time = std::time::Instant::now();
    let max_sleep_time = 3000;
    while provider.last_modified() == last_modified {
        thread::sleep(Duration::from_millis(SLEEP_TIME));
        if start_time.elapsed().as_millis() > max_sleep_time {
            panic!("Provider did not update after {}ms.", max_sleep_time);
        }
    }

    let options = provider.get_options("test", &["test"])?;
    assert_eq!(options.as_i64(), Some(43));

    Ok(())
}
