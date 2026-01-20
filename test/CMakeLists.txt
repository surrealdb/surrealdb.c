# ============================================================================
# SurrealDB C Library - Test Suite
# ============================================================================

# Make sure CMake can find <solution-root>/cmake/FindUnity.cmake
list(APPEND CMAKE_MODULE_PATH "${CMAKE_SOURCE_DIR}/cmake")

find_package(Unity REQUIRED)

# Put test outputs in the build tree (cleaner than writing into source/)
set(_test_bin_dir "${CMAKE_CURRENT_BINARY_DIR}/bin")

add_library(test_granular STATIC
        src/test_granular.c
)
target_include_directories(test_granular PRIVATE
        "${CMAKE_CURRENT_SOURCE_DIR}/src"
)
target_link_libraries(test_granular PRIVATE
        Unity::unity
)

add_executable(test_runner
        src/test.c
)
target_include_directories(test_runner PRIVATE
        "${CMAKE_CURRENT_SOURCE_DIR}/src"
)
target_link_libraries(test_runner PRIVATE
        test_granular
        surrealdb_c
        Unity::unity
)
set_target_properties(test_runner PROPERTIES
        RUNTIME_OUTPUT_DIRECTORY "${_test_bin_dir}"
)

add_executable(test_scratch
        src/test_scratch.c
)
target_include_directories(test_scratch PRIVATE
        "${CMAKE_CURRENT_SOURCE_DIR}/src"
)
target_link_libraries(test_scratch PRIVATE
        surrealdb_c
)
set_target_properties(test_scratch PROPERTIES
        RUNTIME_OUTPUT_DIRECTORY "${_test_bin_dir}"
)

enable_testing()
add_test(NAME surrealdb_c_tests COMMAND test_runner)
set_tests_properties(surrealdb_c_tests PROPERTIES
        WORKING_DIRECTORY "${_test_bin_dir}"
)

message(STATUS "Test binaries will be output to: ${_test_bin_dir}")
