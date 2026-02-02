use std::env;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Once;

static SETUP_ONCE: Once = Once::new();
static mut SETUP_SUCCESS: bool = false;

/// Setup function that configures and builds C tests using CMake
/// This ensures Unity is downloaded, Rust library is built, and C tests are compiled
/// Uses Once to ensure setup only runs once even when tests run in parallel
fn setup() {
    SETUP_ONCE.call_once(|| {
        let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let build_dir = project_root.join("test").join("build");

        println!("Setting up C test suite with CMake...");
        println!("Project root: {}", project_root.display());
        println!("Build directory: {}", build_dir.display());

        // Create build directory if it doesn't exist
        std::fs::create_dir_all(&build_dir).expect("Failed to create build directory");

        // Run CMake configure from project root (test/CMakeLists.txt is included via
        // add_subdirectory)
        println!("Running CMake configure...");

        // Use SKIP_RUST_BUILD since cargo test already built the Rust library
        let configure_status = Command::new("cmake")
            .args(&["-S", ".", "-B", "test/build", "-DSKIP_RUST_BUILD=ON"])
            .current_dir(&project_root)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .expect("Failed to run cmake configure. Is CMake installed?");

        if !configure_status.success() {
            eprintln!("CMake configure failed");
            return;
        }
        println!("CMake configure completed successfully");

        // Run CMake build
        println!("Running CMake build...");
        let build_status = Command::new("cmake")
            .args(&["--build", "test/build", "--config", "Debug"])
            .current_dir(&project_root)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .expect("Failed to run cmake build");

        if !build_status.success() {
            eprintln!("CMake build failed");
            return;
        }
        println!("CMake build completed successfully");

        // Mark setup as successful
        unsafe {
            SETUP_SUCCESS = true;
        }
    });

    // Check if setup was successful
    assert!(unsafe { SETUP_SUCCESS }, "CMake setup failed");
}

/// Test that ensures C tests are built via CMake
/// This test compiles the C test suite and verifies the build succeeds
#[test]
fn test_build_c_tests() {
    setup();

    // Verify that the test executables were created
    // When building from project root with -B test/build, outputs go to test/build/test/bin
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let bin_dir = project_root.join("test").join("build").join("test").join("bin");
    let test_runner = bin_dir.join(if cfg!(windows) {
        "Debug/test_runner.exe"
    } else {
        "test_runner"
    });
    let test_scratch = bin_dir.join(if cfg!(windows) {
        "Debug/test_scratch.exe"
    } else {
        "test_scratch"
    });

    assert!(
        test_runner.exists(),
        "test_runner executable was not created at {}",
        test_runner.display()
    );
    assert!(
        test_scratch.exists(),
        "test_scratch executable was not created at {}",
        test_scratch.display()
    );

    println!("All C test executables built successfully!");
}

/// Test that runs the C test suite via CTest
/// This executes the actual C tests and verifies they pass
#[test]
fn test_run_c_tests() {
    setup();

    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let build_dir = project_root.join("test").join("build");

    println!("Running C test suite via CTest...");

    // On Windows with multi-config generators (MSVC), we need to specify the configuration
    let mut args = vec!["--verbose", "--output-on-failure"];
    if cfg!(windows) {
        args.extend(&["-C", "Debug"]);
    }

    let test_status = Command::new("ctest")
        .args(&args)
        .current_dir(&build_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Failed to run ctest");

    assert!(test_status.success(), "C tests failed");
    println!("All C tests passed!");
}
