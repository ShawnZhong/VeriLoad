VERUS := ./.verus/verus-x86-linux/verus
SOURCE := src/main_impl.rs
BUILD_DIR := build
BINARY := $(BUILD_DIR)/veriload
IMAGE_NAME := veriload

include tests/build.mk

.PHONY: all
all: build tests

.PHONY: verify
verify:
	$(VERUS) $(SOURCE)

.PHONY: build
build: | $(BUILD_DIR)
	$(VERUS) --compile $(SOURCE) -- -C target-feature=+crt-static -o $(BINARY)

.PHONY: clean
clean:
	rm -rf $(BUILD_DIR)
