# NEXORA C++ kernel reference.
#
# No LTO on the FFI translation unit, on purpose: the point of ffi_boundary.cpp
# is to be opaque. Optimising across it would measure inlining rather than the
# boundary. Everything else is -O2, matching the Rust release profile's intent.
#
# No -ffast-math anywhere. The sweep kernel's conformance with the Rust engine
# depends on IEEE-754 semantics holding exactly.

CXX ?= g++
CXXFLAGS ?= -std=c++20 -O2 -Wall -Wextra -Wpedantic
BUILD := build

.PHONY: all clean conformance bench

all: $(BUILD)/conformance $(BUILD)/bench

$(BUILD):
	mkdir -p $(BUILD)

$(BUILD)/conformance: src/conformance.cpp src/kernels.hpp | $(BUILD)
	$(CXX) $(CXXFLAGS) -o $@ src/conformance.cpp

$(BUILD)/ffi_boundary.o: src/ffi_boundary.cpp | $(BUILD)
	$(CXX) $(CXXFLAGS) -fno-lto -c -o $@ src/ffi_boundary.cpp

$(BUILD)/bench: src/bench.cpp src/kernels.hpp $(BUILD)/ffi_boundary.o | $(BUILD)
	$(CXX) $(CXXFLAGS) -fno-lto -o $@ src/bench.cpp $(BUILD)/ffi_boundary.o

conformance: $(BUILD)/conformance
	./$(BUILD)/conformance

bench: $(BUILD)/bench
	./$(BUILD)/bench

clean:
	rm -rf $(BUILD)
