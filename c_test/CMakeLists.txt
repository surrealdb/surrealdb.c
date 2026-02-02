# ============================================================================
# SurrealDB C Library - Test Suite
# ============================================================================

# Make sure CMake can find <solution-root>/cmake/FindUnity.cmake
list(APPEND CMAKE_MODULE_PATH "${PROJECT_SOURCE_DIR}/cmake")

find_package(Unity REQUIRED)

# Put test outputs in the build tree (cleaner than writing into source/)
set(_test_bin_dir "${CMAKE_CURRENT_BINARY_DIR}/bin")

# ============================================================================
# Test Source Files (Unity Fixture based)
# ============================================================================

set(TEST_SOURCES
    src/tests/connection_tests.c
    src/tests/transaction_tests.c
    src/tests/crud_tests.c
    src/tests/query_tests.c
    src/tests/io_tests.c
    src/tests/auth_tests.c
    src/tests/variable_tests.c
    src/tests/rpc_tests.c
    src/tests/stream_tests.c
    src/tests/object_tests.c
    src/tests/memory_tests.c
    src/tests/utility_tests.c
)

# API Tests - individual test implementations
set(API_TEST_SOURCES
    src/api_tests/api_tests.c
)

# ============================================================================
# Test Runner Executable
# ============================================================================

add_executable(test_runner
    src/bin/main.c
    ${TEST_SOURCES}
    ${API_TEST_SOURCES}
)

target_include_directories(test_runner PRIVATE
    "${CMAKE_CURRENT_SOURCE_DIR}/src/tests"
    "${CMAKE_CURRENT_SOURCE_DIR}/src/api_tests"
    "${PROJECT_SOURCE_DIR}/include"
    "${PROJECT_SOURCE_DIR}/ThirdParty/Unity/extras/fixture/src"
)

target_link_libraries(test_runner PRIVATE
    Unity::unity
    surrealdb_c
)

set_target_properties(test_runner PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY "${_test_bin_dir}"
)

# Fix LNK4098: Rust library uses release runtime (/MD), suppress conflict with debug runtime
if(MSVC)
    target_link_options(test_runner PRIVATE /NODEFAULTLIB:MSVCRT)
endif()

# ============================================================================
# Scratch Test Executable (for ad-hoc testing)
# ============================================================================

add_executable(test_scratch
    src/test_scratch.c
)

target_include_directories(test_scratch PRIVATE
    "${CMAKE_CURRENT_SOURCE_DIR}/src"
    "${PROJECT_SOURCE_DIR}/include"
)

target_link_libraries(test_scratch PRIVATE
    surrealdb_c
)

set_target_properties(test_scratch PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY "${_test_bin_dir}"
)

# Fix LNK4098: Rust library uses release runtime (/MD), suppress conflict with debug runtime
if(MSVC)
    target_link_options(test_scratch PRIVATE /NODEFAULTLIB:MSVCRT)
endif()

# ============================================================================
# CTest Integration
# ============================================================================

enable_testing()
add_test(NAME surrealdb_c_tests COMMAND test_runner)
set_tests_properties(surrealdb_c_tests PROPERTIES
    WORKING_DIRECTORY "${_test_bin_dir}"
)

message(STATUS "Test binaries will be output to: ${_test_bin_dir}")
message(STATUS "Unity Fixture extension enabled for test suites")

# ============================================================================
# Examples
# ============================================================================

add_subdirectory(examples)
