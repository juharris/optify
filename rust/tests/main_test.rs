#[test]
fn test_main() {
    // This test will capture the output of the main function
    let output = std::panic::catch_unwind(|| {
        let result = optify::main();
        assert_eq!(result, 3);
    });

    assert!(output.is_ok());
}
