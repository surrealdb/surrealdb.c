# FindUnity.cmake
# 
# This module finds or builds the Unity testing framework.
# If system Unity is found, it exposes Unity::unity target.
# Otherwise, it clones Unity v2.6.1 from GitHub into ThirdParty/Unity and builds it.
#
# Output:
#   Unity::unity - imported target for linking
#   UNITY_FOUND - TRUE if Unity was found or built successfully

include_guard(GLOBAL)

include(FetchContent)
include(FindPackageHandleStandardArgs)

# First, try to find system-installed Unity
find_path(UNITY_INCLUDE_DIR
    NAMES unity.h
    PATHS
        /usr/include
        /usr/local/include
        /opt/local/include
        ${UNITY_ROOT}/include
        $ENV{UNITY_ROOT}/include
    PATH_SUFFIXES
        unity
)

find_library(UNITY_LIBRARY
    NAMES unity libunity
    PATHS
        /usr/lib
        /usr/local/lib
        /opt/local/lib
        ${UNITY_ROOT}/lib
        $ENV{UNITY_ROOT}/lib
)

# Check if we found system Unity
if(UNITY_INCLUDE_DIR AND UNITY_LIBRARY)
    message(STATUS "Found system Unity: ${UNITY_LIBRARY}")
    message(STATUS "Unity include dir: ${UNITY_INCLUDE_DIR}")
    
    # Create imported target for system Unity
    if(NOT TARGET Unity::unity)
        add_library(Unity::unity UNKNOWN IMPORTED)
        set_target_properties(Unity::unity PROPERTIES
            IMPORTED_LOCATION "${UNITY_LIBRARY}"
            INTERFACE_INCLUDE_DIRECTORIES "${UNITY_INCLUDE_DIR}"
        )
    endif()
    
    set(UNITY_FOUND TRUE)
else()
    message(STATUS "System Unity not found, fetching from GitHub...")
    
    # Define paths for Unity
    set(UNITY_SOURCE_DIR "${CMAKE_SOURCE_DIR}/ThirdParty/Unity")
    
    # Check if Unity is already cloned
    if(EXISTS "${UNITY_SOURCE_DIR}/CMakeLists.txt")
        message(STATUS "Unity source found at: ${UNITY_SOURCE_DIR}")
    else()
        message(STATUS "Cloning Unity v2.6.1 into ThirdParty/Unity...")
        
        # Use FetchContent to download Unity
        FetchContent_Declare(
            unity
            GIT_REPOSITORY https://github.com/ThrowTheSwitch/Unity.git
            GIT_TAG        v2.6.1
            SOURCE_DIR     "${UNITY_SOURCE_DIR}"
        )
        
        FetchContent_MakeAvailable(unity)
    endif()
    
    # If Unity was already cloned but not fetched via FetchContent
    if(NOT TARGET unity)
        # Add Unity as a subdirectory
        add_subdirectory("${UNITY_SOURCE_DIR}" "${CMAKE_BINARY_DIR}/ThirdParty/Unity-build")
    endif()
    
    # Create an alias target for consistency
    if(TARGET unity AND NOT TARGET Unity::unity)
        add_library(Unity::unity ALIAS unity)
    endif()
    
    # Set variables for find_package_handle_standard_args
    set(UNITY_INCLUDE_DIR "${UNITY_SOURCE_DIR}/src")
    set(UNITY_LIBRARY "unity")
    set(UNITY_FOUND TRUE)
    
    message(STATUS "Unity will be built from source at: ${UNITY_SOURCE_DIR}")
endif()

find_package_handle_standard_args(Unity
    REQUIRED_VARS UNITY_LIBRARY UNITY_INCLUDE_DIR
)

# Mark variables as advanced
mark_as_advanced(UNITY_INCLUDE_DIR UNITY_LIBRARY)
