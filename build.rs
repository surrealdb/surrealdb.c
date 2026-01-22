extern crate cbindgen;

use std::env;
use std::path::Path;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // Generate C header bindings
    cbindgen::generate(&crate_dir)
        .expect("Unable to generate bindings")
        .write_to_file("include/surrealdb.h");

    // Compile C test library for integration tests
    let api_tests_path = Path::new(&crate_dir).join("test/src/api_tests/api_tests.c");
    if api_tests_path.exists() {
        cc::Build::new()
            .file(&api_tests_path)
            .include(Path::new(&crate_dir).join("include"))
            .include(Path::new(&crate_dir).join("test/src/api_tests"))
            .compile("api_tests");

        // Tell cargo to rerun if C test files change
        println!("cargo:rerun-if-changed=test/src/api_tests/api_tests.c");
        println!("cargo:rerun-if-changed=test/src/api_tests/api_tests.h");
    }
}
