use std::process::Command;
use std::fs;
use std::path::{Path, PathBuf};

/// Clone Criterion test framework from GitHub
/// 
/// Clones Criterion v2.4.3 with depth 1 to ThirdParty/Criterion directory
/// Creates the ThirdParty directory if it doesn't exist
/// Always operates relative to the solution directory (CARGO_MANIFEST_DIR)
/// 
/// # Returns
/// 
/// Returns `Ok(())` on success, or an error message on failure
pub fn clone_criterion() -> Result<(), String> {
    // Get the solution directory (where Cargo.toml is located)
    let solution_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let third_party_path = solution_dir.join("ThirdParty");
    let criterion_path = solution_dir.join("ThirdParty").join("Criterion");
    
    // Check if Criterion directory already exists
    if criterion_path.exists() {
        println!("Criterion directory already exists, skipping clone");
        return Ok(());
    }
    
    // Create ThirdParty directory if it doesn't exist
    if !third_party_path.exists() {
        fs::create_dir(&third_party_path)
            .map_err(|e| format!("Failed to create ThirdParty directory: {}", e))?;
    }
    
    let output = Command::new("git")
        .args(&[
            "clone",
            "--depth",
            "1",
            "--branch",
            "v2.4.3",
            "https://github.com/Snaipe/Criterion.git",
            criterion_path.to_str().ok_or("Invalid path encoding")?,
        ])
        .output()
        .map_err(|e| format!("Failed to execute git command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git clone failed: {}", stderr));
    }

    Ok(())
}

/// Build Criterion test framework using Meson
/// 
/// Configures and builds Criterion using Meson build system
/// Creates a build directory at ThirdParty/Criterion/build
/// Builds the library using meson compile
/// Skips building if Criterion library already exists
/// Always operates relative to the solution directory (CARGO_MANIFEST_DIR)
/// 
/// # Returns
/// 
/// Returns `Ok(())` on success, or an error message on failure
pub fn build_criterion() -> Result<(), String> {
    // Get the solution directory (where Cargo.toml is located)
    let solution_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let criterion_path = solution_dir.join("ThirdParty").join("Criterion");
    let build_path = criterion_path.join("build");
    
    // Check if Criterion directory exists
    if !criterion_path.exists() {
        return Err("Criterion directory does not exist. Run clone_criterion() first.".to_string());
    }
    
    // Check if Criterion is already built by looking for the library file
    let criterion_lib = if cfg!(windows) {
        build_path.join("criterion.lib")
    } else if cfg!(target_os = "macos") {
        build_path.join("libcriterion.dylib")
    } else {
        build_path.join("libcriterion.so")
    };
    
    if criterion_lib.exists() {
        println!("Criterion library already exists at {}, skipping build", criterion_lib.display());
        return Ok(());
    }
    
    println!("Criterion library not found, building from source...");
    
    // Create build directory if it doesn't exist
    if !build_path.exists() {
        fs::create_dir(&build_path)
            .map_err(|e| format!("Failed to create build directory: {}", e))?;
    }
    
    // Configure with Meson (setup build directory)
    let setup_output = Command::new("meson")
        .args(&["setup", "build"])
        .current_dir(&criterion_path)
        .output()
        .map_err(|e| format!("Failed to execute meson setup command: {}", e))?;
    
    if !setup_output.status.success() {
        let stderr = String::from_utf8_lossy(&setup_output.stderr);
        let stdout = String::from_utf8_lossy(&setup_output.stdout);
        eprintln!("Meson setup stdout: {}", stdout);
        eprintln!("Meson setup stderr: {}", stderr);
        return Err(format!("Meson setup failed: {}", stderr));
    }
    
    // Build with Meson
    let compile_output = Command::new("meson")
        .args(&["compile", "-C", "build"])
        .current_dir(&criterion_path)
        .output()
        .map_err(|e| format!("Failed to execute meson compile command: {}", e))?;
    
    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr);
        return Err(format!("Meson compile failed: {}", stderr));
    }
    
    println!("Criterion built successfully!");
    Ok(())
}

/// Delete the Criterion folder
/// 
/// Removes the ThirdParty/Criterion directory and all its contents
/// Always operates relative to the solution directory (CARGO_MANIFEST_DIR)
/// 
/// # Returns
/// 
/// Returns `Ok(())` on success, or an error message on failure
pub fn delete_criterion() -> Result<(), String> {
    // Get the solution directory (where Cargo.toml is located)
    let solution_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let criterion_path = solution_dir.join("ThirdParty").join("Criterion");
    
    if !criterion_path.exists() {
        return Ok(()); // Already deleted or doesn't exist
    }

    fs::remove_dir_all(&criterion_path)
        .map_err(|e| format!("Failed to delete Criterion folder: {}", e))?;

    Ok(())
}

/// Setup Criterion framework by cloning and building it
/// 
/// This function clones Criterion if not already present and builds it
/// Should be called before running C tests that depend on Criterion
/// 
/// # Panics
/// 
/// Panics if cloning or building Criterion fails
pub fn setup_criterion() {
    println!("Building Criterion test framework...");
    
    // Clone Criterion if not already present
    match clone_criterion() {
        Ok(_) => println!("Criterion clone check completed"),
        Err(e) => panic!("Failed to clone Criterion: {}", e),
    }
    
    // Build Criterion
    match build_criterion() {
        Ok(_) => println!("Criterion built successfully"),
        Err(e) => panic!("Failed to build Criterion: {}", e),
    }
}
