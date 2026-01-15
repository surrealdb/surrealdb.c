use std::env;
use std::path::PathBuf;

/// Compile a C test file and link it with the surrealdb_c library
/// 
/// # Arguments
/// 
/// * `test_name` - Name of the test to compile (test, test_scratch, or doc)
/// 
/// # Returns
/// 
/// Returns `Ok(())` on success, or an error message on failure
pub fn compile_test(test_name: &str) -> Result<(), String> {
    // Validate test name
    let valid_tests = ["test", "test_scratch", "doc"];
    if !valid_tests.contains(&test_name) {
        return Err(format!("Invalid test name '{}'. Must be one of: test, test_scratch, doc", test_name));
    }
    
    // Get the project root directory
    let project_root = env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    
    // Determine build profile (debug or release)
    let profile = "debug"; // Default to debug for now
    
    // Set up paths
    let source_file = project_root.join("test").join(format!("{}.c", test_name));
    // Use platform-specific executable extension
    let exe_ext = if cfg!(windows) { "exe" } else { "out" };
    let output_file = project_root.join("test").join(format!("{}.{}", test_name, exe_ext));
    let criterion_include = project_root.join("ThirdParty").join("Criterion").join("include");
    let criterion_lib = project_root.join("ThirdParty").join("Criterion").join("build");
    let target_lib = project_root.join("target").join(&profile);
    
    println!("Compiling {} test...", test_name);
    println!("Source: {}", source_file.display());
    println!("Output: {}", output_file.display());
    
    // Set environment variables required by cc crate
    env::set_var("OPT_LEVEL", "0");
    env::set_var("TARGET", env::var("TARGET").unwrap_or_else(|_| {
        // Detect target triple
        if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
            "x86_64-pc-windows-msvc".to_string()
        } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
            "x86_64-unknown-linux-gnu".to_string()
        } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
            "x86_64-apple-darwin".to_string()
        } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
            "aarch64-apple-darwin".to_string()
        } else {
            "unknown".to_string()
        }
    }));
    env::set_var("HOST", env::var("HOST").unwrap_or_else(|_| env::var("TARGET").unwrap()));
    env::set_var("OUT_DIR", project_root.join("test").to_str().unwrap());
    
    // Get the compiler using cc crate's detection
    let compiler = cc::Build::new().get_compiler();
    
    // Check if we're using MSVC
    let is_msvc = compiler.is_like_msvc();
    
    // Set object file extension based on compiler
    let obj_ext = if is_msvc { "obj" } else { "o" };
    let obj_file = project_root.join("test").join(format!("{}.{}", test_name, obj_ext));
    
    // Compile source to object file
    let mut compile_cmd = compiler.to_command();
    if is_msvc {
        compile_cmd
            .arg("/c")
            .arg(&source_file)
            .arg(format!("/Fo{}", obj_file.display()))
            .arg(format!("/I{}", criterion_include.display()))
            .arg(format!("/I{}", project_root.display()));
    } else {
        compile_cmd
            .arg("-c")
            .arg(&source_file)
            .arg("-o")
            .arg(&obj_file)
            .arg(format!("-I{}", criterion_include.display()))
            .arg(format!("-I{}", project_root.display()));
    }
    
    println!("Running: {:?}", compile_cmd);
    let compile_status = compile_cmd.status()
        .map_err(|e| format!("Failed to execute compile command: {}", e))?;
    
    if !compile_status.success() {
        return Err("Compilation failed".to_string());
    }
    
    // Link the object file
    let mut link_cmd = compiler.to_command();
    
    if is_msvc {
        // MSVC linking
        link_cmd
            .arg(&obj_file)
            .arg(format!("/Fe{}", output_file.display()))
            .arg("/link")
            .arg(format!("/LIBPATH:{}", target_lib.display()))
            .arg("surrealdb_c.lib")
            .arg("kernel32.lib")
            .arg("user32.lib")
            .arg("ws2_32.lib")
            .arg("advapi32.lib")
            .arg("userenv.lib")
            .arg("bcrypt.lib")
            .arg("ole32.lib")
            .arg("oleaut32.lib")
            .arg("propsys.lib")
            .arg("powrprof.lib")
            .arg("secur32.lib")
            .arg("netapi32.lib")
            .arg("iphlpapi.lib")
            .arg("runtimeobject.lib")
            .arg("pdh.lib")
            .arg("ntdll.lib")
            .arg("psapi.lib")
            .arg("shell32.lib");
        
        // Add Criterion library if it exists
        let criterion_lib_file = criterion_lib.join("criterion.lib");
        if criterion_lib_file.exists() {
            link_cmd.arg(format!("/LIBPATH:{}", criterion_lib.display()));
            link_cmd.arg("criterion.lib");
        } else {
            println!("Warning: Criterion library not found at {}, linking without it", criterion_lib_file.display());
        }
    } else {
        // GCC/Clang linking
        link_cmd
            .arg(&obj_file)
            .arg("-o")
            .arg(&output_file)
            .arg(format!("-L{}", target_lib.display()))
            .arg("-lsurrealdb_c");
        
        // Add Criterion library if it exists
        let criterion_lib_file = criterion_lib.join("libcriterion.a");
        if criterion_lib_file.exists() {
            link_cmd.arg(format!("-L{}", criterion_lib.display()));
            link_cmd.arg("-lcriterion");
        } else {
            println!("Warning: Criterion library not found at {}, linking without it", criterion_lib_file.display());
        }
        
        // Add pthread on Unix
        if cfg!(unix) {
            link_cmd.arg("-lpthread");
        }
    }
    
    println!("Running: {:?}", link_cmd);
    let link_status = link_cmd.status()
        .map_err(|e| format!("Failed to execute link command: {}", e))?;
    
    if !link_status.success() {
        return Err("Linking failed".to_string());
    }
    
    println!("Successfully compiled {} to {}", source_file.display(), output_file.display());
    Ok(())
}
