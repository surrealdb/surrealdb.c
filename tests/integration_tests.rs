use std::process::Command;
use std::path::Path;
use std::env;

/// Setup function that configures and builds C tests using CMake
/// This ensures Unity is downloaded, Rust library is built, and C tests are compiled
fn setup() {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let test_dir = project_root.join("test");
    let build_dir = test_dir.join("build");
    
    println!("Setting up C test suite with CMake...");
    println!("Project root: {}", project_root.display());
    println!("Test directory: {}", test_dir.display());
    println!("Build directory: {}", build_dir.display());
    
    // Create build directory if it doesn't exist
    std::fs::create_dir_all(&build_dir).expect("Failed to create build directory");
    
    // Run CMake configure
    println!("Running CMake configure...");
    
    // Use default CMake generator for consistency with Unity build
    let configure_status = Command::new("cmake")
        .args(&["-S", ".", "-B", "build"])
        .current_dir(&test_dir)
        .status()
        .expect("Failed to run cmake configure. Is CMake installed?");
    
    assert!(configure_status.success(), "CMake configure failed");
    println!("CMake configure completed successfully");
    
    // Run CMake build
    println!("Running CMake build...");
    let build_status = Command::new("cmake")
        .args(&["--build", "build", "--config", "Debug"])
        .current_dir(&test_dir)
        .status()
        .expect("Failed to run cmake build");
    
    assert!(build_status.success(), "CMake build failed");
    println!("CMake build completed successfully");
}

/// Test that ensures C tests are built via CMake
/// This test compiles the C test suite and verifies the build succeeds
#[test]
fn test_build_c_tests() {
    setup();
    
    // Verify that the test executables were created
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let test_runner = project_root.join("test").join("bin").join(if cfg!(windows) { "test_runner.exe" } else { "test_runner" });
    let test_scratch = project_root.join("test").join("bin").join(if cfg!(windows) { "test_scratch.exe" } else { "test_scratch" });
    
    assert!(test_runner.exists(), "test_runner executable was not created at {}", test_runner.display());
    assert!(test_scratch.exists(), "test_scratch executable was not created at {}", test_scratch.display());
    
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
    let test_status = Command::new("ctest")
        .args(&["--verbose", "--output-on-failure"])
        .current_dir(&build_dir)
        .status()
        .expect("Failed to run ctest");
    
    assert!(test_status.success(), "C tests failed");
    println!("All C tests passed!");
}
