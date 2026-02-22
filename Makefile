VERUS := ./.verus/verus-x86-linux/verus
SOURCE := src/main_impl.rs
BUILD_DIR := build
BINARY := $(BUILD_DIR)/veriload
IMAGE_NAME := veriload

.DEFAULT_GOAL := all

include tests/build.mk

.PHONY: all
all: compile tests

.PHONY: verify
verify:
	$(VERUS) $(SOURCE)

.PHONY: compile
compile: | $(BUILD_DIR)
	$(VERUS) --compile $(SOURCE) -- -C target-feature=+crt-static -o $(BINARY)

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

.PHONY: clean
clean:
	rm -rf $(BUILD_DIR)
