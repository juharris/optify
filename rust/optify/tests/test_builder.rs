use optify::builder::OptionsProviderBuilder;

#[test]
fn test_builder_circular_imports() {
    let path = std::path::Path::new("tests/circular_imports");
    let mut builder = OptionsProviderBuilder::new();
    builder.add_directory(path).unwrap();
    match builder.build() {
        Ok(_) => panic!("Expected an error."),
        Err(e) => {
            assert_eq!(e, "TODO");
        }
    }
}
