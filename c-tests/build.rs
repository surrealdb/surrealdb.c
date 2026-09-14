//! Compiles Unity and the C test corpus into this package's test binary.
//!
//! This lives here, and not in the library's build script, so that building
//! `surrealdb_c` never requires a C compiler and no test symbols end up in the
//! shipped static library.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let root = manifest.parent().expect("workspace root").to_path_buf();

    let unity = find_unity(&root).unwrap_or_else(|| {
        panic!(
            "Unity not found.\n\
             Set UNITY_DIR, or run `cmake -S . -B build` once in {} to fetch it.",
            root.display()
        )
    });

    let tests = root.join("c_test/src/tests");
    let api_tests = root.join("c_test/src/api_tests");
    let bin = root.join("c_test/src/bin");

    let mut build = cc::Build::new();
    build
        .include(root.join("include"))
        .include(unity.join("src"))
        .include(unity.join("extras/fixture/src"))
        // unity_fixture.h includes unity_memory.h; the fixture extension does
        // not stand alone.
        .include(unity.join("extras/memory/src"))
        .include(&tests)
        .include(&api_tests)
        // main() stays out: the Rust test harness brings its own entry point.
        .file(bin.join("runner.c"))
        .file(api_tests.join("api_tests.c"))
        .file(unity.join("src/unity.c"))
        .file(unity.join("extras/fixture/src/unity_fixture.c"))
        .file(unity.join("extras/memory/src/unity_memory.c"));

    // Discovered rather than listed, so adding a test file to the corpus does
    // not require editing this driver as well.
    for source in c_sources(&tests) {
        build.file(source);
    }

    build.warnings(false).compile("surrealdb_c_test_corpus");

    // The examples are complete programs with their own `main`. Only CMake
    // built them before, so `cargo test` -- the path the README now documents
    // -- could go green while an example no longer compiled or no longer
    // worked. Each is compiled here with `main` renamed to `<stem>_main` and
    // driven by a generated #[test], so they are held to the same standard as
    // the corpus. Both are self-contained against `mem://`.
    let examples = root.join("c_test/examples");
    let mut example_stems: Vec<String> = Vec::new();
    for source in c_sources(&examples) {
        let stem = source
            .file_stem()
            .expect("example file stem")
            .to_string_lossy()
            .to_string();
        cc::Build::new()
            .include(root.join("include"))
            .define("main", format!("{stem}_main").as_str())
            .file(&source)
            .warnings(false)
            .compile(&format!("surrealdb_c_example_{stem}"));
        example_stems.push(stem);
    }

    let generated_examples: String = example_stems
        .iter()
        .map(|stem| {
            format!(
                "extern \"C\" {{ fn {stem}_main() -> ::std::os::raw::c_int; }}\n\
                 #[test]\n\
                 fn example_{stem}() {{\n\
                 \x20   let rc = unsafe {{ {stem}_main() }};\n\
                 \x20   assert_eq!(rc, 0, \"example {stem} exited with {{rc}}\");\n\
                 }}\n"
            )
        })
        .collect();
    fs::write(
        PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR")).join("examples.rs"),
        generated_examples,
    )
    .expect("write examples.rs");

    // Emit one #[test] per Unity group, read out of the corpus itself. CMake
    // derives its CTest entries the same way and runner.c is generated from it,
    // so no group list is maintained by hand anywhere.
    let mut groups: Vec<String> = Vec::new();
    for source in c_sources(&tests) {
        let text = fs::read_to_string(&source).unwrap_or_default();
        for line in text.lines() {
            if let Some(rest) = line.trim().strip_prefix("TEST_GROUP_RUNNER(") {
                if let Some(name) = rest.split(')').next() {
                    groups.push(name.to_string());
                }
            }
        }
    }
    groups.sort();
    groups.dedup();

    // Unity filters groups with strstr(), so one name containing another would
    // silently run both.
    for a in &groups {
        for b in &groups {
            assert!(a == b || !b.contains(a.as_str()), "group {a} is a substring of {b}");
        }
    }

    let generated: String = groups
        .iter()
        .map(|g| format!("test_group!({}, \"{}\");\n", g.to_lowercase(), g))
        .collect();
    fs::write(
        PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR")).join("groups.rs"),
        generated,
    )
    .expect("write groups.rs");

    println!("cargo:rerun-if-env-changed=UNITY_DIR");
    println!("cargo:rerun-if-changed={}", tests.display());
    println!("cargo:rerun-if-changed={}", examples.display());
    println!("cargo:rerun-if-changed={}", api_tests.display());
    println!("cargo:rerun-if-changed={}", bin.join("runner.c").display());
    println!(
        "cargo:rerun-if-changed={}",
        root.join("include/surrealdb.h").display()
    );
}

/// Every `.c` file in a directory, sorted for reproducible builds.
fn c_sources(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "c"))
        .collect();
    files.sort();
    files
}

/// Locate a Unity checkout.
///
/// Prefers an explicit `UNITY_DIR`, then the copy CMake fetches into the
/// project tree, then the sibling checkout carried by surrealdb.cpp.
fn find_unity(root: &Path) -> Option<PathBuf> {
    env::var_os("UNITY_DIR")
        .map(PathBuf::from)
        .into_iter()
        .chain([
            root.join("ThirdParty/Unity"),
            root.join("../ThirdParty/Unity"),
        ])
        .find(|dir| dir.join("src/unity.c").is_file())
}
