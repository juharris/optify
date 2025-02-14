use optify::builder::OptionsProviderBuilder;

#[test]
fn test_builder_circular_imports() {
    let path = std::path::Path::new("tests/circular_imports");
    match OptionsProviderBuilder::new()
        .add_directory(path)
        .unwrap()
        .build()
    {
        Ok(_) => panic!("Expected an error."),
        Err(e) => {
            assert_eq!(e, "TODO");
        }
    }
}
