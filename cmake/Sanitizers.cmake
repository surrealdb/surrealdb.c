# Sanitizer support for the test suite.
#
# The C library hands ownership across an FFI boundary in thirteen different
# ways, and a leak there is invisible to an ordinary test run: the suite's
# Memory group exercises every free function, but "did not crash" is all it can
# assert without a checker underneath it. That gap let sr_create leak a boxed
# object on every call for as long as the function has existed.
#
#   cmake -B build -DSURREALDB_SANITIZE=address
#   ctest --test-dir build --output-on-failure
#
# Accepts a comma-separated list, so `address,undefined` works. LeakSanitizer
# rides along with address on Linux; on macOS it must be asked for at runtime
# with ASAN_OPTIONS=detect_leaks=1.
#
# The Rust staticlib is not itself instrumented, which does not matter for leak
# detection: LeakSanitizer interposes malloc, and the Rust side uses the system
# allocator. It does mean a bad pointer *inside* Rust may go unreported.

set(SURREALDB_SANITIZE "" CACHE STRING
    "Sanitizers for test targets: address, undefined, thread, or a comma-separated list")
set_property(CACHE SURREALDB_SANITIZE PROPERTY STRINGS
             "" "address" "undefined" "address,undefined" "thread")

function(surrealdb_apply_sanitizers target)
    if(NOT SURREALDB_SANITIZE)
        return()
    endif()

    if(MSVC)
        # MSVC supports AddressSanitizer only, spelled differently, and has no
        # leak detection at all -- so it cannot catch the class of bug this
        # exists for. Say so rather than silently producing a weaker build.
        if(SURREALDB_SANITIZE MATCHES "address")
            target_compile_options(${target} PRIVATE /fsanitize=address)
            message(STATUS "  ${target}: /fsanitize=address (MSVC has no leak detection)")
        else()
            message(WARNING "SURREALDB_SANITIZE=${SURREALDB_SANITIZE} is not supported by MSVC; ignoring.")
        endif()
        return()
    endif()

    if(NOT (CMAKE_C_COMPILER_ID MATCHES "GNU|Clang"))
        message(WARNING "SURREALDB_SANITIZE is only wired up for GCC, Clang and MSVC; ignoring.")
        return()
    endif()

    # Frame pointers keep sanitizer stack traces readable; without them a leak
    # report names the allocation site and little else.
    target_compile_options(${target} PRIVATE
        -fsanitize=${SURREALDB_SANITIZE}
        -fno-omit-frame-pointer
        -g
    )
    target_link_options(${target} PRIVATE -fsanitize=${SURREALDB_SANITIZE})
    message(STATUS "  ${target}: -fsanitize=${SURREALDB_SANITIZE}")
endfunction()
