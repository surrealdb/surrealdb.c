cmake_minimum_required(VERSION 3.21) # First version to support C23
project(surrealdb_c VERSION 0.1.0 LANGUAGES C CXX) # We add CXX here to make some IDEs happy.

# ----------------------------------------------------------------------------
# Project Configuration
# ----------------------------------------------------------------------------

set(CMAKE_C_STANDARD 23)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Make custom find modules available
list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake")

# Put binary files on the top level of the build tree for convenience.
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
# DLLs are runtime for windows.
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)

include(${CMAKE_CURRENT_LIST_DIR}/cmake/helpers.cmake)

set(CMAKE_INCLUDE_CURRENT_DIR ON)
set(CMAKE_FOLDER "surrealdb.c")

# Build options
option(BUILD_TESTING "Build tests" ON)
option(SURREALDB_RELEASE "Build Rust library in release mode" OFF)
option(SKIP_RUST_BUILD "Skip building Rust library (use existing)" OFF)
set(SURREALDB_CARGO_TARGET "" CACHE STRING "Cargo target triple to build the Rust library (optional)")
option(SURREALDB_C_RUST_GLOB_DEPENDS "Use CONFIGURE_DEPENDS for Rust source globs" OFF)

# Platform agnostic installation trees.
include(GNUInstallDirs)

# ----------------------------------------------------------------------------
# Rust target selection
# ----------------------------------------------------------------------------

set(_sdb_rust_target "")
if(DEFINED SURREALDB_CARGO_TARGET AND NOT SURREALDB_CARGO_TARGET STREQUAL "")
    set(_sdb_rust_target "${SURREALDB_CARGO_TARGET}")
elseif(DEFINED ENV{CARGO_BUILD_TARGET} AND NOT "$ENV{CARGO_BUILD_TARGET}" STREQUAL "")
    set(_sdb_rust_target "$ENV{CARGO_BUILD_TARGET}")
elseif(MINGW)
    if(CMAKE_SIZEOF_VOID_P EQUAL 8)
        set(_sdb_rust_target "x86_64-pc-windows-gnu")
    else()
        set(_sdb_rust_target "i686-pc-windows-gnu")
    endif()
endif()

set(_sdb_cargo_target_args "")
set(_sdb_rust_target_root "${PROJECT_SOURCE_DIR}/target")
if(_sdb_rust_target)
    list(APPEND _sdb_cargo_target_args "--target" "${_sdb_rust_target}")
    set(_sdb_rust_target_root "${_sdb_rust_target_root}/${_sdb_rust_target}")
endif()

set(_sdb_rust_uses_msvc FALSE)
if(_sdb_rust_target)
    if(_sdb_rust_target MATCHES "-msvc$")
        set(_sdb_rust_uses_msvc TRUE)
    endif()
elseif(MSVC)
    set(_sdb_rust_uses_msvc TRUE)
endif()

if(MSVC AND _sdb_rust_target AND NOT _sdb_rust_uses_msvc)
    message(WARNING "Cargo target '${_sdb_rust_target}' builds GNU .a libraries; MSVC cannot link them.")
elseif(MINGW AND _sdb_rust_target AND _sdb_rust_uses_msvc)
    message(WARNING "Cargo target '${_sdb_rust_target}' builds MSVC .lib libraries; MinGW cannot link them.")
endif()

# ----------------------------------------------------------------------------
# Rust profile selection
# ----------------------------------------------------------------------------

set(_sdb_release_configs Release RelWithDebInfo MinSizeRel)

set(_sdb_is_multi_config FALSE)
if(CMAKE_CONFIGURATION_TYPES)
    set(_sdb_is_multi_config TRUE)
endif()

set(_sdb_force_release FALSE)
if(SURREALDB_RELEASE)
    set(_sdb_force_release TRUE)
endif()

# Cargo target dir - use default output directory (no --target flag).
# Note: cargo puts static libraries in the deps subdirectory.
set(_sdb_rust_target_dir_debug "${_sdb_rust_target_root}/debug/deps")
set(_sdb_rust_target_dir_release "${_sdb_rust_target_root}/release/deps")

