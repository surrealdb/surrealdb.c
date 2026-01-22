cmake_minimum_required(VERSION 3.21) # First version to support C23
project(surrealdb_c VERSION 0.1.0 LANGUAGES C CXX) # We add CXX here to make some IDEs happy.

# ----------------------------------------------------------------------------
# Project Configuration
# ----------------------------------------------------------------------------

set(CMAKE_C_STANDARD 23)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Make custom find modules available (FindOrBuildCriterion.cmake lives here)
list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake")

# Build options
option(BUILD_TESTING "Build tests (requires Criterion)" ON)
option(SURREALDB_RELEASE "Build Rust library in release mode" OFF)
option(SKIP_RUST_BUILD "Skip building Rust library (use existing)" OFF)

# ----------------------------------------------------------------------------
# Rust profile selection
# ----------------------------------------------------------------------------

if(SURREALDB_RELEASE OR CMAKE_BUILD_TYPE STREQUAL "Release")
    set(RUST_BUILD_FLAG "--release")
    set(RUST_PROFILE "release")
else()
    set(RUST_BUILD_FLAG "")
    set(RUST_PROFILE "debug")
endif()

# Cargo target dir - use default output directory (no --target flag)
# This matches where `cargo build` and `cargo test` place output by default
# Note: cargo puts static libraries in the deps subdirectory
set(RUST_TARGET_DIR "${CMAKE_SOURCE_DIR}/target/${RUST_PROFILE}/deps")

message(STATUS "Rust build profile: ${RUST_PROFILE}")
message(STATUS "Rust target directory: ${RUST_TARGET_DIR}")

# ----------------------------------------------------------------------------
# Rust Library Build Target
# ----------------------------------------------------------------------------

if(WIN32 AND MSVC)
    set(SURREALDB_STATIC_LIB "${RUST_TARGET_DIR}/surrealdb_c.lib")
elseif(WIN32)
    set(SURREALDB_STATIC_LIB "${RUST_TARGET_DIR}/libsurrealdb_c.a")
else()
    set(SURREALDB_STATIC_LIB "${RUST_TARGET_DIR}/libsurrealdb_c.a")
endif()

set(SURREALDB_HEADER "${CMAKE_SOURCE_DIR}/include/surrealdb.h")

if(NOT SKIP_RUST_BUILD)
    add_custom_command(
        OUTPUT ${SURREALDB_STATIC_LIB} ${SURREALDB_HEADER}
        COMMAND cargo build ${RUST_BUILD_FLAG}
        WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
        COMMENT "Building Rust library (${RUST_PROFILE}) and generating C header..."
        VERBATIM
    )

    add_custom_target(rust_lib ALL
        DEPENDS ${SURREALDB_STATIC_LIB} ${SURREALDB_HEADER}
    )
else()
    message(STATUS "Skipping Rust build (SKIP_RUST_BUILD=ON, using existing library)")
    add_custom_target(rust_lib ALL)
endif()

add_library(surrealdb_c STATIC IMPORTED GLOBAL)
set_target_properties(surrealdb_c PROPERTIES
    IMPORTED_LOCATION "${SURREALDB_STATIC_LIB}"
    INTERFACE_INCLUDE_DIRECTORIES "${CMAKE_SOURCE_DIR}/include"
)
add_dependencies(surrealdb_c rust_lib)

# Windows system libraries required by the Rust library
if(WIN32)
    set_target_properties(surrealdb_c PROPERTIES
        INTERFACE_LINK_LIBRARIES "ws2_32;userenv;bcrypt;ntdll;pdh;netapi32;iphlpapi;psapi;propsys;runtimeobject;secur32;powrprof"
    )
endif()

message(STATUS "SurrealDB static library: ${SURREALDB_STATIC_LIB}")
message(STATUS "SurrealDB header: ${SURREALDB_HEADER}")

# ----------------------------------------------------------------------------
# Tests
# ----------------------------------------------------------------------------

if(BUILD_TESTING)
    enable_testing()
    list(PREPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_LIST_DIR}/cmake")
    add_subdirectory(test)
endif()

# ----------------------------------------------------------------------------
# Summary
# ----------------------------------------------------------------------------

message(STATUS "")
message(STATUS "=== SurrealDB C Library Build Configuration ===")
message(STATUS "Build type: ${CMAKE_BUILD_TYPE}")
message(STATUS "Rust profile: ${RUST_PROFILE}")
message(STATUS "Build testing: ${BUILD_TESTING}")
message(STATUS "")
message(STATUS "Build with: cmake --build <build_dir>")
if(BUILD_TESTING)
    message(STATUS "Run tests with: ctest --test-dir <build_dir> --verbose")
endif()
message(STATUS "")
