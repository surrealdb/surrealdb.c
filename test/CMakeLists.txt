# ============================================================================
# SurrealDB C Library - Test Suite
# ============================================================================

# Make sure CMake can find <solution-root>/cmake/FindUnity.cmake
list(APPEND CMAKE_MODULE_PATH "${CMAKE_SOURCE_DIR}/cmake")

find_package(Unity REQUIRED)

# Put test outputs in the build tree (cleaner than writing into source/)
set(_test_bin_dir "${CMAKE_CURRENT_BINARY_DIR}/bin")

# ============================================================================
# Test Runner Executable
# All tests are header-only, so we just compile main.c
# ============================================================================

add_executable(test_runner
        src/bin/main.c
)
target_include_directories(test_runner PRIVATE
        "${CMAKE_CURRENT_SOURCE_DIR}/src/tests"
        "${CMAKE_SOURCE_DIR}/include"
)
target_link_libraries(test_runner PRIVATE
        Unity::unity
        surrealdb_c
)
set_target_properties(test_runner PROPERTIES
        RUNTIME_OUTPUT_DIRECTORY "${_test_bin_dir}"
)

# ============================================================================
# Scratch Test Executable (for ad-hoc testing)
# ============================================================================

add_executable(test_scratch
        src/test_scratch.c
)
target_include_directories(test_scratch PRIVATE
        "${CMAKE_CURRENT_SOURCE_DIR}/src"
        "${CMAKE_SOURCE_DIR}/include"
)
target_link_libraries(test_scratch PRIVATE
        surrealdb_c
)
set_target_properties(test_scratch PROPERTIES
        RUNTIME_OUTPUT_DIRECTORY "${_test_bin_dir}"
)

# ============================================================================
# CTest Integration
# ============================================================================

enable_testing()
add_test(NAME surrealdb_c_tests COMMAND test_runner)
set_tests_properties(surrealdb_c_tests PROPERTIES
        WORKING_DIRECTORY "${_test_bin_dir}"
)

message(STATUS "Test binaries will be output to: ${_test_bin_dir}")