if(_sdb_rust_uses_msvc)
    set(_sdb_rust_lib_name "surrealdb_c.lib")
else()
    set(_sdb_rust_lib_name "libsurrealdb_c.a")
endif()

set(_sdb_static_lib_debug "${_sdb_rust_target_dir_debug}/${_sdb_rust_lib_name}")
set(_sdb_static_lib_release "${_sdb_rust_target_dir_release}/${_sdb_rust_lib_name}")

set(SURREALDB_HEADER "${PROJECT_SOURCE_DIR}/include/surrealdb.h")

set(_sdb_rust_deps
    "${PROJECT_SOURCE_DIR}/Cargo.toml"
)
if(EXISTS "${PROJECT_SOURCE_DIR}/cbindgen.toml")
    list(APPEND _sdb_rust_deps "${PROJECT_SOURCE_DIR}/cbindgen.toml")
endif()
if(EXISTS "${PROJECT_SOURCE_DIR}/Cargo.lock")
    list(APPEND _sdb_rust_deps "${PROJECT_SOURCE_DIR}/Cargo.lock")
endif()
if(EXISTS "${PROJECT_SOURCE_DIR}/build.rs")
    list(APPEND _sdb_rust_deps "${PROJECT_SOURCE_DIR}/build.rs")
endif()
if(SURREALDB_C_RUST_GLOB_DEPENDS)
    message(STATUS "Rust source glob recheck: ON")
    file(GLOB_RECURSE _sdb_rust_sources CONFIGURE_DEPENDS
        "${PROJECT_SOURCE_DIR}/src/*.rs"
    )
else()
    message(STATUS "Rust source glob recheck: OFF")
    file(GLOB_RECURSE _sdb_rust_sources
        "${PROJECT_SOURCE_DIR}/src/*.rs"
    )
endif()
list(APPEND _sdb_rust_deps ${_sdb_rust_sources})

if(_sdb_force_release)
    set(RUST_PROFILE "release (forced)")
elseif(_sdb_is_multi_config)
    set(RUST_PROFILE "config-based (Debug=debug, Release/RelWithDebInfo/MinSizeRel=release)")
elseif(CMAKE_BUILD_TYPE IN_LIST _sdb_release_configs)
    set(RUST_PROFILE "release")
else()
    set(RUST_PROFILE "debug")
endif()

set(_sdb_static_lib "${_sdb_static_lib_debug}")
if(_sdb_force_release OR CMAKE_BUILD_TYPE IN_LIST _sdb_release_configs)
    set(_sdb_static_lib "${_sdb_static_lib_release}")
endif()

message(STATUS "Rust build profile: ${RUST_PROFILE}")
if(_sdb_rust_target)
    message(STATUS "Cargo target: ${_sdb_rust_target}")
else()
    message(STATUS "Cargo target: default")
endif()
message(STATUS "Rust target directory (debug): ${_sdb_rust_target_dir_debug}")
message(STATUS "Rust target directory (release): ${_sdb_rust_target_dir_release}")

# ----------------------------------------------------------------------------
# Rust Library Build Target
# ----------------------------------------------------------------------------

