cmake_minimum_required(VERSION 3.21) # First version to support C23

# Must precede project(): CMake configures the Apple toolchain there and
# ignores a deployment target set afterwards.
include(${CMAKE_CURRENT_LIST_DIR}/cmake/AppleDeploymentTarget.cmake)

# Cargo.toml is the single source of truth for the version; build.rs stamps the
# same value into the header. Parsing it here keeps a hand-written copy from
# going stale -- only [package] has a version at the start of a line, as
# dependency versions are all inside braces.
file(STRINGS "${CMAKE_CURRENT_LIST_DIR}/Cargo.toml" SURREALDB_C_VERSION_LINE
    REGEX "^version *= *\"[0-9]+\\.[0-9]+\\.[0-9]+\"" LIMIT_COUNT 1)
string(REGEX MATCH "[0-9]+\\.[0-9]+\\.[0-9]+" SURREALDB_C_VERSION
    "${SURREALDB_C_VERSION_LINE}")
if(NOT SURREALDB_C_VERSION)
    message(FATAL_ERROR "Could not read the package version from Cargo.toml")
endif()

# CXX is enabled despite there being no C++ sources: some IDEs (Visual Studio in
# particular) generate a more useful project when the language is present, and
# consumers of this library are frequently C++.
project(surrealdb_c VERSION ${SURREALDB_C_VERSION} LANGUAGES C CXX)

# ----------------------------------------------------------------------------
# Project Configuration
# ----------------------------------------------------------------------------

set(CMAKE_C_STANDARD 23)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Make custom find modules available
list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake")

# Collect build output in one place. Our own targets set their output
# directories individually; these defaults mostly tidy away vendored
# subprojects such as Unity.
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
# DLLs are runtime for windows.
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)

# IDE project organisation. add_subdirectory_with_folder()
# Generated IDE projects organized by folder.
include(${CMAKE_CURRENT_LIST_DIR}/cmake/helpers.cmake)
set(CMAKE_FOLDER "surrealdb.c")

# Without this clangd cannot resolve unity_fixture.h or any sr_* symbol when
# editing c_test/**, and the whole test tree reads as errors while compiling fine.
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Single-config generators leave this empty, which means no optimisation flags
# and a blank line in the summary below.
if(NOT CMAKE_BUILD_TYPE AND NOT CMAKE_CONFIGURATION_TYPES)
    set(CMAKE_BUILD_TYPE "Debug" CACHE STRING "Build type" FORCE)
    set_property(CACHE CMAKE_BUILD_TYPE PROPERTY STRINGS
                 Debug Release RelWithDebInfo MinSizeRel)
endif()

# Build options
option(BUILD_TESTING "Build tests" ON)
option(SURREALDB_RELEASE "Build Rust library in release mode" OFF)
option(SKIP_RUST_BUILD "Skip building Rust library (use existing)" OFF)
set(SURREALDB_CARGO_TARGET "" CACHE STRING "Cargo target triple to build the Rust library (optional)")

# On by default: with it off, a newly added .rs file does not appear in the
# dependency list until something else forces a reconfigure, so the library can
# go stale. The cost is a directory scan per build.
option(SURREALDB_C_RUST_GLOB_DEPENDS "Use CONFIGURE_DEPENDS for Rust source globs" ON)

# Sanitizers for the test targets. Off by default; see cmake/Sanitizers.cmake
# for why the suite needs them to be able to detect an ownership bug at all.
include(${CMAKE_CURRENT_LIST_DIR}/cmake/Sanitizers.cmake)

# Platform agnostic installation trees.
include(GNUInstallDirs)

include(RustBuild)

# ----------------------------------------------------------------------------
# Installation & Package Config
# ----------------------------------------------------------------------------

# Install the header
install(FILES ${SURREALDB_HEADER}
    DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
)

# Install the static library
if(_sdb_is_multi_config AND NOT _sdb_force_release)
    install(FILES "${_sdb_static_lib_debug}"
        DESTINATION ${CMAKE_INSTALL_LIBDIR}
        CONFIGURATIONS Debug
    )
    install(FILES "${_sdb_static_lib_release}"
        DESTINATION ${CMAKE_INSTALL_LIBDIR}
        CONFIGURATIONS Release RelWithDebInfo MinSizeRel
    )
else()
    install(FILES "${_sdb_static_lib}"
        DESTINATION ${CMAKE_INSTALL_LIBDIR}
    )
endif()

if(_sdb_rust_uses_msvc)
    set(SURREALDB_C_INSTALL_LIBRARY "${CMAKE_INSTALL_LIBDIR}/surrealdb_c.lib")
else()
    set(SURREALDB_C_INSTALL_LIBRARY "${CMAKE_INSTALL_LIBDIR}/libsurrealdb_c.a")
endif()
set(SURREALDB_C_INSTALL_INCLUDEDIR "${CMAKE_INSTALL_INCLUDEDIR}")

# Generate package config file
include(CMakePackageConfigHelpers)
configure_package_config_file(
    "${CMAKE_CURRENT_SOURCE_DIR}/cmake/surrealdb_cConfig.cmake.in"
    "${CMAKE_CURRENT_BINARY_DIR}/surrealdb_cConfig.cmake"
    INSTALL_DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/surrealdb_c
)

write_basic_package_version_file(
    "${CMAKE_CURRENT_BINARY_DIR}/surrealdb_cConfigVersion.cmake"
    VERSION ${PROJECT_VERSION}
    COMPATIBILITY SameMajorVersion
)

install(FILES
    "${CMAKE_CURRENT_BINARY_DIR}/surrealdb_cConfig.cmake"
    "${CMAKE_CURRENT_BINARY_DIR}/surrealdb_cConfigVersion.cmake"
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/surrealdb_c
)

# ----------------------------------------------------------------------------
# Uninstall
# ----------------------------------------------------------------------------

if(PROJECT_IS_TOP_LEVEL AND NOT TARGET uninstall)
    configure_file(
        "${CMAKE_CURRENT_SOURCE_DIR}/cmake/cmake_uninstall.cmake.in"
        "${CMAKE_CURRENT_BINARY_DIR}/cmake_uninstall.cmake"
        IMMEDIATE @ONLY
    )
    add_custom_target(uninstall
        COMMAND ${CMAKE_COMMAND} -P "${CMAKE_CURRENT_BINARY_DIR}/cmake_uninstall.cmake"
        COMMENT "Removing files listed in install_manifest.txt"
    )
endif()

# ----------------------------------------------------------------------------
# Tests
# ----------------------------------------------------------------------------

if(PROJECT_IS_TOP_LEVEL AND BUILD_TESTING)
    enable_testing()
    add_subdirectory(c_test)
elseif(BUILD_TESTING)
    message(STATUS "Skipping surrealdb.c tests when used as subproject")
endif()

# ----------------------------------------------------------------------------
# Summary
# ----------------------------------------------------------------------------

message(STATUS "")
message(STATUS "=== SurrealDB C Library Build Configuration ===")
message(STATUS "Build type: ${CMAKE_BUILD_TYPE}")
message(STATUS "Rust profile: ${RUST_PROFILE}")
message(STATUS "Build testing: ${BUILD_TESTING}")
message(STATUS "Version: ${PROJECT_VERSION}")
message(STATUS "")
message(STATUS "Build with: cmake --build <build_dir>")
if(BUILD_TESTING)
    message(STATUS "Run tests with: ctest --test-dir <build_dir> --verbose")
endif()
message(STATUS "")
