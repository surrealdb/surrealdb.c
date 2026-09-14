cmake_minimum_required(VERSION 3.21) # First version to support C23

# Keep cargo and CMake on one Apple deployment target.
#
# With neither side pinned, CMake links against the host OS version while the C
# and assembly objects vendored inside Rust dependencies (lz4, sha3, aes,
# aws-lc, jitterentropy) are built against the SDK, and every build emits a wall
# of "built for newer macOS version" linker warnings.
#
# The value is asked of rustc rather than hardcoded, because its default varies
# by target — 11.0 for aarch64-apple-darwin but 10.12 for x86_64-apple-darwin —
# and a fixed number would silently raise the floor on Intel builds. rustc also
# echoes MACOSX_DEPLOYMENT_TARGET when it is already set, so exporting that one
# variable configures both toolchains.
#
# Two traps worth knowing: this must precede project(), since CMake configures
# the toolchain there and ignores a value set afterwards; and CMake predefines
# CMAKE_OSX_DEPLOYMENT_TARGET as empty on Apple, so the test is for emptiness
# rather than NOT DEFINED.
if(APPLE AND NOT CMAKE_OSX_DEPLOYMENT_TARGET)
    set(_sdb_rustc_target_args "")
    if(SURREALDB_CARGO_TARGET)
        list(APPEND _sdb_rustc_target_args --target "${SURREALDB_CARGO_TARGET}")
    elseif(DEFINED ENV{CARGO_BUILD_TARGET} AND NOT "$ENV{CARGO_BUILD_TARGET}" STREQUAL "")
        list(APPEND _sdb_rustc_target_args --target "$ENV{CARGO_BUILD_TARGET}")
    endif()

    execute_process(
        COMMAND rustc --print deployment-target ${_sdb_rustc_target_args}
        OUTPUT_VARIABLE _sdb_rustc_deployment
        RESULT_VARIABLE _sdb_rustc_result
        ERROR_QUIET
        OUTPUT_STRIP_TRAILING_WHITESPACE
    )

    # Output looks like MACOSX_DEPLOYMENT_TARGET=11.0, or IPHONEOS_... on iOS;
    # take whatever follows the '=' either way.
    if(_sdb_rustc_result EQUAL 0 AND _sdb_rustc_deployment MATCHES "=([0-9]+(\\.[0-9]+)*)")
        set(CMAKE_OSX_DEPLOYMENT_TARGET "${CMAKE_MATCH_1}" CACHE STRING
            "Minimum Apple OS version, taken from rustc")
        set(_sdb_deployment_origin "matched to rustc")
    else()
        # rustc missing, or too old for --print deployment-target.
        set(CMAKE_OSX_DEPLOYMENT_TARGET "11.0" CACHE STRING
            "Minimum Apple OS version (fallback; rustc could not be queried)")
        set(_sdb_deployment_origin "fallback; rustc could not be queried")
    endif()
endif()