if(NOT SKIP_RUST_BUILD)
    if(_sdb_is_multi_config AND NOT _sdb_force_release)
        add_custom_command(
            OUTPUT ${_sdb_static_lib_debug}
            COMMAND cargo build ${_sdb_cargo_target_args}
            WORKING_DIRECTORY ${PROJECT_SOURCE_DIR}
            COMMENT "Building Rust library (debug) and generating C header..."
            DEPENDS ${_sdb_rust_deps}
            BYPRODUCTS ${SURREALDB_HEADER}
            VERBATIM
        )
        add_custom_command(
            OUTPUT ${_sdb_static_lib_release}
            COMMAND cargo build --release ${_sdb_cargo_target_args}
            WORKING_DIRECTORY ${PROJECT_SOURCE_DIR}
            COMMENT "Building Rust library (release) and generating C header..."
            DEPENDS ${_sdb_rust_deps}
            BYPRODUCTS ${SURREALDB_HEADER}
            VERBATIM
        )

        add_custom_target(rust_lib ALL
            DEPENDS
                $<$<CONFIG:Debug>:${_sdb_static_lib_debug}>
                $<$<OR:$<CONFIG:Release>,$<CONFIG:RelWithDebInfo>,$<CONFIG:MinSizeRel>>:${_sdb_static_lib_release}>
        )
    else()
        set(_sdb_cargo_flags "")
        if(_sdb_force_release OR CMAKE_BUILD_TYPE IN_LIST _sdb_release_configs)
            set(_sdb_cargo_flags "--release")
        endif()

        add_custom_command(
            OUTPUT ${_sdb_static_lib}
            COMMAND cargo build ${_sdb_cargo_flags} ${_sdb_cargo_target_args}
            WORKING_DIRECTORY ${PROJECT_SOURCE_DIR}
            COMMENT "Building Rust library (${RUST_PROFILE}) and generating C header..."
            DEPENDS ${_sdb_rust_deps}
            BYPRODUCTS ${SURREALDB_HEADER}
            VERBATIM
        )

        add_custom_target(rust_lib ALL
            DEPENDS ${_sdb_static_lib}
        )
    endif()
else()
    message(STATUS "Skipping Rust build (SKIP_RUST_BUILD=ON, using existing library)")
    add_custom_target(rust_lib ALL)
endif()

add_custom_target(surrealdb.c SOURCES ${SURREALDB_HEADER})
add_dependencies(surrealdb.c rust_lib)

add_library(surrealdb_c STATIC IMPORTED GLOBAL)
set_target_properties(surrealdb_c PROPERTIES
    INTERFACE_INCLUDE_DIRECTORIES "${PROJECT_SOURCE_DIR}/include"
)
if(_sdb_is_multi_config AND NOT _sdb_force_release)
    set_target_properties(surrealdb_c PROPERTIES
        IMPORTED_LOCATION_DEBUG "${_sdb_static_lib_debug}"
        IMPORTED_LOCATION_RELEASE "${_sdb_static_lib_release}"
        IMPORTED_LOCATION_RELWITHDEBINFO "${_sdb_static_lib_release}"
        IMPORTED_LOCATION_MINSIZEREL "${_sdb_static_lib_release}"
    )
else()
    set(SURREALDB_STATIC_LIB "${_sdb_static_lib}")
    set_target_properties(surrealdb_c PROPERTIES
        IMPORTED_LOCATION "${_sdb_static_lib}"
    )
endif()
add_dependencies(surrealdb_c rust_lib)

# Create namespaced alias for use by subdirectories (matches installed target name)
# This allows examples and tests to use surrealdb::surrealdb_c just like external consumers
add_library(surrealdb::surrealdb_c ALIAS surrealdb_c)

# Platform-specific system libraries required by the Rust library
if(WIN32)
    set_target_properties(surrealdb_c PROPERTIES
        INTERFACE_LINK_LIBRARIES "ws2_32;userenv;bcrypt;ntdll;pdh;netapi32;iphlpapi;psapi;propsys;runtimeobject;secur32;powrprof"
    )
elseif(APPLE)
    set_target_properties(surrealdb_c PROPERTIES
        INTERFACE_LINK_LIBRARIES "-framework Security;-framework SystemConfiguration;-framework CoreFoundation;-framework IOKit;-lobjc"
    )
endif()

if(_sdb_is_multi_config AND NOT _sdb_force_release)
    message(STATUS "SurrealDB static library (debug): ${_sdb_static_lib_debug}")
    message(STATUS "SurrealDB static library (release): ${_sdb_static_lib_release}")
else()
    message(STATUS "SurrealDB static library: ${_sdb_static_lib}")
endif()
message(STATUS "SurrealDB header: ${SURREALDB_HEADER}")

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
# Tests
# ----------------------------------------------------------------------------

if(PROJECT_IS_TOP_LEVEL AND BUILD_TESTING)
    enable_testing()
    list(PREPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_LIST_DIR}/cmake")
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
message(STATUS "")
message(STATUS "Build with: cmake --build <build_dir>")
if(BUILD_TESTING)
    message(STATUS "Run tests with: ctest --test-dir <build_dir> --verbose")
endif()
message(STATUS "")
