BUILD_DIR := build

# Platform-specific generator selection.
# Windows: Let CMake choose the default generator (Visual Studio) which handles MSVC properly.
#          Using Ninja on Windows can pick up wrong compilers (e.g., MinGW from Strawberry Perl).
# Unix/Linux/macOS: Use default generator (usually Make or Ninja)
GENERATOR_FLAG :=

.PHONY: configure clean

configure:
	cmake -S . -B $(BUILD_DIR) $(GENERATOR_FLAG)
	cmake --build $(BUILD_DIR)

clean:
	cmake -E rm -rf $(BUILD_DIR)
