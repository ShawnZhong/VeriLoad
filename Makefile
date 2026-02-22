BUILD_DIR := build

.PHONY: all
all: compile tests

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

include src/build.mk
include tests/build.mk

.PHONY: clean
clean:
	rm -rf $(BUILD_DIR)
