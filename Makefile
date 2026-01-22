BUILD_DIR := build

# Platform-specific generator selection.
# Windows: Let CMake choose the default generator (Visual Studio) which handles MSVC properly.
#          Using Ninja on Windows can pick up wrong compilers (e.g., MinGW from Strawberry Perl).
# Unix/Linux/macOS: Use default generator (usually Make or Ninja)
GENERATOR_FLAG :=

.PHONY: all configure build test clean

all: build

configure:
	cmake -S . -B $(BUILD_DIR) $(GENERATOR_FLAG)

build: configure
	cmake --build $(BUILD_DIR)

test: build
ifeq ($(OS),Windows_NT)
	ctest --test-dir $(BUILD_DIR) --output-on-failure -C Debug
else
	ctest --test-dir $(BUILD_DIR) --output-on-failure
endif

clean:
	cmake -E rm -rf $(BUILD_DIR)
	cargo clean
