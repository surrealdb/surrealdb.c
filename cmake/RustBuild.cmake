# ----------------------------------------------------------------------------
# Building the Rust library, and exposing it as a CMake target
# ----------------------------------------------------------------------------
#
# Extracted from the top-level CMakeLists so that it reads as a project
# definition rather than as a driver for someone else's toolchain.
#
# Inputs (options declared by the caller):
#   SURREALDB_RELEASE, SKIP_RUST_BUILD, SURREALDB_CARGO_TARGET,
#   SURREALDB_C_RUST_GLOB_DEPENDS
#
# Outputs:
#   surrealdb_c              imported static library
#   surrealdb::surrealdb_c   alias, matching the installed target name
#   SURREALDB_HEADER         path to the generated header
#   SURREALDB_C_SYSTEM_LIBS  platform system libraries, also baked into the
#                            installed package config
#   RUST_PROFILE, _sdb_static_lib*, _sdb_is_multi_config, _sdb_force_release,
#   _sdb_rust_uses_msvc      consumed by the install rules
# ----------------------------------------------------------------------------

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

# Environment handed to every cargo invocation below.
set(_sdb_cargo_env "")
if(APPLE AND CMAKE_OSX_DEPLOYMENT_TARGET)
    if(NOT _sdb_deployment_origin)
        set(_sdb_deployment_origin "set by the caller")
    endif()
    message(STATUS "Apple deployment target: ${CMAKE_OSX_DEPLOYMENT_TARGET} (${_sdb_deployment_origin})")
    list(APPEND _sdb_cargo_env "MACOSX_DEPLOYMENT_TARGET=${CMAKE_OSX_DEPLOYMENT_TARGET}")
endif()

if(NOT SKIP_RUST_BUILD)
    if(_sdb_is_multi_config AND NOT _sdb_force_release)
        add_custom_command(
            OUTPUT ${_sdb_static_lib_debug}
            COMMAND ${CMAKE_COMMAND} -E env
                    ${_sdb_cargo_env}
                    cargo build ${_sdb_cargo_target_args}
            WORKING_DIRECTORY ${PROJECT_SOURCE_DIR}
            COMMENT "Building Rust library (debug) and generating C header..."
            DEPENDS ${_sdb_rust_deps}
            BYPRODUCTS ${SURREALDB_HEADER}
            VERBATIM
        )
        add_custom_command(
            OUTPUT ${_sdb_static_lib_release}
            COMMAND ${CMAKE_COMMAND} -E env
                    ${_sdb_cargo_env}
                    cargo build --release ${_sdb_cargo_target_args}
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
            COMMAND ${CMAKE_COMMAND} -E env
                    ${_sdb_cargo_env}
                    cargo build ${_sdb_cargo_flags} ${_sdb_cargo_target_args}
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

# Platform-specific system libraries required by the Rust library.
#
# Defined once and consumed both by the in-tree target below and by
# surrealdb_cConfig.cmake.in, which is configured with this variable. Keeping
# two copies previously left the installed package under-linked on every
# platform except Windows.
if(WIN32)
    set(SURREALDB_C_SYSTEM_LIBS "ws2_32;userenv;bcrypt;ntdll;pdh;netapi32;iphlpapi;psapi;propsys;runtimeobject;secur32;powrprof")
elseif(APPLE)
    set(SURREALDB_C_SYSTEM_LIBS "-framework Security;-framework SystemConfiguration;-framework CoreFoundation;-framework IOKit;-lobjc")
elseif(UNIX)
    set(SURREALDB_C_SYSTEM_LIBS "m;dl;pthread;rt")
else()
    set(SURREALDB_C_SYSTEM_LIBS "")
endif()

set_target_properties(surrealdb_c PROPERTIES
    INTERFACE_LINK_LIBRARIES "${SURREALDB_C_SYSTEM_LIBS}"
)

if(_sdb_is_multi_config AND NOT _sdb_force_release)
    message(STATUS "SurrealDB static library (debug): ${_sdb_static_lib_debug}")
    message(STATUS "SurrealDB static library (release): ${_sdb_static_lib_release}")
else()
    message(STATUS "SurrealDB static library: ${_sdb_static_lib}")
endif()
message(STATUS "SurrealDB header: ${SURREALDB_HEADER}")
