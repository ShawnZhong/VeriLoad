BUILD_DIR := build

.PHONY: all
all: compile tests musl

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

include src/build.mk
include tests/build.mk
include third_party/build.mk

.PHONY: clean
clean:
	rm -rf $(BUILD_DIR)
