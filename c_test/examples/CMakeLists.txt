# ============================================================================
# SurrealDB C Library - Examples
# ============================================================================
#
# This CMakeLists.txt demonstrates how to use the surrealdb_c library.
# It uses the namespaced target surrealdb::surrealdb_c which works both:
#   - When built as a subdirectory of the main project (current setup)
#   - When used as an external consumer via find_package(surrealdb_c)
#
# To use this example as a standalone project, you would:
#   1. Install surrealdb_c: cmake --install <build_dir> --prefix <install_dir>
#   2. Copy this examples directory
#   3. Configure with: cmake -S . -B build -DCMAKE_PREFIX_PATH=<install_dir>
# ============================================================================

cmake_minimum_required(VERSION 3.21)

# When used as subdirectory, project() is optional but helps with IDE organization
if(NOT CMAKE_PROJECT_NAME)
    project(surrealdb_examples LANGUAGES C)
endif()

# Put example binaries in the build tree
set(_example_bin_dir "${CMAKE_CURRENT_BINARY_DIR}/bin")

# ============================================================================
# SaaS Platform Example
# ============================================================================

add_executable(saas_example
    saas_example.c
)

# Link using the namespaced target - this provides both the library and include directories
# When building as subdirectory: uses the ALIAS target surrealdb::surrealdb_c -> surrealdb_c
# When building standalone: uses the installed target from find_package(surrealdb_c)
target_link_libraries(saas_example PRIVATE
    surrealdb::surrealdb_c
)

set_target_properties(saas_example PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY "${_example_bin_dir}"
)

# Fix LNK4098: Rust library uses release runtime (/MD), suppress conflict with debug runtime
if(MSVC)
    target_link_options(saas_example PRIVATE /NODEFAULTLIB:MSVCRT)
endif()

# Link macOS system frameworks required by Rust dependencies (sysinfo, iana_time_zone)
if(APPLE)
    target_link_libraries(saas_example PRIVATE
        "-framework CoreFoundation"
        "-framework IOKit"
        objc
    )
endif()

message(STATUS "Example binaries will be output to: ${_example_bin_dir}")
