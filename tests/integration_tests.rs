mod test_helpers;
mod compile_tests;

use test_helpers::setup_criterion;
use compile_tests::compile_test;

/// Setup function that runs before all tests
/// This ensures Criterion is built and C tests are compiled
fn setup() {
    // Build Criterion framework
    setup_criterion();
    
    // Compile all C test files
    compile_test("test").expect("Failed to compile test.c");
    compile_test("test_scratch").expect("Failed to compile test_scratch.c");
    compile_test("doc").expect("Failed to compile doc.c");
}

/// Test that ensures setup runs first
/// This test will always pass, but its purpose is to trigger the setup
#[test]
fn test_setup_c_tests() {
    setup();
    // If we get here, all C tests were compiled successfully
    assert!(true);
}
