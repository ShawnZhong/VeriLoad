VERUS_BIN := .verus/verus-x86-linux/verus

VERILOAD_MAIN := src/main_impl.rs
VERILOAD_SOURCES := $(wildcard src/*.rs)
VERILOAD_BINARY := $(BUILD_DIR)/veriload

$(VERILOAD_BINARY): $(VERILOAD_SOURCES) | $(BUILD_DIR)
	MAKEFLAGS= $(VERUS_BIN) --compile $(VERILOAD_MAIN) -- -C target-feature=+crt-static -o $(VERILOAD_BINARY)

.PHONY: verify
verify:
	MAKEFLAGS= $(VERUS_BIN) $(VERILOAD_MAIN)

.PHONY: compile
compile: $(VERILOAD_BINARY)
